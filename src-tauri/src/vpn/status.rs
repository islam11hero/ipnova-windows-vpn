//! Runtime status and preflight checks.

use std::process::Command;

use tauri::Manager;

use super::state::{
    active_mode_label, append_support_log, cleanup_active_proxy, set_active_mode,
    singbox_binary, singbox_process_running, tail_text_file, vpn_state_dir, VpnPreflight,
    VpnRuntimeStatus, VPN_CHILD, VPN_EXTERNAL_PID, VPN_PROXY_STATE_DIR,
};
#[cfg(windows)]
use crate::windows_proxy::{
    gather_scenario_inputs, is_local_proxy_listening, is_winhttp_proxy_for_port,
    is_wininet_proxy_for_port, preflight_blocking_messages, SYSTEM_PROXY_PORT,
};
#[cfg(windows)]
use super::state::probe_exit_ip_via_local_proxy;

#[cfg(windows)]
fn winhttp_status_warning() -> Option<String> {
    if is_winhttp_proxy_for_port(SYSTEM_PROXY_PORT) {
        return None;
    }
    Some(
        "WinHTTP is not using the VPN proxy — browsers may work but Windows Update and some apps may not."
            .into(),
    )
}

#[tauri::command]
pub fn vpn_preflight(app: tauri::AppHandle) -> VpnPreflight {
    let binary = singbox_binary(&app);
    let singbox_found = binary.exists();
    let mut messages = Vec::new();
    let mut singbox_runnable = false;

    if singbox_found {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            singbox_runnable = Command::new(&binary)
                .arg("version")
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        }
        #[cfg(not(target_os = "windows"))]
        {
            singbox_runnable = Command::new(&binary)
                .arg("version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
        }
    }

    #[cfg(not(windows))]
    {
        if !singbox_found {
            messages.push(format!(
                "sing-box not found at {} — run scripts/download-singbox.ps1",
                binary.display()
            ));
        } else if !singbox_runnable {
            messages.push(
                "sing-box is present but cannot run — check permissions or reinstall.".into(),
            );
        }
    }

    #[cfg(windows)]
    if singbox_found
        && !singbox_runnable
        && binary.parent().is_some_and(|p| p.exists())
    {
        messages.push(
            "The sing-box folder exists but the .exe cannot run — check Windows Defender quarantine."
                .into(),
        );
    }

    let state_dir = match vpn_state_dir(&app) {
        Ok(d) => Some(d),
        Err(e) => {
            messages.push(format!("Cannot write app data: {e}"));
            None
        }
    };

    #[cfg(windows)]
    {
        let child_active = VPN_CHILD.lock().map(|g| g.is_some()).unwrap_or(false);
        let running = singbox_process_running() || child_active;
        let input = gather_scenario_inputs(
            running,
            singbox_found,
            singbox_runnable,
            state_dir.as_deref(),
        );
        messages.extend(preflight_blocking_messages(&input));
    }

    #[cfg(windows)]
    let port_available = {
        let child_active = VPN_CHILD.lock().map(|g| g.is_some()).unwrap_or(false);
        let running = singbox_process_running() || child_active;
        !is_local_proxy_listening(SYSTEM_PROXY_PORT) || running
    };
    #[cfg(not(windows))]
    let port_available = true;

    let ready = singbox_found && singbox_runnable && port_available && messages.is_empty();

    VpnPreflight {
        ready,
        singbox_found,
        singbox_runnable,
        port_available,
        messages,
    }
}

#[tauri::command]
pub fn vpn_status() -> Result<VpnRuntimeStatus, String> {
    let mode = active_mode_label();

    #[cfg(windows)]
    {
        if let Ok(pid_guard) = VPN_EXTERNAL_PID.lock() {
            if let Some(pid) = *pid_guard {
                let alive = Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {pid}")])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
                    .unwrap_or(false);
                if alive {
                    return Ok(VpnRuntimeStatus {
                        connected: true,
                        message: "Connected (elevated sing-box)".into(),
                        mode: mode.or(Some("tun".into())),
                        warning: None,
                        exit_ip: None,
                        elevation: None,
                    });
                }
                *VPN_EXTERNAL_PID.lock().map_err(|e| e.to_string())? = None;
                set_active_mode(None);
            }
        }
    }

    let mut guard = VPN_CHILD.lock().map_err(|e| e.to_string())?;
    if let Some(child) = guard.as_mut() {
        match child.try_wait() {
            Ok(Some(status)) => {
                let log_dir = VPN_PROXY_STATE_DIR.lock().ok().and_then(|g| g.clone());
                *guard = None;
                if let Some(d) = log_dir.as_ref() {
                    let stderr_tail = tail_text_file(&d.join("sing-box.stderr.log"), 2048);
                    append_support_log(
                        d,
                        "singbox_exit",
                        &format!("status={status}; stderr: {stderr_tail}"),
                    );
                }
                #[cfg(windows)]
                cleanup_active_proxy();
                set_active_mode(None);
                Ok(VpnRuntimeStatus {
                    connected: false,
                    message: format!("sing-box exited ({status}) — proxy restored"),
                    mode: None,
                    warning: None,
                    exit_ip: None,
                    elevation: None,
                })
            }
            Ok(None) => {
                #[cfg(windows)]
                {
                    let proxy_mode = mode.as_deref() == Some("system_proxy");
                    if proxy_mode {
                        let listening = is_local_proxy_listening(SYSTEM_PROXY_PORT);
                        let ie_ok = is_wininet_proxy_for_port(SYSTEM_PROXY_PORT);
                        let winhttp_ok = is_winhttp_proxy_for_port(SYSTEM_PROXY_PORT);
                        if !listening || !ie_ok {
                            cleanup_active_proxy();
                            return Ok(VpnRuntimeStatus {
                                connected: false,
                                message: "VPN was interrupted — system proxy restored. Tap Connect.".into(),
                                mode: None,
                                warning: None,
                                exit_ip: None,
                                elevation: None,
                            });
                        }
                        let exit_ip = probe_exit_ip_via_local_proxy(SYSTEM_PROXY_PORT);
                        let warning = if winhttp_ok {
                            None
                        } else {
                            winhttp_status_warning()
                        };
                        let elevation = if winhttp_ok {
                            None
                        } else {
                            crate::windows_proxy::ElevationNotice::if_needed(true, "winhttp")
                        };
                        return Ok(VpnRuntimeStatus {
                            connected: true,
                            message: "Connected — system proxy".into(),
                            mode: Some("system_proxy".into()),
                            warning,
                            exit_ip,
                            elevation,
                        });
                    }
                }
                Ok(VpnRuntimeStatus {
                    connected: true,
                    message: "Connected".into(),
                    mode: mode.clone(),
                    warning: None,
                    exit_ip: None,
                    elevation: None,
                })
            }
            Err(e) => Err(e.to_string()),
        }
    } else {
        Ok(VpnRuntimeStatus {
            connected: false,
            message: "Disconnected".into(),
            mode: None,
            warning: None,
            exit_ip: None,
            elevation: None,
        })
    }
}
