use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const HOTKEY_ID: i32 = 0x5101;

pub struct HotkeyHandle {
    thread_handle: Option<JoinHandle<()>>,
    thread_id: u32,
}

impl HotkeyHandle {
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

pub fn parse_hotkey_string(str: &str) -> (HOT_KEY_MODIFIERS, u32) {
    let mut modifiers = HOT_KEY_MODIFIERS(0);
    let mut key: u32 = VK_SPACE.0 as u32;

    for part in str.split('+') {
        let trimmed = part.trim().to_lowercase();
        match trimmed.as_str() {
            "ctrl" | "control" => modifiers |= MOD_CONTROL,
            "shift" => modifiers |= MOD_SHIFT,
            "alt" => modifiers |= MOD_ALT,
            "win" | "windows" => modifiers |= MOD_WIN,
            "space" => key = VK_SPACE.0 as u32,
            "f1" => key = VK_F1.0 as u32,
            "f2" => key = VK_F2.0 as u32,
            "f3" => key = VK_F3.0 as u32,
            "f4" => key = VK_F4.0 as u32,
            "f5" => key = VK_F5.0 as u32,
            "f6" => key = VK_F6.0 as u32,
            "f7" => key = VK_F7.0 as u32,
            "f8" => key = VK_F8.0 as u32,
            "f9" => key = VK_F9.0 as u32,
            "f10" => key = VK_F10.0 as u32,
            "f11" => key = VK_F11.0 as u32,
            "f12" => key = VK_F12.0 as u32,
            s if s.len() == 1 => {
                let c = s.chars().next().unwrap().to_ascii_uppercase();
                key = c as u32;
            }
            _ => {}
        }
    }

    modifiers |= MOD_NOREPEAT;
    (modifiers, key)
}

pub fn start_hotkey_listener(
    hotkey_str: &str,
    sender: Sender<()>,
) -> std::result::Result<HotkeyHandle, String> {
    let (modifiers, key) = parse_hotkey_string(hotkey_str);
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();

    let thread_handle = thread::spawn(move || unsafe {
        let thread_id = GetCurrentThreadId();

        let registered = RegisterHotKey(
            None,
            HOTKEY_ID,
            modifiers,
            key,
        );

        if let Err(e) = registered {
            let _ = ready_tx.send(Err(format!("Kısayol tuşu kaydedilemedi: {}", e)));
            return;
        }

        let _ = ready_tx.send(Ok(thread_id));

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID {
                let _ = sender.send(());
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnregisterHotKey(None, HOTKEY_ID);
    });

    match ready_rx.recv_timeout(Duration::from_secs(3)) {
        Ok(Ok(thread_id)) => Ok(HotkeyHandle {
            thread_handle: Some(thread_handle),
            thread_id,
        }),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Kısayol dinleyicisi başlatma zaman aşımına uğradı.".to_string()),
    }
}
