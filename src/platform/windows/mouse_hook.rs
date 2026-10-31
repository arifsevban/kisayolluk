use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use windows::Win32::Foundation::*;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::*;

static HOOK_ENABLED: AtomicBool = AtomicBool::new(true);
static DOUBLE_CLICK_TIME_MS: AtomicI32 = AtomicI32::new(450);

static LAST_RCLICK_X: AtomicI32 = AtomicI32::new(0);
static LAST_RCLICK_Y: AtomicI32 = AtomicI32::new(0);
static LAST_RCLICK_TIME_MS: AtomicU64 = AtomicU64::new(0);

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

    pub fn set_double_click_time_ms(&self, ms: u64) {
        DOUBLE_CLICK_TIME_MS.store(ms as i32, Ordering::SeqCst);
    }

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

pub fn start_mouse_hook(sender: Sender<(i32, i32)>) -> std::result::Result<MouseHookHandle, String> {
    unsafe {
        EVENT_SENDER = Some(sender);
    }

    let (ready_tx, ready_rx) = std::sync::mpsc::channel();

    let thread_handle = thread::spawn(move || unsafe {
        let thread_id = GetCurrentThreadId();
        let hook = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(low_level_mouse_proc),
            None,
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

        let mut msg = MSG::default();
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

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 && HOOK_ENABLED.load(Ordering::Relaxed) {
        let msg = w_param.0 as u32;
        if msg == WM_RBUTTONUP {
            let ms_hook = *(l_param.0 as *const MSLLHOOKSTRUCT);
            let x = ms_hook.pt.x;
            let y = ms_hook.pt.y;

            let now_ms = GetTickCount64();
            let last_ms = LAST_RCLICK_TIME_MS.load(Ordering::Relaxed);
            let last_x = LAST_RCLICK_X.load(Ordering::Relaxed);
            let last_y = LAST_RCLICK_Y.load(Ordering::Relaxed);

            let max_time_ms = (DOUBLE_CLICK_TIME_MS.load(Ordering::Relaxed) as u64).max(500);
            let elapsed = now_ms.saturating_sub(last_ms);

            // Generous spatial tolerance for right-click: at least 24px
            let max_dx = (GetSystemMetrics(SM_CXDOUBLECLK) * 3).max(24);
            let max_dy = (GetSystemMetrics(SM_CYDOUBLECLK) * 3).max(24);

            let is_double_click = elapsed >= 20
                && elapsed <= max_time_ms
                && (x - last_x).abs() <= max_dx
                && (y - last_y).abs() <= max_dy;

            if is_double_click {
                LAST_RCLICK_TIME_MS.store(0, Ordering::Relaxed);
                if let Some(ref sender) = EVENT_SENDER {
                    let _ = sender.send((x, y));
                }
            } else {
                LAST_RCLICK_TIME_MS.store(now_ms, Ordering::Relaxed);
                LAST_RCLICK_X.store(x, Ordering::Relaxed);
                LAST_RCLICK_Y.store(y, Ordering::Relaxed);
            }
        }
    }

    CallNextHookEx(Some(HOOK_HANDLE), n_code, w_param, l_param)
}
