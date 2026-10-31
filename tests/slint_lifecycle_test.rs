use std::time::Duration;
use slint::ComponentHandle;

slint::slint! {
    export component TestWin inherits Window {
        width: 100px;
        height: 100px;
    }
}

#[test]
fn test_slint_event_loop_until_quit_keeps_running() {
    let win = TestWin::new().unwrap();
    win.show().unwrap();

    let win_weak = win.as_weak();
    // After 200ms, hide the window
    slint::Timer::single_shot(Duration::from_millis(200), move || {
        println!("Hiding window now...");
        if let Some(w) = win_weak.upgrade() {
            w.hide().unwrap();
        }
    });

    // After 600ms (window is hidden for 400ms), show window again!
    let win_weak_2 = win.as_weak();
    slint::Timer::single_shot(Duration::from_millis(600), move || {
        println!("Showing window again while event loop was running!");
        if let Some(w) = win_weak_2.upgrade() {
            w.show().unwrap();
        }
    });

    // After 1000ms, hide again and explicitly call quit_event_loop
    slint::Timer::single_shot(Duration::from_millis(1000), move || {
        println!("Calling quit_event_loop() to exit cleanly!");
        let _ = slint::quit_event_loop();
    });

    println!("Starting run_event_loop_until_quit...");
    let res = slint::run_event_loop_until_quit();
    println!("Event loop exited! Result: {:?}", res);
    assert!(res.is_ok());
}
