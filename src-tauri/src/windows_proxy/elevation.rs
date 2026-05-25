//! Structured «admin / UAC required» signal for the frontend (Full repair, WinHTTP, WCM 2026).

use serde::Serialize;

/// Tells the UI to show an administrator notice with optional UAC actions.
#[derive(Debug, Clone, Serialize)]
pub struct ElevationNotice {
    pub required: bool,
    /// Machine id: `wcm_2026` | `winhttp` | `full_repair` | `machine_proxy`
    pub reason: String,
    pub message_en: String,
    pub message_ar: String,
    pub can_use_uac: bool,
    /// `uac_wcm_2026` | `uac_winhttp` | `run_as_admin` | `auto_repair_full`
    pub suggested_action: String,
}

impl ElevationNotice {
    pub fn for_reason(reason: &str) -> Self {
        match reason {
            "wcm_2026" => Self::wcm_2026(),
            "winhttp" => Self::winhttp(),
            "machine_proxy" => Self::machine_proxy(),
            _ => Self::full_repair(),
        }
    }

    pub fn full_repair() -> Self {
        Self {
            required: true,
            reason: "full_repair".into(),
            message_en: "Windows requires administrator rights to repair network proxy settings. Approve UAC or run IPNOVA as administrator.".into(),
            message_ar: "نظام ويندوز يتطلب صلاحيات المسؤول لإصلاح إعدادات الشبكة. وافق على UAC أو شغّل IPNOVA كمسؤول.".into(),
            can_use_uac: true,
            suggested_action: "auto_repair_full".into(),
        }
    }

    pub fn wcm_2026() -> Self {
        Self {
            required: true,
            reason: "wcm_2026".into(),
            message_en: "Windows requires administrator rights to fix the 2026 Wi‑Fi / proxy stack (WinHttpAutoProxySvc + WcmSvc).".into(),
            message_ar: "نظام ويندوز يتطلب صلاحيات المسؤول لإصلاح مكدس البروكسي 2026 (Wi‑Fi / WinHttpAutoProxySvc).".into(),
            can_use_uac: true,
            suggested_action: "uac_wcm_2026".into(),
        }
    }

    pub fn winhttp() -> Self {
        Self {
            required: true,
            reason: "winhttp".into(),
            message_en: "Windows requires administrator rights to apply WinHTTP proxy (system apps and Windows Update).".into(),
            message_ar: "نظام ويندوز يتطلب صلاحيات المسؤول لتطبيق بروكسي WinHTTP (تطبيقات النظام وWindows Update).".into(),
            can_use_uac: true,
            suggested_action: "uac_winhttp".into(),
        }
    }

    pub fn machine_proxy() -> Self {
        Self {
            required: true,
            reason: "machine_proxy".into(),
            message_en: "Machine-wide proxy for all users requires running IPNOVA as administrator.".into(),
            message_ar: "بروكسي جميع مستخدمي الجهاز يتطلب تشغيل IPNOVA كمسؤول.".into(),
            can_use_uac: false,
            suggested_action: "run_as_admin".into(),
        }
    }

    #[cfg(windows)]
    pub fn none() -> Option<Self> {
        None
    }

    #[cfg(windows)]
    pub fn if_needed(required: bool, reason: &str) -> Option<Self> {
        if required && !crate::windows_security::is_process_elevated() {
            Some(Self::for_reason(reason))
        } else {
            None
        }
    }
}

#[cfg(windows)]
pub fn elevation_from_wcm_fix(wcm: &super::wcm::WcmFixResult) -> Option<ElevationNotice> {
    if wcm.ok || !wcm.needs_admin {
        return None;
    }
    if wcm.elevated_used {
        return None;
    }
    ElevationNotice::if_needed(true, "wcm_2026")
}
