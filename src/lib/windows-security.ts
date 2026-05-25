import type { ElevationNotice } from "./elevation-notice";
import {
  elevationPrimaryMessage,
  elevationSecondaryMessage,
} from "./elevation-notice";
import { exportVpnDiagnostics } from "./vpn";
import { isTauriRuntime } from "./platform";
import { tauriInvoke } from "./tauri-invoke";

export type { ElevationNotice } from "./elevation-notice";
export { elevationPrimaryMessage, elevationSecondaryMessage };

export type WindowsSecurityStatus = {
  is_admin: boolean;
  can_elevate: boolean;
  install_dir: string;
  singbox_path: string;
  app_data_dir: string;
  defender_exclusions_recommended: boolean;
  smartscreen_help_url: string;
  wdsi_submit_url: string;
};

export type DefenderExclusionResult = {
  ok: boolean;
  message: string;
};

export type ProxyDiagnostics = {
  ie_proxy_enabled: boolean;
  ie_proxy_server: string;
  pac_url: string;
  auto_detect: boolean;
  winhttp_line: string;
  winhttp_autoproxy_service: string;
  wininet_ok: boolean;
  winhttp_ok: boolean;
  wcmsvc_dependency_issue: boolean;
  edge_proxy_policy: string | null;
  firefox_detected: boolean;
  other_vpn_hints: string[];
  port_2080_in_use: boolean;
  port_2080_pid: number | null;
  port_2080_process: string;
  backup_available: boolean;
  recommendations_en: string[];
};

export type TroubleshootingCheck = {
  id: string;
  title: string;
  status: string;
  detail: string;
};

export type VpnConnectMode = "auto" | "tun" | "system_proxy";

export type WinhttpRetryResult = {
  ok: boolean;
  method: string | null;
  message: string;
  needs_admin: boolean;
  elevation?: ElevationNotice | null;
};

export function elevationFromWcmFix(r: WcmFixResult): ElevationNotice | null {
  if (r.ok || !r.needs_admin) return null;
  return {
    required: true,
    reason: "wcm_2026",
    message_ar:
      "نظام ويندوز يتطلب صلاحيات المسؤول لإصلاح مكدس البروكسي 2026 (Wi‑Fi / WinHttpAutoProxySvc).",
    message_en: r.message,
    can_use_uac: true,
    suggested_action: "uac_wcm_2026",
  };
}

export type ProxyScenario = {
  id: string;
  title: string;
  status: "pass" | "warn" | "fail" | "skip" | string;
  detail: string;
  blocks_connect: boolean;
};

export type ProxyScenarioReport = {
  ready_for_connect: boolean;
  connected_healthy: boolean;
  scenarios: ProxyScenario[];
};

export type AutoRepairStep = {
  id: string;
  title: string;
  status: string;
  detail: string;
};

export type AutoRepairReport = {
  ok: boolean;
  mode: string;
  steps: AutoRepairStep[];
  summary_en: string;
  recommend_reconnect: boolean;
  issues_2026_found: string[];
  needs_admin: boolean;
  elevation?: ElevationNotice | null;
};

export type AutoRepairMode = "startup" | "full";

export type WcmServiceState = {
  name: string;
  start_type: string;
  running: boolean;
};

export type WcmStackStatus = {
  windows_build: number | null;
  modern_stack: boolean;
  winhttp_autoproxy_start: string;
  wcm_depend_on: string;
  has_dependency_conflict: boolean;
  winhttp_autoproxy_running: boolean;
  wcmsvc_running: boolean;
  wlansvc_running: boolean;
  services: WcmServiceState[];
  recommendation_en: string;
};

export type WcmFixStep = {
  id: string;
  status: string;
  detail: string;
};

export type WcmFixResult = {
  ok: boolean;
  needs_admin: boolean;
  elevated_used: boolean;
  steps: WcmFixStep[];
  message: string;
  status_after: WcmStackStatus | null;
};

export type PermissionCapability = {
  id: string;
  title_en: string;
  needs_admin: boolean;
  needs_uac: boolean;
  available_now: boolean;
  hint_en: string;
};

export type AppPermissionsStatus = {
  is_admin: boolean;
  can_request_uac: boolean;
  windows_build: number | null;
  capabilities: PermissionCapability[];
  wcm_stack: WcmStackStatus;
};

export type ProxyConflictCheck = {
  has_conflict: boolean;
  proxy_enabled: boolean;
  current_proxy_server: string;
  has_pac: boolean;
  pac_url: string;
  auto_detect: boolean;
  message: string;
};

