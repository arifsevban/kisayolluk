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
fn test_ctrl_right_click_event_channel_capacity_and_repeatability() {
    let event_manager = EventManager::new();
    let sender = event_manager.get_sender();

    // Verify sending 25 consecutive CtrlRightClick events without drops or blockage
    for i in 1..=25 {
        sender.send(AppEvent::CtrlRightClick {
            x: 100 + i,
            y: 200 + i,
        });
    }

    let mut count = 0;
    while let Ok(event) = event_manager.receiver().try_recv() {
        if let AppEvent::CtrlRightClick { x, y } = event {
            count += 1;
            assert_eq!(x, 100 + count);
            assert_eq!(y, 200 + count);
        }
    }

    assert_eq!(count, 25, "All 25 consecutive CtrlRightClick events must be received accurately");
}

