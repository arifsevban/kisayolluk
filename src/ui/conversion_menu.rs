use crate::files::file_type::SupportedImageFormat;

pub struct ConversionMenuModel {
    pub source_format: Option<SupportedImageFormat>,
    pub available_targets: Vec<SupportedImageFormat>,
}

impl ConversionMenuModel {
    pub fn new(source: Option<SupportedImageFormat>) -> Self {
        let available_targets = SupportedImageFormat::targets_for(source);
        Self {
            source_format: source,
            available_targets,
        }
    }
}
