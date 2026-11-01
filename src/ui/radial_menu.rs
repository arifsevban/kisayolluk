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

    export struct PetalItem {
        id: string,
        title: string,
        subtitle: string,
        icon: string,
        enabled: bool,
        pos_x: float,
        pos_y: float,
        size: float,
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
        in property <[PetalItem]> petals: [];
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

        callback petal_clicked(string);
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

        // ============================================
        // 1. DAISY FLOWER RADIAL MENU (PAPATYA ÇİÇEĞİ)
        // ============================================
        if !root.is_submenu && !root.has_toast: Rectangle {
            width: 100%;
            height: 100%;

            // 1a. Round Daisy Petals radiating around the cursor
            for petal[index] in root.petals: Rectangle {
                x: petal.pos_x * 1px;
                y: petal.pos_y * 1px;
                width: petal.size * 1px;
                height: petal.size * 1px;
                border-radius: (petal.size / 2) * 1px;
                background: petal.enabled
                    ? (petal-touch.has-hover ? rgb(48, 48, 54) : rgb(28, 28, 32))
                    : rgba(24, 24, 27, 0.72);
                border-width: petal.enabled ? (petal-touch.has-hover ? 1.5px : 1.0px) : 1.0px;
                border-color: petal.enabled
                    ? (petal-touch.has-hover ? rgb(228, 228, 231) : rgba(113, 113, 122, 0.45))
                    : rgba(63, 63, 70, 0.35);
                drop-shadow-blur: 16px;
                drop-shadow-color: rgba(0, 0, 0, 0.75);
                clip: true;
                animate background { duration: 140ms; }
                animate border-color { duration: 140ms; }

                petal-touch := TouchArea {
                    width: 100%;
                    height: 100%;
                    enabled: petal.enabled;
                    clicked => {
                        root.petal_clicked(petal.id);
                    }

                    VerticalLayout {
                        alignment: center;
                        spacing: 2px;
                        padding: 8px;

                        Text {
                            text: petal.icon;
                            font-size: petal.size > 90.0 ? 24px : 20px;
                            horizontal-alignment: center;
                        }
                        Text {
                            text: petal.title;
                            font-size: petal.size > 90.0 ? 13px : 11px;
                            font-weight: 700;
                            color: petal.enabled
                                ? (petal-touch.has-hover ? rgb(250, 250, 250) : rgb(228, 228, 231))
                                : rgb(113, 113, 122);
                            horizontal-alignment: center;
                            overflow: elide;
                        }
                        if petal.subtitle != "": Text {
                            text: petal.subtitle;
                            font-size: petal.size > 90.0 ? 9px : 8px;
                            color: petal.enabled ? rgb(161, 161, 170) : rgb(82, 82, 91);
                            horizontal-alignment: center;
                            overflow: elide;
                        }
                    }
                }
            }

            // 1b. Central Floral Disc ("Papatya Göbeği" - Mouse cursor center)
            Rectangle {
                width: 96px;
                height: 96px;
                x: (parent.width - self.width) / 2;
                y: (parent.height - self.height) / 2;
                border-radius: 48px;
                background: rgb(18, 18, 21);
                border-width: 1.5px;
                border-color: rgb(113, 113, 122);
                drop-shadow-blur: 14px;
                drop-shadow-color: rgba(0, 0, 0, 0.75);
                clip: true;

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
                            color: rgb(244, 244, 245);
                            horizontal-alignment: center;
                        }
                        Text {
                            text: root.file_name;
                            font-size: 9px;
                            color: rgb(212, 212, 216);
                            horizontal-alignment: center;
                            overflow: elide;
                        }
                        Text {
                            text: root.file_format + " • " + root.file_size;
                            font-size: 8px;
                            color: rgb(161, 161, 170);
                            horizontal-alignment: center;
                        }
                    }
                }
            }
        }

        // ============================================
        // 2. CONVERSION SUBMENU / TOAST OVERLAY MODAL
        // ============================================
        if root.is_submenu || root.has_toast: Rectangle {
            width: 330px;
            height: 330px;
            x: (parent.width - self.width) / 2;
            y: (parent.height - self.height) / 2;
            border-radius: 165px;
            background: rgba(24, 24, 27, 0.96);
            border-width: 1.5px;
            border-color: rgba(113, 113, 122, 0.4);
            drop-shadow-blur: 24px;
            drop-shadow-color: rgba(0, 0, 0, 0.85);
            clip: true;

            // Stop click propagation
            TouchArea { }

            // Submenu
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
                        color: rgb(244, 244, 245);
                        horizontal-alignment: center;
                    }
                    Text {
                        text: root.file_name;
                        font-size: 11px;
                        color: rgb(212, 212, 216);
                        horizontal-alignment: center;
                        overflow: elide;
                    }
                    Text {
                        text: "Mevcut: " + root.file_format;
                        font-size: 10px;
                        color: rgb(113, 113, 122);
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
                                background: fmt-touch.has-hover ? rgb(63, 63, 70) : rgb(39, 39, 42);
                                border-width: 1px;
                                border-color: fmt-touch.has-hover ? rgb(161, 161, 170) : rgb(63, 63, 70);
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
                                        color: rgb(161, 161, 170);
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
                        background: back-touch.has-hover ? rgb(63, 63, 70) : rgb(39, 39, 42);
                        border-width: 1px;
                        border-color: rgb(63, 63, 70);

                        HorizontalLayout {
                            alignment: center;
                            spacing: 6px;

                            Text {
                                text: "◀";
                                font-size: 11px;
                                color: rgb(212, 212, 216);
                                vertical-alignment: center;
                            }
                            Text {
                                text: "Geri Dön";
                                font-size: 12px;
                                font-weight: 600;
                                color: rgb(244, 244, 245);
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }

            // Toast overlay
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
                            background: open-touch.has-hover ? rgb(63, 63, 70) : rgb(39, 39, 42);
                            border-width: 1px;
                            border-color: rgb(82, 82, 91);

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
                            background: close-touch.has-hover ? rgb(63, 63, 70) : rgb(39, 39, 42);
                            border-width: 1px;
                            border-color: rgb(63, 63, 70);

                            Text {
                                text: "Kapat";
                                font-size: 11px;
                                color: rgb(212, 212, 216);
                                horizontal-alignment: center;
                                vertical-alignment: center;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PetalDefinition {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon: String,
    pub enabled: bool,
}

/// Dynamically calculates flower petal sizes and symmetrical coordinates for any number of petals N
pub fn create_daisy_petals(definitions: &[PetalDefinition], center_x: f32, center_y: f32) -> Vec<PetalItem> {
    let count = definitions.len().max(3);
    let n = count as f32;

    // Angle step: 360 / N
    let step = 2.0 * std::f32::consts::PI / n;
    let sin_half = (std::f32::consts::PI / n).sin();

    // Geometric formula balancing petal diameter and radius within available window space
    let d = (162.0 / (1.0 + sin_half * 0.95)).clamp(75.0, 115.0);
    let s = (2.0 * d * sin_half * 0.95).clamp(65.0, 106.0);

    definitions
        .iter()
        .enumerate()
        .map(|(i, def)| {
            // First petal is at top (-90 degrees / -PI/2)
            let angle = -std::f32::consts::FRAC_PI_2 + (i as f32) * step;
            let cx = center_x + d * angle.cos();
            let cy = center_y + d * angle.sin();
            let px = (cx - s / 2.0).round();
            let py = (cy - s / 2.0).round();

            PetalItem {
                id: def.id.clone().into(),
                title: def.title.clone().into(),
                subtitle: def.subtitle.clone().into(),
                icon: def.icon.clone().into(),
                enabled: def.enabled,
                pos_x: px,
                pos_y: py,
                size: s.round(),
            }
        })
        .collect()
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
            crate::platform::windows::mouse_hook::set_menu_closed();
            if let Some(win) = win_weak_close.upgrade() {
                let _ = win.hide();
            }
            let _ = sender_clone.send(AppEvent::CloseMenu);
        });

        let win_weak_petal = window.as_weak();
        window.on_petal_clicked(move |petal_id| {
            if petal_id == "convert" {
                if let Some(win) = win_weak_petal.upgrade() {
                    win.set_is_submenu(true);
                }
            }
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

        // 1. Initial 4 Daisy Petals (Expandable to N petals in the future)
        let default_petals = vec![
            PetalDefinition {
                id: "convert".to_string(),
                title: "Dönüştür".to_string(),
                subtitle: "Görsel formatı".to_string(),
                icon: "⚡".to_string(),
                enabled: true,
            },
            PetalDefinition {
                id: "action_2".to_string(),
                title: "İşlem 2".to_string(),
                subtitle: "Yakında".to_string(),
                icon: "📦".to_string(),
                enabled: false,
            },
            PetalDefinition {
                id: "action_3".to_string(),
                title: "İşlem 3".to_string(),
                subtitle: "Yakında".to_string(),
                icon: "🔒".to_string(),
                enabled: false,
            },
            PetalDefinition {
                id: "action_4".to_string(),
                title: "İşlem 4".to_string(),
                subtitle: "Yakında".to_string(),
                icon: "✂️".to_string(),
                enabled: false,
            },
        ];

        let petals = create_daisy_petals(&default_petals, 180.0, 180.0);
        let petal_model = Rc::new(VecModel::from(petals));
        self.window.set_petals(ModelRc::from(petal_model));

        // 2. Target formats for the conversion submenu
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

        // 3. Center on mouse cursor (cursor is at center 180, 180 of the daisy disc)
        let half_w = 180;
        let half_h = 180;
        let pos_x = (x - half_w).max(20);
        let pos_y = (y - half_h).max(20);
        self.window.window().set_position(slint::PhysicalPosition::new(pos_x, pos_y));

        crate::platform::windows::mouse_hook::set_menu_bounds(true, pos_x, pos_y);

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

        crate::platform::windows::mouse_hook::set_menu_bounds(true, pos_x, pos_y);

        let _ = self.window.show();

        let win_weak = self.window.as_weak();
        slint::Timer::single_shot(std::time::Duration::from_millis(3500), move || {
            if let Some(win) = win_weak.upgrade() {
                if win.get_has_toast() {
                    crate::platform::windows::mouse_hook::set_menu_closed();
                    let _ = win.hide();
                }
            }
        });
    }

    pub fn hide(&self) {
        crate::platform::windows::mouse_hook::set_menu_closed();
        let _ = self.window.hide();
    }
}
