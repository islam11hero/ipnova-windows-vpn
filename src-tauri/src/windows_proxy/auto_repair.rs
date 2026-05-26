//! Auto-Repair — orphan proxy, WinINet/WinHTTP, and Windows 11 24H2–2026 stack issues.

#[cfg(windows)]
use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoRepairMode {
    /// App startup: safe fixes only (no UAC, no DNS flush).
    Startup,
    /// User-triggered or post-connect: full repair including 2026 stack.
    Full,
}

#[derive(Debug, Clone, Serialize)]
pub struct AutoRepairStep {
    pub id: String,
    pub title: String,
    /// `ok` | `warn` | `skip` | `fail`
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AutoRepairReport {
    pub ok: bool,
    pub mode: String,
    pub steps: Vec<AutoRepairStep>,
    pub summary_en: String,
    /// User should tap Connect (or Reconnect) after repair.
    pub recommend_reconnect: bool,
    pub issues_2026_found: Vec<String>,
    /// True when Full (or related) repair could not finish without admin/UAC.
    pub needs_admin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<super::elevation::ElevationNotice>,
}

fn step(id: &str, title: &str, status: &str, detail: impl Into<String>) -> AutoRepairStep {
    AutoRepairStep {
        id: id.into(),
        title: title.into(),
        status: status.into(),
        detail: detail.into(),
    }
}

#[cfg(windows)]
fn read_windows_build() -> Option<u32> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let ver = hklm
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .ok()?;
    let build: String = ver.get_value("CurrentBuildNumber").ok()?;
    build.parse().ok()
}

#[cfg(windows)]
fn is_modern_windows_stack() -> bool {
    read_windows_build().is_some_and(|b| b >= 26_100)
}

#[cfg(windows)]
fn detect_issues_2026(input: &super::scenarios::ScenarioInputs) -> Vec<String> {
    let mut issues = Vec::new();
    if is_modern_windows_stack() {
        issues.push("Windows 11 24H2+ build (26100+) — enhanced proxy stack".into());
    }
    if input.wcm_issue {
        issues.push("WcmSvc ↔ WinHttpAutoProxySvc dependency (24H2/2025/2026 Wi‑Fi)".into());
    }
    if super::scenario_logic::has_pac_or_wpad(&input.wininet) {
        issues.push("PAC/WPAD still active — browsers may ignore manual proxy".into());
    }
    if input.edge_policy.is_some() {
        issues.push("Microsoft Edge enterprise proxy policy".into());
    }
    if input.gpo_hint.is_some() {
        issues.push("WinINet locked by group policy".into());
    }
    if !input.winhttp_ok && input.singbox_running {
        issues.push("WinHTTP not on VPN port while connected (2026 system apps bypass)".into());
    }
    if input.port_listening
        && !input.singbox_running
        && super::scenario_logic::port_holder_blocks_connect(&input.port_process, false)
    {
        issues.push("Port 2080 held by another process".into());
    }
    issues
}

#[cfg(windows)]
fn flush_dns_cache() -> Result<String, String> {
    use super::process::hidden_command;
    let status = hidden_command("ipconfig")
        .args(["/flushdns"])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok("DNS resolver cache flushed".into())
    } else {
        Err("ipconfig /flushdns failed".into())
    }
}

#[cfg(windows)]
fn clear_stale_advproxy_file(state_dir: &Path) -> bool {
    let path = super::winhttp::winhttp_advproxy_settings_path(state_dir);
    if path.exists() {
        return std::fs::remove_file(&path).is_ok();
    }
    false
}

