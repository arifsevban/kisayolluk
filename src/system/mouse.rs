use std::sync::mpsc::Sender;
use crate::platform::windows::mouse_hook::{start_mouse_hook, MouseEvent, MouseHookHandle};

pub struct MouseManager {
    handle: Option<MouseHookHandle>,
}

impl MouseManager {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn start(&mut self, sender: Sender<MouseEvent>) -> Result<(), String> {
        let handle = start_mouse_hook(sender)?;
        self.handle = Some(handle);
        Ok(())
    }

    pub fn set_enabled(&self, enabled: bool) {
        if let Some(ref handle) = self.handle {
            handle.set_enabled(enabled);
        }
    }

    pub fn set_double_click_time_ms(&self, ms: u64) {
        if let Some(ref handle) = self.handle {
            handle.set_double_click_time_ms(ms);
        }
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.stop();
        }
    }
}

impl Drop for MouseManager {
    fn drop(&mut self) {
        self.stop();
    }
}

