use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use windows::core::PCWSTR;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL};
use windows::Win32::UI::WindowsAndMessaging::*;

static HOOK_ENABLED: AtomicBool = AtomicBool::new(true);
static WAS_CTRL_RCLICK_DOWN: AtomicBool = AtomicBool::new(false);
static LAST_HOOK_DURATION_NS: AtomicU64 = AtomicU64::new(0);

static mut EVENT_SENDER: Option<Sender<(i32, i32)>> = None;
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

pub fn start_mouse_hook(sender: Sender<(i32, i32)>) -> std::result::Result<MouseHookHandle, String> {
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
unsafe fn is_ctrl_down() -> bool {
    (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0
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

            if msg == WM_RBUTTONDOWN {
                if is_ctrl_down() {
                    WAS_CTRL_RCLICK_DOWN.store(true, Ordering::Relaxed);
                    // Swallow WM_RBUTTONDOWN so Windows doesn't initiate a context menu or selection focus
                    return LRESULT(1);
                } else {
                    WAS_CTRL_RCLICK_DOWN.store(false, Ordering::Relaxed);
                }
            } else if msg == WM_RBUTTONUP {
                let was_down = WAS_CTRL_RCLICK_DOWN.swap(false, Ordering::Relaxed);
                let ctrl_active = is_ctrl_down();

                if was_down {
                    // Paired: WM_RBUTTONDOWN was swallowed, so WM_RBUTTONUP must also be swallowed.
                    // Windows Explorer never saw DOWN, so it never has capture.
                    let ms_hook = *(l_param.0 as *const MSLLHOOKSTRUCT);
                    let x = ms_hook.pt.x;
                    let y = ms_hook.pt.y;

                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.send((x, y));
                    }

                    return LRESULT(1);
                } else if ctrl_active {
                    // User pressed Right Click DOWN first, then Ctrl:
                    // WM_RBUTTONDOWN was already sent to target window (e.g. Explorer).
                    // We MUST pass WM_RBUTTONUP to the target window so it releases mouse capture cleanly!
                    let ms_hook = *(l_param.0 as *const MSLLHOOKSTRUCT);
                    let x = ms_hook.pt.x;
                    let y = ms_hook.pt.y;

                    if let Some(ref sender) = EVENT_SENDER {
                        let _ = sender.send((x, y));
                    }

                    // CRITICAL: DO NOT return LRESULT(1) here!
                    // Let CallNextHookEx deliver WM_RBUTTONUP so Explorer never gets stuck in a 15-20s capture lockup!
                }
            } else if msg == WM_LBUTTONDOWN || msg == WM_MBUTTONDOWN {
                // Clear any pending state if user clicks left/middle button
                WAS_CTRL_RCLICK_DOWN.store(false, Ordering::Relaxed);
            }
        }

        CallNextHookEx(None, n_code, w_param, l_param)
    })();

    LAST_HOOK_DURATION_NS.store(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
    result
}
