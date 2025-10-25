#[cfg(target_os = "windows")]
pub(crate) mod windows_proxy {
    use anyhow::Result;
    use std::ptr;
    use winreg::enums::*;
    use winreg::RegKey;
    use windows_sys::Win32::Networking::WinInet::{InternetSetOptionW, INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED};
    use win_inet::HINTERNET;

    #[derive(Debug, Clone)]
    pub struct WindowsProxyBackup {
        pub enabled: u32,
        pub server: String,
        pub override_list: String,
    }

    pub fn set_socks5_proxy(host: &str, port: i32) -> Result<WindowsProxyBackup> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            KEY_READ | KEY_WRITE,
        )?;

        // сохраним текущие значения, чтобы потом вернуть
        let enabled: u32 = settings.get_value("ProxyEnable").unwrap_or(0);
        let server: String = settings.get_value("ProxyServer").unwrap_or_default();
        let override_list: String = settings.get_value("ProxyOverride").unwrap_or_default();

        let backup = WindowsProxyBackup {
            enabled,
            server,
            override_list,
        };

        // применяем SOCKS5
        settings.set_value("ProxyEnable", &1u32)?;
        settings.set_value("ProxyServer", &format!("socks={}:{}", host, port))?;
        settings.set_value("ProxyOverride", &"localhost;127.0.0.1;<local>")?;

        unsafe {
            InternetSetOptionW(ptr::null_mut::<core::ffi::c_void>() as HINTERNET, INTERNET_OPTION_SETTINGS_CHANGED, ptr::null_mut(), 0);
            InternetSetOptionW(ptr::null_mut::<core::ffi::c_void>() as HINTERNET, INTERNET_OPTION_REFRESH, ptr::null_mut(), 0);
        }

        Ok(backup)
    }

    pub fn restore_proxy(backup: &WindowsProxyBackup) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            KEY_READ | KEY_WRITE,
        )?;

        settings.set_value("ProxyEnable", &backup.enabled)?;
        settings.set_value("ProxyServer", &backup.server)?;
        settings.set_value("ProxyOverride", &backup.override_list)?;

        unsafe {
            InternetSetOptionW(ptr::null_mut::<core::ffi::c_void>() as HINTERNET, INTERNET_OPTION_SETTINGS_CHANGED, ptr::null_mut(), 0);
            InternetSetOptionW(ptr::null_mut::<core::ffi::c_void>() as HINTERNET, INTERNET_OPTION_REFRESH, ptr::null_mut(), 0);
        }

        println!("♻️ Windows proxy восстановлен");
        Ok(())
    }
}
