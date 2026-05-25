use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::Manager;

#[cfg(windows)]
use crate::windows_powershell::run_powershell;

#[derive(Serialize, Clone)]
pub struct WindowsSecurityStatus {
    pub is_admin: bool,
    pub can_elevate: bool,
    pub install_dir: String,
    pub singbox_path: String,
    pub app_data_dir: String,
    pub defender_exclusions_recommended: bool,
    pub smartscreen_help_url: String,
    pub wdsi_submit_url: String,
}

#[cfg(windows)]
pub fn is_process_elevated() -> bool {
    use std::mem::size_of;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
pub fn is_process_elevated() -> bool {
    false
}

#[cfg(not(windows))]
pub fn security_status(app: &tauri::AppHandle, singbox: &Path) -> WindowsSecurityStatus {
    let _ = (app, singbox);
    WindowsSecurityStatus {
        is_admin: false,
        can_elevate: false,
        install_dir: String::new(),
        singbox_path: String::new(),
        app_data_dir: String::new(),
        defender_exclusions_recommended: false,
        smartscreen_help_url: String::new(),
        wdsi_submit_url: String::new(),
    }
}

#[cfg(windows)]
pub fn security_status(app: &tauri::AppHandle, singbox: &Path) -> WindowsSecurityStatus {
    let install_dir = app
        .path()
        .resource_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".".into());

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    WindowsSecurityStatus {
        is_admin: is_process_elevated(),
        can_elevate: cfg!(windows),
        install_dir,
        singbox_path: singbox.to_string_lossy().into_owned(),
        app_data_dir,
        defender_exclusions_recommended: true,
        smartscreen_help_url:
            "https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation"
                .into(),
        wdsi_submit_url: "https://www.microsoft.com/en-us/wdsi/filesubmission".into(),
    }
}

#[derive(Serialize, Clone)]
pub struct DefenderStatus {
    pub real_time_protection_enabled: bool,
    pub tamper_protection_enabled: bool,
    pub controlled_folder_access_enabled: bool,
    pub smart_app_control_state: String,
    pub singbox_exists: bool,
    pub singbox_excluded: bool,
    pub wintun_exists: bool,
    pub recommendations_en: Vec<String>,
}

