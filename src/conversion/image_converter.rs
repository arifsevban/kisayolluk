use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat, Rgb, RgbImage};
use crate::config::settings::AppSettings;
use crate::files::file_type::SupportedImageFormat;

pub struct ImageConverter;

impl ImageConverter {
    pub fn convert(
        source_path: &Path,
        target_path: &Path,
        target_format: SupportedImageFormat,
        settings: &AppSettings,
    ) -> Result<PathBuf, String> {
        let img = image::ImageReader::open(source_path)
            .map_err(|e| format!("Dosya açılamadı: {}", e))?
            .decode()
            .map_err(|e| format!("Görsel çözümlenemedi (bozuk dosya veya desteklenmeyen alt format): {}", e))?;

        let file = File::create(target_path)
            .map_err(|e| format!("Hedef dosya oluşturulamadı: {}", e))?;
        let mut writer = BufWriter::new(file);

        match target_format {
            SupportedImageFormat::Jpeg => {
                let rgb_img = Self::composite_on_background(&img, settings.conversion.jpeg_background_color);
                let quality = settings.conversion.jpeg_quality.clamp(1, 100);
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
                encoder
                    .encode(
                        rgb_img.as_raw(),
                        rgb_img.width(),
                        rgb_img.height(),
                        ColorType::Rgb8.into(),
                    )
                    .map_err(|e| format!("JPEG kodlama hatası: {}", e))?;
            }
            SupportedImageFormat::Png => {
                img.write_to(&mut writer, ImageFormat::Png)
                    .map_err(|e| format!("PNG dönüştürme hatası: {}", e))?;
            }
            SupportedImageFormat::Webp => {
                img.write_to(&mut writer, ImageFormat::WebP)
                    .map_err(|e| format!("WEBP dönüştürme hatası: {}", e))?;
            }
            SupportedImageFormat::Bmp => {
                // If the source image has alpha, blend it on white/bg to prevent BMP alpha glitches
                let rgb_img = Self::composite_on_background(&img, settings.conversion.jpeg_background_color);
                let dyn_rgb = DynamicImage::ImageRgb8(rgb_img);
                dyn_rgb
                    .write_to(&mut writer, ImageFormat::Bmp)
                    .map_err(|e| format!("BMP dönüştürme hatası: {}", e))?;
            }
            SupportedImageFormat::Tiff => {
                img.write_to(&mut writer, ImageFormat::Tiff)
                    .map_err(|e| format!("TIFF dönüştürme hatası: {}", e))?;
            }
        }

        Ok(target_path.to_path_buf())
    }

    fn composite_on_background(img: &DynamicImage, bg: [u8; 3]) -> RgbImage {
        let (width, height) = img.dimensions();
        let mut rgb_img = RgbImage::new(width, height);

        match img {
            DynamicImage::ImageRgb8(rgb) => return rgb.clone(),
            _ => {}
        }

        let rgba = img.to_rgba8();
        for y in 0..height {
            for x in 0..width {
                let p = rgba.get_pixel(x, y);
                let a = p[3] as f32 / 255.0;
                let r = ((p[0] as f32 * a) + (bg[0] as f32 * (1.0 - a))).round() as u8;
                let g = ((p[1] as f32 * a) + (bg[1] as f32 * (1.0 - a))).round() as u8;
                let b = ((p[2] as f32 * a) + (bg[2] as f32 * (1.0 - a))).round() as u8;
                rgb_img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }

        rgb_img
    }
}
