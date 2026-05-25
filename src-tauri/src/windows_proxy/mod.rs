//! Windows system-proxy mode (no admin / no TUN).
//!
//! WinINet (browsers) + WinHTTP (system services) → sing-box mixed on 127.0.0.1:2080.

mod config;
mod constants;
mod diagnostics;
mod log;
mod auto_repair;
mod scenario_logic;
mod scenarios;
#[cfg(windows)]
mod process;
mod probe;
mod retry;
mod types;
mod elevation;
mod permissions;
mod wcm;

#[cfg(windows)]
mod backup;
#[cfg(windows)]
mod conflict;
#[cfg(windows)]
mod hints;
#[cfg(windows)]
mod machine;
#[cfg(windows)]
mod notify;
#[cfg(windows)]
mod system;
#[cfg(windows)]
mod troubleshooting;
#[cfg(windows)]
mod winhttp;
#[cfg(windows)]
mod wininet;

#[cfg(not(windows))]
mod stubs;

pub use config::{harden_singbox_config, prepare_system_proxy_config};
pub use constants::SYSTEM_PROXY_PORT;
pub use diagnostics::ProxyDiagnostics;
pub use retry::WinhttpRetryResult;
pub use troubleshooting::TroubleshootingCheck;
pub use types::{ProxyConflictCheck, ProxyEnableResult};

pub use probe::is_local_proxy_listening;
pub use auto_repair::{run_auto_repair, AutoRepairMode, AutoRepairReport, AutoRepairStep};
pub use elevation::ElevationNotice;
pub use scenarios::{
    evaluate_proxy_scenarios, gather_scenario_inputs, preflight_blocking_messages,
    ProxyScenario, ProxyScenarioReport,
};

#[cfg(windows)]
pub use backup::{backup_exists, backup_proxy_settings};
#[cfg(windows)]
pub use conflict::check_proxy_conflict;
#[cfg(windows)]
pub use diagnostics::{proxy_diagnostics, proxy_diagnostics_for_dir};
#[cfg(windows)]
pub use hints::detect_other_vpn_hints;
#[cfg(windows)]
pub use machine::apply_machine_wide_wininet;
#[cfg(windows)]
pub use probe::preflight_port_message;
#[cfg(windows)]
pub use retry::retry_winhttp_proxy;
#[cfg(windows)]
pub use system::{
    disable_system_proxy, enable_system_proxy, reset_orphaned_proxies,
    reset_winhttp_if_orphaned,
};
#[cfg(windows)]
pub use troubleshooting::troubleshooting_checks;
#[cfg(windows)]
pub use permissions::{query_app_permissions, AppPermissionsStatus, PermissionCapability};
pub use wcm::{
    apply_wcm_fix_elevated, apply_wcm_fix_full, query_wcm_stack_status, try_apply_wcm_fix,
    wcm_remediation_script, WcmFixResult, WcmStackStatus,
};
#[cfg(windows)]
pub use winhttp::is_winhttp_proxy_for_port;
#[cfg(windows)]
pub use wininet::is_wininet_proxy_for_port;

#[cfg(not(windows))]
pub use stubs::*;
