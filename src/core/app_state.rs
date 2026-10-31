use std::sync::{Arc, Mutex};
use crate::config::settings::AppSettings;
use crate::files::file_context::FileContext;

#[derive(Debug, Clone)]
pub struct AppStateData {
    pub settings: AppSettings,
    pub enabled: bool,
    pub active_file: Option<FileContext>,
    pub is_menu_open: bool,
}

impl Default for AppStateData {
    fn default() -> Self {
        Self {
            settings: AppSettings::default(),
            enabled: true,
            active_file: None,
            is_menu_open: false,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<AppStateData>>,
}

impl AppState {
    pub fn new(settings: AppSettings) -> Self {
        let enabled = settings.general.enabled;
        Self {
            inner: Arc::new(Mutex::new(AppStateData {
                settings,
                enabled,
                active_file: None,
                is_menu_open: false,
            })),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.lock().unwrap().enabled
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut data = self.inner.lock().unwrap();
        data.enabled = enabled;
        data.settings.general.enabled = enabled;
    }

    pub fn toggle_enabled(&self) -> bool {
        let mut data = self.inner.lock().unwrap();
        data.enabled = !data.enabled;
        data.settings.general.enabled = data.enabled;
        data.enabled
    }

    pub fn get_settings(&self) -> AppSettings {
        self.inner.lock().unwrap().settings.clone()
    }

    pub fn update_settings<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut AppSettings),
    {
        let mut data = self.inner.lock().unwrap();
        update_fn(&mut data.settings);
    }

    pub fn set_active_file(&self, file: Option<FileContext>) {
        self.inner.lock().unwrap().active_file = file;
    }

    pub fn get_active_file(&self) -> Option<FileContext> {
        self.inner.lock().unwrap().active_file.clone()
    }

    pub fn set_menu_open(&self, open: bool) {
        self.inner.lock().unwrap().is_menu_open = open;
    }

    pub fn is_menu_open(&self) -> bool {
        self.inner.lock().unwrap().is_menu_open
    }
}

