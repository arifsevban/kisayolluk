use std::path::PathBuf;

pub struct NotificationItem {
    pub message: String,
    pub is_error: bool,
    pub target_folder: Option<PathBuf>,
}
