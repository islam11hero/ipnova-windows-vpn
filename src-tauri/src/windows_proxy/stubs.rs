//! Non-Windows stubs (compile on macOS/Linux for `cargo check`).

use std::path::Path;

use serde_json::Value;

use super::constants::SYSTEM_PROXY_PORT;
use super::diagnostics::ProxyDiagnostics;
use super::retry::WinhttpRetryResult;
use super::troubleshooting::TroubleshootingCheck;
use super::types::{ProxyConflictCheck, ProxyEnableResult};

pub fn troubleshooting_checks(_singbox_path: &Path, _singbox_exists: bool) -> Vec<TroubleshootingCheck> {
    vec![]
}

pub fn apply_machine_wide_wininet(_port: u16) -> Result<String, String> {
    Err("Windows only".into())
}

pub fn detect_other_vpn_hints() -> Vec<String> {
    vec![]
}

pub fn harden_singbox_config(_config: &mut Value) {}

pub fn wcm_remediation_script() -> String {
    String::new()
}

pub fn check_proxy_conflict() -> ProxyConflictCheck {
    ProxyConflictCheck {
        has_conflict: false,
        proxy_enabled: false,
        current_proxy_server: String::new(),
        has_pac: false,
        pac_url: String::new(),
        auto_detect: false,
        message: String::new(),
    }
}

pub fn try_apply_wcm_fix() -> Result<String, String> {
    Err("Windows only".into())
}

pub fn prepare_system_proxy_config(_config: &mut Value) {}

pub fn is_winhttp_proxy_for_port(_port: u16) -> bool {
    false
}

pub fn is_local_proxy_listening(_port: u16) -> bool {
    false
}

pub fn is_wininet_proxy_for_port(_port: u16) -> bool {
    false
}

pub fn enable_system_proxy(_state_dir: &Path) -> Result<ProxyEnableResult, String> {
    Err("System proxy mode is Windows-only".into())
}

pub fn disable_system_proxy(_state_dir: &Path) -> Result<(), String> {
    Ok(())
}

pub fn retry_winhttp_proxy(_state_dir: &Path) -> Result<WinhttpRetryResult, String> {
    Ok(WinhttpRetryResult {
        ok: false,
        method: None,
        message: "WinHTTP retry is Windows-only.".into(),
        needs_admin: false,
        elevation: None,
    })
}

pub fn proxy_diagnostics() -> ProxyDiagnostics {
    ProxyDiagnostics {
        ie_proxy_enabled: false,
        ie_proxy_server: String::new(),
        pac_url: String::new(),
        auto_detect: false,
        winhttp_line: String::new(),
        winhttp_autoproxy_service: String::new(),
        wininet_ok: false,
        winhttp_ok: false,
        wcmsvc_dependency_issue: false,
        edge_proxy_policy: None,
        firefox_detected: false,
        other_vpn_hints: vec![],
        port_2080_in_use: false,
        port_2080_pid: None,
        port_2080_process: String::new(),
        backup_available: false,
        recommendations_en: vec!["System proxy mode is Windows-only.".into()],
    }
}

pub fn reset_winhttp_if_orphaned(_port: u16) -> bool {
    false
}

pub fn reset_orphaned_proxies(_port: u16) -> (bool, bool) {
    (false, false)
}

pub fn preflight_port_message(_port: u16, _singbox_running: bool) -> Option<String> {
    None
}

#[allow(dead_code)]
pub const SYSTEM_PROXY_PORT_STUB: u16 = SYSTEM_PROXY_PORT;