#[cfg(windows)]
fn path_in_defender_exclusions(target: &Path) -> bool {
    let path = target.to_string_lossy().replace('\'', "''");
    let script = format!(
        r#"$t = (Resolve-Path -LiteralPath '{path}' -ErrorAction SilentlyContinue).Path
if (-not $t) {{ 'false'; exit }}
$norm = $t.TrimEnd('\')
$prefs = Get-MpPreference -ErrorAction SilentlyContinue
$hit = $false
foreach ($e in @($prefs.ExclusionPath)) {{
  if ($e -and ($norm -like ($e.TrimEnd('\') + '*'))) {{ $hit = $true; break }}
}}
if ($hit) {{ 'true' }} else {{ 'false' }}"#
    );
    run_powershell(&script)
        .map(|s| s.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[cfg(windows)]
pub fn query_defender_status(singbox: &Path) -> DefenderStatus {
    let singbox_exists = singbox.exists();
    let wintun = singbox
        .parent()
        .map(|p| p.join("wintun.dll"))
        .filter(|p| p.exists())
        .is_some();

    let sac_script = r#"
$v = (Get-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy' -Name VerifiedAndReputablePolicyState -ErrorAction SilentlyContinue).VerifiedAndReputablePolicyState
switch ($v) { 0 { 'off' } 1 { 'evaluation' } 2 { 'on' } default { 'unknown' } }
"#;
    let smart_app_control_state = run_powershell(sac_script).unwrap_or_else(|_| "unknown".into());

    let mp_script = r#"
$s = Get-MpComputerStatus -ErrorAction SilentlyContinue
if (-not $s) { '|||false' ; exit }
$rtp = if ($s.RealTimeProtectionEnabled) { 'true' } else { 'false' }
$tp = if ($s.IsTamperProtected) { 'true' } else { 'false' }
$cfa = 'false'
try {
  $cfaMode = (Get-MpPreference -ErrorAction SilentlyContinue).EnableControlledFolderAccess
  if ($cfaMode -eq 1) { $cfa = 'true' }
} catch {}
Write-Output ($rtp + '|' + $tp + '|' + $cfa)
"#;
    let mp_line = run_powershell(mp_script).unwrap_or_default();
    let mut mp = mp_line.splitn(3, '|');
    let real_time_protection_enabled = mp.next().unwrap_or("false") == "true";
    let tamper_protection_enabled = mp.next().unwrap_or("false") == "true";
    let controlled_folder_access_enabled = mp.next().unwrap_or("false") == "true";

    let singbox_excluded = singbox_exists && path_in_defender_exclusions(singbox);

    let mut recommendations = Vec::new();
    if !singbox_exists {
        recommendations.push(
            "sing-box.exe is missing — run scripts/download-singbox.ps1 or restore from Defender quarantine."
                .into(),
        );
    } else if !singbox_excluded {
        recommendations.push(
            "sing-box is not in Defender exclusions — use «Add Defender exclusion» (requires UAC)."
                .into(),
        );
    }
    if tamper_protection_enabled {
        recommendations.push(
            "Tamper Protection is on — exclusions need Administrator approval via UAC.".into(),
        );
    }
    if smart_app_control_state == "on" {
        recommendations.push(
            "Smart App Control is ON — unsigned or low-reputation binaries may be blocked. Sign the app or turn SAC off in Windows Security.".into(),
        );
    } else if smart_app_control_state == "evaluation" {
        recommendations.push(
            "Smart App Control is in evaluation — check CodeIntegrity event log if sing-box is blocked.".into(),
        );
    }
    if controlled_folder_access_enabled && !singbox_excluded {
        recommendations.push(
            "Controlled folder access is on — add sing-box to allowed apps when applying exclusions.".into(),
        );
    }
    if !real_time_protection_enabled {
        recommendations.push(
            "Real-time protection is off — another antivirus may be active.".into(),
        );
    }

    DefenderStatus {
        real_time_protection_enabled,
        tamper_protection_enabled,
        controlled_folder_access_enabled,
        smart_app_control_state,
        singbox_exists,
        singbox_excluded,
        wintun_exists: wintun,
        recommendations_en: recommendations,
    }
}

#[cfg(not(windows))]
pub fn query_defender_status(_singbox: &Path) -> DefenderStatus {
    DefenderStatus {
        real_time_protection_enabled: false,
        tamper_protection_enabled: false,
        controlled_folder_access_enabled: false,
        smart_app_control_state: "n/a".into(),
        singbox_exists: false,
        singbox_excluded: false,
        wintun_exists: false,
        recommendations_en: vec!["Windows Defender status is Windows-only.".into()],
    }
}

#[cfg(windows)]
pub fn defender_troubleshooting_checks(singbox: &Path) -> Vec<crate::windows_proxy::TroubleshootingCheck> {
    use crate::windows_proxy::TroubleshootingCheck;

    let d = query_defender_status(singbox);
    let mut checks = vec![
        TroubleshootingCheck {
            id: "defender_rtp".into(),
            title: "Defender real-time protection".into(),
            status: if d.real_time_protection_enabled {
                "pass"
            } else {
                "warn"
            }
            .into(),
            detail: if d.real_time_protection_enabled {
                "Enabled.".into()
            } else {
                "Disabled or managed by another AV.".into()
            },
        },
        TroubleshootingCheck {
            id: "defender_exclusion".into(),
            title: "sing-box Defender exclusion".into(),
            status: if !d.singbox_exists {
                "fail"
            } else if d.singbox_excluded {
                "pass"
            } else {
                "warn"
            }
            .into(),
            detail: if !d.singbox_exists {
                "sing-box.exe not found.".into()
            } else if d.singbox_excluded {
                "Install path is excluded.".into()
            } else {
                "Not excluded — tap Add Defender exclusion.".into()
            },
        },
        TroubleshootingCheck {
            id: "smart_app_control".into(),
            title: "Smart App Control".into(),
            status: if d.smart_app_control_state == "on" {
                "warn"
            } else {
                "pass"
            }
            .into(),
            detail: format!("State: {}.", d.smart_app_control_state),
        },
    ];

    if d.tamper_protection_enabled {
        checks.push(TroubleshootingCheck {
            id: "tamper_protection".into(),
            title: "Tamper Protection".into(),
            status: "warn".into(),
            detail: "Exclusions require admin UAC — approve the elevation prompt.".into(),
        });
    }

    checks
}

#[cfg(not(windows))]
pub fn defender_troubleshooting_checks(
    _singbox: &Path,
) -> Vec<crate::windows_proxy::TroubleshootingCheck> {
    vec![]
}

#[derive(Serialize, Clone)]
pub struct WdsiFileHash {
    pub path: String,
    pub sha256: String,
    pub exists: bool,
}

#[cfg(windows)]
pub fn file_hashes_for_wdsi(singbox: &Path, app_exe: Option<&Path>) -> Vec<WdsiFileHash> {
    let mut targets: Vec<PathBuf> = Vec::new();
    if singbox.exists() {
        targets.push(singbox.to_path_buf());
    }
    if let Some(dir) = singbox.parent() {
        let wintun = dir.join("wintun.dll");
        if wintun.exists() {
            targets.push(wintun);
        }
    }
    if let Some(exe) = app_exe {
        if exe.exists() {
            targets.push(exe.to_path_buf());
        }
    }

    let mut out = Vec::new();
    for path in targets {
        let escaped = path.to_string_lossy().replace('\'', "''");
        let script = format!(
            r#"$p = '{escaped}'
if (-not (Test-Path -LiteralPath $p)) {{ Write-Output 'MISSING|' ; exit }}
$h = (Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash
Write-Output ('OK|' + $h)"#
        );
        let line = run_powershell(&script).unwrap_or_else(|_| "MISSING|".into());
        let mut parts = line.splitn(2, '|');
        let tag = parts.next().unwrap_or("MISSING");
        let hash = parts.next().unwrap_or("").to_string();
        out.push(WdsiFileHash {
            path: path.to_string_lossy().into_owned(),
            sha256: if tag == "OK" { hash } else { String::new() },
            exists: tag == "OK",
        });
    }
    out
}

#[cfg(not(windows))]
pub fn file_hashes_for_wdsi(_singbox: &Path, _app_exe: Option<&Path>) -> Vec<WdsiFileHash> {
    vec![]
}

/// Launch elevated PowerShell once to add Defender exclusions (user UAC consent).
#[cfg(windows)]
pub fn request_defender_exclusions(paths: &[PathBuf], singbox: &Path) -> Result<String, String> {
    if paths.is_empty() {
        return Err("No paths to exclude".into());
    }

    let singbox_esc = singbox.to_string_lossy().replace('\'', "''");
    let wintun_esc = singbox
        .parent()
        .map(|p| p.join("wintun.dll"))
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().replace('\'', "''"))
        .unwrap_or_default();

    let mut script = String::from(
        "$ErrorActionPreference='Stop'; \
         if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { \
           throw 'Administrator required'; \
         }; \
         $paths = @(",
    );
    for (i, p) in paths.iter().enumerate() {
        if i > 0 {
            script.push(',');
        }
        let escaped = p.to_string_lossy().replace('\'', "''");
        script.push('\'');
        script.push_str(&escaped);
        script.push('\'');
    }
    script.push_str(
        "); \
         foreach ($p in $paths) { \
           if (Test-Path $p) { Add-MpPreference -ExclusionPath $p -ErrorAction SilentlyContinue }; \
         }; \
         $sb = '",
    );
    script.push_str(&singbox_esc);
    script.push_str(
        "'; \
         if (Test-Path -LiteralPath $sb) { \
           Add-MpPreference -ExclusionPath $sb -ErrorAction SilentlyContinue; \
           Add-MpPreference -ExclusionProcess (Split-Path -Leaf $sb) -ErrorAction SilentlyContinue; \
           Add-MpPreference -ControlledFolderAccessAllowedApplications $sb -ErrorAction SilentlyContinue; \
         };",
    );
    if !wintun_esc.is_empty() {
        script.push_str(" $wt = '");
        script.push_str(&wintun_esc);
        script.push_str(
            "'; \
             if (Test-Path -LiteralPath $wt) { \
               Add-MpPreference -ExclusionPath $wt -ErrorAction SilentlyContinue; \
             };",
        );
    }
    script.push_str(" Write-Output 'OK'");

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &format!(
                "Start-Process powershell -Verb RunAs -Wait -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-Command',{}",
                serde_json::to_string(&script).map_err(|e| e.to_string())?
            ),
        ])
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err("UAC was denied or PowerShell failed".into());
    }

    if !singbox.exists() {
        return Ok(
            "Exclusions added, but sing-box.exe is still missing — run download-singbox.ps1 or restore from quarantine."
                .into(),
        );
    }

    if path_in_defender_exclusions(singbox) {
        Ok("Defender exclusions applied (paths, sing-box process, Controlled folder access).".into())
    } else {
        Ok(
            "UAC completed but sing-box path is not listed in exclusions — Tamper Protection or company policy may block changes."
                .into(),
        )
    }
}

#[cfg(not(windows))]
pub fn request_defender_exclusions(_paths: &[PathBuf], _singbox: &Path) -> Result<String, String> {
    Err("Defender exclusions are Windows-only".into())
}

#[cfg(windows)]
pub fn open_windows_security_settings() -> Result<(), String> {
    Command::new("cmd")
        .args(["/C", "start", "ms-settings:windowsdefender"])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(windows))]
pub fn open_windows_security_settings() -> Result<(), String> {
    Err("Windows only".into())
}

/// Spawn sing-box elevated (UAC once per connect) — returns PID written by helper.
#[cfg(windows)]
pub fn spawn_singbox_elevated(
    binary: &Path,
    config_path: &Path,
    workdir: &Path,
    pid_file: &Path,
) -> Result<u32, String> {
    let binary_s = binary.to_string_lossy();
    let config_s = config_path.to_string_lossy();
    let workdir_s = workdir.to_string_lossy();
    let pid_s = pid_file.to_string_lossy();

    let ps = format!(
        "$p = Start-Process -FilePath '{binary_s}' \
         -ArgumentList 'run','-c','{config_s}','-D','{workdir_s}' \
         -WorkingDirectory '{workdir_s}' \
         -Verb RunAs -PassThru -WindowStyle Hidden; \
         if (-not $p) {{ throw 'Start-Process failed' }}; \
         $p.Id | Out-File -FilePath '{pid_s}' -Encoding ascii -NoNewline"
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps])
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err("UAC denied or failed to start sing-box elevated".into());
    }

    std::thread::sleep(std::time::Duration::from_millis(800));
    let pid_text = std::fs::read_to_string(pid_file).map_err(|e| e.to_string())?;
    pid_text
        .trim()
        .parse::<u32>()
        .map_err(|_| "Could not read sing-box PID".into())
}

#[cfg(not(windows))]
pub fn spawn_singbox_elevated(
    _binary: &Path,
    _config_path: &Path,
    _workdir: &Path,
    _pid_file: &Path,
) -> Result<u32, String> {
    Err("Elevation is Windows-only".into())
}

#[cfg(windows)]
pub fn kill_pid(pid: u32) -> Result<(), String> {
    let status = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("taskkill failed for pid {pid}"))
    }
}

#[cfg(not(windows))]
pub fn kill_pid(_pid: u32) -> Result<(), String> {
    Ok(())
}
