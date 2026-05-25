//! WinINet proxy state via registry (fast reads; writes without PowerShell).

#[cfg(windows)]
const IE_SETTINGS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

#[cfg(windows)]
const IE_POLICY_KEY: &str =
    r"Software\Policies\Microsoft\Windows\CurrentVersion\Internet Settings";

#[cfg(windows)]
pub const PROXY_OVERRIDE: &str = "<local>;localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*";

#[cfg(windows)]
pub fn proxy_server_value(port: u16) -> String {
    let host = format!("127.0.0.1:{port}");
    format!("http={host};https={host};socks={host}")
}

#[cfg(windows)]
#[derive(Clone, Debug, Default)]
pub struct WininetProxyState {
    pub enabled: bool,
    pub server: String,
    pub auto_detect: bool,
    pub pac_url: String,
}

#[cfg(windows)]
fn open_ie_settings_write() -> Result<winreg::RegKey, String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
    use winreg::RegKey;
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(IE_SETTINGS_KEY, KEY_WRITE)
        .map_err(|e| e.to_string())
}

#[cfg(windows)]
fn open_ie_settings_read() -> Result<winreg::RegKey, String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(IE_SETTINGS_KEY)
        .map_err(|e| e.to_string())
}

#[cfg(windows)]
pub fn read_wininet_proxy_state() -> Option<WininetProxyState> {
    let settings = open_ie_settings_read().ok()?;
    let enabled: u32 = settings.get_value("ProxyEnable").unwrap_or(0);
    let server: String = settings.get_value("ProxyServer").unwrap_or_default();
    let auto_detect: u32 = settings.get_value("AutoDetect").unwrap_or(0);
    let pac_url: String = settings.get_value("AutoConfigURL").unwrap_or_default();
    Some(WininetProxyState {
        enabled: enabled == 1,
        server,
        auto_detect: auto_detect != 0,
        pac_url,
    })
}

#[cfg(not(windows))]
pub fn read_wininet_proxy_state() -> Option<()> {
    None
}

/// `(ProxyEnable == 1, ProxyServer value)`.
#[cfg(windows)]
pub fn read_wininet_proxy() -> Option<(bool, String)> {
    read_wininet_proxy_state().map(|s| (s.enabled, s.server))
}

#[cfg(not(windows))]
pub fn read_wininet_proxy() -> Option<(bool, String)> {
    None
}

#[cfg(windows)]
pub fn wininet_has_pac_or_wpad() -> bool {
    read_wininet_proxy_state().is_some_and(|s| !s.pac_url.trim().is_empty() || s.auto_detect)
}

#[cfg(not(windows))]
pub fn wininet_has_pac_or_wpad() -> bool {
    false
}

#[cfg(windows)]
pub fn wininet_points_at_port(port: u16) -> bool {
    let Some(state) = read_wininet_proxy_state() else {
        return false;
    };
    if !state.enabled {
        return false;
    }
    let port_s = port.to_string();
    let server = state.server.to_ascii_lowercase();
    server.contains("127.0.0.1") && server.contains(&port_s)
}

#[cfg(not(windows))]
pub fn wininet_points_at_port(_port: u16) -> bool {
    false
}

/// Apply manual proxy for browsers (WinINet / Chromium on Windows).
#[cfg(windows)]
pub fn write_wininet_manual_proxy(port: u16) -> Result<(), String> {
    let settings = open_ie_settings_write()?;
    let proxy = proxy_server_value(port);
    settings.set_value("ProxyEnable", &1u32).map_err(|e| e.to_string())?;
    settings
        .set_value("ProxyServer", &proxy)
        .map_err(|e| e.to_string())?;
    settings
        .set_value("ProxyOverride", &PROXY_OVERRIDE)
        .map_err(|e| e.to_string())?;
    settings.set_value("AutoDetect", &0u32).map_err(|e| e.to_string())?;
    let _ = settings.delete_value("AutoConfigURL");
    let _ = settings.delete_value("ProxySettingsPerUser");
    Ok(())
}

#[cfg(windows)]
pub fn disable_wininet_manual_proxy() -> Result<(), String> {
    let settings = open_ie_settings_write()?;
    settings.set_value("ProxyEnable", &0u32).map_err(|e| e.to_string())?;
    settings.set_value("AutoDetect", &0u32).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(windows)]
pub fn restore_wininet_from_backup(
    enable: Option<u32>,
    server: Option<&str>,
    override_list: Option<&str>,
    auto_detect: Option<u32>,
    pac_url: Option<&str>,
) -> Result<(), String> {
    let settings = open_ie_settings_write()?;
    if let Some(v) = enable {
        settings.set_value("ProxyEnable", &v).map_err(|e| e.to_string())?;
    }
    if let Some(s) = server {
        settings
            .set_value("ProxyServer", &s)
            .map_err(|e| e.to_string())?;
    } else {
        let _ = settings.delete_value("ProxyServer");
    }
    if let Some(o) = override_list {
        settings
            .set_value("ProxyOverride", &o)
            .map_err(|e| e.to_string())?;
    }
    if let Some(v) = auto_detect {
        settings.set_value("AutoDetect", &v).map_err(|e| e.to_string())?;
    }
    if let Some(url) = pac_url {
        if url.is_empty() {
            let _ = settings.delete_value("AutoConfigURL");
        } else {
            settings
                .set_value("AutoConfigURL", &url)
                .map_err(|e| e.to_string())?;
        }
    } else {
        let _ = settings.delete_value("AutoConfigURL");
    }
    Ok(())
}

/// Registry-based GPO lock detection (no PowerShell).
#[cfg(windows)]
pub fn wininet_gpo_hint() -> Option<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let policy = hklm.open_subkey(IE_POLICY_KEY).ok()?;
    let keys = [
        "ProxyEnable",
        "ProxyServer",
        "ProxySettings",
        "AutoConfigURL",
    ];
    let mut hit = Vec::new();
    for k in keys {
        if policy.get_raw_value(k).is_ok() {
            hit.push(k);
        }
    }
    if hit.is_empty() {
        None
    } else {
        Some(format!("WinINet GPO locks: {}", hit.join(", ")))
    }
}

#[cfg(not(windows))]
pub fn write_wininet_manual_proxy(_port: u16) -> Result<(), String> {
    Err("Windows only".into())
}

#[cfg(not(windows))]
pub fn disable_wininet_manual_proxy() -> Result<(), String> {
    Err("Windows only".into())
}

#[cfg(not(windows))]
pub fn restore_wininet_from_backup(
    _enable: Option<u32>,
    _server: Option<&str>,
    _override_list: Option<&str>,
    _auto_detect: Option<u32>,
    _pac_url: Option<&str>,
) -> Result<(), String> {
    Err("Windows only".into())
}

#[cfg(not(windows))]
pub fn wininet_gpo_hint() -> Option<String> {
    None
}
