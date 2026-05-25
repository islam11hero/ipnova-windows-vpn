import type { ElevationNotice } from "./elevation-notice";
import type { VpnConnectMode } from "./windows-security";
import { isTauriRuntime } from "./platform";
import { tauriInvoke } from "./tauri-invoke";

const SESSION_STORAGE_KEY = "ipnova.secure_session";

export type VpnRuntimeStatus = {
  connected: boolean;
  message: string;
  mode?: string | null;
  warning?: string | null;
  exit_ip?: string | null;
  elevation?: ElevationNotice | null;
};

export type VpnPreflight = {
  ready: boolean;
  singbox_found: boolean;
  singbox_runnable: boolean;
  port_available: boolean;
  messages: string[];
};

export type VpnRecoveryReport = {
  orphan_proxy_cleaned: boolean;
  stale_port_in_use: boolean;
  singbox_running: boolean;
  message: string;
};

const browserVpnMessage =
  "Browser preview: VPN runs only in the Windows app (npm run tauri dev or the .exe installer)";

const browserPreflight: VpnPreflight = {
  ready: false,
  singbox_found: false,
  singbox_runnable: false,
  port_available: true,
  messages: [browserVpnMessage],
};

export async function vpnPreflight(): Promise<VpnPreflight> {
  if (!isTauriRuntime()) return browserPreflight;
  return tauriInvoke<VpnPreflight>("vpn_preflight");
}

export async function connectVpn(
  config: Record<string, unknown>,
  mode: VpnConnectMode = "auto",
): Promise<VpnRuntimeStatus> {
  if (!isTauriRuntime()) {
    return { connected: false, message: browserVpnMessage, mode: null };
  }
  return tauriInvoke<VpnRuntimeStatus>("vpn_connect", { config, mode });
}

export async function disconnectVpn(): Promise<VpnRuntimeStatus> {
  if (!isTauriRuntime()) {
    return { connected: false, message: "Disconnected", mode: null };
  }
  return tauriInvoke<VpnRuntimeStatus>("vpn_disconnect");
}

export async function getVpnRuntimeStatus(): Promise<VpnRuntimeStatus> {
  if (!isTauriRuntime()) {
    return { connected: false, message: "Browser preview", mode: null };
  }
  return tauriInvoke<VpnRuntimeStatus>("vpn_status");
}

export async function recoverStaleVpn(): Promise<VpnRecoveryReport> {
  if (!isTauriRuntime()) {
    return {
      orphan_proxy_cleaned: false,
      stale_port_in_use: false,
      singbox_running: false,
      message: "",
    };
  }
  return tauriInvoke<VpnRecoveryReport>("vpn_recover_stale");
}

export async function exportVpnDiagnostics(): Promise<string> {
  if (!isTauriRuntime()) return "{}";
  return tauriInvoke<string>("vpn_export_diagnostics");
}

export async function saveSessionSecure(sessionJson: string): Promise<void> {
  if (!isTauriRuntime()) {
    localStorage.setItem(SESSION_STORAGE_KEY, sessionJson);
    return;
  }
  await tauriInvoke("secure_store_session", { sessionJson });
}

export async function loadSessionSecure(): Promise<string | null> {
  if (!isTauriRuntime()) {
    return localStorage.getItem(SESSION_STORAGE_KEY);
  }
  return tauriInvoke<string | null>("secure_load_session");
}

export async function clearSessionSecure(): Promise<void> {
  if (!isTauriRuntime()) {
    localStorage.removeItem(SESSION_STORAGE_KEY);
    return;
  }
  await tauriInvoke("secure_clear_session");
}
