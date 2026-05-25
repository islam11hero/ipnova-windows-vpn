//! Live proxy diagnostics for the UI.

#[cfg(windows)]
use std::path::Path;

#[cfg(windows)]
use super::backup::backup_exists;
#[cfg(windows)]
use super::constants::SYSTEM_PROXY_PORT;
#[cfg(windows)]
use super::hints::{detect_edge_proxy_policy, detect_firefox_installed, detect_other_vpn_hints};
#[cfg(windows)]
use super::probe::{is_local_proxy_listening, port_listener_info};
#[cfg(windows)]
use super::winhttp::{is_winhttp_proxy_for_port, netsh_output};
#[cfg(windows)]
use super::wininet::is_wininet_proxy_for_port;
#[cfg(windows)]
use crate::wininet_registry::{read_wininet_proxy_state, wininet_gpo_hint};
#[derive(serde::Serialize, Clone)]
pub struct ProxyDiagnostics {
    pub ie_proxy_enabled: bool,
    pub ie_proxy_server: String,
    pub pac_url: String,
    pub auto_detect: bool,
    pub winhttp_line: String,
    pub winhttp_autoproxy_service: String,
    pub wininet_ok: bool,
    pub winhttp_ok: bool,
    pub wcmsvc_dependency_issue: bool,
    pub edge_proxy_policy: Option<String>,
    pub firefox_detected: bool,
    pub other_vpn_hints: Vec<String>,
    pub port_2080_in_use: bool,
    pub port_2080_pid: Option<u32>,
    pub port_2080_process: String,
    pub backup_available: bool,
    pub recommendations_en: Vec<String>,
}

#[cfg(windows)]
pub fn proxy_diagnostics() -> ProxyDiagnostics {
    proxy_diagnostics_for_dir(None)
}

#[cfg(windows)]
pub fn proxy_diagnostics_for_dir(state_dir: Option<&Path>) -> ProxyDiagnostics {
    let mut recommendations = Vec::new();

    let wininet = read_wininet_proxy_state().unwrap_or_default();
    let ie_enabled = wininet.enabled;
    let ie_server = wininet.server.clone();
    let pac_url = wininet.pac_url.clone();
    let auto_detect = wininet.auto_detect;

    let winhttp_line = super::winhttp::netsh_output(&["winhttp", "show", "proxy"]);

    let autoproxy_svc = read_service_start_type("WinHttpAutoProxySvc");
    let wcm_deps = read_wcm_depend_on_service();
    let wcmsvc_dependency_issue =
        wcm_deps.contains("WinHttpAutoProxySvc") && autoproxy_svc.eq_ignore_ascii_case("disabled");

    let edge_proxy_policy = detect_edge_proxy_policy();
    let firefox_detected = detect_firefox_installed();
    let other_vpn_hints = detect_other_vpn_hints();

    if !other_vpn_hints.is_empty() {
        recommendations.push(format!(
            "Other VPN detected ({}) — disconnect it before using system proxy mode.",
            other_vpn_hints.join(", ")
        ));
    }

    if wcmsvc_dependency_issue {
        recommendations.push(
            "Windows 11 24H2: WinHttpAutoProxySvc is disabled while WcmSvc depends on it — use «Apply 24H2 fix» or copy the admin script below.".into(),
        );
    }

    if let Some(ref edge) = edge_proxy_policy {
        recommendations.push(edge.clone());
    }

    if firefox_detected {
        recommendations.push(
            "Firefox detected — set Settings → Network → «Use system proxy settings» or switch to TUN mode.".into(),
        );
    }

    if auto_detect || !pac_url.is_empty() {
        recommendations.push(
            "PAC or WPAD is enabled — IPNOVA disables it on Connect; if proxy fails, clear it in Internet Options → LAN settings.".into(),
        );
    }

    if let Some(gpo) = wininet_gpo_hint() {
        recommendations.push(gpo);
    }

    let (port_pid, port_proc) = port_listener_info(SYSTEM_PROXY_PORT);
    let port_in_use = is_local_proxy_listening(SYSTEM_PROXY_PORT);
    if port_in_use {
        if let Some(pid) = port_pid {
            recommendations.push(format!(
                "Port {SYSTEM_PROXY_PORT} is used by PID {pid}{} — close that app if Connect fails.",
                if port_proc.is_empty() {
                    String::new()
                } else {
                    format!(" ({port_proc})")
                }
            ));
        }
    }

    if !ie_enabled {
        recommendations.push(
            "WinINet proxy is off — press Connect in the app or check sing-box.".into(),
        );
    } else if !ie_server.contains(&SYSTEM_PROXY_PORT.to_string()) {
        recommendations.push(format!(
            "System proxy points to «{ie_server}» instead of port {} — may conflict with VPN.",
            SYSTEM_PROXY_PORT
        ));
    }

    let wininet_ok = is_wininet_proxy_for_port(SYSTEM_PROXY_PORT);
    let winhttp_ok = is_winhttp_proxy_for_port(SYSTEM_PROXY_PORT);
    if !winhttp_ok {
        let adv_line = netsh_output(&["winhttp", "show", "advproxy"]);
        if !adv_line.is_empty() && adv_line.len() < 200 {
            recommendations.push(format!("WinHTTP advproxy: {adv_line}"));
        }
        recommendations.push(
            "WinHTTP is not using the local proxy — some Windows services may bypass VPN. Reconnect or run Check proxy.".into(),
        );
    }

    recommendations.push(
        "System proxy covers browsers and HTTP/HTTPS apps. Games, UDP/QUIC and some apps need TUN mode.".into(),
    );

    let backup_available = state_dir.is_some_and(backup_exists);

    ProxyDiagnostics {
        ie_proxy_enabled: ie_enabled,
        ie_proxy_server: ie_server,
        pac_url,
        auto_detect,
        winhttp_line,
        winhttp_autoproxy_service: autoproxy_svc,
        wininet_ok,
        winhttp_ok,
        wcmsvc_dependency_issue,
        edge_proxy_policy,
        firefox_detected,
        other_vpn_hints,
        port_2080_in_use: port_in_use,
        port_2080_pid: port_pid,
        port_2080_process: port_proc,
        backup_available,
        recommendations_en: recommendations,
    }
}

#[cfg(windows)]
fn read_service_start_type(name: &str) -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let path = format!(r"SYSTEM\CurrentControlSet\Services\{name}");
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.open_subkey(&path)
        .ok()
        .and_then(|k| k.get_value::<u32, _>("Start").ok())
        .map(|v| match v {
            2 => "Automatic",
            3 => "Manual",
            4 => "Disabled",
            _ => "Unknown",
        })
        .unwrap_or_else(|| "Unknown".into())
}

#[cfg(windows)]
fn read_wcm_depend_on_service() -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let wcm = match hklm.open_subkey(r"SYSTEM\CurrentControlSet\Services\WcmSvc") {
        Ok(k) => k,
        Err(_) => return String::new(),
    };
    wcm.get_raw_value("DependOnService")
        .ok()
        .and_then(|raw| {
            let bytes = raw.bytes;
            if bytes.len() < 4 || bytes.len() % 2 != 0 {
                return None;
            }
            let utf16: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let s = String::from_utf16_lossy(&utf16);
            Some(s.trim_end_matches('\0').to_string())
        })
        .unwrap_or_default()
}
