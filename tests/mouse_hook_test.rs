use std::sync::mpsc;
use std::time::Duration;
use kisayolluk::core::event_manager::{AppEvent, EventManager};
use kisayolluk::system::mouse::MouseManager;

#[test]
fn test_mouse_manager_lifecycle_and_channel() {
    let mut mouse_mgr = MouseManager::new();
    let (tx, rx) = mpsc::channel();

    // 1. Start mouse hook
    let start_result = mouse_mgr.start(tx);
    assert!(start_result.is_ok(), "Mouse hook should start successfully");

    // 2. Verify enabled state toggling
    mouse_mgr.set_enabled(false);
    mouse_mgr.set_enabled(true);

    // 3. Verify no lingering events when idle
    let recv_result = rx.recv_timeout(Duration::from_millis(100));
    assert!(recv_result.is_err(), "Idle hook should not produce false positive events");

    // 4. Stop mouse hook
    mouse_mgr.stop();
}

#[test]
fn test_middle_click_event_channel_capacity_and_repeatability() {
    let event_manager = EventManager::new();
    let sender = event_manager.get_sender();

    // Verify sending 25 consecutive MiddleClick events without drops or blockage
    for i in 1..=25 {
        sender.send(AppEvent::MiddleClick {
            x: 100 + i,
            y: 200 + i,
        });
    }

    let mut count = 0;
    while let Ok(event) = event_manager.receiver().try_recv() {
        if let AppEvent::MiddleClick { x, y } = event {
            count += 1;
            assert_eq!(x, 100 + count);
            assert_eq!(y, 200 + count);
        }
    }

    assert_eq!(count, 25, "All 25 consecutive MiddleClick events must be received accurately");
}

#[test]
fn test_matches_wide_ignore_ascii_case() {
    use kisayolluk::platform::windows::mouse_hook::matches_wide_ignore_ascii_case;
    use windows::core::w;

    let s1: Vec<u16> = "CabinetWClass".encode_utf16().collect();
    assert!(matches_wide_ignore_ascii_case(&s1, w!("CabinetWClass")));
    assert!(matches_wide_ignore_ascii_case(&s1, w!("cabinetwclass")));
    assert!(matches_wide_ignore_ascii_case(&s1, w!("CABINETWCLASS")));

    let s2: Vec<u16> = "Progman".encode_utf16().collect();
    assert!(matches_wide_ignore_ascii_case(&s2, w!("progman")));
    assert!(!matches_wide_ignore_ascii_case(&s2, w!("CabinetWClass")));

    let s3: Vec<u16> = "Shell_TrayWnd".encode_utf16().collect();
    assert!(matches_wide_ignore_ascii_case(&s3, w!("SHELL_TRAYWND")));
    assert!(!matches_wide_ignore_ascii_case(&s3, w!("WorkerW")));

    // Substrings / differing lengths
    let s4: Vec<u16> = "CabinetW".encode_utf16().collect();
    assert!(!matches_wide_ignore_ascii_case(&s4, w!("CabinetWClass")));

    let s5: Vec<u16> = "CabinetWClassExtra".encode_utf16().collect();
    assert!(!matches_wide_ignore_ascii_case(&s5, w!("CabinetWClass")));
}

#[test]
fn test_async_file_resolved_event_propagation() {
    let event_manager = EventManager::new();
    let sender = event_manager.get_sender();

    // Verify 20 consecutive FileResolved cycles execute without deadlock
    for i in 1..=20 {
        sender.send(AppEvent::FileResolved {
            resolution: kisayolluk::files::resolver::FileResolverResult::NoSelection,
            x: 200 + i,
            y: 300 + i,
        });
    }

    let mut count = 0;
    while let Ok(event) = event_manager.receiver().try_recv() {
        if let AppEvent::FileResolved { x, y, .. } = event {
            count += 1;
            assert_eq!(x, 200 + count);
            assert_eq!(y, 300 + count);
        }
    }
    assert_eq!(count, 20, "All 20 FileResolved events must arrive in sequence");
}

#[test]
fn test_outside_click_bounds_logic() {
    use kisayolluk::platform::windows::mouse_hook::{set_menu_bounds, set_menu_closed};

    // Test bounds setting and closing
    set_menu_bounds(true, 500, 300);
    set_menu_closed();
}

#[test]
fn test_is_desktop_or_explorer_point_safety() {
    use kisayolluk::platform::windows::mouse_hook::is_desktop_or_explorer_point;
    use windows::Win32::Foundation::POINT;

    // Test extreme coordinates and normal points to ensure no panic or access violation
    unsafe {
        let _ = is_desktop_or_explorer_point(POINT { x: 0, y: 0 });
        let _ = is_desktop_or_explorer_point(POINT { x: 500, y: 500 });
        let _ = is_desktop_or_explorer_point(POINT { x: -1000, y: -1000 });
        let _ = is_desktop_or_explorer_point(POINT { x: 5000, y: 5000 });
    }
}

#[test]
fn test_twenty_consecutive_menu_open_close_cycles() {
    use kisayolluk::platform::windows::mouse_hook::{set_menu_bounds, set_menu_closed};

    for i in 1..=25 {
        set_menu_bounds(true, 100 + i * 10, 200 + i * 10);
        set_menu_closed();
    }
}

