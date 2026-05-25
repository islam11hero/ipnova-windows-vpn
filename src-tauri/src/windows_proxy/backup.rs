//! Backup / restore WinINet + WinHTTP before VPN overrides.

#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::path::Path;

#[cfg(windows)]
use serde::{Deserialize, Serialize};

#[cfg(windows)]
use crate::wininet_registry::{
    read_wininet_proxy_state, restore_wininet_from_backup, WininetProxyState,
};

#[cfg(windows)]
use super::notify::notify_proxy_change;
#[cfg(windows)]
use super::winhttp::{
    disable_winhttp_proxy, netsh_output, parse_winhttp_show_proxy, reset_winhttp_stack,
    winhttp_advproxy_settings_path,
};
#[cfg(windows)]
use super::wininet::refresh_wininet_connections;

#[cfg(windows)]
#[derive(Debug, Serialize, Deserialize)]
struct IeBackupJson {
    #[serde(rename = "ProxyEnable", default)]
    proxy_enable: Option<u32>,
    #[serde(rename = "ProxyServer", default)]
    proxy_server: Option<String>,
    #[serde(rename = "ProxyOverride", default)]
    proxy_override: Option<String>,
    #[serde(rename = "AutoDetect", default)]
    auto_detect: Option<u32>,
    #[serde(rename = "AutoConfigURL", default)]
    auto_config_url: Option<String>,
}

#[cfg(windows)]
#[derive(Debug, Serialize, Deserialize)]
struct WinhttpBackupJson {
    direct: bool,
    proxy: Option<String>,
    bypass: Option<String>,
    show_proxy: String,
    show_advproxy: String,
}

#[cfg(windows)]
#[derive(Debug, Serialize, Deserialize)]
struct ProxyBackupFile {
    ie: IeBackupJson,
    winhttp: WinhttpBackupJson,
}

#[cfg(windows)]
pub fn backup_path(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join("proxy-backup.json")
}

#[cfg(windows)]
pub fn backup_exists(state_dir: &Path) -> bool {
    backup_path(state_dir).is_file()
}

#[cfg(windows)]
fn ie_from_state(state: &WininetProxyState) -> IeBackupJson {
    IeBackupJson {
        proxy_enable: Some(if state.enabled { 1 } else { 0 }),
        proxy_server: if state.server.is_empty() {
            None
        } else {
            Some(state.server.clone())
        },
        proxy_override: None,
        auto_detect: Some(if state.auto_detect { 1 } else { 0 }),
        auto_config_url: if state.pac_url.is_empty() {
            None
        } else {
            Some(state.pac_url.clone())
        },
    }
}

#[cfg(windows)]
pub fn backup_proxy_settings(state_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(state_dir).map_err(|e| e.to_string())?;

    // Keep the first snapshot until Disconnect — avoids overwriting with VPN proxy on reconnect.
    if backup_exists(state_dir) {
        return Ok(());
    }

    let state = read_wininet_proxy_state().unwrap_or_default();
    let mut ie = ie_from_state(&state);
    if let Ok(settings) = read_proxy_override() {
        ie.proxy_override = settings;
    }

    let show_proxy = netsh_output(&["winhttp", "show", "proxy"]);
    let show_advproxy = netsh_output(&["winhttp", "show", "advproxy"]);
    let (direct, proxy, bypass) = parse_winhttp_show_proxy(&show_proxy);

    let payload = ProxyBackupFile {
        ie,
        winhttp: WinhttpBackupJson {
            direct,
            proxy,
            bypass,
            show_proxy,
            show_advproxy,
        },
    };

    let json = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    fs::write(backup_path(state_dir), json).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(windows)]
fn read_proxy_override() -> Result<Option<String>, String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Internet Settings")
        .map_err(|e| e.to_string())?;
    let v: String = settings.get_value("ProxyOverride").unwrap_or_default();
    if v.is_empty() {
        Ok(None)
    } else {
        Ok(Some(v))
    }
}

#[cfg(windows)]
pub fn restore_proxy_from_backup(state_dir: &Path) -> bool {
    let path = backup_path(state_dir);
    if !path.is_file() {
        return false;
    }

    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let data: ProxyBackupFile = match serde_json::from_str(&raw) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let adv = winhttp_advproxy_settings_path(state_dir);
    if adv.exists() {
        let _ = fs::remove_file(&adv);
    }

    let pac = data.ie.auto_config_url.as_deref();
    let _ = restore_wininet_from_backup(
        data.ie.proxy_enable,
        data.ie.proxy_server.as_deref(),
        data.ie.proxy_override.as_deref(),
        data.ie.auto_detect,
        pac,
    );

    reset_winhttp_stack();
    if !data.winhttp.direct {
        if let Some(ref proxy) = data.winhttp.proxy {
            let bypass = data
                .winhttp
                .bypass
                .as_deref()
                .unwrap_or("<local>");
            let _ = enable_winhttp_legacy_from_strings(proxy, bypass);
        }
    }

    let _ = refresh_wininet_connections();
    notify_proxy_change();
    let _ = fs::remove_file(&path);
    true
}

#[cfg(windows)]
fn enable_winhttp_legacy_from_strings(proxy: &str, bypass: &str) -> Result<(), String> {
    use super::process::hidden_command;
    let status = hidden_command("netsh")
        .args(["winhttp", "set", "proxy", proxy, bypass])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("netsh winhttp set proxy failed".into())
    }
}

#[cfg(windows)]
pub fn restore_or_disable(state_dir: &Path) {
    if !restore_proxy_from_backup(state_dir) {
        let _ = super::wininet::disable_wininet_proxy();
        let _ = disable_winhttp_proxy();
    }
}
