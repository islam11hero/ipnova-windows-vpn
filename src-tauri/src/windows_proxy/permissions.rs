//! App permissions matrix (admin / UAC) for Settings UI.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PermissionCapability {
    pub id: String,
    pub title_en: String,
    pub needs_admin: bool,
    pub needs_uac: bool,
    pub available_now: bool,
    pub hint_en: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppPermissionsStatus {
    pub is_admin: bool,
    pub can_request_uac: bool,
    pub windows_build: Option<u32>,
    pub capabilities: Vec<PermissionCapability>,
    pub wcm_stack: super::wcm::WcmStackStatus,
}

#[cfg(windows)]
pub fn query_app_permissions(connected: bool, proxy_mode: bool) -> AppPermissionsStatus {
    use crate::windows_security::is_process_elevated;

    let is_admin = is_process_elevated();
    let build = super::wcm::read_windows_build();
    let wcm = super::wcm::query_wcm_stack_status();

    let capabilities = vec![
        PermissionCapability {
            id: "system_proxy".into(),
            title_en: "System proxy (browsers)".into(),
            needs_admin: false,
            needs_uac: false,
            available_now: true,
            hint_en: "Works as normal user — WinINet on 127.0.0.1:2080".into(),
        },
        PermissionCapability {
            id: "winhttp".into(),
            title_en: "WinHTTP (Windows Update, apps)".into(),
            needs_admin: false,
            needs_uac: true,
            available_now: is_admin,
            hint_en: "Often needs UAC once — use Retry WinHTTP or Auto-Repair".into(),
        },
        PermissionCapability {
            id: "wcm_2026".into(),
            title_en: "2026 Wi‑Fi / proxy stack (WcmSvc)".into(),
            needs_admin: true,
            needs_uac: true,
            available_now: is_admin,
            hint_en: "Fixes WinHttpAutoProxySvc + restarts WcmSvc/WlanSvc — admin or UAC".into(),
        },
        PermissionCapability {
            id: "tun".into(),
            title_en: "TUN full tunnel".into(),
            needs_admin: false,
            needs_uac: true,
            available_now: is_admin,
            hint_en: "sing-box elevated per Connect (UAC)".into(),
        },
        PermissionCapability {
            id: "machine_proxy".into(),
            title_en: "Proxy for all users".into(),
            needs_admin: true,
            needs_uac: false,
            available_now: is_admin,
            hint_en: "Run IPNOVA as administrator".into(),
        },
        PermissionCapability {
            id: "defender".into(),
            title_en: "Defender exclusions".into(),
            needs_admin: false,
            needs_uac: true,
            available_now: is_admin,
            hint_en: "One UAC prompt in setup".into(),
        },
        PermissionCapability {
            id: "auto_repair".into(),
            title_en: "Auto-Repair (full)".into(),
            needs_admin: false,
            needs_uac: false,
            available_now: true,
            hint_en: if connected && proxy_mode {
                "Startup steps without admin; WCM/WinHTTP may prompt UAC"
            } else {
                "Safe fixes without admin; 2026 stack uses UAC if needed"
            }
            .into(),
        },
    ];

    AppPermissionsStatus {
        is_admin,
        can_request_uac: true,
        windows_build: build,
        capabilities,
        wcm_stack: wcm,
    }
}

#[cfg(not(windows))]
pub fn query_app_permissions(_connected: bool, _proxy_mode: bool) -> AppPermissionsStatus {
    AppPermissionsStatus {
        is_admin: false,
        can_request_uac: false,
        windows_build: None,
        capabilities: vec![],
        wcm_stack: super::wcm::query_wcm_stack_status(),
    }
}
