use std::path::Path;
use std::process::Command;

pub struct NotificationManager;

impl NotificationManager {
    pub fn open_in_explorer(file_path: &Path) {
        let path_str = file_path.to_string_lossy().to_string();
        let _ = Command::new("explorer.exe")
            .arg(format!("/select,\"{}\"", path_str))
            .spawn();
    }
}

