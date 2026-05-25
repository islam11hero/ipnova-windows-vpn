//! WinHTTP retry with optional UAC elevation.

#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
use super::constants::SYSTEM_PROXY_PORT;
#[cfg(windows)]
use super::log::append_proxy_log;
#[cfg(windows)]
use super::notify::notify_proxy_change;
#[cfg(windows)]
use super::winhttp::{
    enable_winhttp_proxy, winhttp_advproxy_settings_path, write_winhttp_advproxy_settings,
};
#[cfg(windows)]
use crate::wininet_registry::PROXY_OVERRIDE;
#[cfg(windows)]
use crate::windows_security::is_process_elevated;

#[derive(serde::Serialize, Clone)]
pub struct WinhttpRetryResult {
    pub ok: bool,
    pub method: Option<String>,
    pub message: String,
    pub needs_admin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<super::elevation::ElevationNotice>,
}

#[cfg(windows)]
pub fn retry_winhttp_proxy(state_dir: &Path) -> Result<WinhttpRetryResult, String> {
    append_proxy_log(state_dir, "winhttp_retry", "start");
    let _ = write_winhttp_advproxy_settings(state_dir, SYSTEM_PROXY_PORT)?;

    if is_process_elevated() {
        return finish_winhttp_retry(state_dir, enable_winhttp_proxy(state_dir, SYSTEM_PROXY_PORT));
    }

    let settings_path = winhttp_advproxy_settings_path(state_dir);
    let result_file = state_dir.join("winhttp-retry-result.txt");
    let _ = fs::remove_file(&result_file);

    let settings_esc = settings_path.to_string_lossy().replace('\'', "''");
    let result_esc = result_file.to_string_lossy().replace('\'', "''");
    let bypass = PROXY_OVERRIDE.replace('\'', "''");
    let port = SYSTEM_PROXY_PORT;

    let inner = format!(
        r#"$ErrorActionPreference = 'Stop'
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {{
  throw 'Administrator rights required'
}}
$sf = '{settings_esc}'
$rf = '{result_esc}'
netsh winhttp set advproxy setting-scope=machine settings-file=$sf 2>&1 | Out-Null
$adv = (netsh winhttp show advproxy 2>&1 | Out-String)
if ($adv -match '127.0.0.1' -and $adv -match '{port}') {{
  'OK|advproxy-machine' | Set-Content -LiteralPath $rf -Encoding ASCII -Force
  exit 0
}}
netsh winhttp set advproxy setting-scope=currentuser settings-file=$sf 2>&1 | Out-Null
$adv2 = (netsh winhttp show advproxy 2>&1 | Out-String)
if ($adv2 -match '127.0.0.1' -and $adv2 -match '{port}') {{
  'OK|advproxy-currentuser' | Set-Content -LiteralPath $rf -Encoding ASCII -Force
  exit 0
}}
netsh winhttp set proxy 127.0.0.1:{port} '{bypass}' 2>&1 | Out-Null
$leg = (netsh winhttp show proxy 2>&1 | Out-String)
if ($leg -match '127.0.0.1' -and $leg -match '{port}') {{
  'OK|legacy' | Set-Content -LiteralPath $rf -Encoding ASCII -Force
  exit 0
}}
('FAIL|' + ($adv + $leg)) | Set-Content -LiteralPath $rf -Encoding ASCII -Force
exit 1"#
    );

    let launch = format!(
        "Start-Process powershell -Verb RunAs -Wait -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-Command',{}",
        serde_json::to_string(&inner).map_err(|e| e.to_string())?
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &launch])
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        append_proxy_log(state_dir, "winhttp_retry", "uac_denied");
        return Ok(WinhttpRetryResult {
            ok: false,
            method: None,
            message:
                "UAC was denied. Close IPNOVA, right-click → Run as administrator, then retry."
                    .into(),
            needs_admin: true,
            elevation: super::elevation::ElevationNotice::if_needed(true, "winhttp"),
        });
    }

    let raw = fs::read_to_string(&result_file).unwrap_or_default();
    let raw = raw.trim();
    if raw.starts_with("OK|") {
        let method = raw.strip_prefix("OK|").unwrap_or("advproxy").to_string();
        append_proxy_log(state_dir, "winhttp_retry_ok", &method);
        notify_proxy_change();
        return Ok(WinhttpRetryResult {
            ok: true,
            method: Some(method.clone()),
            message: format!(
                "WinHTTP proxy applied ({method}). Windows Update and system apps should use the VPN."
            ),
            needs_admin: false,
            elevation: None,
        });
    }

    let detail = raw.strip_prefix("FAIL|").unwrap_or(raw);
    append_proxy_log(state_dir, "winhttp_retry_fail", detail);
    Ok(WinhttpRetryResult {
        ok: false,
        method: None,
        message: format!(
            "WinHTTP could not be set even as administrator. {detail} Try «Apply 24H2 fix» if on Windows 11."
        ),
        needs_admin: false,
        elevation: None,
    })
}

#[cfg(windows)]
fn finish_winhttp_retry(
    state_dir: &Path,
    result: Result<String, String>,
) -> Result<WinhttpRetryResult, String> {
    match result {
        Ok(method) => {
            append_proxy_log(state_dir, "winhttp_retry_ok", &method);
            notify_proxy_change();
            Ok(WinhttpRetryResult {
                ok: true,
                method: Some(method.clone()),
                message: format!("WinHTTP proxy applied ({method})."),
                needs_admin: false,
                elevation: None,
            })
        }
        Err(e) => {
            append_proxy_log(state_dir, "winhttp_retry_fail", &e);
            Ok(WinhttpRetryResult {
                ok: false,
                method: None,
                message: e,
                needs_admin: !crate::windows_security::is_process_elevated(),
                elevation: super::elevation::ElevationNotice::if_needed(
                    !crate::windows_security::is_process_elevated(),
                    "winhttp",
                ),
            })
        }
    }
}
