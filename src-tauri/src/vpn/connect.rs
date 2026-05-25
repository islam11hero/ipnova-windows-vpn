//! Connect / disconnect and sing-box spawn (system proxy + TUN).

use std::path::Path;

use serde_json::Value;
use tauri::Manager;

#[cfg(windows)]
use crate::windows_proxy::{
    enable_system_proxy, evaluate_proxy_scenarios, gather_scenario_inputs, ProxyEnableResult,
    SYSTEM_PROXY_PORT,
};
#[cfg(windows)]
use crate::windows_security::spawn_singbox_elevated;

use super::state::{
    append_support_log, normalize_mode, set_active_mode, singbox_binary, spawn_singbox_process,
    stop_child, tail_text_file, vpn_state_dir, wait_for_local_proxy, write_config, VpnRuntimeStatus,
    VPN_CHILD, VPN_EXTERNAL_PID, VPN_PROXY_STATE_DIR, VPN_SYSTEM_PROXY_ACTIVE,
};
#[cfg(windows)]
use super::state::probe_exit_ip_via_local_proxy;

#[cfg(windows)]
fn winhttp_warning_message(proxy: &ProxyEnableResult) -> Option<String> {
    if proxy.winhttp_ok {
        return None;
    }
    Some(format!(
        "WinHTTP proxy was not applied ({}). Browsers usually work; some Windows services may bypass VPN.",
        proxy
            .winhttp_warning
            .as_deref()
            .unwrap_or("netsh failed")
    ))
}

#[cfg(windows)]
fn connect_system_proxy(
    app: &tauri::AppHandle,
    binary: &Path,
    config: Value,
) -> Result<VpnRuntimeStatus, String> {
    let config_path = write_config(app, config, "system_proxy")?;
    let working_dir = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| vpn_state_dir(app).unwrap_or_default());

    append_support_log(&working_dir, "connect_start", "system_proxy mode");

    let scenario_input =
        gather_scenario_inputs(false, true, true, Some(&working_dir));
    let scenario_report = evaluate_proxy_scenarios(&scenario_input);
    if let Ok(json) = serde_json::to_string(&scenario_report.scenarios) {
        append_support_log(&working_dir, "scenarios_pre", &json);
    }

    let mut child = spawn_singbox_process(binary, &config_path, &working_dir)?;
    if !wait_for_local_proxy(SYSTEM_PROXY_PORT, 12_000) {
        let _ = child.kill();
        let _ = child.wait();
        let stderr_tail = tail_text_file(&working_dir.join("sing-box.stderr.log"), 4096);
        append_support_log(
            &working_dir,
            "connect_fail",
            &format!("port {SYSTEM_PROXY_PORT} closed; stderr: {stderr_tail}"),
        );
        return Err(format!(
            "sing-box did not open port {} — check Defender or reinstall",
            SYSTEM_PROXY_PORT
        ));
    }

    let proxy_result = match enable_system_proxy(&working_dir) {
        Ok(r) => r,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = crate::windows_proxy::disable_system_proxy(&working_dir);
            append_support_log(&working_dir, "connect_fail", &format!("proxy enable: {e}"));
            return Err(e);
        }
    };
    let warning = winhttp_warning_message(&proxy_result);

    *VPN_PROXY_STATE_DIR.lock().map_err(|e| e.to_string())? = Some(working_dir.clone());
    *VPN_SYSTEM_PROXY_ACTIVE.lock().map_err(|e| e.to_string())? = true;
    *VPN_CHILD.lock().map_err(|e| e.to_string())? = Some(child);
    set_active_mode(Some("system_proxy"));

    let exit_ip = probe_exit_ip_via_local_proxy(SYSTEM_PROXY_PORT);
    let mut message = match proxy_result.winhttp_method.as_deref() {
        Some(method) => format!("Connected — system proxy (WinHTTP: {method})"),
        None => "Connected — system proxy (HTTP/HTTPS)".into(),
    };
    if exit_ip.is_none() {
        message.push_str(" — could not verify exit IP (check Marzban)");
    }
    if warning.is_some() {
        message.push_str(" — restart the browser if the IP does not change");
    }

    let scenario_input =
        gather_scenario_inputs(true, true, true, Some(&working_dir));
    let post_report = evaluate_proxy_scenarios(&scenario_input);
    if let Ok(json) = serde_json::to_string(&post_report) {
        append_support_log(&working_dir, "scenarios_post", &json);
    }

    append_support_log(
        &working_dir,
        "connect_ok",
        &format!(
            "winhttp={:?} exit_ip={:?} healthy={}",
            proxy_result.winhttp_method,
            exit_ip,
            post_report.connected_healthy
        ),
    );

    let elevation = if proxy_result.winhttp_ok {
        None
    } else {
        crate::windows_proxy::ElevationNotice::if_needed(true, "winhttp")
    };

    Ok(VpnRuntimeStatus {
        connected: true,
        message,
        mode: Some("system_proxy".into()),
        warning,
        exit_ip,
        elevation,
    })
}

