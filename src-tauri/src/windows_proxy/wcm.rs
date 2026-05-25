//! Windows 11 24H2–2026 Wi‑Fi / WinHttpAutoProxySvc / WcmSvc stack.

#[cfg(windows)]
use std::process::Command;

use serde::Serialize;
#[cfg(windows)]
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_WRITE};
#[cfg(windows)]
use winreg::RegKey;

#[cfg(windows)]
use super::process::hidden_command;
#[cfg(windows)]
use super::scenario_logic::wcm_24h2_issue;
#[cfg(windows)]
use crate::windows_powershell::run_powershell;
#[cfg(windows)]
use crate::windows_security::is_process_elevated;

#[derive(Debug, Clone, Serialize, Default)]
pub struct WcmServiceState {
    pub name: String,
    pub start_type: String,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct WcmStackStatus {
    pub windows_build: Option<u32>,
    pub modern_stack: bool,
    pub winhttp_autoproxy_start: String,
    pub wcm_depend_on: String,
    pub has_dependency_conflict: bool,
    pub winhttp_autoproxy_running: bool,
    pub wcmsvc_running: bool,
    pub wlansvc_running: bool,
    pub services: Vec<WcmServiceState>,
    pub recommendation_en: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WcmFixStep {
    pub id: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WcmFixResult {
    pub ok: bool,
    pub needs_admin: bool,
    pub elevated_used: bool,
    pub steps: Vec<WcmFixStep>,
    pub message: String,
    pub status_after: Option<WcmStackStatus>,
}

pub fn wcm_remediation_script() -> String {
    r#"# IPNOVA — Windows 11 24H2+ / 2026 Wi‑Fi & proxy stack (run as Administrator)
# 1) Enable WinHttpAutoProxySvc (Manual)
Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Services\WinHttpAutoProxySvc' -Name Start -Value 3
# 2) Start services
Start-Service WinHttpAutoProxySvc -ErrorAction SilentlyContinue
Restart-Service WcmSvc -ErrorAction SilentlyContinue
Restart-Service WlanSvc -ErrorAction SilentlyContinue
Write-Host 'Done. Reboot if Wi‑Fi or system proxy is still unstable.'

# IT only — remove WinHttpAutoProxySvc from WcmSvc dependencies (rare):
# $wcm = 'HKLM:\SYSTEM\CurrentControlSet\Services\WcmSvc'
# $deps = (Get-ItemProperty $wcm -Name DependOnService).DependOnService | Where-Object { $_ -ne 'WinHttpAutoProxySvc' }
# Set-ItemProperty $wcm -Name DependOnService -Value $deps
"#
    .to_string()
}

#[cfg(windows)]
fn fix_step(id: &str, status: &str, detail: impl Into<String>) -> WcmFixStep {
    WcmFixStep {
        id: id.into(),
        status: status.into(),
        detail: detail.into(),
    }
}

#[cfg(windows)]
pub fn read_windows_build() -> Option<u32> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let ver = hklm
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .ok()?;
    let build: String = ver.get_value("CurrentBuildNumber").ok()?;
    build.parse().ok()
}

#[cfg(windows)]
fn service_start_type(name: &str) -> String {
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
fn service_running(name: &str) -> bool {
    let out = hidden_command("sc")
        .args(["query", name])
        .output()
        .ok();
    out.map(|o| String::from_utf8_lossy(&o.stdout).contains("RUNNING"))
        .unwrap_or(false)
}

#[cfg(windows)]
fn read_wcm_depend_on_service() -> String {
    let wcm = match RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SYSTEM\CurrentControlSet\Services\WcmSvc")
    {
        Ok(k) => k,
        Err(_) => return String::new(),
    };
    wcm.get_raw_value("DependOnService")
        .ok()
        .and_then(|raw| decode_multi_sz(&raw.bytes))
        .unwrap_or_default()
}

#[cfg(windows)]
fn decode_multi_sz(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 4 || bytes.len() % 2 != 0 {
        return None;
    }
    let utf16: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    Some(String::from_utf16_lossy(&utf16).trim_end_matches('\0').to_string())
}

#[cfg(windows)]
pub fn query_wcm_stack_status() -> WcmStackStatus {
    let build = read_windows_build();
    let modern_stack = build.is_some_and(|b| b >= 26_100);
    let winhttp_autoproxy_start = service_start_type("WinHttpAutoProxySvc");
    let wcm_depend_on = read_wcm_depend_on_service();
    let has_dependency_conflict =
        wcm_24h2_issue(&wcm_depend_on, &winhttp_autoproxy_start);

    let winhttp_autoproxy_running = service_running("WinHttpAutoProxySvc");
    let wcmsvc_running = service_running("WcmSvc");
    let wlansvc_running = service_running("WlanSvc");

    let services = vec![
        WcmServiceState {
            name: "WinHttpAutoProxySvc".into(),
            start_type: winhttp_autoproxy_start.clone(),
            running: winhttp_autoproxy_running,
        },
        WcmServiceState {
            name: "WcmSvc".into(),
            start_type: service_start_type("WcmSvc"),
            running: wcmsvc_running,
        },
        WcmServiceState {
            name: "WlanSvc".into(),
            start_type: service_start_type("WlanSvc"),
            running: wlansvc_running,
        },
    ];

    let recommendation_en = if has_dependency_conflict {
        "WinHttpAutoProxySvc is Disabled but WcmSvc depends on it — apply the 2026 stack fix (admin)."
            .into()
    } else if !winhttp_autoproxy_running && winhttp_autoproxy_start != "Disabled" {
        "WinHttpAutoProxySvc is not running — start it with the 2026 stack fix.".into()
    } else if modern_stack {
        "2026-era proxy stack looks OK. Use Auto-Repair if Wi‑Fi or WinHTTP still fails.".into()
    } else {
        "No WcmSvc dependency conflict detected on this build.".into()
    };

    WcmStackStatus {
        windows_build: build,
        modern_stack,
        winhttp_autoproxy_start,
        wcm_depend_on,
        has_dependency_conflict,
        winhttp_autoproxy_running,
        wcmsvc_running,
        wlansvc_running,
        services,
        recommendation_en,
    }
}

#[cfg(not(windows))]
pub fn query_wcm_stack_status() -> WcmStackStatus {
    WcmStackStatus {
        windows_build: None,
        modern_stack: false,
        winhttp_autoproxy_start: String::new(),
        wcm_depend_on: String::new(),
        has_dependency_conflict: false,
        winhttp_autoproxy_running: false,
        wcmsvc_running: false,
        wlansvc_running: false,
        services: vec![],
        recommendation_en: "Windows only.".into(),
    }
}

#[cfg(windows)]
fn set_service_start_manual(name: &str) -> Result<(), String> {
    let path = format!(r"SYSTEM\CurrentControlSet\Services\{name}");
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(&path, KEY_WRITE)
        .map_err(|e| format!("{name}: {e}"))?;
    key.set_value("Start", &3u32).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn start_service(name: &str) -> Result<(), String> {
    let status = hidden_command("sc")
        .args(["start", name])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() || service_running(name) {
        Ok(())
    } else {
        Err(format!("sc start {name} failed"))
    }
}

#[cfg(windows)]
fn restart_service(name: &str) -> Result<(), String> {
    let _ = hidden_command("sc").args(["stop", name]).status();
    std::thread::sleep(std::time::Duration::from_millis(400));
    start_service(name)
}

#[cfg(windows)]
fn apply_wcm_fix_registry_and_services() -> Result<Vec<WcmFixStep>, String> {
    let mut steps = Vec::new();

    match set_service_start_manual("WinHttpAutoProxySvc") {
        Ok(()) => steps.push(fix_step(
            "registry",
            "ok",
            "WinHttpAutoProxySvc → Manual (Start=3)",
        )),
        Err(e) => {
            steps.push(fix_step("registry", "fail", e.clone()));
            return Err(e);
        }
    }

    match start_service("WinHttpAutoProxySvc") {
        Ok(()) => steps.push(fix_step(
            "start_autoproxy",
            "ok",
            "WinHttpAutoProxySvc started",
        )),
        Err(e) => steps.push(fix_step(
            "start_autoproxy",
            "warn",
            format!("{e} — reboot may be required"),
        )),
    }

    match restart_service("WcmSvc") {
        Ok(()) => steps.push(fix_step("restart_wcm", "ok", "WcmSvc restarted")),
        Err(e) => steps.push(fix_step("restart_wcm", "warn", e)),
    }

    match restart_service("WlanSvc") {
        Ok(()) => steps.push(fix_step("restart_wlan", "ok", "WlanSvc restarted")),
        Err(e) => steps.push(fix_step("restart_wlan", "warn", e)),
    }

    Ok(steps)
}

#[cfg(windows)]
pub fn apply_wcm_fix_full() -> WcmFixResult {
    let before = query_wcm_stack_status();
    if !is_process_elevated() {
        return WcmFixResult {
            ok: false,
            needs_admin: true,
            elevated_used: false,
            steps: vec![fix_step(
                "admin",
                "fail",
                "Administrator rights required for registry and services",
            )],
            message: "Run IPNOVA as administrator, or use «Apply 2026 fix (UAC)».".into(),
            status_after: Some(before),
        };
    }

    match apply_wcm_fix_registry_and_services() {
        Ok(steps) => {
            let after = query_wcm_stack_status();
            let ok = !after.has_dependency_conflict;
            WcmFixResult {
                ok,
                needs_admin: false,
                elevated_used: true,
                message: if ok {
                    "2026 proxy stack repaired. Reconnect Wi‑Fi or reboot if needed.".into()
                } else {
                    "Partial fix applied — reboot or use the admin script.".into()
                },
                status_after: Some(after),
                steps,
            }
        }
        Err(e) => WcmFixResult {
            ok: false,
            needs_admin: true,
            elevated_used: true,
            steps: vec![fix_step("error", "fail", e.clone())],
            message: e,
            status_after: Some(query_wcm_stack_status()),
        },
    }
}

#[cfg(windows)]
pub fn apply_wcm_fix_elevated() -> Result<WcmFixResult, String> {
    if is_process_elevated() {
        return Ok(apply_wcm_fix_full());
    }

    let result_file = std::env::temp_dir().join("ipnova-wcm-fix-result.txt");
    let _ = std::fs::remove_file(&result_file);
    let rf = result_file.to_string_lossy().replace('\'', "''");

    let inner = format!(
        r#"$ErrorActionPreference = 'Stop'
$rf = '{rf}'
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {{
  throw 'Administrator required'
}}
Set-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Services\WinHttpAutoProxySvc' -Name Start -Value 3
Start-Service WinHttpAutoProxySvc -ErrorAction SilentlyContinue
Restart-Service WcmSvc -ErrorAction SilentlyContinue
Restart-Service WlanSvc -ErrorAction SilentlyContinue
'OK|2026 stack fix applied (services restarted)' | Set-Content -LiteralPath $rf -Encoding ASCII -Force"#
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
        return Ok(WcmFixResult {
            ok: false,
            needs_admin: true,
            elevated_used: false,
            steps: vec![fix_step("uac", "fail", "UAC denied")],
            message: "UAC was denied. Right-click IPNOVA → Run as administrator.".into(),
            status_after: Some(query_wcm_stack_status()),
        });
    }

    let raw = std::fs::read_to_string(&result_file).unwrap_or_default();
    if raw.starts_with("OK|") {
        let after = query_wcm_stack_status();
        return Ok(WcmFixResult {
            ok: !after.has_dependency_conflict,
            needs_admin: false,
            elevated_used: true,
            steps: vec![
                fix_step("uac", "ok", "Elevated repair completed"),
                fix_step(
                    "registry",
                    "ok",
                    "WinHttpAutoProxySvc Manual + services restarted",
                ),
            ],
            message: raw
                .strip_prefix("OK|")
                .unwrap_or("2026 stack fix applied.")
                .to_string(),
            status_after: Some(after),
        });
    }

    Ok(WcmFixResult {
        ok: false,
        needs_admin: true,
        elevated_used: false,
        steps: vec![fix_step("uac", "fail", raw)],
        message: "Elevated repair failed.".into(),
        status_after: Some(query_wcm_stack_status()),
    })
}

/// Legacy API — registry only when elevated; PowerShell fallback otherwise.
#[cfg(windows)]
pub fn try_apply_wcm_fix() -> Result<String, String> {
    let result = apply_wcm_fix_full();
    if result.ok {
        Ok(result.message)
    } else if result.needs_admin {
        apply_wcm_fix_elevated().map(|r| r.message)
    } else {
        Err(result.message)
    }
}

#[cfg(not(windows))]
pub fn apply_wcm_fix_full() -> WcmFixResult {
    WcmFixResult {
        ok: false,
        needs_admin: false,
        elevated_used: false,
        steps: vec![],
        message: "Windows only.".into(),
        status_after: None,
    }
}

#[cfg(not(windows))]
pub fn apply_wcm_fix_elevated() -> Result<WcmFixResult, String> {
    Ok(apply_wcm_fix_full())
}

#[cfg(not(windows))]
pub fn try_apply_wcm_fix() -> Result<String, String> {
    Err("Windows only".into())
}
