//! Machine-wide WinINet + WinHTTP (elevated).

#[cfg(windows)]
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_WRITE};
#[cfg(windows)]
use winreg::RegKey;

#[cfg(windows)]
use super::constants::SYSTEM_PROXY_PORT;
#[cfg(windows)]
use super::notify::notify_proxy_change;
#[cfg(windows)]
use super::winhttp::enable_winhttp_proxy;
#[cfg(windows)]
use super::wininet::refresh_wininet_connections;
#[cfg(windows)]
use crate::wininet_registry::{proxy_server_value, write_wininet_manual_proxy, PROXY_OVERRIDE};
#[cfg(windows)]
use crate::windows_security::is_process_elevated;

#[cfg(windows)]
const IE_SETTINGS: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";
#[cfg(windows)]
const IE_POLICY: &str = r"Software\Policies\Microsoft\Windows\CurrentVersion\Internet Settings";

#[cfg(windows)]
fn write_ie_proxy(key: &RegKey) -> Result<(), String> {
    let proxy = proxy_server_value(SYSTEM_PROXY_PORT);
    key.set_value("ProxyEnable", &1u32).map_err(|e| e.to_string())?;
    key.set_value("ProxyServer", &proxy).map_err(|e| e.to_string())?;
    key.set_value("ProxyOverride", &PROXY_OVERRIDE)
        .map_err(|e| e.to_string())?;
    key.set_value("AutoDetect", &0u32).map_err(|e| e.to_string())?;
    let _ = key.delete_value("AutoConfigURL");
    Ok(())
}

#[cfg(windows)]
pub fn apply_machine_wide_wininet(port: u16) -> Result<String, String> {
    if !is_process_elevated() {
        return Err(
            "Administrator rights required — close IPNOVA, right-click → Run as administrator, then try again."
                .into(),
        );
    }

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let policy = hklm
        .create_subkey(IE_POLICY)
        .map_err(|e| e.to_string())?
        .0;
    policy
        .set_value("ProxySettingsPerUser", &0u32)
        .map_err(|e| e.to_string())?;

    let hkcu_settings = hkcu
        .open_subkey_with_flags(IE_SETTINGS, KEY_WRITE)
        .map_err(|e| e.to_string())?;
    write_ie_proxy(&hkcu_settings)?;

    let hklm_settings = hklm
        .open_subkey_with_flags(IE_SETTINGS, KEY_WRITE)
        .map_err(|e| e.to_string())?;
    write_ie_proxy(&hklm_settings)?;

    write_wininet_manual_proxy(port)?;
    refresh_wininet_connections()?;

    let state_dir = std::env::temp_dir().join("ipnova-machine-proxy");
    let _ = std::fs::create_dir_all(&state_dir);
    let wh_msg = match enable_winhttp_proxy(&state_dir, port) {
        Ok(m) => format!(" WinHTTP: {m}."),
        Err(e) => format!(" WinHTTP warning: {e}."),
    };

    notify_proxy_change();
    Ok(format!(
        "Machine-wide proxy applied for all Windows users.{wh_msg}"
    ))
}