#[cfg(windows)]
pub fn run_auto_repair(
    app: &tauri::AppHandle,
    mode: AutoRepairMode,
) -> Result<AutoRepairReport, String> {
    use super::constants::SYSTEM_PROXY_PORT;
    use super::log::append_proxy_log;
    use super::notify::notify_proxy_change;
    use super::scenarios::{evaluate_proxy_scenarios, gather_scenario_inputs};
    use super::system::{disable_system_proxy, enable_system_proxy, reset_orphaned_proxies};
    use super::winhttp::{is_winhttp_proxy_for_port, reset_winhttp_stack};
    use super::wininet::{is_wininet_proxy_for_port, refresh_wininet_connections};
    use crate::vpn::singbox_binary;
    use crate::vpn::{singbox_process_running, vpn_state_dir};
    use crate::vpn::state::{VPN_CHILD, VPN_PROXY_STATE_DIR, VPN_SYSTEM_PROXY_ACTIVE};
    use crate::wininet_registry::wininet_has_pac_or_wpad;

    let state_dir = vpn_state_dir(app)?;
    let binary = singbox_binary(app);
    let exists = binary.exists();
    let runnable = if exists {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new(&binary)
            .arg("version")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    };

    let singbox_running = singbox_process_running()
        || VPN_CHILD
            .lock()
            .ok()
            .map(|g| g.is_some())
            .unwrap_or(false);
    let proxy_active = VPN_SYSTEM_PROXY_ACTIVE
        .lock()
        .ok()
        .is_some_and(|g| *g);

    append_proxy_log(
        &state_dir,
        "auto_repair",
        &format!("mode={mode:?} running={singbox_running}"),
    );

    let mut steps = Vec::new();
    let mut issues_2026 = Vec::new();
    let mut admin_reason: Option<String> = None;

    let mut input = gather_scenario_inputs(
        singbox_running,
        exists,
        runnable,
        Some(state_dir.as_path()),
    );
    issues_2026 = detect_issues_2026(&input);

    // 1 — Orphan proxy (crash / force-kill)
    if !singbox_running
        && (is_wininet_proxy_for_port(SYSTEM_PROXY_PORT)
            || is_winhttp_proxy_for_port(SYSTEM_PROXY_PORT))
    {
        let _ = disable_system_proxy(&state_dir);
        let (ie, wh) = reset_orphaned_proxies(SYSTEM_PROXY_PORT);
        if let Ok(mut g) = VPN_SYSTEM_PROXY_ACTIVE.lock() {
            *g = false;
        }
        if let Ok(mut g) = VPN_PROXY_STATE_DIR.lock() {
            *g = None;
        }
        steps.push(step(
            "orphan_proxy",
            "Orphan system proxy",
            "ok",
            format!(
                "Restored proxy after abnormal exit (WinINet={ie}, WinHTTP={wh})"
            ),
        ));
    } else {
        steps.push(step(
            "orphan_proxy",
            "Orphan system proxy",
            "skip",
            "No orphan proxy detected",
        ));
    }

    // 2 — PAC / WPAD (2026 browsers + corporate networks)
    if wininet_has_pac_or_wpad() {
        match refresh_wininet_connections() {
            Ok(()) => steps.push(step(
                "pac_wpad",
                "PAC / WPAD",
                "ok",
                "Cleared auto-config and refreshed WinINet (Chromium/Edge)",
            )),
            Err(e) => steps.push(step(
                "pac_wpad",
                "PAC / WPAD",
                "warn",
                format!("Could not clear PAC/WPAD: {e}"),
            )),
        }
    } else {
        steps.push(step(
            "pac_wpad",
            "PAC / WPAD",
            "skip",
            "Not enabled",
        ));
    }

    // 3 — WinINet refresh (2026 Chromium Connections store)
    match refresh_wininet_connections() {
        Ok(()) => steps.push(step(
            "wininet_refresh",
            "Browser proxy refresh",
            "ok",
            "WinINet + WM_SETTINGCHANGE broadcast",
        )),
        Err(e) => steps.push(step(
            "wininet_refresh",
            "Browser proxy refresh",
            "warn",
            e,
        )),
    }

    if mode == AutoRepairMode::Full {
        // 4 — 2026 stack: WinHttpAutoProxySvc / WcmSvc
        if input.wcm_issue {
            let wcm = if crate::windows_security::is_process_elevated() {
                super::wcm::apply_wcm_fix_full()
            } else {
                super::wcm::apply_wcm_fix_elevated().unwrap_or_else(|e| super::wcm::WcmFixResult {
                    ok: false,
                    needs_admin: true,
                    elevated_used: false,
                    steps: vec![],
                    message: e,
                    status_after: None,
                })
            };
            if !wcm.ok && wcm.needs_admin && !wcm.elevated_used {
                admin_reason = Some("wcm_2026".into());
            }
            let st = if wcm.ok { "ok" } else { "warn" };
            let detail = if wcm.elevated_used && wcm.ok {
                format!("{} (UAC)", wcm.message)
            } else {
                format!("{} — Settings → Permissions → Apply 2026 fix", wcm.message)
            };
            steps.push(step("wcm_2026", "2026 proxy stack (Wi‑Fi)", st, detail));
        } else {
            steps.push(step(
                "wcm_2026",
                "2026 proxy stack (Wi‑Fi)",
                "skip",
                "WinHttpAutoProxySvc dependency OK",
            ));
        }

        // 5 — Reset stale WinHTTP advproxy artifact
        if clear_stale_advproxy_file(&state_dir) {
            reset_winhttp_stack();
            steps.push(step(
                "winhttp_reset",
                "WinHTTP cache",
                "ok",
                "Removed stale advproxy settings and reset WinHTTP",
            ));
        } else {
            steps.push(step(
                "winhttp_reset",
                "WinHTTP cache",
                "skip",
                "No stale advproxy file",
            ));
        }

        // 6 — Re-apply proxy if VPN session active
        if singbox_running && proxy_active {
            match enable_system_proxy(&state_dir) {
                Ok(r) => {
                    let detail = if r.winhttp_ok {
                        format!(
                            "WinINet + WinHTTP reapplied ({})",
                            r.winhttp_method.as_deref().unwrap_or("ok")
                        )
                    } else {
                        format!(
                            "WinINet OK; WinHTTP: {}",
                            r.winhttp_warning.as_deref().unwrap_or("needs admin retry")
                        )
                    };
                    let st = if r.winhttp_ok { "ok" } else { "warn" };
                    if !r.winhttp_ok {
                        admin_reason.get_or_insert_with(|| "winhttp".into());
                    }
                    steps.push(step("reapply_vpn_proxy", "VPN proxy", st, detail));
                }
                Err(e) => steps.push(step(
                    "reapply_vpn_proxy",
                    "VPN proxy",
                    "fail",
                    e,
                )),
            }
        } else if singbox_running {
            match super::retry::retry_winhttp_proxy(&state_dir) {
                Ok(r) if r.ok => steps.push(step(
                    "winhttp_retry",
                    "WinHTTP retry",
                    "ok",
                    r.message,
                )),
                Ok(r) => {
                    if !r.ok {
                        admin_reason.get_or_insert_with(|| "winhttp".into());
                    }
                    steps.push(step(
                        "winhttp_retry",
                        "WinHTTP retry",
                        if r.ok { "ok" } else { "warn" },
                        r.message,
                    ))
                }
                Err(e) => {
                    admin_reason.get_or_insert_with(|| "winhttp".into());
                    steps.push(step("winhttp_retry", "WinHTTP retry", "warn", e))
                }
            }
        } else {
            steps.push(step(
                "reapply_vpn_proxy",
                "VPN proxy",
                "skip",
                "Not connected — Connect after repair",
            ));
        }

        // 7 — DNS flush (2026: stale DNS after proxy changes)
        match flush_dns_cache() {
            Ok(msg) => steps.push(step("dns_flush", "DNS cache", "ok", msg)),
            Err(e) => steps.push(step("dns_flush", "DNS cache", "warn", e)),
        }
    }

    notify_proxy_change();

    input = gather_scenario_inputs(
        singbox_running,
        exists,
        runnable,
        Some(state_dir.as_path()),
    );
    let report = evaluate_proxy_scenarios(&input);
    issues_2026 = detect_issues_2026(&input);

    let has_fail = steps.iter().any(|s| s.status == "fail");
    let has_ok = steps.iter().any(|s| s.status == "ok");
    let ok = has_ok && !has_fail;

    let recommend_reconnect =
        !singbox_running || !report.connected_healthy || steps.iter().any(|s| s.id == "orphan_proxy" && s.status == "ok");

    let summary_en: String = if ok && report.connected_healthy {
        "Auto-Repair completed — VPN proxy is healthy.".to_string()
    } else if ok {
        "Auto-Repair completed — tap Connect or Reconnect.".to_string()
    } else if has_ok {
        "Auto-Repair partially applied — see steps; admin may be required for WinHTTP/Wi‑Fi."
            .to_string()
    } else {
        "Auto-Repair could not fix all issues — check Settings → Proxy diagnostics.".to_string()
    };

    append_proxy_log(&state_dir, "auto_repair_done", &summary_en);

    let mode_label: String = match mode {
        AutoRepairMode::Startup => "startup".to_string(),
        AutoRepairMode::Full => "full".to_string(),
    };

    let elevation = admin_reason
        .as_deref()
        .and_then(|r| super::elevation::ElevationNotice::if_needed(true, r));
    let needs_admin = elevation.is_some();

    Ok(AutoRepairReport {
        ok,
        mode: mode_label,
        steps,
        summary_en,
        recommend_reconnect,
        issues_2026_found: issues_2026,
        needs_admin,
        elevation,
    })
}

#[cfg(not(windows))]
pub fn run_auto_repair(_app: &tauri::AppHandle, mode: AutoRepairMode) -> Result<AutoRepairReport, String> {
    let mode_label = match mode {
        AutoRepairMode::Startup => "startup",
        AutoRepairMode::Full => "full",
    };
    Ok(AutoRepairReport {
        ok: false,
        mode: mode_label,
        steps: vec![step(
            "platform",
            "Platform",
            "skip",
            "Auto-Repair is Windows-only",
        )],
        summary_en: "Auto-Repair is only available in the Windows desktop app.".into(),
        recommend_reconnect: false,
        issues_2026_found: vec![],
        needs_admin: false,
        elevation: None,
    })
}
