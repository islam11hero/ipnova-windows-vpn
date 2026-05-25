//! Support logs, recovery, and diagnostics export.

use std::process::Command;

use serde::Serialize;

use tauri::Manager;

use super::state::{
    append_support_log, log_unix_secs, singbox_binary, singbox_process_running,
    support_log_path, tail_text_file, vpn_state_dir, VpnRuntimeStatus,
};
#[cfg(windows)]
use crate::windows_proxy::{is_local_proxy_listening, SYSTEM_PROXY_PORT};

use super::status::vpn_status;

#[derive(Serialize, Clone)]
pub struct SupportLogInfo {
    pub log_path: String,
    pub log_dir: String,
    pub tail: String,
}

#[tauri::command]
pub fn vpn_support_log_info(app: tauri::AppHandle) -> Result<SupportLogInfo, String> {
    let dir = vpn_state_dir(&app)?;
    let log_path = support_log_path(&dir);
    let tail = tail_text_file(&log_path, 12_000);
    Ok(SupportLogInfo {
        log_path: log_path.to_string_lossy().into_owned(),
        log_dir: dir.to_string_lossy().into_owned(),
        tail,
    })
}

#[derive(Serialize, Clone)]
pub struct VpnRecoveryReport {
    pub orphan_proxy_cleaned: bool,
    pub stale_port_in_use: bool,
    pub singbox_running: bool,
    pub message: String,
}

/// On app start: Auto-Repair (startup mode) + port check.
#[tauri::command]
pub fn vpn_recover_stale(app: tauri::AppHandle) -> VpnRecoveryReport {
    let running = singbox_process_running();
    let mut orphan_proxy_cleaned = false;
    let mut messages: Vec<String> = Vec::new();

    #[cfg(windows)]
    {
        let port_busy = is_local_proxy_listening(SYSTEM_PROXY_PORT) && !running;

        match crate::windows_proxy::run_auto_repair(
            &app,
            crate::windows_proxy::AutoRepairMode::Startup,
        ) {
            Ok(repair) => {
                orphan_proxy_cleaned = repair.steps.iter().any(|s| {
                    s.id == "orphan_proxy" && s.status == "ok"
                });
                if !repair.summary_en.is_empty() && orphan_proxy_cleaned {
                    messages.push(repair.summary_en);
                } else if orphan_proxy_cleaned {
                    messages.push(
                        "Restored Windows proxy settings left from a previous IPNOVA session.".into(),
                    );
                }
                for s in &repair.steps {
                    if s.status == "ok" && (s.id == "pac_wpad" || s.id == "wininet_refresh") {
                        messages.push(s.detail.clone());
                    }
                }
            }
            Err(e) => {
                append_support_log(
                    &vpn_state_dir(&app).unwrap_or_default(),
                    "recover_fail",
                    &e,
                );
            }
        }

        if port_busy {
            messages.push(
                "Port 2080 is in use — close other VPN apps or restart IPNOVA before connecting.".into(),
            );
        }

        return VpnRecoveryReport {
            orphan_proxy_cleaned,
            stale_port_in_use: port_busy,
            singbox_running: running,
            message: messages.join(" "),
        };
    }

    #[cfg(not(windows))]
    VpnRecoveryReport {
        orphan_proxy_cleaned: false,
        stale_port_in_use: false,
        singbox_running: running,
        message: String::new(),
    }
}

#[tauri::command]
pub fn vpn_export_diagnostics(app: tauri::AppHandle) -> Result<String, String> {
    let dir = vpn_state_dir(&app)?;
    let runtime = vpn_status()?;

    #[cfg(windows)]
    let proxy = crate::windows_proxy::proxy_diagnostics_for_dir(Some(dir.as_path()));
    #[cfg(not(windows))]
    let proxy = crate::windows_proxy::proxy_diagnostics();

    #[cfg(windows)]
    let proxy_scenarios = {
        use crate::windows_proxy::{evaluate_proxy_scenarios, gather_scenario_inputs};
        use super::state::singbox_process_running;
        let binary = singbox_binary(&app);
        let exists = binary.exists();
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let runnable = exists
            && std::process::Command::new(&binary)
                .arg("version")
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        let input = gather_scenario_inputs(
            singbox_process_running(),
            exists,
            runnable,
            Some(dir.as_path()),
        );
        evaluate_proxy_scenarios(&input)
    };
    #[cfg(not(windows))]
    let proxy_scenarios = serde_json::Value::Null;

    #[cfg(windows)]
    let defender = {
        let singbox = singbox_binary(&app);
        let app_exe = std::env::current_exe().ok();
        serde_json::json!({
            "status": crate::windows_security::query_defender_status(&singbox),
            "wdsi_hashes": crate::windows_security::file_hashes_for_wdsi(&singbox, app_exe.as_deref()),
        })
    };
    #[cfg(not(windows))]
    let defender = serde_json::Value::Null;

    let bundle = serde_json::json!({
        "exported_at_unix": log_unix_secs(),
        "app": "ipnova-windows-vpn",
        "runtime": runtime,
        "proxy": proxy,
        "proxy_scenarios": proxy_scenarios,
        "defender": defender,
        "last_connect_log": tail_text_file(&support_log_path(&dir), 16_000),
        "singbox_stderr": tail_text_file(&dir.join("sing-box.stderr.log"), 8_000),
    });

    serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn vpn_open_support_logs(app: tauri::AppHandle) -> Result<(), String> {
    let dir = vpn_state_dir(&app)?;
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(&dir).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let _ = dir;
        return Err("Open log folder is not supported on this OS".into());
    }
    Ok(())
}
