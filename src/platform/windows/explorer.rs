use std::path::PathBuf;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Variant::*;
use windows::Win32::UI::Shell::*;

use crate::files::file_context::FileContext;
use crate::files::resolver::FileResolverResult;

fn get_window_class(hwnd: HWND) -> String {
    let mut class_name = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut class_name) };
    if len > 0 {
        String::from_utf16_lossy(&class_name[..len as usize])
    } else {
        String::new()
    }
}

fn get_root_window(hwnd: HWND) -> HWND {
    let root = unsafe { GetAncestor(hwnd, GA_ROOT) };
    if root.0 != std::ptr::null_mut() {
        root
    } else {
        hwnd
    }
}

fn extract_selection_from_dispatch(disp: &IDispatch) -> FileResolverResult {
    unsafe {
        let sp: Result<IServiceProvider> = disp.cast();
        let sp = match sp {
            Ok(s) => s,
            Err(e) => return FileResolverResult::Error(format!("IServiceProvider hatası: {}", e)),
        };

        let browser: Result<IShellBrowser> = sp.QueryService(&SID_STopLevelBrowser);
        let browser = match browser {
            Ok(b) => b,
            Err(e) => return FileResolverResult::Error(format!("IShellBrowser hatası: {}", e)),
        };

        let view = match browser.QueryActiveShellView() {
            Ok(v) => v,
            Err(e) => return FileResolverResult::Error(format!("ShellView hatası: {}", e)),
        };

        let folder_view: Result<IFolderView> = view.cast();
        let folder_view = match folder_view {
            Ok(fv) => fv,
            Err(e) => return FileResolverResult::Error(format!("IFolderView hatası: {}", e)),
        };

        let item_array: Result<IShellItemArray> = folder_view.Items(SVGIO_SELECTION);
        let item_array = match item_array {
            Ok(ia) => ia,
            Err(_) => return FileResolverResult::NoSelection,
        };

        let count = match item_array.GetCount() {
            Ok(c) => c as usize,
            Err(_) => 0,
        };

        if count == 0 {
            return FileResolverResult::NoSelection;
        }

        if count > 1 {
            return FileResolverResult::MultipleSelection(count);
        }

        let item = match item_array.GetItemAt(0) {
            Ok(i) => i,
            Err(e) => return FileResolverResult::Error(format!("Öğe okunamadı: {}", e)),
        };

        let path_pwstr = match item.GetDisplayName(SIGDN_FILESYSPATH) {
            Ok(p) => p,
            Err(e) => return FileResolverResult::Error(format!("Dosya yolu okunamadı: {}", e)),
        };

        let path_str = match path_pwstr.to_string() {
            Ok(s) => s,
            Err(e) => return FileResolverResult::Error(format!("Karakter kodlama hatası: {}", e)),
        };

        let path = PathBuf::from(path_str);
        match FileContext::from_path(path) {
            Ok(ctx) => FileResolverResult::Selected(ctx),
            Err(e) => FileResolverResult::Error(e),
        }
    }
}

pub fn get_explorer_selection() -> FileResolverResult {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        // Check foreground window and cursor window
        let mut target_hwnd = GetForegroundWindow();
        if target_hwnd.0 == std::ptr::null_mut() {
            let mut pt = POINT::default();
            if GetCursorPos(&mut pt).is_ok() {
                target_hwnd = WindowFromPoint(pt);
            }
        }

        if target_hwnd.0 == std::ptr::null_mut() {
            return FileResolverResult::UnsupportedApp;
        }

        let root_hwnd = get_root_window(target_hwnd);
        let class_name = get_window_class(root_hwnd);

        let is_explorer = class_name == "CabinetWClass" || class_name == "ExploreWClass";
        let is_desktop = class_name == "Progman" || class_name == "WorkerW";

        if !is_explorer && !is_desktop {
            return FileResolverResult::UnsupportedApp;
        }

        let sw: Result<IShellWindows> = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL);
        let sw = match sw {
            Ok(s) => s,
            Err(e) => return FileResolverResult::Error(format!("ShellWindows başlatılamadı: {}", e)),
        };

        if is_desktop {
            let empty = VARIANT::default();
            let hwnd_var = VARIANT::default();
            let mut phwnd = 0;
            if let Ok(disp) = sw.FindWindowSW(&empty, &hwnd_var, SWC_DESKTOP, &mut phwnd, SWFO_NEEDDISPATCH) {
                return extract_selection_from_dispatch(&disp);
            }
        }

        let count = sw.Count().unwrap_or(0);
        for i in 0..count {
            let var_index = VARIANT::from(i);
            if let Ok(disp) = sw.Item(&var_index) {
                if let Ok(wb) = disp.cast::<IWebBrowserApp>() {
                    if let Ok(hwnd_val) = wb.HWND() {
                        let explorer_hwnd = HWND(hwnd_val.0 as *mut _);
                        if explorer_hwnd == root_hwnd || explorer_hwnd == target_hwnd {
                            return extract_selection_from_dispatch(&disp);
                        }
                    }
                }
            }
        }

        // If target hwnd didn't match directly, try the active window among IShellWindows
        for i in 0..count {
            let var_index = VARIANT::from(i);
            if let Ok(disp) = sw.Item(&var_index) {
                if let Ok(wb) = disp.cast::<IWebBrowserApp>() {
                    if let Ok(hwnd_val) = wb.HWND() {
                        let explorer_hwnd = HWND(hwnd_val.0 as *mut _);
                        if GetForegroundWindow() == explorer_hwnd {
                            return extract_selection_from_dispatch(&disp);
                        }
                    }
                }
            }
        }

        FileResolverResult::NoSelection
    }
}
