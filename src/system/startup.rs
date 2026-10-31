use std::env;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Registry::*;

const SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "Kisayolluk";

pub fn is_startup_enabled() -> bool {
    unsafe {
        let subkey_hstring = HSTRING::from(SUBKEY);
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_hstring.as_ptr()),
            Some(0),
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return false;
        }

        let app_name_hstring = HSTRING::from(APP_NAME);
        let mut data_type = REG_VALUE_TYPE::default();
        let mut data_len = 0u32;

        let query_res = RegQueryValueExW(
            hkey,
            PCWSTR(app_name_hstring.as_ptr()),
            None,
            Some(&mut data_type),
            None,
            Some(&mut data_len),
        );

        let _ = RegCloseKey(hkey);
        query_res.is_ok()
    }
}

pub fn set_startup(enable: bool) -> std::result::Result<(), String> {
    unsafe {
        let subkey_hstring = HSTRING::from(SUBKEY);
        let mut hkey = HKEY::default();
        let create_res = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_hstring.as_ptr()),
            Some(0),
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            None,
            &mut hkey,
            None,
        );

        if create_res.is_err() {
            return Err(format!("Kayıt defteri anahtarı açılamadı (kod: {})", create_res.0));
        }

        let app_name_hstring = HSTRING::from(APP_NAME);

        if enable {
            let current_exe = env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = current_exe.to_string_lossy();
            let quoted_path = format!("\"{}\"", exe_str);
            let wide_path: Vec<u16> = quoted_path.encode_utf16().chain(std::iter::once(0)).collect();

            let bytes: &[u8] = std::slice::from_raw_parts(
                wide_path.as_ptr() as *const u8,
                wide_path.len() * 2,
            );

            let res = RegSetValueExW(
                hkey,
                PCWSTR(app_name_hstring.as_ptr()),
                Some(0),
                REG_SZ,
                Some(bytes),
            );

            let _ = RegCloseKey(hkey);
            if res.is_err() {
                return Err(format!("Başlangıç değeri yazılamadı (kod: {})", res.0));
            }
        } else {
            let res = RegDeleteValueW(hkey, PCWSTR(app_name_hstring.as_ptr()));
            let _ = RegCloseKey(hkey);
            if res.is_err() && res != ERROR_FILE_NOT_FOUND {
                return Err(format!("Başlangıç değeri silinemedi (kod: {})", res.0));
            }
        }

        Ok(())
    }
}