export type SupportLogInfo = {
  log_path: string;
  log_dir: string;
  tail: string;
};

export type DefenderStatus = {
  real_time_protection_enabled: boolean;
  tamper_protection_enabled: boolean;
  controlled_folder_access_enabled: boolean;
  smart_app_control_state: string;
  singbox_exists: boolean;
  singbox_excluded: boolean;
  wintun_exists: boolean;
  recommendations_en: string[];
};

export type WdsiFileHash = {
  path: string;
  sha256: string;
  exists: boolean;
};

const BROWSER_SECURITY: WindowsSecurityStatus = {
  is_admin: false,
  can_elevate: false,
  install_dir: "—",
  singbox_path: "—",
  app_data_dir: "—",
  defender_exclusions_recommended: false,
  smartscreen_help_url:
    "https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation",
  wdsi_submit_url: "https://www.microsoft.com/en-us/wdsi/filesubmission",
};

const BROWSER_PROXY_DIAG: ProxyDiagnostics = {
  ie_proxy_enabled: false,
  ie_proxy_server: "",
  pac_url: "",
  auto_detect: false,
  winhttp_line: "",
  winhttp_autoproxy_service: "",
  wininet_ok: false,
  winhttp_ok: false,
  wcmsvc_dependency_issue: false,
  edge_proxy_policy: null,
  firefox_detected: false,
  other_vpn_hints: [],
  port_2080_in_use: false,
  port_2080_pid: null,
  port_2080_process: "",
  backup_available: false,
  recommendations_en: ["Available in the installed Windows app only."],
};

const BROWSER_DEFENDER: DefenderStatus = {
  real_time_protection_enabled: false,
  tamper_protection_enabled: false,
  controlled_folder_access_enabled: false,
  smart_app_control_state: "n/a",
  singbox_exists: false,
  singbox_excluded: false,
  wintun_exists: false,
  recommendations_en: ["Available in the installed Windows app only."],
};

const SETUP_KEY = "ipnova.security_setup_done";

export async function getWindowsSecurityStatus(): Promise<WindowsSecurityStatus> {
  if (!isTauriRuntime()) return BROWSER_SECURITY;
  return tauriInvoke<WindowsSecurityStatus>("windows_security_status");
}

export async function addDefenderExclusions(): Promise<DefenderExclusionResult> {
  if (!isTauriRuntime()) {
    return { ok: false, message: "Available only in the installed Windows app" };
  }
  return tauriInvoke<DefenderExclusionResult>("windows_add_defender_exclusions");
}

export async function openDefenderSettings(): Promise<void> {
  if (!isTauriRuntime()) return;
  await tauriInvoke("windows_open_defender_settings");
}

export async function getProxyDiagnostics(): Promise<ProxyDiagnostics> {
  if (!isTauriRuntime()) return BROWSER_PROXY_DIAG;
  return tauriInvoke<ProxyDiagnostics>("windows_proxy_diagnostics");
}

/** Auto-Repair: orphan proxy, PAC, 2026 WinHTTP/Wi‑Fi stack, optional DNS flush. */
export async function runAutoRepair(
  mode: AutoRepairMode = "full",
): Promise<AutoRepairReport> {
  if (!isTauriRuntime()) {
    return {
      ok: false,
      mode,
      steps: [],
      summary_en: "Available only in the Windows desktop app.",
      recommend_reconnect: false,
      issues_2026_found: [],
      needs_admin: false,
      elevation: null,
    };
  }
  return tauriInvoke<AutoRepairReport>("windows_auto_repair", { mode });
}

export async function getProxyScenarios(): Promise<ProxyScenarioReport> {
  if (!isTauriRuntime()) {
    return {
      ready_for_connect: false,
      connected_healthy: false,
      scenarios: [],
    };
  }
  return tauriInvoke<ProxyScenarioReport>("windows_proxy_scenarios");
}

export async function getWcmRemediationScript(): Promise<string> {
  if (!isTauriRuntime()) return "";
  return tauriInvoke<string>("windows_wcm_remediation_script");
}

export async function applyWcmFix(): Promise<string> {
  if (!isTauriRuntime()) return "Available only in the Windows app";
  return tauriInvoke<string>("windows_apply_wcm_fix");
}

export async function getWcmStackStatus(): Promise<WcmStackStatus> {
  if (!isTauriRuntime()) {
    return {
      windows_build: null,
      modern_stack: false,
      winhttp_autoproxy_start: "",
      wcm_depend_on: "",
      has_dependency_conflict: false,
      winhttp_autoproxy_running: false,
      wcmsvc_running: false,
      wlansvc_running: false,
      services: [],
      recommendation_en: "Windows desktop app only.",
    };
  }
  return tauriInvoke<WcmStackStatus>("windows_wcm_stack_status");
}

