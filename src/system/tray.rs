use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const WM_TRAY_CALLBACK: u32 = WM_APP + 101;
const ID_TRAY_TOGGLE: usize = 1001;
const ID_TRAY_SETTINGS: usize = 1002;
const ID_TRAY_ABOUT: usize = 1003;
const ID_TRAY_EXIT: usize = 1004;

static TRAY_ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    ToggleEnabled,
    OpenSettings,
    About,
    Exit,
}

pub struct SystemTray {
    thread_handle: Option<JoinHandle<()>>,
    hwnd: isize,
}

impl SystemTray {
    pub fn new(command_sender: Sender<TrayCommand>) -> std::result::Result<Self, String> {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();

        let thread_handle = thread::spawn(move || unsafe {
            let class_name = w!("KisayollukTrayWindowClass");
            let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(PCWSTR::null())
                .unwrap_or_default();
            let hinstance_cast = HINSTANCE(hinstance.0);

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(tray_wnd_proc),
                hInstance: hinstance_cast,
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassExW(&wnd_class);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                w!("KisayollukTrayWindow"),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(hinstance_cast),
                None,
            );

            let hwnd = match hwnd {
                Ok(h) => h,
                Err(e) => {
                    let _ = ready_tx.send(Err(format!("Tray penceresi oluşturulamadı: {}", e)));
                    return;
                }
            };

            // Save command sender in user data
            let sender_boxed = Box::new(command_sender);
            let sender_ptr = Box::into_raw(sender_boxed);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, sender_ptr as isize);

            // Create Tray Icon
            let icon = LoadIconW(None, IDI_APPLICATION)
                .or_else(|_| LoadIconW(None, IDI_ASTERISK))
                .unwrap_or(HICON(std::ptr::null_mut()));
            let mut tip = [0u16; 128];
            let tip_str = "Kısayolluk - Radyal İşlem Aracı";
            for (i, c) in tip_str.encode_utf16().enumerate().take(127) {
                tip[i] = c;
            }

            let mut flags = NIF_MESSAGE | NIF_TIP;
            if !icon.is_invalid() && !icon.0.is_null() {
                flags |= NIF_ICON;
            }

            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: flags,
                uCallbackMessage: WM_TRAY_CALLBACK,
                hIcon: icon,
                szTip: tip,
                ..Default::default()
            };

            let mut added = false;
            for _ in 0..5 {
                if Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
                    added = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(150));
            }

            if !added {
                let err = windows::Win32::Foundation::GetLastError();
                let _ = ready_tx.send(Err(format!("Sistem tepsisi simgesi eklenemedi (err: {:?}, icon: {:?}).", err, icon.0)));
                return;
            }

            let _ = ready_tx.send(Ok(hwnd.0 as isize));

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Remove Tray Icon
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);

            // Clean up sender
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Sender<TrayCommand>;
            if !ptr.is_null() {
                let _ = Box::from_raw(ptr);
            }
        });

        match ready_rx.recv() {
            Ok(Ok(hwnd)) => Ok(Self {
                thread_handle: Some(thread_handle),
                hwnd,
            }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Tray başlatma zaman aşımına uğradı.".to_string()),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        TRAY_ENABLED.store(enabled, Ordering::SeqCst);
    }

    pub fn stop(&mut self) {
        if self.hwnd != 0 {
            unsafe {
                let _ = PostMessageW(Some(HWND(self.hwnd as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        self.stop();
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAY_CALLBACK => {
            let event = (lparam.0 & 0xffff) as u32;
            if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
                show_tray_menu(hwnd);
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xffff) as usize;
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Sender<TrayCommand>;
            if !ptr.is_null() {
                let sender = &*ptr;
                match id {
                    ID_TRAY_TOGGLE => {
                        let cur = TRAY_ENABLED.load(Ordering::SeqCst);
                        TRAY_ENABLED.store(!cur, Ordering::SeqCst);
                        let _ = sender.send(TrayCommand::ToggleEnabled);
                    }
                    ID_TRAY_SETTINGS => {
                        let _ = sender.send(TrayCommand::OpenSettings);
                    }
                    ID_TRAY_ABOUT => {
                        let _ = sender.send(TrayCommand::About);
                    }
                    ID_TRAY_EXIT => {
                        let _ = sender.send(TrayCommand::Exit);
                    }
                    _ => {}
                }
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let menu = match CreatePopupMenu() {
        Ok(m) => m,
        Err(_) => return,
    };

    let is_enabled = TRAY_ENABLED.load(Ordering::SeqCst);
    let toggle_text = if is_enabled {
        w!("Kısayolluk: Etkin (Tıkla ve Devre Dışı Bırak)")
    } else {
        w!("Kısayolluk: Devre Dışı (Tıkla ve Etkinleştir)")
    };

    let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_TOGGLE, toggle_text);
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_SETTINGS, w!("Ayarlar..."));
    let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_ABOUT, w!("Hakkında"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT, w!("Çıkış"));

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);

    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON,
        pt.x,
        pt.y,
        Some(0),
        hwnd,
        None,
    );
    let _ = DestroyMenu(menu);
}
