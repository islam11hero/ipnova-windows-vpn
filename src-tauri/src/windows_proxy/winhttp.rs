//! WinHTTP proxy (advproxy, legacy netsh, import IE).

#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use super::process::hidden_command;
#[cfg(windows)]
use crate::wininet_registry::{proxy_server_value, PROXY_OVERRIDE};

#[cfg(windows)]
pub fn netsh_output(args: &[&str]) -> String {
    hidden_command("netsh")
        .args(args)
        .output()
        .map(|o| {
            format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            )
            .trim()
            .to_string()
        })
        .unwrap_or_default()
}

#[cfg(windows)]
pub fn winhttp_advproxy_settings_path(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join("winhttp-advproxy-active.json")
}

#[cfg(windows)]
pub fn write_winhttp_advproxy_settings(state_dir: &Path, port: u16) -> Result<std::path::PathBuf, String> {
    fs::create_dir_all(state_dir).map_err(|e| e.to_string())?;
    let path = winhttp_advproxy_settings_path(state_dir);
    let settings = serde_json::json!({
        "ProxyIsEnabled": true,
        "Proxy": proxy_server_value(port),
        "ProxyBypass": PROXY_OVERRIDE,
        "AutoConfigIsEnabled": false,
        "AutoDetect": false
    });
    fs::write(
        &path,
        serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(path)
}

#[cfg(windows)]
pub fn enable_winhttp_advproxy(state_dir: &Path, port: u16) -> Result<(), String> {
    let path = write_winhttp_advproxy_settings(state_dir, port)?;
    let path_arg = format!("settings-file={}", path.to_string_lossy());

    let machine_ok = hidden_command("netsh")
        .args([
            "winhttp",
            "set",
            "advproxy",
            "setting-scope=machine",
            &path_arg,
        ])
        .status()
        .map_err(|e| e.to_string())?
        .success()
        && is_winhttp_proxy_for_port(port);

    if machine_ok {
        return Ok(());
    }

    let user_ok = hidden_command("netsh")
        .args([
            "winhttp",
            "set",
            "advproxy",
            "setting-scope=currentuser",
            &path_arg,
        ])
        .status()
        .map_err(|e| e.to_string())?
        .success()
        && is_winhttp_proxy_for_port(port);

    if user_ok {
        return Ok(());
    }

    Err(format!(
        "advproxy failed (machine + currentuser): {}",
        netsh_output(&["winhttp", "show", "advproxy"])
    ))
}

#[cfg(windows)]
pub fn enable_winhttp_legacy(port: u16) -> Result<(), String> {
    let proxy = format!("127.0.0.1:{port}");
    let status = hidden_command("netsh")
        .args(["winhttp", "set", "proxy", &proxy, PROXY_OVERRIDE])
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err("netsh winhttp set proxy failed".into())
    }
}

#[cfg(windows)]
pub fn enable_winhttp_proxy(state_dir: &Path, port: u16) -> Result<String, String> {
    if enable_winhttp_advproxy(state_dir, port).is_ok() && is_winhttp_proxy_for_port(port)
    {
        return Ok("advproxy".into());
    }

    if enable_winhttp_legacy(port).is_ok() && is_winhttp_proxy_for_port(port) {
        return Ok("legacy".into());
    }

    let import = hidden_command("netsh")
        .args(["winhttp", "import", "proxy", "source=ie"])
        .status()
        .map_err(|e| e.to_string())?;
    if import.success() && is_winhttp_proxy_for_port(port) {
        return Ok("import_ie".into());
    }

    Err("Failed to set WinHTTP proxy (advproxy, legacy, and import ie)".into())
}

#[cfg(windows)]
pub fn reset_winhttp_stack() {
    let _ = hidden_command("netsh")
        .args(["winhttp", "reset", "advproxy"])
        .status();
    let _ = hidden_command("netsh")
        .args(["winhttp", "reset", "proxy"])
        .status();
}

#[cfg(windows)]
pub fn disable_winhttp_proxy() -> Result<(), String> {
    reset_winhttp_stack();
    Ok(())
}

#[cfg(windows)]
pub fn parse_winhttp_show_proxy(output: &str) -> (bool, Option<String>, Option<String>) {
    super::scenario_logic::parse_winhttp_show_proxy(output)
}

/// True when WinHTTP (legacy or advproxy) points at our local mixed inbound port.
#[cfg(windows)]
pub fn is_winhttp_proxy_for_port(port: u16) -> bool {
    let port_s = port.to_string();
    let legacy = netsh_output(&["winhttp", "show", "proxy"]);
    if legacy.contains("127.0.0.1") && legacy.contains(&port_s) {
        return true;
    }
    let adv = netsh_output(&["winhttp", "show", "advproxy"]);
    adv.contains("127.0.0.1") && adv.contains(&port_s)
}
