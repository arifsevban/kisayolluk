use crate::files::file_type::SupportedImageFormat;

#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub format: SupportedImageFormat,
    pub name: &'static str,
    pub extension: &'static str,
    pub mime_type: &'static str,
    pub supports_alpha: bool,
}

pub struct FormatRegistry;

impl FormatRegistry {
    pub fn get_info(format: SupportedImageFormat) -> FormatInfo {
        match format {
            SupportedImageFormat::Png => FormatInfo {
                format,
                name: "PNG",
                extension: "png",
                mime_type: "image/png",
                supports_alpha: true,
            },
            SupportedImageFormat::Jpeg => FormatInfo {
                format,
                name: "JPEG",
                extension: "jpg",
                mime_type: "image/jpeg",
                supports_alpha: false,
            },
            SupportedImageFormat::Webp => FormatInfo {
                format,
                name: "WEBP",
                extension: "webp",
                mime_type: "image/webp",
                supports_alpha: true,
            },
            SupportedImageFormat::Bmp => FormatInfo {
                format,
                name: "BMP",
                extension: "bmp",
                mime_type: "image/bmp",
                supports_alpha: false,
            },
            SupportedImageFormat::Tiff => FormatInfo {
                format,
                name: "TIFF",
                extension: "tiff",
                mime_type: "image/tiff",
                supports_alpha: true,
            },
        }
    }
}

