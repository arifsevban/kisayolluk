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
        let wb: Result<IWebBrowserApp> = disp.cast();
        let wb = match wb {
            Ok(w) => w,
            Err(_) => return FileResolverResult::UnsupportedApp,
        };

        let doc = match wb.Document() {
            Ok(d) => d,
            Err(_) => return FileResolverResult::UnsupportedApp,
        };

        let sfvd: Result<IShellFolderViewDual> = doc.cast();
        let sfvd = match sfvd {
            Ok(s) => s,
            Err(_) => return FileResolverResult::UnsupportedApp,
        };

        let items = match sfvd.SelectedItems() {
            Ok(i) => i,
            Err(_) => return FileResolverResult::NoSelection,
        };

        let count = items.Count().unwrap_or(0);
        if count == 0 {
            return FileResolverResult::NoSelection;
        }

        if count > 1 {
            return FileResolverResult::MultipleSelection(count as usize);
        }

        let item = match items.Item(&VARIANT::from(0i32)) {
            Ok(it) => it,
            Err(e) => return FileResolverResult::Error(format!("Öğe okunamadı: {}", e)),
        };

        let path_bstr = match item.Path() {
            Ok(p) => p,
            Err(e) => return FileResolverResult::Error(format!("Dosya yolu okunamadı: {}", e)),
        };

        let path_str = path_bstr.to_string();
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

        let sw: Result<IShellWindows> = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL);
        let sw = match sw {
            Ok(s) => s,
            Err(e) => return FileResolverResult::Error(format!("ShellWindows başlatılamadı: {}", e)),
        };

        // Check foreground window and cursor window
        let mut target_hwnd = GetForegroundWindow();
        let mut cursor_hwnd = HWND(std::ptr::null_mut());
        let mut pt = POINT::default();
        if GetCursorPos(&mut pt).is_ok() {
            cursor_hwnd = WindowFromPoint(pt);
            if target_hwnd.0.is_null() {
                target_hwnd = cursor_hwnd;
            }
        }

        let root_hwnd = get_root_window(target_hwnd);
        let class_name = get_window_class(root_hwnd);
        let cursor_class = get_window_class(cursor_hwnd);

        let is_desktop = class_name == "Progman"
            || class_name == "WorkerW"
            || cursor_class == "Progman"
            || cursor_class == "WorkerW"
            || cursor_class == "SHELLDLL_DefView"
            || cursor_class == "SysListView32";

        let is_explorer = class_name == "CabinetWClass"
            || class_name == "ExploreWClass"
            || cursor_class == "CabinetWClass";

        // 1. Direct Desktop check if desktop is active/under cursor
        if is_desktop {
            let empty = VARIANT::default();
            let hwnd_var = VARIANT::default();
            let mut phwnd = 0;
            if let Ok(disp) = sw.FindWindowSW(&empty, &hwnd_var, SWC_DESKTOP, &mut phwnd, SWFO_NEEDDISPATCH) {
                return extract_selection_from_dispatch(&disp);
            }
        }

        // 2. Direct Explorer check for matching window
        if is_explorer {
            let count = sw.Count().unwrap_or(0);
            for i in 0..count {
                let var_index = VARIANT::from(i);
                if let Ok(disp) = sw.Item(&var_index) {
                    if let Ok(wb) = disp.cast::<IWebBrowserApp>() {
                        if let Ok(hwnd_val) = wb.HWND() {
                            let explorer_hwnd = HWND(hwnd_val.0 as *mut _);
                            if explorer_hwnd == root_hwnd || explorer_hwnd == target_hwnd || explorer_hwnd == cursor_hwnd {
                                return extract_selection_from_dispatch(&disp);
                            }
                        }
                    }
                }
            }
        }

        // 3. Fallback: check Desktop for selection
        let empty = VARIANT::default();
        let hwnd_var = VARIANT::default();
        let mut phwnd = 0;
        if let Ok(disp) = sw.FindWindowSW(&empty, &hwnd_var, SWC_DESKTOP, &mut phwnd, SWFO_NEEDDISPATCH) {
            let res = extract_selection_from_dispatch(&disp);
            if matches!(res, FileResolverResult::Selected(_) | FileResolverResult::MultipleSelection(_)) {
                return res;
            }
        }

        // 4. Fallback: check open Explorer windows for selection
        let count = sw.Count().unwrap_or(0);
        for i in 0..count {
            let var_index = VARIANT::from(i);
            if let Ok(disp) = sw.Item(&var_index) {
                let res = extract_selection_from_dispatch(&disp);
                if matches!(res, FileResolverResult::Selected(_) | FileResolverResult::MultipleSelection(_)) {
                    return res;
                }
            }
        }

        if is_desktop || is_explorer {
            FileResolverResult::NoSelection
        } else {
            FileResolverResult::UnsupportedApp
        }
    }
}
