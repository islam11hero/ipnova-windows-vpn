//! VPN core: sing-box lifecycle, system proxy, diagnostics.

mod connect;
mod logs;
mod state;
mod status;

pub use connect::{vpn_connect, vpn_disconnect};
pub use logs::{
    vpn_export_diagnostics, vpn_open_support_logs, vpn_recover_stale, vpn_support_log_info,
    SupportLogInfo, VpnRecoveryReport,
};
pub use state::{
    active_mode_label, emergency_vpn_cleanup, singbox_binary, singbox_process_running,
    vpn_state_dir, VpnPreflight, VpnRuntimeStatus, VPN_CHILD, VPN_EXTERNAL_PID,
    VPN_PROXY_STATE_DIR, VPN_SYSTEM_PROXY_ACTIVE,
};
pub use status::{vpn_preflight, vpn_status};
