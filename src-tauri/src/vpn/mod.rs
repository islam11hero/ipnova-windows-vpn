//! VPN core: sing-box lifecycle, system proxy, diagnostics.

pub mod connect;
pub mod logs;
pub mod state;
pub mod status;

pub use connect::{vpn_connect, vpn_disconnect};
pub use logs::{
    vpn_export_diagnostics, vpn_open_support_logs, vpn_recover_stale, vpn_support_log_info,
    SupportLogInfo, VpnRecoveryReport,
};
pub use state::{
    active_mode_label, emergency_vpn_cleanup, singbox_binary, singbox_process_running,
    vpn_state_dir, VpnPreflight, VpnRuntimeStatus,
};
pub use status::{vpn_preflight, vpn_status};
