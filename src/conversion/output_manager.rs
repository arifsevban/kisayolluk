use std::path::{Path, PathBuf};
use crate::config::settings::CollisionStrategy;
use crate::files::file_type::SupportedImageFormat;

pub struct OutputManager;

impl OutputManager {
    pub fn determine_output_path(
        source_path: &Path,
        target_format: SupportedImageFormat,
        output_dir_override: Option<&Path>,
        strategy: CollisionStrategy,
    ) -> Result<PathBuf, String> {
        let parent_dir = if let Some(dir) = output_dir_override {
            dir.to_path_buf()
        } else {
            source_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        };

        if !parent_dir.exists() {
            std::fs::create_dir_all(&parent_dir)
                .map_err(|e| format!("Hedef klasör oluşturulamadı: {}", e))?;
        }

        let stem = source_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "donusturulen".to_string());

        let target_ext = target_format.default_extension();
        let direct_target = parent_dir.join(format!("{}.{}", stem, target_ext));

        if !direct_target.exists() {
            return Ok(direct_target);
        }

        match strategy {
            CollisionStrategy::Overwrite => Ok(direct_target),
            CollisionStrategy::Skip => Err(format!(
                "Hedef dosya zaten mevcut: {}",
                direct_target.display()
            )),
            CollisionStrategy::AddSuffix => {
                let mut counter = 1;
                loop {
                    let candidate = parent_dir.join(format!("{}_{}.{}", stem, counter, target_ext));
                    if !candidate.exists() {
                        return Ok(candidate);
                    }
                    counter += 1;
                    if counter > 10000 {
                        return Err("Uygun dosya adı bulunamadı (çok fazla çakışma).".to_string());
                    }
                }
            }
        }
    }
}

