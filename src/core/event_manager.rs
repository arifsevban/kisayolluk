use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::config::settings::AppSettings;
use crate::files::file_type::SupportedImageFormat;
use crate::files::resolver::FileResolverResult;

#[derive(Debug, Clone)]
pub enum AppEvent {
    CtrlRightClick { x: i32, y: i32 },
    FileResolved {
        resolution: FileResolverResult,
        x: i32,
        y: i32,
    },
    ToggleEnabled,
    OpenSettings,
    CloseMenu,
    ConvertRequested { target_format: SupportedImageFormat },
    OpenFolder { path: PathBuf },
    ShowToast { message: String, is_error: bool },
    SettingsUpdated(AppSettings),
    ExitApp,
}

#[derive(Clone)]
pub struct EventSender {
    sender: Sender<AppEvent>,
}

impl EventSender {
    pub fn new(sender: Sender<AppEvent>) -> Self {
        Self { sender }
    }

    pub fn send(&self, event: AppEvent) {
        let _ = self.sender.send(event);
    }
}

pub struct EventManager {
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
}

impl EventManager {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }

    pub fn get_sender(&self) -> EventSender {
        EventSender::new(self.sender.clone())
    }

    pub fn get_raw_sender(&self) -> Sender<AppEvent> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> &Receiver<AppEvent> {
        &self.receiver
    }
}