/** Full 2026 stack fix (admin process). */
export async function applyWcmFixFull(): Promise<WcmFixResult> {
  if (!isTauriRuntime()) {
    return {
      ok: false,
      needs_admin: false,
      elevated_used: false,
      steps: [],
      message: "Windows only",
      status_after: null,
    };
  }
  return tauriInvoke<WcmFixResult>("windows_apply_wcm_fix_full");
}

/** 2026 stack fix with UAC elevation. */
export async function applyWcmFixUac(): Promise<WcmFixResult> {
  if (!isTauriRuntime()) {
    return {
      ok: false,
      needs_admin: true,
      elevated_used: false,
      steps: [],
      message: "Windows only",
      status_after: null,
    };
  }
  return tauriInvoke<WcmFixResult>("windows_apply_wcm_fix_uac");
}

export async function getAppPermissionsStatus(): Promise<AppPermissionsStatus> {
  if (!isTauriRuntime()) {
    return {
      is_admin: false,
      can_request_uac: false,
      windows_build: null,
      capabilities: [],
      wcm_stack: await getWcmStackStatus(),
    };
  }
  return tauriInvoke<AppPermissionsStatus>("windows_permissions_status");
}

export async function checkProxyConflict(): Promise<ProxyConflictCheck> {
  if (!isTauriRuntime()) {
    return {
      has_conflict: false,
      proxy_enabled: false,
      current_proxy_server: "",
      has_pac: false,
      pac_url: "",
      auto_detect: false,
      message: "",
    };
  }
  return tauriInvoke<ProxyConflictCheck>("windows_check_proxy_conflict");
}

export async function getSupportLogInfo(): Promise<SupportLogInfo> {
  if (!isTauriRuntime()) return { log_path: "", log_dir: "", tail: "" };
  return tauriInvoke<SupportLogInfo>("vpn_support_log_info");
}

export async function openSupportLogsFolder(): Promise<void> {
  if (!isTauriRuntime()) return;
  await tauriInvoke("vpn_open_support_logs");
}

export async function copyDiagnosticsBundle(): Promise<void> {
  const json = await exportVpnDiagnostics();
  await navigator.clipboard.writeText(json);
}

export async function runTroubleshootingChecks(): Promise<TroubleshootingCheck[]> {
  if (!isTauriRuntime()) return [];
  return tauriInvoke<TroubleshootingCheck[]>("windows_troubleshooting_checks");
}

export async function applyMachineWideProxy(): Promise<string> {
  if (!isTauriRuntime()) return "Available only in the Windows app";
  return tauriInvoke<string>("windows_apply_machine_proxy");
}

/** Re-apply WinHTTP via advproxy; prompts UAC when not running as admin. */
export async function retryWinhttpAdmin(): Promise<WinhttpRetryResult> {
  if (!isTauriRuntime()) {
    return {
      ok: false,
      method: null,
      message: "Available only in the installed Windows app",
      needs_admin: false,
      elevation: null,
    };
  }
  return tauriInvoke<WinhttpRetryResult>("windows_retry_winhttp");
}

export async function getDefenderStatus(): Promise<DefenderStatus> {
  if (!isTauriRuntime()) return BROWSER_DEFENDER;
  return tauriInvoke<DefenderStatus>("windows_defender_status");
}

export async function getWdsiFileHashes(): Promise<WdsiFileHash[]> {
  if (!isTauriRuntime()) return [];
  return tauriInvoke<WdsiFileHash[]>("windows_wdsi_file_hashes");
}

export async function copyWdsiHashesForSubmission(): Promise<void> {
  const hashes = await getWdsiFileHashes();
  const lines = [
    "IPNOVA — submit as false positive at https://www.microsoft.com/en-us/wdsi/filesubmission",
    "Role: Software developer",
    "",
    ...hashes.map((h) =>
      h.exists ? `${h.path}\nSHA256: ${h.sha256}` : `${h.path}\n(missing)`,
    ),
  ];
  await navigator.clipboard.writeText(lines.join("\n\n"));
}

export function isSecuritySetupDone(): boolean {
  return localStorage.getItem(SETUP_KEY) === "1";
}

export function markSecuritySetupDone(): void {
  localStorage.setItem(SETUP_KEY, "1");
}

export function shouldShowSecuritySetup(): boolean {
  if (!isTauriRuntime()) return false;
  return !isSecuritySetupDone();
}
