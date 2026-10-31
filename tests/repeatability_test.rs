use kisayolluk::core::event_manager::EventManager;
use kisayolluk::files::file_context::FileContext;
use kisayolluk::files::file_type::SupportedImageFormat;
use kisayolluk::ui::radial_menu::RadialMenuController;

#[test]
fn test_menu_and_toast_repeatability() {
    let event_manager = EventManager::new();
    let sender = event_manager.get_raw_sender();
    let controller = RadialMenuController::new(sender).unwrap();

    let dummy_file = FileContext {
        path: std::path::PathBuf::from(r"C:\test\sample.png"),
        file_name: "sample.png".to_string(),
        extension: "png".to_string(),
        format: Some(SupportedImageFormat::Png),
        size_bytes: 1024 * 50,
        is_accessible: true,
        is_directory: false,
    };

    // 1. Open and close menu 20 times sequentially
    for i in 1..=20 {
        controller.show_menu(&dummy_file, 500, 400);
        controller.hide();
        println!("Menu open/close cycle {} successful", i);
    }

    // 2. Open and close toast 10 times sequentially
    for i in 1..=10 {
        controller.show_toast(
            &format!("Test bildirim {}", i),
            i % 2 == 0,
            false,
            500,
            400,
        );
        controller.hide();
        println!("Toast open/close cycle {} successful", i);
    }
}
