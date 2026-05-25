//! End-to-end proxy scenarios (preflight, connected health, troubleshooting).

use std::path::Path;

use serde::Serialize;

use super::constants::SYSTEM_PROXY_PORT;
use super::scenario_logic::{
    has_pac_or_wpad, is_foreign_manual_proxy, is_singbox_process_name, port_holder_blocks_connect,
    server_points_at_local_port, wcm_24h2_issue, WininetSnapshot,
};

#[derive(Debug, Clone, Serialize)]
pub struct ProxyScenario {
    pub id: String,
    pub title: String,
    /// `pass` | `warn` | `fail` | `skip`
    pub status: String,
    pub detail: String,
    pub blocks_connect: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProxyScenarioReport {
    pub ready_for_connect: bool,
    pub connected_healthy: bool,
    pub scenarios: Vec<ProxyScenario>,
}

fn scen(
    id: &str,
    title: &str,
    status: &str,
    detail: impl Into<String>,
    blocks_connect: bool,
) -> ProxyScenario {
    ProxyScenario {
        id: id.into(),
        title: title.into(),
        status: status.into(),
        detail: detail.into(),
        blocks_connect,
    }
}

/// Inputs gathered once; safe to call from preflight without extra syscalls per scenario.
#[derive(Clone, Debug)]
pub struct ScenarioInputs {
    pub port: u16,
    pub singbox_running: bool,
    pub singbox_exists: bool,
    pub singbox_runnable: bool,
    pub port_listening: bool,
    pub port_pid: Option<u32>,
    pub port_process: String,
    pub wininet: WininetSnapshot,
    pub wininet_ok: bool,
    pub winhttp_ok: bool,
    pub gpo_hint: Option<String>,
    pub edge_policy: Option<String>,
    pub firefox_installed: bool,
    pub other_vpn: Vec<String>,
    pub wcm_issue: bool,
    pub backup_available: bool,
    pub elevated: bool,
}

impl Default for ScenarioInputs {
    fn default() -> Self {
        Self {
            port: SYSTEM_PROXY_PORT,
            singbox_running: false,
            singbox_exists: false,
            singbox_runnable: false,
            port_listening: false,
            port_pid: None,
            port_process: String::new(),
            wininet: WininetSnapshot::default(),
            wininet_ok: false,
            winhttp_ok: false,
            gpo_hint: None,
            edge_policy: None,
            firefox_installed: false,
            other_vpn: Vec::new(),
            wcm_issue: false,
            backup_available: false,
            elevated: false,
        }
    }
}

#[cfg(windows)]
pub fn gather_scenario_inputs(
    singbox_running: bool,
    singbox_exists: bool,
    singbox_runnable: bool,
    state_dir: Option<&Path>,
) -> ScenarioInputs {
    use super::backup::backup_exists;
    use super::hints::{
        detect_edge_proxy_policy, detect_firefox_installed, detect_other_vpn_hints,
    };
    use super::probe::{is_local_proxy_listening, port_listener_info};
    use super::winhttp::is_winhttp_proxy_for_port;
    use super::wininet::is_wininet_proxy_for_port;
    use crate::wininet_registry::read_wininet_proxy_state;
    use crate::windows_security::is_process_elevated;

    let port = SYSTEM_PROXY_PORT;
    let wininet_state = read_wininet_proxy_state().unwrap_or_default();
    let wininet = WininetSnapshot {
        enabled: wininet_state.enabled,
        server: wininet_state.server,
        auto_detect: wininet_state.auto_detect,
        pac_url: wininet_state.pac_url,
    };

    let port_listening = is_local_proxy_listening(port);
    let (port_pid, port_process) = if port_listening {
        port_listener_info(port)
    } else {
        (None, String::new())
    };

    let autoproxy = read_service_start_type("WinHttpAutoProxySvc");
    let wcm_deps = read_wcm_depend_on_service();

    ScenarioInputs {
        port,
        singbox_running,
        singbox_exists,
        singbox_runnable,
        port_listening,
        port_pid,
        port_process,
        wininet: wininet.clone(),
        wininet_ok: is_wininet_proxy_for_port(port),
        winhttp_ok: is_winhttp_proxy_for_port(port),
        gpo_hint: crate::wininet_registry::wininet_gpo_hint(),
        edge_policy: detect_edge_proxy_policy(),
        firefox_installed: detect_firefox_installed(),
        other_vpn: detect_other_vpn_hints(),
        wcm_issue: wcm_24h2_issue(&wcm_deps, &autoproxy),
        backup_available: state_dir.is_some_and(backup_exists),
        elevated: is_process_elevated(),
    }
}

#[cfg(not(windows))]
pub fn gather_scenario_inputs(
    _singbox_running: bool,
    singbox_exists: bool,
    singbox_runnable: bool,
    _state_dir: Option<&Path>,
) -> ScenarioInputs {
    ScenarioInputs {
        singbox_exists,
        singbox_runnable,
        ..Default::default()
    }
}

/// Evaluate all known system-proxy scenarios from gathered inputs.
pub fn evaluate_proxy_scenarios(input: &ScenarioInputs) -> ProxyScenarioReport {
    let mut scenarios = Vec::new();
    let port = input.port;

    // --- Connect prerequisites ---
    scenarios.push(if !input.singbox_exists {
        scen(
            "singbox_file",
            "sing-box binary",
            "fail",
            "sing-box.exe missing — run scripts/download-singbox.ps1",
            true,
        )
    } else {
        scen(
            "singbox_file",
            "sing-box binary",
            "pass",
            "sing-box.exe found",
            false,
        )
    });

    scenarios.push(if !input.singbox_exists {
        scen(
            "singbox_run",
            "sing-box can execute",
            "skip",
            "Install sing-box first",
            false,
        )
    } else if !input.singbox_runnable {
        scen(
            "singbox_run",
            "sing-box can execute",
            "fail",
            "Blocked or quarantined — add Defender exclusion",
            true,
        )
    } else {
        scen(
            "singbox_run",
            "sing-box can execute",
            "pass",
            "Version command OK",
            false,
        )
    });

    if input.port_listening {
        let blocks = port_holder_blocks_connect(&input.port_process, input.singbox_running);
        let (status, detail) = if input.singbox_running {
            (
                "pass",
                format!("sing-box listening on {port} (expected while connected)"),
            )
        } else if is_singbox_process_name(&input.port_process) {
            ("warn", format!("Port {port}: sing-box still listening — restart IPNOVA if Connect fails"))
        } else if let Some(pid) = input.port_pid {
            let proc = if input.port_process.is_empty() {
                String::new()
            } else {
                format!(" · {}", input.port_process)
            };
            if blocks {
                (
                    "fail",
                    format!("Port {port} used by PID {pid}{proc} — close it before Connect"),
                )
            } else {
                ("warn", format!("Port {port} in use (PID {pid}{proc})"))
            }
        } else {
            (
                "fail",
                format!("Port {port} in use — close other VPN apps or restart IPNOVA"),
            )
        };
        scenarios.push(scen(
            "port_2080",
            "Local port 2080",
            status,
            detail,
            blocks,
        ));
    } else {
        scenarios.push(scen(
            "port_2080",
            "Local port 2080",
            "pass",
            format!("Port {port} free for sing-box mixed inbound"),
            false,
        ));
    }

    // --- WinINet / PAC / GPO ---
    if has_pac_or_wpad(&input.wininet) {
        let detail = if input.wininet.pac_url.trim().is_empty() {
            "WPAD auto-detect on — disabled on Connect".into()
        } else {
            format!("PAC: {}", input.wininet.pac_url.trim())
        };
        scenarios.push(scen("pac_wpad", "PAC / WPAD", "warn", detail, false));
    } else {
        scenarios.push(scen(
            "pac_wpad",
            "PAC / WPAD",
            "pass",
            "No PAC or auto-detect",
            false,
        ));
    }

    if is_foreign_manual_proxy(&input.wininet, port) {
        scenarios.push(scen(
            "foreign_proxy",
            "Third-party manual proxy",
            "warn",
            format!(
                "Current proxy «{}» — replaced with 127.0.0.1:{port} while connected",
                input.wininet.server.trim()
            ),
            false,
        ));
    }

    if let Some(ref gpo) = input.gpo_hint {
        scenarios.push(scen(
            "wininet_gpo",
            "WinINet group policy",
            "warn",
            gpo.clone(),
            false,
        ));
    }

    if let Some(ref edge) = input.edge_policy {
        scenarios.push(scen(
            "edge_policy",
            "Microsoft Edge policy",
            "warn",
            edge.clone(),
            false,
        ));
    }

    // --- While connected / after Connect ---
    if input.singbox_running {
        scenarios.push(if input.wininet_ok {
            scen(
                "wininet",
                "WinINet (browsers)",
                "pass",
                format!("127.0.0.1:{port}"),
                false,
            )
        } else if server_points_at_local_port(&input.wininet.server, port) && !input.wininet.enabled
        {
            scen(
                "wininet",
                "WinINet (browsers)",
                "warn",
                "ProxyServer set but ProxyEnable is off — retry Connect",
                false,
            )
        } else {
            scen(
                "wininet",
                "WinINet (browsers)",
                "fail",
                "Browser proxy not on VPN port — check GPO/PAC or restart browser",
                false,
            )
        });

        scenarios.push(if input.winhttp_ok {
            scen(
                "winhttp",
                "WinHTTP (system apps)",
                "pass",
                "Uses local proxy",
                false,
            )
        } else if input.elevated {
            scen(
                "winhttp",
                "WinHTTP (system apps)",
                "warn",
                "WinHTTP not applied — use Retry WinHTTP in Settings",
                false,
            )
        } else {
            scen(
                "winhttp",
                "WinHTTP (system apps)",
                "warn",
                "WinHTTP not applied — Retry WinHTTP (admin) or run app as administrator",
                false,
            )
        });
    } else {
        scenarios.push(scen(
            "wininet",
            "WinINet (browsers)",
            "skip",
            "Check after Connect",
            false,
        ));
        scenarios.push(scen(
            "winhttp",
            "WinHTTP (system apps)",
            "skip",
            "Check after Connect",
            false,
        ));
    }

    if !input.other_vpn.is_empty() {
        scenarios.push(scen(
            "other_vpn",
            "Other VPN software",
            "warn",
            input.other_vpn.join("; "),
            false,
        ));
    } else {
        scenarios.push(scen(
            "other_vpn",
            "Other VPN software",
            "pass",
            "No conflicting VPN detected",
            false,
        ));
    }

    if input.wcm_issue {
        scenarios.push(scen(
            "wcm_24h2",
            "Windows 11 24H2 Wi‑Fi",
            "warn",
            "WinHttpAutoProxySvc disabled but WcmSvc depends on it — Apply 24H2 fix",
            false,
        ));
    }

    if input.firefox_installed {
        scenarios.push(scen(
            "firefox",
            "Mozilla Firefox",
            "warn",
            "Use «Use system proxy settings» in Firefox or switch to TUN mode",
            false,
        ));
    }

    scenarios.push(if input.backup_available {
        scen(
            "backup",
            "Proxy backup",
            "pass",
            "proxy-backup.json present — settings restore on disconnect",
            false,
        )
    } else {
        scen(
            "backup",
            "Proxy backup",
            "skip",
            "Created on first successful Connect",
            false,
        )
    });

    let ready_for_connect = !scenarios.iter().any(|s| s.blocks_connect);
    let connected_healthy = input.singbox_running
        && input.wininet_ok
        && scenarios
            .iter()
            .filter(|s| s.id == "wininet" || s.id == "winhttp")
            .all(|s| s.status == "pass");

    ProxyScenarioReport {
        ready_for_connect,
        connected_healthy,
        scenarios,
    }
}

/// Human-readable messages for `vpn_preflight` (blocks Connect only).
pub fn preflight_blocking_messages(input: &ScenarioInputs) -> Vec<String> {
    let report = evaluate_proxy_scenarios(input);
    report
        .scenarios
        .into_iter()
        .filter(|s| s.blocks_connect)
        .map(|s| s.detail)
        .collect()
}

/// Map scenarios to troubleshooting checklist rows (Settings → live checks).
pub fn scenarios_to_troubleshooting_checks(report: &ProxyScenarioReport) -> Vec<super::troubleshooting::TroubleshootingCheck> {
    report
        .scenarios
        .iter()
        .filter(|s| {
            matches!(
                s.id.as_str(),
                "singbox_file"
                    | "singbox_run"
                    | "port_2080"
                    | "pac_wpad"
                    | "other_vpn"
                    | "wininet"
                    | "winhttp"
                    | "wcm_24h2"
            )
        })
        .map(|s| super::troubleshooting::TroubleshootingCheck {
            id: s.id.clone(),
            title: s.title.clone(),
            status: s.status.clone(),
            detail: s.detail.clone(),
        })
        .collect()
}

#[cfg(windows)]
fn read_service_start_type(name: &str) -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let path = format!(r"SYSTEM\CurrentControlSet\Services\{name}");
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(&path)
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
    let wcm = match RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SYSTEM\CurrentControlSet\Services\WcmSvc")
    {
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
            Some(String::from_utf16_lossy(&utf16).trim_end_matches('\0').to_string())
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_when_port_free_and_singbox_ok() {
        let input = ScenarioInputs {
            port: 2080,
            singbox_exists: true,
            singbox_runnable: true,
            port_listening: false,
            ..Default::default()
        };
        let report = evaluate_proxy_scenarios(&input);
        assert!(report.ready_for_connect);
    }

    #[test]
    fn blocked_when_port_held_by_other_app() {
        let input = ScenarioInputs {
            port: 2080,
            singbox_exists: true,
            singbox_runnable: true,
            port_listening: true,
            port_pid: Some(999),
            port_process: "openvpn".into(),
            ..Default::default()
        };
        let report = evaluate_proxy_scenarios(&input);
        assert!(!report.ready_for_connect);
        assert!(report.scenarios.iter().any(|s| s.id == "port_2080" && s.blocks_connect));
    }

    #[test]
    fn connected_healthy_requires_wininet_and_winhttp() {
        let input = ScenarioInputs {
            port: 2080,
            singbox_running: true,
            wininet_ok: true,
            winhttp_ok: true,
            ..Default::default()
        };
        let report = evaluate_proxy_scenarios(&input);
        assert!(report.connected_healthy);
    }
}
