mod windows;

#[cfg(unix)]
use gio::prelude::*;
#[cfg(unix)]
use gio::Settings;
#[cfg(unix)]
use std::process::Command;

#[cfg(target_os = "windows")]
use windows::windows_proxy::WindowsProxyBackup;
#[cfg(target_os = "windows")]
use crate::windows::windows_proxy;

#[derive(Debug, Clone)]
pub(crate) struct ProxyBackup {
    mode: String,
    host: String,
    port: i32,
    ignore_hosts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SocksManager {
    backup: Option<ProxyBackup>,
    #[cfg(target_os = "windows")]
    win_backup: Option<WindowsProxyBackup>,
}

impl SocksManager {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self { backup: None, win_backup: None }
        }

        #[cfg(unix)]
        {
            Self { backup: None }
        }
    }

    pub fn detect_desktop_env() -> Option<String> {
        std::env::var("XDG_CURRENT_DESKTOP").ok()
    }

    pub unsafe fn connect(&mut self, host: &str, port: i32) -> anyhow::Result<()> {
        #[cfg(target_os = "windows")]
        {
            let backup = windows_proxy::set_socks5_proxy(host, port)?;
            self.win_backup = Some(backup);
            return Ok(());
        }

        let de = Self::detect_desktop_env().unwrap_or_default().to_lowercase();

        if de.contains("gnome") {
            #[cfg(unix)]
            self.connect_gnome(host, port)?;
        } else if de.contains("kde") {
            #[cfg(unix)]
            self.connect_kde(host, port)?;
        } else {
            unsafe {std::env::set_var("ALL_PROXY", format!("socks5://{}:{}", host, port));}
        }

        Ok(())
    }

    pub unsafe fn disconnect(&mut self) -> anyhow::Result<()> {
        #[cfg(target_os = "windows")]
        {
            if let Some(backup) = &self.win_backup {
                windows_proxy::restore_proxy(backup)?;
            }

            return Ok(());
        }

        let de = Self::detect_desktop_env().unwrap_or_default().to_lowercase();

        if de.contains("gnome") {
            #[cfg(unix)]
            self.disconnect_gnome()?;
        } else if de.contains("kde") {
            #[cfg(unix)]
            self.disconnect_kde()?;
        } else {
            unsafe {std::env::remove_var("ALL_PROXY");}
        }

        Ok(())
    }

    #[cfg(unix)]
    fn connect_gnome(&mut self, host: &str, port: i32) -> anyhow::Result<()> {
        let proxy_settings = Settings::new("org.gnome.system.proxy");
        let socks_settings = Settings::new("org.gnome.system.proxy.socks");

        let backup = ProxyBackup {
            mode: proxy_settings.get::<String>("mode"),
            host: socks_settings.get::<String>("host"),
            port: socks_settings.get::<i32>("port"),
            ignore_hosts: proxy_settings.get::<Vec<String>>("ignore-hosts"),
        };
        self.backup = Some(backup);

        proxy_settings.set("mode", &"manual")?;
        socks_settings.set("host", &host)?;
        socks_settings.set("port", &port)?;

        Ok(())
    }

    #[cfg(unix)]
    fn disconnect_gnome(&mut self) -> anyhow::Result<()> {
        if let Some(backup) = &self.backup {
            let proxy_settings = Settings::new("org.gnome.system.proxy");
            let socks_settings = Settings::new("org.gnome.system.proxy.socks");

            proxy_settings.set("mode", &backup.mode)?;
            socks_settings.set("host", &backup.host)?;
            socks_settings.set("port", &backup.port)?;
            proxy_settings.set("ignore-hosts", &backup.ignore_hosts)?;
        }

        Ok(())
    }

    #[cfg(unix)]
    fn connect_kde(&mut self, host: &str, port: i32) -> anyhow::Result<()> {
        Command::new("kwriteconfig5")
            .args(["--file", "kioslaverc", "--group", "Proxy Settings", "--key", "ProxyType", "1"])
            .output()?;
        Command::new("kwriteconfig5")
            .args(["--file", "kioslaverc", "--group", "Proxy Settings", "--key", "socksProxy", &format!("{}:{}", host, port)])
            .output()?;
        Command::new("qdbus")
            .args(["org.kde.kded5", "/modules/proxyscout", "reparseConfiguration"])
            .output()
            .ok();

        Ok(())
    }

    #[cfg(unix)]
    fn disconnect_kde(&mut self) -> anyhow::Result<()> {
        Command::new("kwriteconfig5")
            .args(["--file", "kioslaverc", "--group", "Proxy Settings", "--key", "ProxyType", "0"])
            .output()?;
        Command::new("qdbus")
            .args(["org.kde.kded5", "/modules/proxyscout", "reparseConfiguration"])
            .output()
            .ok();

        Ok(())
    }
}
