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
        pos_x: float,
        pos_y: float,
        size_w: float,
        size_h: float,
        path_data: string,
    }

    export struct PetalItem {
        id: string,
        title: string,
        subtitle: string,
        icon: string,
        arrow: string,
        enabled: bool,
        pos_x: float,
        pos_y: float,
        size: float,
        path_data: string,
    }

    export component RadialMenuWindow inherits Window {
        title: "Kısayolluk";
        width: 440px;
        height: 440px;
        no-frame: true;
        background: transparent;
        always-on-top: true;

        in property <string> file_name: "resim.png";
        in property <string> file_format: "PNG";
        in property <string> file_size: "1.2 MB";
        in property <bool> is_submenu: false;
        in property <[PetalItem]> petals: [];
        in property <[TargetFormatItem]> target_formats: [];
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

        pure function get_sector(mx: float, my: float, count: int) -> int {
            if (count <= 0) {
                return -1;
            }
            let dx = mx - 220.0;
            let dy = my - 220.0;
            let dist = Math.sqrt(dx * dx + dy * dy);
            if (dist < 52.0 || dist > 138.0) {
                return -1;
            }
            let a = Math.atan2(dy, dx) / 1deg; // -180 to 180 deg
            let sector_span = 360.0 / count;
            // Shift so sector 0 is centered at top (-90 degrees)
            let shifted = a + 90.0 + (sector_span / 2.0);
            let positive_angle = shifted < 0.0 ? shifted + 360.0 : (shifted >= 360.0 ? shifted - 360.0 : shifted);
            return Math.floor(positive_angle / sector_span);
        }

        pure function get_sat_sector(mx: float, my: float, count: int) -> int {
            if (count <= 0) {
                return -1;
            }
            let dx = mx - 220.0;
            let dy = my - 220.0;
            let dist = Math.sqrt(dx * dx + dy * dy);
            if (dist < 139.0 || dist > 212.0) {
                return -1;
            }
            let a = Math.atan2(dy, dx) / 1deg;
            // Satellite arc spans from -145 to -35 deg
            if (a < -146.0 || a > -34.0) {
                return -1;
            }
            let step = 110.0 / count;
            let offset = a - (-145.0);
            let idx = Math.floor(offset / step);
            return idx < 0 ? 0 : (idx >= count ? count - 1 : idx);
        }

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

        // ============================================
        // 1. BLENDER PIE MENU (ANNULAR RING + OUTER SATELLITE TIER)
        // ============================================
        if !root.has_toast: menu-box := Rectangle {
            width: 100%;
            height: 100%;

            property <int> hovered_sector: touch.has-hover ? root.get_sector(touch.mouse-x / 1px, touch.mouse-y / 1px, root.petals.length) : -1;
            property <int> hovered_sat: (root.is_submenu && touch.has-hover) ? root.get_sat_sector(touch.mouse-x / 1px, touch.mouse-y / 1px, root.target_formats.length) : -1;

            // 1a. Curved Annular Sector Wedges (Primary Ring)
            for petal[index] in root.petals: Path {
                width: 440px;
                height: 440px;
                x: 0px;
                y: 0px;
                viewbox-x: 0;
                viewbox-y: 0;
                viewbox-width: 440;
                viewbox-height: 440;
                commands: petal.path_data;
                fill: (root.is_submenu && index == 0)
                    ? rgb(228, 126, 32)
                    : ((menu-box.hovered_sector == index && petal.enabled)
                        ? rgb(228, 126, 32)
                        : (petal.enabled ? rgba(42, 45, 53, 0.92) : rgba(26, 28, 33, 0.82)));
                stroke: (root.is_submenu && index == 0)
                    ? rgb(250, 164, 70)
                    : ((menu-box.hovered_sector == index && petal.enabled)
                        ? rgb(250, 164, 70)
                        : (petal.enabled ? rgba(255, 255, 255, 0.12) : rgba(255, 255, 255, 0.04)));
                stroke-width: ((menu-box.hovered_sector == index && petal.enabled) || (root.is_submenu && index == 0)) ? 1.6px : 1.0px;
            }

            // 1b. Slice Content: Centered Icon, Label, and Outward Arrow
            for petal[index] in root.petals: Rectangle {
                x: petal.pos_x * 1px;
                y: petal.pos_y * 1px;
                width: petal.size * 1px;
                height: petal.size * 1px;
                background: transparent;

                VerticalLayout {
                    alignment: center;
                    spacing: 2px;

                    if petal.arrow != "": Text {
                        text: petal.arrow;
                        font-size: 8px;
                        color: ((menu-box.hovered_sector == index && petal.enabled) || (root.is_submenu && index == 0))
                            ? white
                            : (petal.enabled ? rgba(255, 255, 255, 0.8) : rgb(95, 100, 110));
                        horizontal-alignment: center;
                    }
                    Text {
                        text: petal.icon;
                        font-size: 20px;
                        horizontal-alignment: center;
                    }
                    Text {
                        text: petal.title;
                        font-size: 11px;
                        font-weight: 700;
                        color: ((menu-box.hovered_sector == index && petal.enabled) || (root.is_submenu && index == 0))
                            ? white
                            : (petal.enabled ? rgb(240, 240, 242) : rgb(113, 113, 122));
                        horizontal-alignment: center;
                        overflow: elide;
                    }
                    if petal.subtitle != "": Text {
                        text: petal.subtitle;
                        font-size: 8px;
                        color: ((menu-box.hovered_sector == index && petal.enabled) || (root.is_submenu && index == 0))
                            ? rgb(254, 243, 199)
                            : (petal.enabled ? rgb(161, 161, 170) : rgb(82, 82, 91));
                        horizontal-alignment: center;
                        overflow: elide;
                    }
                }
            }

            // 1c. Center Floating Capsule in Hollow Opening (Cursor is at center 220, 220)
            Rectangle {
                width: 90px;
                height: 22px;
                x: (parent.width - self.width) / 2;
                y: (parent.height - self.height) / 2;
                border-radius: 11px;
                background: rgba(22, 24, 30, 0.82);
                border-width: 1px;
                border-color: rgba(255, 255, 255, 0.12);
                drop-shadow-blur: 8px;
                drop-shadow-color: rgba(0, 0, 0, 0.5);

                HorizontalLayout {
                    alignment: center;
                    spacing: 4px;
                    padding-left: 6px;
                    padding-right: 6px;

                    Text {
                        text: "📄";
                        font-size: 9px;
                        vertical-alignment: center;
                    }
                    Text {
                        text: root.file_name;
                        font-size: 9px;
                        font-weight: 600;
                        color: rgb(228, 231, 236);
                        vertical-alignment: center;
                        overflow: elide;
                    }
                }
            }

            // ============================================
            // 1d. SATELLITE TIER (OUTER ARC SLICES ABOVE CONVERT)
            // ============================================
            if root.is_submenu: Rectangle {
                width: 100%;
                height: 100%;

                // Satellite Arc Annular Wedges
                for fmt[index] in root.target_formats: Path {
                    width: 440px;
                    height: 440px;
                    x: 0px;
                    y: 0px;
                    viewbox-x: 0;
                    viewbox-y: 0;
                    viewbox-width: 440;
                    viewbox-height: 440;
                    commands: fmt.path_data;
                    fill: (menu-box.hovered_sat == index)
                        ? rgb(228, 126, 32)
                        : rgba(42, 45, 53, 0.95);
                    stroke: (menu-box.hovered_sat == index)
                        ? rgb(250, 164, 70)
                        : rgba(255, 255, 255, 0.14);
                    stroke-width: (menu-box.hovered_sat == index) ? 1.6px : 1.0px;
                }

                // Satellite Arc Labels
                for fmt[index] in root.target_formats: Rectangle {
                    x: fmt.pos_x * 1px;
                    y: fmt.pos_y * 1px;
                    width: fmt.size_w * 1px;
                    height: fmt.size_h * 1px;
                    background: transparent;

                    VerticalLayout {
                        alignment: center;
                        spacing: 1px;

                        Text {
                            text: "📄";
                            font-size: 13px;
                            horizontal-alignment: center;
                        }
                        Text {
                            text: fmt.name;
                            font-size: 11px;
                            font-weight: 700;
                            color: (menu-box.hovered_sat == index) ? white : rgb(240, 240, 244);
                            horizontal-alignment: center;
                        }
                    }
                }
            }

            // 1e. Master Interactive Touch Area
            touch := TouchArea {
                width: 100%;
                height: 100%;
                mouse-cursor: (((menu-box.hovered_sector >= 0 && menu-box.hovered_sector < root.petals.length && root.petals[menu-box.hovered_sector].enabled)) || (menu-box.hovered_sat >= 0)) ? pointer : default;

                clicked => {
                    let dx = (touch.mouse-x - 220px) / 1px;
                    let dy = (touch.mouse-y - 220px) / 1px;
                    let dist = Math.sqrt(dx * dx + dy * dy);

                    // If satellite tier is active and clicked on a satellite slice:
                    if (root.is_submenu && dist >= 139.0 && dist <= 212.0) {
                        let sat = root.get_sat_sector(touch.mouse-x / 1px, touch.mouse-y / 1px, root.target_formats.length);
                        if (sat >= 0 && sat < root.target_formats.length) {
                            root.format_selected(root.target_formats[sat].id);
                            return;
                        }
                    }

                    // If clicked on primary ring:
                    if (dist >= 52.0 && dist <= 138.0) {
                        let s = root.get_sector(touch.mouse-x / 1px, touch.mouse-y / 1px, root.petals.length);
                        if (s >= 0 && s < root.petals.length) {
                            if (root.petals[s].enabled) {
                                root.petal_clicked(root.petals[s].id);
                                return;
                            }
                        }
                    }

                    // Otherwise, clicked outside or in empty center -> close
                    root.close_requested();
                }
            }
        }

        // ============================================
        // 2. TOAST OVERLAY MODAL
        // ============================================
        if root.has_toast: Rectangle {
            width: 300px;
            height: 300px;
            x: (parent.width - self.width) / 2;
            y: (parent.height - self.height) / 2;
            border-radius: 150px;
            background: rgba(22, 24, 28, 0.96);
            border-width: 1.5px;
            border-color: rgba(113, 113, 122, 0.4);
            drop-shadow-blur: 24px;
            drop-shadow-color: rgba(0, 0, 0, 0.85);
            clip: true;

            // Stop click propagation
            TouchArea { }

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

#[derive(Debug, Clone)]
pub struct PetalDefinition {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon: String,
    pub enabled: bool,
}

/// Generates an SVG path for an annular sector (curved ring wedge) with rounded corners.
pub fn generate_annular_wedge_path(
    cx: f32,
    cy: f32,
    r_in: f32,
    r_out: f32,
    angle_start_rad: f32,
    angle_end_rad: f32,
    corner_radius: f32,
) -> String {
    let r_c = corner_radius.min((r_out - r_in) * 0.25);
    let delta_out = r_c / r_out;
    let delta_in = r_c / r_in;

    let (sin_s, cos_s) = angle_start_rad.sin_cos();
    let (sin_e, cos_e) = angle_end_rad.sin_cos();

    // Sharp vertices used as quadratic bezier control points for rounded corners
    let v1x = cx + r_out * cos_s;
    let v1y = cy + r_out * sin_s;

    let v2x = cx + r_out * cos_e;
    let v2y = cy + r_out * sin_e;

    let v3x = cx + r_in * cos_e;
    let v3y = cy + r_in * sin_e;

    let v4x = cx + r_in * cos_s;
    let v4y = cy + r_in * sin_s;

    // Straight edge endpoints
    let e1x = cx + (r_out - r_c) * cos_s;
    let e1y = cy + (r_out - r_c) * sin_s;

    let e2x = cx + (r_out - r_c) * cos_e;
    let e2y = cy + (r_out - r_c) * sin_e;

    let e3x = cx + (r_in + r_c) * cos_e;
    let e3y = cy + (r_in + r_c) * sin_e;

    let e4x = cx + (r_in + r_c) * cos_s;
    let e4y = cy + (r_in + r_c) * sin_s;

    // Arc endpoints
    let (sin_s_out, cos_s_out) = (angle_start_rad + delta_out).sin_cos();
    let a1x = cx + r_out * cos_s_out;
    let a1y = cy + r_out * sin_s_out;

    let (sin_e_out, cos_e_out) = (angle_end_rad - delta_out).sin_cos();
    let a2x = cx + r_out * cos_e_out;
    let a2y = cy + r_out * sin_e_out;

    let (sin_e_in, cos_e_in) = (angle_end_rad - delta_in).sin_cos();
    let a3x = cx + r_in * cos_e_in;
    let a3y = cy + r_in * sin_e_in;

    let (sin_s_in, cos_s_in) = (angle_start_rad + delta_in).sin_cos();
    let a4x = cx + r_in * cos_s_in;
    let a4y = cy + r_in * sin_s_in;

    format!(
        "M {:.1} {:.1} L {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1} A {:.1} {:.1} 0 0 1 {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1} L {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1} A {:.1} {:.1} 0 0 0 {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1} Z",
        e4x, e4y,
        e1x, e1y,
        v1x, v1y, a1x, a1y,
        r_out, r_out, a2x, a2y,
        v2x, v2y, e2x, e2y,
        e3x, e3y,
        v3x, v3y, a3x, a3y,
        r_in, r_in, a4x, a4y,
        v4x, v4y, e4x, e4y,
    )
}

/// Dynamically calculates annular wedge sectors and coordinates for any number of petals N
pub fn create_daisy_petals(definitions: &[PetalDefinition], center_x: f32, center_y: f32) -> Vec<PetalItem> {
    let count = definitions.len().max(3);
    let n = count as f32;

    let step = 2.0 * std::f32::consts::PI / n;
    // Clean, distinct radial gap between sectors (4.0 degrees for clear separation)
    let gap = (4.0_f32).to_radians();

    // Compact, balanced ring: r_in = 55.0 (diameter 110px), r_out = 135.0 (diameter 270px)
    let r_in = 55.0_f32;
    let r_out = 135.0_f32;
    let r_mid = (r_in + r_out) / 2.0; // 95.0
    let content_size = 64.0_f32;

    definitions
        .iter()
        .enumerate()
        .map(|(i, def)| {
            // First sector is centered at top (-90 degrees / -PI/2)
            let mid_angle = -std::f32::consts::FRAC_PI_2 + (i as f32) * step;
            let start_angle = mid_angle - (step - gap) / 2.0;
            let end_angle = mid_angle + (step - gap) / 2.0;

            let path_data = generate_annular_wedge_path(
                center_x, center_y, r_in, r_out, start_angle, end_angle, 4.0,
            );

            let cx = center_x + r_mid * mid_angle.cos();
            let cy = center_y + r_mid * mid_angle.sin();
            let px = (cx - content_size / 2.0).round();
            let py = (cy - content_size / 2.0).round();

            let arrow = match i {
                0 => "▲",
                1 => "▶",
                2 => "▼",
                3 => "◀",
                _ => "",
            };

            PetalItem {
                id: def.id.clone().into(),
                title: def.title.clone().into(),
                subtitle: def.subtitle.clone().into(),
                icon: def.icon.clone().into(),
                arrow: arrow.into(),
                enabled: def.enabled,
                pos_x: px,
                pos_y: py,
                size: content_size,
                path_data: path_data.into(),
            }
        })
        .collect()
}

/// Dynamically calculates satellite arc pie slices for any number of sub-items
pub fn create_satellite_petals(
    formats: &[SupportedImageFormat],
    cx: f32,
    cy: f32,
    parent_mid_angle: f32,
) -> Vec<TargetFormatItem> {
    let count = formats.len().max(1);
    let k = count as f32;

    // Arc span for the satellite tier: 110 degrees centered at parent_mid_angle
    let total_span = (110.0_f32).to_radians();
    let step = total_span / k;
    let gap = (3.5_f32).to_radians();

    let r_in = 140.0_f32;
    let r_out = 205.0_f32;
    let r_mid = (r_in + r_out) / 2.0; // 172.5
    let box_w = 60.0_f32;
    let box_h = 44.0_f32;
    let corner_radius = 4.0_f32;

    let start_base = parent_mid_angle - total_span / 2.0;

    formats
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let mid_angle = start_base + (i as f32 + 0.5) * step;
            let start_angle = mid_angle - (step - gap) / 2.0;
            let end_angle = mid_angle + (step - gap) / 2.0;

            let path_data = generate_annular_wedge_path(
                cx, cy, r_in, r_out, start_angle, end_angle, corner_radius,
            );

            let label_cx = cx + r_mid * mid_angle.cos();
            let label_cy = cy + r_mid * mid_angle.sin();
            let px = (label_cx - box_w / 2.0).round();
            let py = (label_cy - box_h / 2.0).round();

            TargetFormatItem {
                id: f.default_extension().into(),
                name: f.display_name().into(),
                extension: f.default_extension().into(),
                pos_x: px,
                pos_y: py,
                size_w: box_w,
                size_h: box_h,
                path_data: path_data.into(),
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
                    let cur = win.get_is_submenu();
                    win.set_is_submenu(!cur);
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
        let win_weak_fmt = window.as_weak();
        window.on_format_selected(move |fmt_str| {
            crate::platform::windows::mouse_hook::set_menu_closed();
            if let Some(win) = win_weak_fmt.upgrade() {
                let _ = win.hide();
            }
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

        let petals = create_daisy_petals(&default_petals, 220.0, 220.0);
        let petal_model = Rc::new(VecModel::from(petals));
        self.window.set_petals(ModelRc::from(petal_model));

        // 2. Target formats for the conversion satellite arc (above "Dönüştür", which is at -PI/2)
        let targets = SupportedImageFormat::targets_for(file.format);
        let sat_items = create_satellite_petals(&targets, 220.0, 220.0, -std::f32::consts::FRAC_PI_2);
        let model = Rc::new(VecModel::from(sat_items));
        self.window.set_target_formats(ModelRc::from(model));

        self.window.set_is_submenu(false);
        self.window.set_has_toast(false);

        // 3. Center on mouse cursor (cursor is at center 220, 220 of the 440x440 window)
        let half_w = 220;
        let half_h = 220;
        let pos_x = (x - half_w).max(0);
        let pos_y = (y - half_h).max(0);
        self.window.window().set_position(slint::PhysicalPosition::new(pos_x, pos_y));

        crate::platform::windows::mouse_hook::set_menu_bounds(true, pos_x, pos_y);

        let _ = self.window.show();
    }

    pub fn show_toast(&self, message: &str, is_error: bool, can_open_folder: bool, x: i32, y: i32) {
        self.window.set_toast_message(message.into());
        self.window.set_toast_is_error(is_error);
        self.window.set_can_open_folder(can_open_folder);
        self.window.set_has_toast(true);

        let half_w = 220;
        let half_h = 220;
        let pos_x = (x - half_w).max(0);
        let pos_y = (y - half_h).max(0);
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
