pub mod settings;
pub mod persistence;

pub use settings::{AppSettings, CollisionStrategy};
pub use persistence::{load_settings, save_settings, get_config_file_path};

