use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollisionStrategy {
    AddSuffix,
    Overwrite,
    Skip,
}

impl Default for CollisionStrategy {
    fn default() -> Self {
        Self::AddSuffix
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub start_with_windows: bool,
    pub enabled: bool,
    pub show_tray_icon: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            start_with_windows: false,
            enabled: true,
            show_tray_icon: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutSettings {
    pub global_shortcut: String,
    pub double_right_click_enabled: bool,
    pub double_click_time_ms: u32,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            global_shortcut: "Ctrl+Shift+Space".to_string(),
            double_right_click_enabled: true,
            double_click_time_ms: 400,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub menu_size: u32,
    pub animation_duration_ms: u32,
    pub theme: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            menu_size: 260,
            animation_duration_ms: 150,
            theme: "dark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionSettings {
    pub default_output_dir: Option<PathBuf>,
    pub jpeg_quality: u8,
    pub jpeg_background_color: [u8; 3],
    pub collision_strategy: CollisionStrategy,
}

impl Default for ConversionSettings {
    fn default() -> Self {
        Self {
            default_output_dir: None,
            jpeg_quality: 90,
            jpeg_background_color: [255, 255, 255],
            collision_strategy: CollisionStrategy::AddSuffix,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub shortcuts: ShortcutSettings,
    pub appearance: AppearanceSettings,
    pub conversion: ConversionSettings,
}

