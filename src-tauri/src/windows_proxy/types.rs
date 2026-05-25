//! Shared types (all platforms).

use serde::Serialize;

#[derive(Clone, Debug, serde::Serialize)]
pub struct ProxyEnableResult {
    pub winhttp_ok: bool,
    pub winhttp_warning: Option<String>,
    pub winhttp_method: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct ProxyConflictCheck {
    pub has_conflict: bool,
    pub proxy_enabled: bool,
    pub current_proxy_server: String,
    pub has_pac: bool,
    pub pac_url: String,
    pub auto_detect: bool,
    pub message: String,
}
