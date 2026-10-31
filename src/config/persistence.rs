use std::fs;
use std::path::PathBuf;
use crate::config::settings::AppSettings;

pub fn get_config_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("Kisayolluk")
    } else {
        PathBuf::from(".").join("config")
    }
}

pub fn get_config_file_path() -> PathBuf {
    get_config_dir().join("config.json")
}

pub fn load_settings() -> AppSettings {
    let path = get_config_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                return settings;
            }
        }
    }
    // Return default and attempt to save it
    let default_settings = AppSettings::default();
    let _ = save_settings(&default_settings);
    default_settings
}

pub fn save_settings(settings: &AppSettings) -> Result<(), std::io::Error> {
    let dir = get_config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    let path = get_config_file_path();
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    fs::write(path, json)?;
    Ok(())
}

