use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEvent {
    MiddleClick { x: i32, y: i32 },
    OutsideClick,
}

static HOOK_ENABLED: AtomicBool = AtomicBool::new(true);
static WAS_MBUTTON_DOWN: AtomicBool = AtomicBool::new(false);
static MBUTTON_DOWN_POS_X: AtomicI32 = AtomicI32::new(0);
static MBUTTON_DOWN_POS_Y: AtomicI32 = AtomicI32::new(0);
static LAST_HOOK_DURATION_NS: AtomicU64 = AtomicU64::new(0);

static MENU_OPEN: AtomicBool = AtomicBool::new(false);
static MENU_POS_X: AtomicI32 = AtomicI32::new(0);
static MENU_POS_Y: AtomicI32 = AtomicI32::new(0);

static mut EVENT_SENDER: Option<Sender<MouseEvent>> = None;
static mut HOOK_HANDLE: HHOOK = HHOOK(std::ptr::null_mut());

pub struct MouseHookHandle {
    thread_handle: Option<JoinHandle<()>>,
    thread_id: u32,
}

impl MouseHookHandle {
    pub fn set_enabled(&self, enabled: bool) {
        HOOK_ENABLED.store(enabled, Ordering::SeqCst);
    }

    /// Kept for API compatibility; double-click logic has been replaced with Ctrl + Right Click.
    pub fn set_double_click_time_ms(&self, _ms: u64) {}

