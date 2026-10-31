use kisayolluk::config::settings::AppSettings;
use kisayolluk::files::file_type::SupportedImageFormat;
use kisayolluk::platform::windows::keyboard::parse_hotkey_string;

#[test]
fn test_hotkey_parsing() {
    let (_mods, key) = parse_hotkey_string("Ctrl+Shift+Space");
    assert_eq!(key, 0x20); // VK_SPACE

    let (_mods_f1, key_f1) = parse_hotkey_string("Alt+F1");
    assert_eq!(key_f1, 0x70); // VK_F1
}

#[test]
fn test_settings_serialization_roundtrip() {
    let mut settings = AppSettings::default();
    settings.conversion.jpeg_quality = 85;
    settings.shortcuts.double_click_time_ms = 450;

    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.conversion.jpeg_quality, 85);
    assert_eq!(deserialized.shortcuts.double_click_time_ms, 450);
}

#[test]
fn test_supported_format_extensions() {
    assert_eq!(SupportedImageFormat::from_extension("png"), Some(SupportedImageFormat::Png));
    assert_eq!(SupportedImageFormat::from_extension("jpg"), Some(SupportedImageFormat::Jpeg));
    assert_eq!(SupportedImageFormat::from_extension("jpeg"), Some(SupportedImageFormat::Jpeg));
    assert_eq!(SupportedImageFormat::from_extension("webp"), Some(SupportedImageFormat::Webp));
    assert_eq!(SupportedImageFormat::from_extension("bmp"), Some(SupportedImageFormat::Bmp));
    assert_eq!(SupportedImageFormat::from_extension("tiff"), Some(SupportedImageFormat::Tiff));
    assert_eq!(SupportedImageFormat::from_extension("exe"), None);
    assert_eq!(SupportedImageFormat::from_extension("txt"), None);
}

