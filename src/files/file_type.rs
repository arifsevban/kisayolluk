use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedImageFormat {
    Png,
    Jpeg,
    Webp,
    Bmp,
    Tiff,
}

impl SupportedImageFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::Webp),
            "bmp" => Some(Self::Bmp),
            "tif" | "tiff" => Some(Self::Tiff),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPG",
            Self::Webp => "WEBP",
            Self::Bmp => "BMP",
            Self::Tiff => "TIFF",
        }
    }

    pub fn default_extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
        }
    }

    pub fn all_formats() -> &'static [SupportedImageFormat] {
        &[
            Self::Png,
            Self::Jpeg,
            Self::Webp,
            Self::Bmp,
            Self::Tiff,
        ]
    }

    /// Returns target conversion options, excluding current source format
    pub fn targets_for(source: Option<SupportedImageFormat>) -> Vec<SupportedImageFormat> {
        Self::all_formats()
            .iter()
            .copied()
            .filter(|&f| Some(f) != source)
            .collect()
    }
}