    pub fn stop(mut self) {
        if self.thread_id != 0 {
            unsafe {
                let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

pub fn get_last_hook_duration_ns() -> u64 {
    LAST_HOOK_DURATION_NS.load(Ordering::Relaxed)
}

pub fn set_menu_bounds(open: bool, pos_x: i32, pos_y: i32) {
    MENU_POS_X.store(pos_x, Ordering::Relaxed);
    MENU_POS_Y.store(pos_y, Ordering::Relaxed);
    MENU_OPEN.store(open, Ordering::Relaxed);
}

pub fn set_menu_closed() {
    MENU_OPEN.store(false, Ordering::Relaxed);
}

pub fn start_mouse_hook(sender: Sender<MouseEvent>) -> std::result::Result<MouseHookHandle, String> {
    unsafe {
        EVENT_SENDER = Some(sender);
    }

    let (ready_tx, ready_rx) = std::sync::mpsc::channel();

    let thread_handle = thread::spawn(move || unsafe {
        let thread_id = GetCurrentThreadId();

        // 1. Ensure message queue exists for this thread before registering hook
        let mut msg = MSG::default();
        let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);

        // 2. Obtain current module handle for global hook registration
        let hinstance = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let hinstance_cast = HINSTANCE(hinstance.0);

        let hook = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(low_level_mouse_proc),
            Some(hinstance_cast),
            0,
        );

        match hook {
            Ok(h) => {
                HOOK_HANDLE = h;
                let _ = ready_tx.send(Ok(thread_id));
            }
            Err(e) => {
                let _ = ready_tx.send(Err(format!("Mouse hook başlatılamadı: {}", e)));
                return;
            }
        }

        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if !HOOK_HANDLE.0.is_null() {
            let _ = UnhookWindowsHookEx(HOOK_HANDLE);
            HOOK_HANDLE = HHOOK(std::ptr::null_mut());
        }
    });

    match ready_rx.recv_timeout(Duration::from_secs(3)) {
        Ok(Ok(thread_id)) => Ok(MouseHookHandle {
            thread_handle: Some(thread_handle),
            thread_id,
        }),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Mouse hook başlatma zaman aşımına uğradı.".to_string()),
    }
}

#[inline]
pub fn matches_wide_ignore_ascii_case(slice: &[u16], target: PCWSTR) -> bool {
    unsafe {
        let mut ptr = target.0;
        let mut i = 0;
        while *ptr != 0 {
            if i >= slice.len() {
                return false;
            }
            let c1 = slice[i];
            let c2 = *ptr;
            if c1 != c2 {
                let lower1 = if c1 <= 0x7F { (c1 as u8).to_ascii_lowercase() as u16 } else { c1 };
                let lower2 = if c2 <= 0x7F { (c2 as u8).to_ascii_lowercase() as u16 } else { c2 };
                if lower1 != lower2 {
                    return false;
                }
            }
            ptr = ptr.add(1);
            i += 1;
        }
        i == slice.len()
    }
}

#[inline]
pub unsafe fn is_desktop_or_explorer_point(pt: POINT) -> bool {
    let hwnd = WindowFromPoint(pt);
    if hwnd.0.is_null() {
        return false;
    }

    let mut buf = [0u16; 64];
    let len = GetClassNameW(hwnd, &mut buf);
    if len > 0 {
        let name = &buf[..len as usize];
        // Exclude Windows Taskbar explicitly so taskbar middle-clicks are never intercepted
        if matches_wide_ignore_ascii_case(name, w!("Shell_TrayWnd"))
            || matches_wide_ignore_ascii_case(name, w!("Shell_SecondaryTrayWnd"))
        {
            return false;
        }
        // Immediate desktop window classes under cursor
        if matches_wide_ignore_ascii_case(name, w!("SysListView32"))
            || matches_wide_ignore_ascii_case(name, w!("SHELLDLL_DefView"))
            || matches_wide_ignore_ascii_case(name, w!("Progman"))
            || matches_wide_ignore_ascii_case(name, w!("WorkerW"))
        {
            return true;
        }
    }

    let root = GetAncestor(hwnd, GA_ROOT);
    let target = if root.0.is_null() { hwnd } else { root };

    let len_root = GetClassNameW(target, &mut buf);
    if len_root > 0 {
        let root_name = &buf[..len_root as usize];
        if matches_wide_ignore_ascii_case(root_name, w!("Shell_TrayWnd"))
            || matches_wide_ignore_ascii_case(root_name, w!("Shell_SecondaryTrayWnd"))
        {
            return false;
        }
        if matches_wide_ignore_ascii_case(root_name, w!("CabinetWClass"))
            || matches_wide_ignore_ascii_case(root_name, w!("ExploreWClass"))
            || matches_wide_ignore_ascii_case(root_name, w!("Progman"))
            || matches_wide_ignore_ascii_case(root_name, w!("WorkerW"))
        {
            return true;
        }
    }

    false
}

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let start = Instant::now();

    let result = (|| {
        if n_code >= 0 && HOOK_ENABLED.load(Ordering::Relaxed) {
            let msg = w_param.0 as u32;

            // 1. Outside Click Detection: close menu if user clicks outside the radial menu bounds
            let mut is_outside_click = false;
            if MENU_OPEN.load(Ordering::Relaxed) {
                if msg == WM_LBUTTONDOWN || msg == WM_RBUTTONDOWN || msg == WM_MBUTTONDOWN {
                    let ms_hook = *(l_param.0 as *const MSLLHOOKSTRUCT);
                    let click_x = ms_hook.pt.x;
                    let click_y = ms_hook.pt.y;

                    let menu_x = MENU_POS_X.load(Ordering::Relaxed);
                    let menu_y = MENU_POS_Y.load(Ordering::Relaxed);

                    // Circular menu has radius up to 215px centered at (menu_x + 220, menu_y + 220)
                    let center_x = menu_x + 220;
                    let center_y = menu_y + 220;
                    let dx = click_x - center_x;
                    let dy = click_y - center_y;
                    let dist_sq = dx * dx + dy * dy;

                    // If click is outside the 225px radius circular menu:
                    if dist_sq > 225 * 225 {
                        is_outside_click = true;
                        if MENU_OPEN.swap(false, Ordering::Relaxed) {
                            if let Some(ref sender) = EVENT_SENDER {
                                let _ = sender.send(MouseEvent::OutsideClick);
                            }
                        }
                    }
                }
            }

            // 2. Middle Mouse Button (Mouse Wheel Click) Activation:
            // Only active on Desktop or File Explorer surfaces when the menu is not already open
            if !is_outside_click && !MENU_OPEN.load(Ordering::Relaxed) {
                if msg == WM_MBUTTONDOWN {
                    let ms_hook = *(l_param.0 as *const MSLLHOOKSTRUCT);
                    if is_desktop_or_explorer_point(ms_hook.pt) {
                        WAS_MBUTTON_DOWN.store(true, Ordering::Relaxed);
                        MBUTTON_DOWN_POS_X.store(ms_hook.pt.x, Ordering::Relaxed);
                        MBUTTON_DOWN_POS_Y.store(ms_hook.pt.y, Ordering::Relaxed);
                        // Swallow WM_MBUTTONDOWN so Windows Explorer/Desktop doesn't start auto-scroll or selection
                        return LRESULT(1);
                    }
                }
            }

            if msg == WM_MBUTTONUP {
                let was_down = WAS_MBUTTON_DOWN.swap(false, Ordering::Relaxed);
                if was_down {
                    let ms_hook = *(l_param.0 as *const MSLLHOOKSTRUCT);
                    let start_x = MBUTTON_DOWN_POS_X.load(Ordering::Relaxed);
                    let start_y = MBUTTON_DOWN_POS_Y.load(Ordering::Relaxed);
                    let dx = ms_hook.pt.x - start_x;
                    let dy = ms_hook.pt.y - start_y;

                    // Small tolerance for mouse jitter during click (<= 25px)
                    if dx * dx + dy * dy <= 25 * 25 {
                        if let Some(ref sender) = EVENT_SENDER {
                            let _ = sender.send(MouseEvent::MiddleClick {
                                x: ms_hook.pt.x,
                                y: ms_hook.pt.y,
                            });
                        }
                    }

                    // Swallow WM_MBUTTONUP to cleanly pair with the swallowed WM_MBUTTONDOWN
                    return LRESULT(1);
                }
            } else if msg == WM_LBUTTONDOWN || msg == WM_RBUTTONDOWN {
                // Clear any pending middle-click state if other buttons are pressed
                WAS_MBUTTON_DOWN.store(false, Ordering::Relaxed);
            }
        }

        CallNextHookEx(None, n_code, w_param, l_param)
    })();

    LAST_HOOK_DURATION_NS.store(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
    result
}
