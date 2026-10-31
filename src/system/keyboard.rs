use std::sync::mpsc::Sender;
use crate::platform::windows::keyboard::{start_hotkey_listener, HotkeyHandle};

pub struct KeyboardManager {
    handle: Option<HotkeyHandle>,
}

impl KeyboardManager {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn start(&mut self, hotkey: &str, sender: Sender<()>) -> Result<(), String> {
        let handle = start_hotkey_listener(hotkey, sender)?;
        self.handle = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.stop();
        }
    }
}

impl Drop for KeyboardManager {
    fn drop(&mut self) {
        self.stop();
    }
}

