#![windows_subsystem = "windows"]

use std::rc::Rc;
use std::cell::RefCell;
use std::thread;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use kisayolluk::config::persistence::load_settings;
use kisayolluk::core::event_manager::EventManager;
use kisayolluk::app::KisayollukApp;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 0. Default to lightweight software backend if not explicitly set (ensures ~20 MB idle RAM)
    if std::env::var("SLINT_BACKEND").is_err() {
        std::env::set_var("SLINT_BACKEND", "software");
    }

    // 1. Single Instance Protection via Named Win32 Mutex
    let mutex_name = w!("Global\\Kisayolluk_SingleInstance_Mutex");
    let mutex = unsafe { CreateMutexW(None, true, PCWSTR(mutex_name.as_ptr())) };

    if let Ok(h_mutex) = mutex {
        let last_error = unsafe { GetLastError() };
        if last_error == ERROR_ALREADY_EXISTS {
            unsafe {
                let msg = w!("Kısayolluk zaten arka planda çalışıyor.");
                let title = w!("Kısayolluk");
                let _ = MessageBoxW(None, msg, title, MB_OK | MB_ICONINFORMATION);
                let _ = CloseHandle(h_mutex);
            }
            return Ok(());
        }
    }

    // 2. Load Persisted Settings
    let settings = load_settings();

    // 3. Setup Event Manager
    let event_manager = EventManager::new();
    let event_sender = event_manager.get_raw_sender();

    // 4. Initialize Application
    let app = match KisayollukApp::new(settings, event_sender) {
        Ok(a) => Rc::new(RefCell::new(a)),
        Err(e) => {
            let error_msg: Vec<u16> = format!("Başlatma hatası: {}\0", e).encode_utf16().collect();
            unsafe {
                let _ = MessageBoxW(
                    None,
                    PCWSTR(error_msg.as_ptr()),
                    w!("Kısayolluk Hata"),
                    MB_OK | MB_ICONERROR,
                );
            }
            return Err(e.into());
        }
    };

    // 5. Connect Background Event Channel to Slint Main Thread
    let (dispatch_tx, dispatch_rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        while let Ok(event) = event_manager.receiver().recv() {
            let _ = dispatch_tx.send(event);
            let _ = slint::invoke_from_event_loop(|| {});
        }
    });

    // Event pump on main thread
    let timer = slint::Timer::default();
    let app_clone = app.clone();
    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(16), move || {
        while let Ok(event) = dispatch_rx.try_recv() {
            app_clone.borrow_mut().handle_event(event);
        }
    });

    // 6. Run Native Slint Event Loop
    let res = slint::run_event_loop();

    // 7. Cleanup Mutex on exit
    if let Ok(h_mutex) = mutex {
        unsafe {
            let _ = ReleaseMutex(h_mutex);
            let _ = CloseHandle(h_mutex);
        }
    }

    res.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
