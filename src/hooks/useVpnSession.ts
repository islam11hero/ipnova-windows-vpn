import { useCallback, useEffect, useState } from "react";

import type { VpnStatus } from "../lib/api";
import {
  fetchGuestVpnProfile,
  fetchVpnProfile,
  type VpnProfile,
} from "../lib/api";
import { translateApiError } from "../lib/errors";
import { isTauriRuntime } from "../lib/platform";
import {
  connectVpn,
  disconnectVpn,
  getVpnRuntimeStatus,
  recoverStaleVpn,
  vpnPreflight,
  type VpnPreflight,
  type VpnRecoveryReport,
} from "../lib/vpn";
import type { ElevationNotice } from "../lib/elevation-notice";
import type { VpnConnectMode } from "../lib/windows-security";
import { checkProxyConflict } from "../lib/windows-security";

import type { AccountMode, AppView } from "./useAccount";
import type { GuestSession } from "../lib/guest-session";

type AccountSlice = {
  view: AppView;
  accountMode: AccountMode;
  accessToken: string | null;
  guestSession: GuestSession | null;
  status: VpnStatus | null;
  guestExpired: boolean;
  refreshStatus: () => Promise<VpnStatus | null>;
  setView: (v: AppView) => void;
  selectedNodeId?: string;
};

export function useVpnSession(account: AccountSlice) {
  const {
    view,
    accountMode,
    accessToken,
    guestSession,
    status,
    guestExpired,
    refreshStatus,
    selectedNodeId,
  } = account;

  const [connected, setConnected] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [connectMode, setConnectMode] = useState<VpnConnectMode>("system_proxy");
  const [preflight, setPreflight] = useState<VpnPreflight | null>(null);
  const [connectWarning, setConnectWarning] = useState<string | null>(null);
  const [exitIp, setExitIp] = useState<string | null>(null);
  const [recoveryNotice, setRecoveryNotice] = useState<string | null>(null);
  const [runtimeMessage, setRuntimeMessage] = useState<string | null>(null);
  const [elevationNotice, setElevationNotice] = useState<ElevationNotice | null>(
    null,
  );

  const applyRuntimeStatus = useCallback(
    (s: {
      connected: boolean;
      message: string;
      warning?: string | null;
      exit_ip?: string | null;
      elevation?: ElevationNotice | null;
    }) => {
      setConnected(s.connected);
      setExitIp(s.exit_ip ?? null);
      setConnectWarning(s.warning ?? null);
      setRuntimeMessage(s.connected ? null : s.message);
      if (s.elevation?.required) {
        setElevationNotice(s.elevation);
      } else if (!s.warning) {
        setElevationNotice(null);
      }
    },
    [],
  );

  useEffect(() => {
    void getVpnRuntimeStatus().then(applyRuntimeStatus).catch(() => undefined);
  }, [applyRuntimeStatus]);

  useEffect(() => {
    if (!isTauriRuntime() || view !== "dashboard") return;
    void vpnPreflight().then(setPreflight).catch(() => undefined);
    void recoverStaleVpn().then((r: VpnRecoveryReport) => {
      if (r.orphan_proxy_cleaned && r.message) {
        setRecoveryNotice(r.message);
      }
      if (r.stale_port_in_use && r.message) {
        setConnectWarning(r.message);
      }
      if (r.singbox_running) {
        void getVpnRuntimeStatus().then(applyRuntimeStatus).catch(() => undefined);
      }
    });
  }, [view, applyRuntimeStatus]);

  useEffect(() => {
    if (!isTauriRuntime() || view !== "dashboard") return;
    const sync = () => {
      void getVpnRuntimeStatus().then(applyRuntimeStatus).catch(() => undefined);
    };
    sync();
    const id = window.setInterval(sync, 12_000);
    const onVisible = () => {
      if (document.visibilityState === "visible") sync();
    };
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      window.clearInterval(id);
      document.removeEventListener("visibilitychange", onVisible);
    };
  }, [view, applyRuntimeStatus]);

  const loadProfile = useCallback(async (): Promise<VpnProfile> => {
    const node = selectedNodeId?.trim() || undefined;
    if (accountMode === "member" && accessToken) {
      return fetchVpnProfile(accessToken, { node });
    }
    if (guestSession) {
      return fetchGuestVpnProfile(guestSession, { node });
    }
    throw new Error("No active session");
  }, [accountMode, accessToken, guestSession, selectedNodeId]);

  const handleConnect = useCallback(async () => {
    setBusy(true);
    setError(null);
    setConnectWarning(null);
    setElevationNotice(null);
    setExitIp(null);
    try {
      if (isTauriRuntime()) {
        const check = await vpnPreflight();
        setPreflight(check);
        if (!check.ready) {
          throw new Error(check.messages.join(" "));
        }
        const conflict = await checkProxyConflict();
        if (conflict.has_conflict) {
          const ok = window.confirm(
            `${conflict.message}\n\nContinue and replace the current proxy while VPN is connected?`,
          );
          if (!ok) {
            throw new Error("Connection cancelled — existing system proxy kept");
          }
        }
      }

      const current = (await refreshStatus()) ?? status;
      if (!current || !current.eligibility.canConnect) {
        throw new Error(
          current?.eligibility.reason ??
            current?.eligibility.reasonAr ??
            "Cannot connect — check your plan or try the free trial",
        );
      }

      const profile = await loadProfile();
      const runtime = await connectVpn(profile.config, connectMode);
      applyRuntimeStatus(runtime);
      setRecoveryNotice(null);
      if (!runtime.connected) throw new Error(runtime.message);
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Connection failed";
      setError(translateApiError(msg));
      setConnected(false);
    } finally {
      setBusy(false);
    }
  }, [
    applyRuntimeStatus,
    connectMode,
    loadProfile,
    refreshStatus,
    status,
  ]);

  const handleDisconnect = useCallback(async () => {
    setBusy(true);
    try {
      const runtime = await disconnectVpn();
      applyRuntimeStatus(runtime);
    } finally {
      setBusy(false);
    }
  }, [applyRuntimeStatus]);

  const handleConnectToggle = useCallback(
    async (onExpired: () => void) => {
      if (guestExpired) {
        onExpired();
        return;
      }
      if (connected) {
        await handleDisconnect();
        return;
      }
      await handleConnect();
    },
    [connected, guestExpired, handleConnect, handleDisconnect],
  );

  const runBusy = useCallback(async (fn: () => Promise<void>) => {
    setBusy(true);
    try {
      await fn();
    } finally {
      setBusy(false);
    }
  }, []);

  return {
    connected,
    busy,
    setBusy,
    runBusy,
    error,
    setError,
    connectMode,
    setConnectMode,
    preflight,
    setPreflight,
    connectWarning,
    setConnectWarning,
    exitIp,
    recoveryNotice,
    setRecoveryNotice,
    elevationNotice,
    setElevationNotice,
    runtimeMessage,
    handleConnect,
    handleDisconnect,
    handleConnectToggle,
    applyRuntimeStatus,
  };
}
