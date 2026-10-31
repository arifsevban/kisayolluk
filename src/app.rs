use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Instant;

use crate::config::persistence::save_settings;
use crate::config::settings::AppSettings;
use crate::conversion::engine::ConversionEngine;
use crate::core::app_state::AppState;
use crate::core::event_manager::{AppEvent, EventSender};
use crate::files::resolver::{resolve_selected_file, FileResolverResult};
use crate::system::keyboard::KeyboardManager;
use crate::system::mouse::MouseManager;
use crate::system::notifications::NotificationManager;
use crate::system::startup::set_startup;
use crate::system::tray::{SystemTray, TrayCommand};
use crate::ui::radial_menu::RadialMenuController;
use crate::ui::settings::SettingsController;

pub struct KisayollukApp {
    state: AppState,
    radial_ui: RadialMenuController,
    settings_ui: SettingsController,
    mouse_mgr: MouseManager,
    _keyboard_mgr: KeyboardManager,
    tray: Option<SystemTray>,
    _event_sender: EventSender,
    last_converted_path: Option<PathBuf>,
    last_click_pos: (i32, i32),
}

impl KisayollukApp {
    pub fn new(
        settings: AppSettings,
        event_sender: Sender<AppEvent>,
    ) -> Result<Self, String> {
        let app_state = AppState::new(settings.clone());
        let es = EventSender::new(event_sender.clone());

        let radial_ui = RadialMenuController::new(event_sender.clone())
            .map_err(|e| format!("Radial UI başlatılamadı: {}", e))?;

        let settings_ui = SettingsController::new(event_sender.clone())
            .map_err(|e| format!("Settings UI başlatılamadı: {}", e))?;

        let mut mouse_mgr = MouseManager::new();
        let es_mouse = event_sender.clone();
        let (mouse_tx, mouse_rx) = std::sync::mpsc::channel();
        mouse_mgr
            .start(mouse_tx)
            .map_err(|e| format!("Mouse yöneticisi başlatılamadı: {}", e))?;

        thread::spawn(move || {
            while let Ok(event) = mouse_rx.recv() {
                match event {
                    crate::platform::windows::mouse_hook::MouseEvent::CtrlRightClick { x, y } => {
                        let _ = es_mouse.send(AppEvent::CtrlRightClick { x, y });
                    }
                    crate::platform::windows::mouse_hook::MouseEvent::OutsideClick => {
                        let _ = es_mouse.send(AppEvent::CloseMenu);
                    }
                }
            }
        });

        let mut keyboard_mgr = KeyboardManager::new();
        let es_kb = event_sender.clone();
        let (kb_tx, kb_rx) = std::sync::mpsc::channel();
        if let Err(e) = keyboard_mgr.start(&settings.shortcuts.global_shortcut, kb_tx) {
            eprintln!("Klavye kısayolu kaydedilemedi: {}", e);
        }

        thread::spawn(move || {
            while let Ok(()) = kb_rx.recv() {
                let _ = es_kb.send(AppEvent::ToggleEnabled);
            }
        });

        let es_tray = event_sender.clone();
        let (tray_tx, tray_rx) = std::sync::mpsc::channel();
        let tray = if settings.general.show_tray_icon {
            match SystemTray::new(tray_tx) {
                Ok(t) => {
                    thread::spawn(move || {
                        while let Ok(cmd) = tray_rx.recv() {
                            match cmd {
                                TrayCommand::ToggleEnabled => {
                                    let _ = es_tray.send(AppEvent::ToggleEnabled);
                                }
                                TrayCommand::OpenSettings => {
                                    let _ = es_tray.send(AppEvent::OpenSettings);
                                }
                                TrayCommand::About => {
                                    let _ = es_tray.send(AppEvent::ShowToast {
                                        message: "Kısayolluk v0.1.0\nHafif ve Yerel Radyal İşlem Aracı".to_string(),
                                        is_error: false,
                                    });
                                }
                                TrayCommand::Exit => {
                                    let _ = es_tray.send(AppEvent::ExitApp);
                                }
                            }
                        }
                    });
                    Some(t)
                }
                Err(e) => {
                    eprintln!("Sistem tepsisi başlatılamadı: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            state: app_state,
            radial_ui,
            settings_ui,
            mouse_mgr,
            _keyboard_mgr: keyboard_mgr,
            tray,
            _event_sender: es,
            last_converted_path: None,
            last_click_pos: (400, 300),
        })
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::CtrlRightClick { x, y } => {
                self.last_click_pos = (x, y);

                if !self.state.is_enabled() {
                    return;
                }

                // Explicitly release any active mouse capture so clicks immediately reach our UI
                unsafe {
                    let _ = windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                }

                // Hide previous menu/toast before querying so our own window doesn't interfere
                self.radial_ui.hide();

                let es_clone = self._event_sender.clone();
                thread::spawn(move || {
                    let start_time = Instant::now();
                    let resolution = resolve_selected_file();
                    let elapsed_ms = start_time.elapsed().as_millis();
                    println!("Dosya algılama tamamlandı: {} ms", elapsed_ms);
                    es_clone.send(AppEvent::FileResolved {
                        resolution,
                        x,
                        y,
                    });
                });
            }

            AppEvent::FileResolved { resolution, x, y } => {
                match resolution {
                    FileResolverResult::Selected(ctx) => {
                        if ctx.is_directory {
                            self.radial_ui.show_toast(
                                "Klasör seçildi. Dönüştürmek için bir görsel dosyası seçin.",
                                true,
                                false,
                                x,
                                y,
                            );
                            return;
                        }

                        if ctx.format.is_none() {
                            self.radial_ui.show_toast(
                                &format!("Desteklenmeyen dosya formatı (.{})", ctx.extension),
                                true,
                                false,
                                x,
                                y,
                            );
                            return;
                        }

                        self.state.set_active_file(Some(ctx.clone()));
                        self.state.set_menu_open(true);
                        self.radial_ui.show_menu(&ctx, x, y);
                    }
                    FileResolverResult::NoSelection => {
                        self.radial_ui.show_toast("Önce bir dosya seçmelisin.", true, false, x, y);
                    }
                    FileResolverResult::MultipleSelection(count) => {
                        self.radial_ui.show_toast(
                            &format!("Devam etmek için tek bir dosya seç ({} dosya seçili).", count),
                            true,
                            false,
                            x,
                            y,
                        );
                    }
                    FileResolverResult::UnsupportedApp => {
                        self.radial_ui.show_toast(
                            "Dosya Gezgini veya Masaüstü'nde bir dosya seçin.",
                            true,
                            false,
                            x,
                            y,
                        );
                    }
                    FileResolverResult::Error(err) => {
                        self.radial_ui.show_toast(&err, true, false, x, y);
                    }
                }
            }

            AppEvent::ConvertRequested { target_format } => {
                if let Some(file) = self.state.get_active_file() {
                    let settings = self.state.get_settings();
                    let start_time = Instant::now();
                    match ConversionEngine::execute_conversion(&file.path, target_format, &settings) {
                        Ok(output_path) => {
                            let duration_ms = start_time.elapsed().as_millis();
                            let file_name = output_path
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| "dosya".to_string());

                            println!(
                                "Dönüştürme tamamlandı ({} ms): {}",
                                duration_ms,
                                output_path.display()
                            );

                            self.last_converted_path = Some(output_path);
                            self.radial_ui.show_toast(
                                &format!("Dönüştürme başarılı: {}", file_name),
                                false,
                                true,
                                self.last_click_pos.0,
                                self.last_click_pos.1,
                            );
                        }
                        Err(err) => {
                            eprintln!("Dönüştürme başarısız: {}", err);
                            self.radial_ui.show_toast(
                                &format!("Dönüştürme hatası: {}", err),
                                true,
                                false,
                                self.last_click_pos.0,
                                self.last_click_pos.1,
                            );
                        }
                    }
                }
            }

            AppEvent::OpenFolder { path } => {
                NotificationManager::open_in_explorer(&path);
            }

            AppEvent::CloseMenu => {
                self.state.set_menu_open(false);
                crate::platform::windows::mouse_hook::set_menu_closed();
                self.radial_ui.hide();
                if let Some(ref path) = self.last_converted_path.take() {
                    NotificationManager::open_in_explorer(path);
                }
            }

            AppEvent::ToggleEnabled => {
                let new_state = self.state.toggle_enabled();
                self.mouse_mgr.set_enabled(new_state);
                if let Some(ref tray) = self.tray {
                    tray.set_enabled(new_state);
                }
                let status_msg = if new_state {
                    "Kısayolluk Etkinleştirildi"
                } else {
                    "Kısayolluk Devre Dışı Bırakıldı"
                };
                println!("{}", status_msg);
            }

            AppEvent::OpenSettings => {
                let current_settings = self.state.get_settings();
                self.settings_ui.show(&current_settings);
            }

            AppEvent::SettingsUpdated(new_settings) => {
                let _ = save_settings(&new_settings);
                let _ = set_startup(new_settings.general.start_with_windows);
                self.mouse_mgr
                    .set_double_click_time_ms(new_settings.shortcuts.double_click_time_ms as u64);
                self.mouse_mgr.set_enabled(new_settings.general.enabled);
                self.state.update_settings(|s| *s = new_settings);
                println!("Ayarlar güncellendi ve kaydedildi.");
            }

            AppEvent::ShowToast { message, is_error } => {
                self.radial_ui.show_toast(
                    &message,
                    is_error,
                    false,
                    self.last_click_pos.0,
                    self.last_click_pos.1,
                );
            }

            AppEvent::ExitApp => {
                println!("Kısayolluk kapatılıyor...");
                self.radial_ui.hide();
                self.settings_ui.hide();
                let _ = slint::quit_event_loop();
            }
        }
    }
}
