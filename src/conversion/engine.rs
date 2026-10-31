use std::path::{Path, PathBuf};
use crate::config::settings::AppSettings;
use crate::conversion::image_converter::ImageConverter;
use crate::conversion::output_manager::OutputManager;
use crate::files::file_type::SupportedImageFormat;

pub struct ConversionEngine;

impl ConversionEngine {
    pub fn execute_conversion(
        source_path: &Path,
        target_format: SupportedImageFormat,
        settings: &AppSettings,
    ) -> Result<PathBuf, String> {
        let output_path = OutputManager::determine_output_path(
            source_path,
            target_format,
            settings.conversion.default_output_dir.as_deref(),
            settings.conversion.collision_strategy,
        )?;

        let final_path = ImageConverter::convert(source_path, &output_path, target_format, settings)?;
        Ok(final_path)
    }
}

