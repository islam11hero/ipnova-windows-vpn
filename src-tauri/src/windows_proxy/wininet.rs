//! WinINet (browser) proxy enable/verify/disable.

#[cfg(windows)]
use std::path::Path;

#[cfg(windows)]
use crate::wininet_registry::{
    disable_wininet_manual_proxy, write_wininet_manual_proxy, wininet_gpo_hint,
    wininet_has_pac_or_wpad, wininet_points_at_port,
};

#[cfg(windows)]
use super::log::append_proxy_log;
#[cfg(windows)]
use super::notify::notify_proxy_change;

/// Stamp Connections key so Chromium reloads proxy from registry.
#[cfg(windows)]
pub fn refresh_wininet_connections() -> Result<(), String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            KEY_WRITE,
        )
        .map_err(|e| e.to_string())?;
    let _ = settings.delete_value("AutoConfigURL");
    let _ = settings.set_value("AutoDetect", &0u32);

    let conn_path = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings\Connections";
    if let Ok(conn) = hkcu.open_subkey_with_flags(conn_path, KEY_WRITE) {
        let stamp = chrono_lite_timestamp();
        let _ = conn.set_value("IPNOVAProxyRefresh", &stamp);
    }

    notify_proxy_change();
    Ok(())
}

#[cfg(windows)]
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

#[cfg(windows)]
pub fn enable_wininet_proxy(port: u16) -> Result<(), String> {
    write_wininet_manual_proxy(port)?;
    refresh_wininet_connections()
}

#[cfg(windows)]
pub fn enable_wininet_proxy_verified(port: u16, state_dir: &Path) -> Result<(), String> {
    const MAX_ATTEMPTS: u32 = 2;
    let mut last_err = String::new();

    for attempt in 1..=MAX_ATTEMPTS {
        if let Err(e) = enable_wininet_proxy(port) {
            last_err = e;
            append_proxy_log(
                state_dir,
                "wininet_enable_fail",
                &format!("attempt {attempt}: {last_err}"),
            );
            continue;
        }
        if is_wininet_proxy_for_port(port) {
            append_proxy_log(state_dir, "wininet_ok", &format!("attempt {attempt}"));
            return Ok(());
        }
        last_err = format!(
            "ProxyEnable/ProxyServer set but registry still does not show 127.0.0.1:{port}"
        );
        append_proxy_log(state_dir, "wininet_verify_fail", &format!("attempt {attempt}"));
        std::thread::sleep(std::time::Duration::from_millis(350));
    }

    let pac = wininet_has_pac_or_wpad();
    let gpo = wininet_gpo_hint();
    let mut msg = format!(
        "Windows did not apply the browser proxy ({last_err}). \
         Close other VPN apps, disable PAC/WPAD in Internet Options, then retry."
    );
    if pac {
        msg.push_str(" PAC or auto-detect is still enabled.");
    }
    if let Some(gpo) = gpo {
        msg.push_str(&format!(" {gpo}"));
    }
    Err(msg)
}

#[cfg(windows)]
pub fn disable_wininet_proxy() -> Result<(), String> {
    disable_wininet_manual_proxy()?;
    refresh_wininet_connections()
}

/// True when WinINet proxy points at our local mixed inbound port.
#[cfg(windows)]
pub fn is_wininet_proxy_for_port(port: u16) -> bool {
    wininet_points_at_port(port)
}

/// Clear WinINet if it still points at our port but sing-box is not running (crash recovery).
#[cfg(windows)]
pub fn reset_wininet_if_orphaned(port: u16) -> bool {
    if !is_wininet_proxy_for_port(port) {
        return false;
    }
    let _ = disable_wininet_manual_proxy();
    let _ = refresh_wininet_connections();
    !is_wininet_proxy_for_port(port)
}
