use std::rc::Rc;
use std::sync::mpsc::Sender;
use slint::{ComponentHandle, ModelRc, VecModel};
use crate::core::event_manager::AppEvent;
use crate::files::file_context::FileContext;
use crate::files::file_type::SupportedImageFormat;

slint::slint! {
    export struct TargetFormatItem {
        id: string,
        name: string,
        extension: string,
    }

    export component RadialMenuWindow inherits Window {
        width: 360px;
        height: 360px;
        no-frame: true;
        background: transparent;
        always-on-top: true;

        in property <string> file_name: "resim.png";
        in property <string> file_format: "PNG";
        in property <string> file_size: "1.2 MB";
        in property <bool> is_submenu: false;
        in property <[TargetFormatItem]> target_formats: [
            { id: "jpg", name: "JPG", extension: "jpg" },
            { id: "webp", name: "WEBP", extension: "webp" },
            { id: "bmp", name: "BMP", extension: "bmp" },
            { id: "tiff", name: "TIFF", extension: "tiff" },
        ];
        in property <string> toast_message: "";
        in property <bool> has_toast: false;
        in property <bool> toast_is_error: false;
        in property <bool> can_open_folder: false;

        callback convert_clicked();
        callback format_selected(string);
        callback back_clicked();
        callback close_requested();
        callback open_folder_clicked();

        forward-focus: key-handler;

        key-handler := FocusScope {
            key-pressed(event) => {
                if (event.text == Key.Escape) {
                    root.close_requested();
                    return accept;
                }
                return reject;
            }
        }

        // Click outside closes the menu
        TouchArea {
            width: 100%;
            height: 100%;
            clicked => {
                root.close_requested();
            }
        }

        // Circular Menu Container
        Rectangle {
            width: 330px;
            height: 330px;
            x: (parent.width - self.width) / 2;
            y: (parent.height - self.height) / 2;
            border-radius: 165px;
            background: rgba(15, 23, 42, 0.94);
            border-width: 1.5px;
            border-color: rgba(59, 130, 246, 0.4);
            drop-shadow-blur: 24px;
            drop-shadow-color: rgba(0, 0, 0, 0.8);
            clip: true;

            // Stop click propagation
            TouchArea { }

            // ============================================
            // 1. MAIN 4-QUADRANT RADIAL MENU
            // ============================================
            if !root.is_submenu && !root.has_toast: Rectangle {
                width: 100%;
                height: 100%;

                // TOP SLICE: "Dönüştür" (ACTIVE)
                top-touch := TouchArea {
                    x: 20px;
                    y: 10px;
                    width: 290px;
                    height: 115px;
                    clicked => { root.convert_clicked(); }

                    Rectangle {
                        width: 100%;
                        height: 100%;
                        background: top-touch.has-hover ? rgba(59, 130, 246, 0.25) : transparent;
                        border-radius: 18px;
                        animate background { duration: 150ms; }

                        VerticalLayout {
                            alignment: center;
                            spacing: 3px;
                            padding-top: 12px;

                            Text {
                                text: "⚡";
                                font-size: 24px;
                                horizontal-alignment: center;
                            }
                            Text {
                                text: "Dönüştür";
                                font-size: 15px;
                                font-weight: 700;
                                color: top-touch.has-hover ? rgb(96, 165, 250) : white;
                                horizontal-alignment: center;
                            }
                            Text {
                                text: "Görsel formatını değiştir";
                                font-size: 10px;
                                color: rgb(148, 163, 184);
                                horizontal-alignment: center;
                            }
                        }
                    }
                }

                // RIGHT SLICE: "İşlem 2" (DISABLED)
                Rectangle {
                    x: 205px;
                    y: 105px;
                    width: 115px;
                    height: 115px;
                    opacity: 0.35;

                    VerticalLayout {
                        alignment: center;
                        spacing: 3px;

                        Text {
                            text: "📦";
                            font-size: 20px;
                            horizontal-alignment: center;
                        }
                        Text {
                            text: "İşlem 2";
                            font-size: 12px;
                            font-weight: 600;
                            color: rgb(203, 213, 225);
                            horizontal-alignment: center;
                        }
                        Text {
                            text: "Yakında";
                            font-size: 9px;
                            color: rgb(100, 116, 139);
                            horizontal-alignment: center;
                        }
                    }
                }

                // BOTTOM SLICE: "İşlem 3" (DISABLED)
                Rectangle {
                    x: 20px;
                    y: 205px;
                    width: 290px;
                    height: 115px;
                    opacity: 0.35;

                    VerticalLayout {
                        alignment: center;
                        spacing: 3px;
                        padding-bottom: 12px;

                        Text {
                            text: "İşlem 3";
                            font-size: 12px;
                            font-weight: 600;
                            color: rgb(203, 213, 225);
                            horizontal-alignment: center;
                        }
                        Text {
                            text: "🔒 Yakında";
                            font-size: 9px;
                            color: rgb(100, 116, 139);
                            horizontal-alignment: center;
                        }
                    }
                }

                // LEFT SLICE: "İşlem 4" (DISABLED)
                Rectangle {
                    x: 10px;
                    y: 105px;
                    width: 115px;
                    height: 115px;
                    opacity: 0.35;

                    VerticalLayout {
                        alignment: center;
                        spacing: 3px;

                        Text {
                            text: "✂️";
                            font-size: 20px;
                            horizontal-alignment: center;
                        }
                        Text {
                            text: "İşlem 4";
                            font-size: 12px;
                            font-weight: 600;
                            color: rgb(203, 213, 225);
                            horizontal-alignment: center;
                        }
                        Text {
                            text: "Yakında";
                            font-size: 9px;
                            color: rgb(100, 116, 139);
                            horizontal-alignment: center;
                        }
                    }
                }
            }

            // ============================================
            // 2. CONVERSION SUBMENU (DYNAMIC TARGET FORMATS)
            // ============================================
            if root.is_submenu && !root.has_toast: Rectangle {
                width: 100%;
                height: 100%;

                VerticalLayout {
                    x: 25px;
                    y: 18px;
                    width: 280px;
                    spacing: 2px;
                    alignment: start;

                    Text {
                        text: "Hedef Format Seçin";
                        font-size: 13px;
                        font-weight: 700;
                        color: rgb(147, 197, 253);
                        horizontal-alignment: center;
                    }
                    Text {
                        text: root.file_name;
                        font-size: 11px;
                        color: rgb(203, 213, 225);
                        horizontal-alignment: center;
                        overflow: elide;
                    }
                    Text {
                        text: "Mevcut: " + root.file_format;
                        font-size: 10px;
                        color: rgb(100, 116, 139);
                        horizontal-alignment: center;
                    }
                }

                HorizontalLayout {
                    x: 20px;
                    y: 90px;
                    width: 290px;
                    height: 135px;
                    spacing: 6px;
                    alignment: center;

                    for item in root.target_formats: Rectangle {
                        width: 64px;
                        height: 115px;

                        fmt-touch := TouchArea {
                            clicked => {
                                root.format_selected(item.id);
                            }

                            Rectangle {
                                width: 100%;
                                height: 100%;
                                border-radius: 12px;
                                background: fmt-touch.has-hover ? rgb(37, 99, 235) : rgb(30, 41, 59);
                                border-width: 1px;
                                border-color: fmt-touch.has-hover ? rgb(96, 165, 250) : rgb(51, 65, 85);
                                animate background { duration: 120ms; }

                                VerticalLayout {
                                    alignment: center;
                                    spacing: 4px;

                                    Text {
                                        text: "📄";
                                        font-size: 18px;
                                        horizontal-alignment: center;
                                    }
                                    Text {
                                        text: item.name;
                                        font-size: 13px;
                                        font-weight: 700;
                                        color: white;
                                        horizontal-alignment: center;
                                    }
                                    Text {
                                        text: "." + item.extension;
                                        font-size: 9px;
                                        color: rgb(148, 163, 184);
                                        horizontal-alignment: center;
                                    }
                                }
                            }
                        }
                    }
                }

                // Back Button
                back-touch := TouchArea {
                    x: 95px;
                    y: 245px;
                    width: 140px;
                    height: 38px;
                    clicked => { root.back_clicked(); }

                    Rectangle {
                        width: 100%;
                        height: 100%;
                        border-radius: 19px;
                        background: back-touch.has-hover ? rgb(55, 65, 81) : rgb(31, 41, 55);
                        border-width: 1px;
                        border-color: rgb(75, 85, 99);

                        HorizontalLayout {
                            alignment: center;
                            spacing: 6px;

                            Text {
                                text: "◀";
                                font-size: 11px;
                                color: rgb(203, 213, 225);
                                vertical-alignment: center;
                            }
                            Text {
                                text: "Geri Dön";
                                font-size: 12px;
                                font-weight: 600;
                                color: rgb(226, 232, 240);
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }

            // ============================================
            // 3. TOAST OVERLAY
            // ============================================
            if root.has_toast: Rectangle {
                width: 100%;
                height: 100%;

                VerticalLayout {
                    alignment: center;
                    spacing: 10px;
                    padding-left: 28px;
                    padding-right: 28px;

                    Text {
                        text: root.toast_is_error ? "⚠️" : "✅";
                        font-size: 32px;
                        horizontal-alignment: center;
                    }
                    Text {
                        text: root.toast_message;
                        font-size: 12px;
                        font-weight: 600;
                        color: root.toast_is_error ? rgb(248, 113, 113) : rgb(74, 222, 128);
                        horizontal-alignment: center;
                        wrap: word-wrap;
                    }

                    if root.can_open_folder: open-touch := TouchArea {
                        height: 36px;
                        clicked => { root.open_folder_clicked(); }

                        Rectangle {
                            border-radius: 18px;
                            background: open-touch.has-hover ? rgb(37, 99, 235) : rgb(29, 78, 216);

                            HorizontalLayout {
                                alignment: center;
                                spacing: 6px;

                                Text {
                                    text: "📁";
                                    font-size: 12px;
                                    vertical-alignment: center;
                                }
                                Text {
                                    text: "Klasörü Aç";
                                    font-size: 12px;
                                    font-weight: 600;
                                    color: white;
                                    vertical-alignment: center;
                                }
                            }
                        }
                    }

                    close-touch := TouchArea {
                        height: 32px;
                        clicked => { root.close_requested(); }

                        Rectangle {
                            border-radius: 16px;
                            background: close-touch.has-hover ? rgb(55, 65, 81) : rgb(31, 41, 55);

                            Text {
                                text: "Kapat";
                                font-size: 11px;
                                color: rgb(148, 163, 184);
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }

            // ============================================
            // CENTER DISC
            // ============================================
            if !root.is_submenu && !root.has_toast: Rectangle {
                width: 105px;
                height: 105px;
                x: (parent.width - self.width) / 2;
                y: (parent.height - self.height) / 2;
                border-radius: 53px;
                background: rgb(17, 24, 39);
                border-width: 2px;
                border-color: rgb(37, 99, 235);
                drop-shadow-blur: 10px;
                drop-shadow-color: rgba(0, 0, 0, 0.5);

                TouchArea {
                    clicked => { root.close_requested(); }

                    VerticalLayout {
                        alignment: center;
                        spacing: 2px;
                        padding-left: 6px;
                        padding-right: 6px;

                        Text {
                            text: "Kısayolluk";
                            font-size: 10px;
                            font-weight: 700;
                            color: rgb(96, 165, 250);
                            horizontal-alignment: center;
                        }
                        Text {
                            text: root.file_name;
                            font-size: 9px;
                            color: rgb(226, 232, 240);
                            horizontal-alignment: center;
                            overflow: elide;
                        }
                        Text {
                            text: root.file_format + " • " + root.file_size;
                            font-size: 8px;
                            color: rgb(148, 163, 184);
                            horizontal-alignment: center;
                        }
                    }
                }
            }
        }
    }
}

pub struct RadialMenuController {
    window: RadialMenuWindow,
}

impl RadialMenuController {
    pub fn new(event_sender: Sender<AppEvent>) -> Result<Self, slint::PlatformError> {
        let window = RadialMenuWindow::new()?;

        let sender_clone = event_sender.clone();
        let win_weak_close = window.as_weak();
        window.on_close_requested(move || {
            if let Some(win) = win_weak_close.upgrade() {
                let _ = win.hide();
            }
            let _ = sender_clone.send(AppEvent::CloseMenu);
        });

        let win_weak = window.as_weak();
        window.on_convert_clicked(move || {
            if let Some(win) = win_weak.upgrade() {
                win.set_is_submenu(true);
            }
        });

        let win_weak = window.as_weak();
        window.on_back_clicked(move || {
            if let Some(win) = win_weak.upgrade() {
                win.set_is_submenu(false);
            }
        });

        let sender_clone = event_sender.clone();
        window.on_format_selected(move |fmt_str| {
            if let Some(fmt) = match fmt_str.as_str() {
                "png" => Some(SupportedImageFormat::Png),
                "jpg" | "jpeg" => Some(SupportedImageFormat::Jpeg),
                "webp" => Some(SupportedImageFormat::Webp),
                "bmp" => Some(SupportedImageFormat::Bmp),
                "tiff" => Some(SupportedImageFormat::Tiff),
                _ => None,
            } {
                let _ = sender_clone.send(AppEvent::ConvertRequested { target_format: fmt });
            }
        });

        let sender_clone = event_sender.clone();
        window.on_open_folder_clicked(move || {
            let _ = sender_clone.send(AppEvent::CloseMenu);
        });

        Ok(Self {
            window,
        })
    }

    pub fn show_menu(&self, file: &FileContext, x: i32, y: i32) {
        self.window.set_file_name(file.file_name.clone().into());
        let fmt_display = file
            .format
            .map(|f| f.display_name())
            .unwrap_or_else(|| &file.extension)
            .to_uppercase();
        self.window.set_file_format(fmt_display.into());

        let size_kb = file.size_bytes as f64 / 1024.0;
        let size_str = if size_kb > 1024.0 {
            format!("{:.1} MB", size_kb / 1024.0)
        } else {
            format!("{:.0} KB", size_kb)
        };
        self.window.set_file_size(size_str.into());

        let targets = SupportedImageFormat::targets_for(file.format);
        let items: Vec<TargetFormatItem> = targets
            .into_iter()
            .map(|f| TargetFormatItem {
                id: f.default_extension().into(),
                name: f.display_name().into(),
                extension: f.default_extension().into(),
            })
            .collect();

        let model = Rc::new(VecModel::from(items));
        self.window.set_target_formats(ModelRc::from(model));

        self.window.set_is_submenu(false);
        self.window.set_has_toast(false);

        let half_w = 180;
        let half_h = 180;
        let pos_x = (x - half_w).max(20);
        let pos_y = (y - half_h).max(20);
        self.window.window().set_position(slint::PhysicalPosition::new(pos_x, pos_y));

        let _ = self.window.show();
    }

    pub fn show_toast(&self, message: &str, is_error: bool, can_open_folder: bool, x: i32, y: i32) {
        self.window.set_toast_message(message.into());
        self.window.set_toast_is_error(is_error);
        self.window.set_can_open_folder(can_open_folder);
        self.window.set_has_toast(true);

        let half_w = 180;
        let half_h = 180;
        let pos_x = (x - half_w).max(20);
        let pos_y = (y - half_h).max(20);
        self.window.window().set_position(slint::PhysicalPosition::new(pos_x, pos_y));

        let _ = self.window.show();

        let win_weak = self.window.as_weak();
        slint::Timer::single_shot(std::time::Duration::from_millis(3500), move || {
            if let Some(win) = win_weak.upgrade() {
                if win.get_has_toast() {
                    let _ = win.hide();
                }
            }
        });
    }

    pub fn hide(&self) {
        let _ = self.window.hide();
    }
}
