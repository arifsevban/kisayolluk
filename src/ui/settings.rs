use std::sync::mpsc::Sender;
use slint::ComponentHandle;
use crate::config::settings::AppSettings;
use crate::core::event_manager::AppEvent;

slint::slint! {
    import { Button, CheckBox, Slider, LineEdit } from "std-widgets.slint";

    export component SettingsWindow inherits Window {
        title: "Kısayolluk Ayarları";
        width: 480px;
        height: 560px;
        background: rgb(20, 20, 23);

        in-out property <bool> start_with_windows: false;
        in-out property <bool> app_enabled: true;
        in-out property <bool> show_tray_icon: true;
        in-out property <string> global_shortcut: "Ctrl+Shift+Space";
        in-out property <bool> double_right_click_enabled: true;
        in-out property <int> double_click_time_ms: 400;
        in-out property <int> jpeg_quality: 90;

        callback save_clicked();
        callback cancel_clicked();

        VerticalLayout {
            padding: 24px;
            spacing: 16px;

            // Title Header
            HorizontalLayout {
                spacing: 12px;
                Text {
                    text: "⚙️";
                    font-size: 24px;
                    vertical-alignment: center;
                }
                VerticalLayout {
                    Text {
                        text: "Kısayolluk Ayarları";
                        font-size: 18px;
                        font-weight: 700;
                        color: rgb(244, 244, 245);
                    }
                    Text {
                        text: "Hızlı, hafif ve yerel radyal işlem aracı";
                        font-size: 11px;
                        color: rgb(161, 161, 170);
                    }
                }
            }

            // Section: Genel
            Rectangle {
                background: rgb(32, 32, 36);
                border-radius: 8px;
                border-width: 1px;
                border-color: rgb(63, 63, 70);

                VerticalLayout {
                    padding: 12px;
                    spacing: 8px;

                    Text {
                        text: "GENEL";
                        font-size: 11px;
                        font-weight: 700;
                        color: rgb(212, 212, 216);
                    }

                    CheckBox {
                        text: "Kısayolluk Aracını Etkinleştir";
                        checked <=> root.app_enabled;
                    }

                    CheckBox {
                        text: "Windows Açıldığında Başlat";
                        checked <=> root.start_with_windows;
                    }

                    CheckBox {
                        text: "Sistem Tepsisi Simgesini Göster";
                        checked <=> root.show_tray_icon;
                    }
                }
            }

            // Section: Kısayollar
            Rectangle {
                background: rgb(32, 32, 36);
                border-radius: 8px;
                border-width: 1px;
                border-color: rgb(63, 63, 70);

                VerticalLayout {
                    padding: 12px;
                    spacing: 8px;

                    Text {
                        text: "KISAYOLLAR & ALGILAMA";
                        font-size: 11px;
                        font-weight: 700;
                        color: rgb(212, 212, 216);
                    }

                    HorizontalLayout {
                        spacing: 8px;
                        Text {
                            text: "Global Kısayol:";
                            font-size: 12px;
                            color: rgb(228, 228, 231);
                            vertical-alignment: center;
                        }
                        LineEdit {
                            text <=> root.global_shortcut;
                            width: 160px;
                        }
                    }

                    CheckBox {
                        text: "Çift Sağ Tıklama ile Menüyü Aç";
                        checked <=> root.double_right_click_enabled;
                    }

                    HorizontalLayout {
                        spacing: 8px;
                        Text {
                            text: "Çift Tık Aralığı (" + root.double_click_time_ms + " ms):";
                            font-size: 12px;
                            color: rgb(228, 228, 231);
                            vertical-alignment: center;
                        }
                        Slider {
                            minimum: 200;
                            maximum: 800;
                            value: root.double_click_time_ms;
                            changed(val) => { root.double_click_time_ms = Math.round(val); }
                        }
                    }
                }
            }

            // Section: Görsel Dönüştürme
            Rectangle {
                background: rgb(32, 32, 36);
                border-radius: 8px;
                border-width: 1px;
                border-color: rgb(63, 63, 70);

                VerticalLayout {
                    padding: 12px;
                    spacing: 8px;

                    Text {
                        text: "GÖRSEL DÖNÜŞTÜRME";
                        font-size: 11px;
                        font-weight: 700;
                        color: rgb(212, 212, 216);
                    }

                    HorizontalLayout {
                        spacing: 8px;
                        Text {
                            text: "JPEG Kalitesi (%" + root.jpeg_quality + "):";
                            font-size: 12px;
                            color: rgb(228, 228, 231);
                            vertical-alignment: center;
                        }
                        Slider {
                            minimum: 10;
                            maximum: 100;
                            value: root.jpeg_quality;
                            changed(val) => { root.jpeg_quality = Math.round(val); }
                        }
                    }

                    Text {
                        text: "PNG'den JPG'ye dönüşümde saydam alanlar beyaz arka planla doldurulur.";
                        font-size: 10px;
                        color: rgb(148, 163, 184);
                    }
                }
            }

            // Buttons
            HorizontalLayout {
                alignment: end;
                spacing: 12px;

                Button {
                    text: "İptal";
                    clicked => { root.cancel_clicked(); }
                }

                Button {
                    text: "Kaydet ve Uygula";
                    primary: true;
                    clicked => { root.save_clicked(); }
                }
            }
        }
    }
}

pub struct SettingsController {
    window: SettingsWindow,
}

impl SettingsController {
    pub fn new(event_sender: Sender<AppEvent>) -> Result<Self, slint::PlatformError> {
        let window = SettingsWindow::new()?;

        let win_weak = window.as_weak();
        let sender_clone = event_sender.clone();
        window.on_save_clicked(move || {
            if let Some(win) = win_weak.upgrade() {
                let mut settings = AppSettings::default();
                settings.general.start_with_windows = win.get_start_with_windows();
                settings.general.enabled = win.get_app_enabled();
                settings.general.show_tray_icon = win.get_show_tray_icon();
                settings.shortcuts.global_shortcut = win.get_global_shortcut().to_string();
                settings.shortcuts.double_right_click_enabled = win.get_double_right_click_enabled();
                settings.shortcuts.double_click_time_ms = win.get_double_click_time_ms() as u32;
                settings.conversion.jpeg_quality = win.get_jpeg_quality() as u8;

                let _ = sender_clone.send(AppEvent::SettingsUpdated(settings));
                let _ = win.hide();
            }
        });

        let win_weak = window.as_weak();
        window.on_cancel_clicked(move || {
            if let Some(win) = win_weak.upgrade() {
                let _ = win.hide();
            }
        });

        Ok(Self { window })
    }

    pub fn show(&self, current: &AppSettings) {
        self.window.set_start_with_windows(current.general.start_with_windows);
        self.window.set_app_enabled(current.general.enabled);
        self.window.set_show_tray_icon(current.general.show_tray_icon);
        self.window.set_global_shortcut(current.shortcuts.global_shortcut.clone().into());
        self.window.set_double_right_click_enabled(current.shortcuts.double_right_click_enabled);
        self.window.set_double_click_time_ms(current.shortcuts.double_click_time_ms as i32);
        self.window.set_jpeg_quality(current.conversion.jpeg_quality as i32);

        let _ = self.window.show();
    }

    pub fn hide(&self) {
        let _ = self.window.hide();
    }
}