#[cfg(not(windows))]
fn connect_system_proxy(
    _app: &tauri::AppHandle,
    _binary: &Path,
    _config: Value,
) -> Result<VpnRuntimeStatus, String> {
    Err("System proxy mode is not supported on this OS".into())
}

fn connect_tun(
    app: &tauri::AppHandle,
    binary: &Path,
    config: Value,
    force_elevation: bool,
) -> Result<VpnRuntimeStatus, String> {
    #[cfg(target_os = "windows")]
    {
        let wintun = binary.parent().map(|p| p.join("wintun.dll"));
        if wintun.as_ref().is_none_or(|d| !d.exists()) {
            return Err("wintun.dll missing next to sing-box.exe".into());
        }
    }

    let config_path = write_config(app, config, "tun")?;
    let working_dir = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| vpn_state_dir(app).unwrap_or_default());

    #[cfg(windows)]
    if force_elevation {
        let pid_file = working_dir.join("sing-box.pid");
        let pid = spawn_singbox_elevated(binary, &config_path, &working_dir, &pid_file)?;
        *VPN_EXTERNAL_PID.lock().map_err(|e| e.to_string())? = Some(pid);
        set_active_mode(Some("tun"));
        return Ok(VpnRuntimeStatus {
            connected: true,
            message: "Connected (TUN) — approve UAC".into(),
            mode: Some("tun".into()),
            warning: None,
            exit_ip: None,
            elevation: None,
        });
    }

    match spawn_singbox_process(binary, &config_path, &working_dir) {
        Ok(child) => {
            *VPN_CHILD.lock().map_err(|e| e.to_string())? = Some(child);
            set_active_mode(Some("tun"));
            Ok(VpnRuntimeStatus {
                connected: true,
                message: "Connected (TUN)".into(),
                mode: Some("tun".into()),
                warning: None,
                exit_ip: None,
                elevation: None,
            })
        }
        Err(first_err) => {
            #[cfg(windows)]
            {
                let pid_file = working_dir.join("sing-box.pid");
                match spawn_singbox_elevated(binary, &config_path, &working_dir, &pid_file) {
                    Ok(pid) => {
                        *VPN_EXTERNAL_PID.lock().map_err(|e| e.to_string())? = Some(pid);
                        set_active_mode(Some("tun"));
                        Ok(VpnRuntimeStatus {
                            connected: true,
                            message: "Connected (TUN) — approve UAC when prompted".into(),
                            mode: Some("tun".into()),
                            warning: None,
                            exit_ip: None,
                            elevation: None,
                        })
                    }
                    Err(elevate_err) => Err(format!(
                        "TUN failed: {first_err}. UAC denied or elevation failed: {elevate_err}. Try system proxy mode (no admin)."
                    )),
                }
            }
            #[cfg(not(windows))]
            Err(format!(
                "Failed to start sing-box: {first_err}. On Windows use system proxy or run as admin."
            ))
        }
    }
}

#[tauri::command]
pub fn vpn_disconnect() -> Result<VpnRuntimeStatus, String> {
    stop_child()?;
    Ok(VpnRuntimeStatus {
        connected: false,
        message: "Disconnected".into(),
        mode: None,
        warning: None,
        exit_ip: None,
        elevation: None,
    })
}

#[tauri::command]
pub fn vpn_connect(
    app: tauri::AppHandle,
    config: Value,
    mode: Option<String>,
) -> Result<VpnRuntimeStatus, String> {
    stop_child()?;

    let mode = normalize_mode(mode);
    let binary = singbox_binary(&app);
    if !binary.exists() {
        return Err(format!(
            "sing-box not found at {} — run scripts/download-singbox.ps1",
            binary.display()
        ));
    }

    if mode == "system_proxy" {
        return connect_system_proxy(&app, &binary, config);
    }

    if mode == "tun" {
        return connect_tun(&app, &binary, config, true);
    }

    // auto: Windows tries system proxy first, then TUN
    #[cfg(windows)]
    {
        match connect_system_proxy(&app, &binary, config.clone()) {
            Ok(status) => return Ok(status),
            Err(proxy_err) => match connect_tun(&app, &binary, config, false) {
                Ok(status) => {
                    let warning = Some(format!(
                        "System proxy failed ({proxy_err}). Connected via TUN instead — UAC may have been required."
                    ));
                    return Ok(VpnRuntimeStatus {
                        warning,
                        ..status
                    });
                }
                Err(tun_err) => {
                    return Err(format!(
                        "System proxy failed: {proxy_err}. TUN fallback failed: {tun_err}"
                    ));
                }
            },
        }
    }

    connect_tun(&app, &binary, config, false)
}
