//! Pre-connect proxy conflict detection.

#[cfg(windows)]
use super::constants::SYSTEM_PROXY_PORT;
#[cfg(windows)]
use super::scenario_logic::{evaluate_proxy_conflict, WininetSnapshot};
#[cfg(windows)]
use super::types::ProxyConflictCheck;
#[cfg(windows)]
use crate::wininet_registry::read_wininet_proxy_state;

#[cfg(windows)]
pub fn check_proxy_conflict() -> ProxyConflictCheck {
    let state = read_wininet_proxy_state().unwrap_or_default();
    let snapshot = WininetSnapshot {
        enabled: state.enabled,
        server: state.server,
        auto_detect: state.auto_detect,
        pac_url: state.pac_url,
    };
    evaluate_proxy_conflict(&snapshot, SYSTEM_PROXY_PORT)
}
