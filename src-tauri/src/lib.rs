mod secure_store;
mod vpn;
#[cfg(windows)]
mod wininet_registry;
#[cfg(windows)]
mod windows_powershell;
mod windows_proxy;
mod windows_security;

use serde::Serialize;
use tauri::Manager;

#[derive(Serialize)]
pub struct DefenderExclusionResult {
    pub ok: bool,
    pub message: String,
}

#[tauri::command]
fn windows_security_status(app: tauri::AppHandle) -> Result<windows_security::WindowsSecurityStatus, String> {
    let singbox = vpn::singbox_binary(&app);
    Ok(windows_security::security_status(&app, &singbox))
}

#[tauri::command]
fn windows_add_defender_exclusions(app: tauri::AppHandle) -> Result<DefenderExclusionResult, String> {
    let mut paths = Vec::new();
    if let Ok(dir) = app.path().resource_dir() {
        paths.push(dir);
    }
    if let Ok(dir) = app.path().app_data_dir() {
        paths.push(dir);
    }
    let singbox = vpn::singbox_binary(&app);
    if let Some(parent) = singbox.parent() {
        paths.push(parent.to_path_buf());
    }

    match windows_security::request_defender_exclusions(&paths, &singbox) {
        Ok(message) => {
            let ok = !message.contains("not listed") && !message.contains("still missing");
            Ok(DefenderExclusionResult { ok, message })
        }
        Err(message) => Ok(DefenderExclusionResult { ok: false, message }),
    }
}

#[tauri::command]
fn windows_defender_status(app: tauri::AppHandle) -> windows_security::DefenderStatus {
    let singbox = vpn::singbox_binary(&app);
    windows_security::query_defender_status(&singbox)
}

#[tauri::command]
fn windows_wdsi_file_hashes(app: tauri::AppHandle) -> Vec<windows_security::WdsiFileHash> {
    let singbox = vpn::singbox_binary(&app);
    let app_exe = std::env::current_exe().ok();
    windows_security::file_hashes_for_wdsi(&singbox, app_exe.as_deref())
}

#[tauri::command]
fn windows_open_defender_settings() -> Result<(), String> {
    windows_security::open_windows_security_settings()
}

#[tauri::command]
fn windows_proxy_diagnostics(app: tauri::AppHandle) -> windows_proxy::ProxyDiagnostics {
    #[cfg(windows)]
    {
        let dir = vpn::vpn_state_dir(&app).ok();
        return windows_proxy::proxy_diagnostics_for_dir(dir.as_deref());
    }
    #[cfg(not(windows))]
    let _ = app;
    windows_proxy::proxy_diagnostics()
}

#[tauri::command]
fn windows_wcm_remediation_script() -> String {
    windows_proxy::wcm_remediation_script()
}

#[tauri::command]
fn windows_apply_wcm_fix() -> Result<String, String> {
    windows_proxy::try_apply_wcm_fix()
}

#[tauri::command]
fn windows_wcm_stack_status() -> windows_proxy::WcmStackStatus {
    windows_proxy::query_wcm_stack_status()
}

#[tauri::command]
fn windows_apply_wcm_fix_full() -> windows_proxy::WcmFixResult {
    windows_proxy::apply_wcm_fix_full()
}

#[tauri::command]
fn windows_apply_wcm_fix_uac() -> Result<windows_proxy::WcmFixResult, String> {
    windows_proxy::apply_wcm_fix_elevated()
}

#[tauri::command]
fn windows_permissions_status(
    app: tauri::AppHandle,
) -> windows_proxy::AppPermissionsStatus {
    let connected = vpn::singbox_process_running();
    let mode = vpn::active_mode_label();
    let proxy_mode = mode.as_deref() == Some("system_proxy");
    let _ = app;
    windows_proxy::query_app_permissions(connected, proxy_mode)
}

#[tauri::command]
fn windows_check_proxy_conflict() -> windows_proxy::ProxyConflictCheck {
    windows_proxy::check_proxy_conflict()
}

#[tauri::command]
fn windows_troubleshooting_checks(app: tauri::AppHandle) -> Vec<windows_proxy::TroubleshootingCheck> {
    let binary = vpn::singbox_binary(&app);
    let mut checks = windows_proxy::troubleshooting_checks(&binary, binary.exists());
    checks.extend(windows_security::defender_troubleshooting_checks(&binary));
    checks
}

#[tauri::command]
fn windows_apply_machine_proxy() -> Result<String, String> {
    windows_proxy::apply_machine_wide_wininet(windows_proxy::SYSTEM_PROXY_PORT)
}

#[tauri::command]
fn windows_retry_winhttp(app: tauri::AppHandle) -> Result<windows_proxy::WinhttpRetryResult, String> {
    let dir = vpn::vpn_state_dir(&app)?;
    windows_proxy::retry_winhttp_proxy(&dir)
}

#[tauri::command]
fn windows_auto_repair(
    app: tauri::AppHandle,
    mode: Option<String>,
) -> Result<windows_proxy::AutoRepairReport, String> {
    let repair_mode = match mode.as_deref() {
        Some("full") => windows_proxy::AutoRepairMode::Full,
        _ => windows_proxy::AutoRepairMode::Startup,
    };
    windows_proxy::run_auto_repair(&app, repair_mode)
}

#[tauri::command]
fn windows_proxy_scenarios(app: tauri::AppHandle) -> windows_proxy::ProxyScenarioReport {
    let binary = vpn::singbox_binary(&app);
    let exists = binary.exists();
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let runnable = exists
            && std::process::Command::new(&binary)
                .arg("version")
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        let running = vpn::singbox_process_running();
        let dir = vpn::vpn_state_dir(&app).ok();
        let input =
            windows_proxy::gather_scenario_inputs(running, exists, runnable, dir.as_deref());
        return windows_proxy::evaluate_proxy_scenarios(&input);
    }
    #[cfg(not(windows))]
    {
        let _ = (app, exists, binary);
        windows_proxy::evaluate_proxy_scenarios(&windows_proxy::gather_scenario_inputs(
            false, false, false, None,
        ))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            vpn::connect::vpn_connect,
            vpn::connect::vpn_disconnect,
            vpn::status::vpn_status,
            vpn::status::vpn_preflight,
            vpn::logs::vpn_support_log_info,
            vpn::logs::vpn_open_support_logs,
            vpn::logs::vpn_recover_stale,
            vpn::logs::vpn_export_diagnostics,
            secure_store::secure_store_session,
            secure_store::secure_load_session,
            secure_store::secure_clear_session,
            windows_security_status,
            windows_add_defender_exclusions,
            windows_defender_status,
            windows_wdsi_file_hashes,
            windows_open_defender_settings,
            windows_proxy_diagnostics,
            windows_wcm_remediation_script,
            windows_apply_wcm_fix,
            windows_wcm_stack_status,
            windows_apply_wcm_fix_full,
            windows_apply_wcm_fix_uac,
            windows_permissions_status,
            windows_check_proxy_conflict,
            windows_troubleshooting_checks,
            windows_apply_machine_proxy,
            windows_retry_winhttp,
            windows_proxy_scenarios,
            windows_auto_repair,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            vpn::emergency_vpn_cleanup(app_handle);
        }
    });
}
