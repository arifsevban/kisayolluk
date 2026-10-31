use std::fs;
use std::path::{Path, PathBuf};
use crate::files::file_type::SupportedImageFormat;

#[derive(Debug, Clone)]
pub struct FileContext {
    pub path: PathBuf,
    pub file_name: String,
    pub extension: String,
    pub format: Option<SupportedImageFormat>,
    pub size_bytes: u64,
    pub is_accessible: bool,
    pub is_directory: bool,
}

impl FileContext {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_buf = path.as_ref().to_path_buf();
        if !path_buf.exists() {
            return Err("Dosya bulunamadı.".to_string());
        }

        let file_name = path_buf
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Bilinmeyen Dosya".to_string());

        let extension = path_buf
            .extension()
            .map(|e| e.to_string_lossy().to_string().to_lowercase())
            .unwrap_or_default();

        let format = SupportedImageFormat::from_extension(&extension);

        let metadata = fs::metadata(&path_buf).map_err(|e| format!("Dosya bilgisi okunamadı: {}", e))?;
        let is_directory = metadata.is_dir();
        let size_bytes = metadata.len();

        // Check accessibility by trying to open read-only
        let is_accessible = if is_directory {
            true
        } else {
            fs::File::open(&path_buf).is_ok()
        };

        Ok(Self {
            path: path_buf,
            file_name,
            extension,
            format,
            size_bytes,
            is_accessible,
            is_directory,
        })
    }
}

