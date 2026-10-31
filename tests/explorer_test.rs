use windows::core::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Variant::*;
use windows::Win32::UI::Shell::*;

#[test]
fn test_all_explorer_windows() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let sw: Result<IShellWindows> = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL);
        if let Ok(sw) = sw {
            let count = sw.Count().unwrap_or(0);
            println!("Shell windows count: {}", count);
            for i in 0..count {
                let var_i = VARIANT::from(i);
                if let Ok(disp) = sw.Item(&var_i) {
                    if let Ok(wb) = disp.cast::<IWebBrowserApp>() {
                        if let Ok(doc) = wb.Document() {
                            if let Ok(sfvd) = doc.cast::<IShellFolderViewDual>() {
                                if let Ok(sel) = sfvd.SelectedItems() {
                                    let sel_count = sel.Count().unwrap_or(0);
                                    println!("Window {} selected items: {}", i, sel_count);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
