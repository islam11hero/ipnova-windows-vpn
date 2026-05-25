import { useCallback, useEffect, useState } from "react";

import {
  ensureGuestSession,
  fetchGuestVpnStatus,
  fetchVpnStatus,
  linkGuestOrder,
  refreshVpnProfile,
  type VpnStatus,
} from "../lib/api";
import {
  clearGuestSession,
  formatCountdown,
  isGuestExpired,
  loadGuestSession,
  saveGuestSession,
  type GuestSession,
} from "../lib/guest-session";
import { supabase, supabaseConfigured } from "../lib/supabase";
import {
  clearSessionSecure,
  loadSessionSecure,
  saveSessionSecure,
} from "../lib/vpn";
import { shouldShowSecuritySetup } from "../lib/windows-security";

export type AppView = "booting" | "dashboard" | "auth" | "setup";
export type AccountMode = "guest" | "member";

function guestFromResponse(g: {
  orderId: string;
  deviceHash: string;
  marzbanUsername: string;
  expiresAt: number;
  planName: string;
  expired?: boolean;
}): GuestSession {
  return {
    orderId: g.orderId,
    deviceHash: g.deviceHash,
    marzbanUsername: g.marzbanUsername,
    expiresAt: g.expiresAt,
    planName: g.planName,
    expired: g.expired ?? false,
  };
}

export function useAccount() {
  const [view, setView] = useState<AppView>("booting");
  const [accountMode, setAccountMode] = useState<AccountMode>("guest");
  const [authMode, setAuthMode] = useState<"signin" | "signup">("signup");
  const [guestSession, setGuestSession] = useState<GuestSession | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [status, setStatus] = useState<VpnStatus | null>(null);
  const [countdown, setCountdown] = useState("");

  const guestExpired =
    accountMode === "guest" &&
    guestSession != null &&
    (isGuestExpired(guestSession) || status?.eligibility.canConnect === false);

  const loadStatusForCurrentAccount = useCallback(async () => {
    if (accountMode === "member" && accessToken) {
      return fetchVpnStatus(accessToken);
    }
    if (accountMode === "guest" && guestSession) {
      try {
        return await fetchGuestVpnStatus(guestSession);
      } catch (e) {
        const code = (e as Error & { code?: string }).code;
        if (code === "GUEST_SESSION_INVALID") {
          const fresh = await ensureGuestSession();
          const g = guestFromResponse(fresh);
          setGuestSession(g);
          return fetchGuestVpnStatus(g);
        }
        throw e;
      }
    }
    return null;
  }, [accountMode, accessToken, guestSession]);

  const refreshStatus = useCallback(async () => {
    const next = await loadStatusForCurrentAccount();
    setStatus(next);
    return next;
  }, [loadStatusForCurrentAccount]);

  const enterGuestMode = useCallback(async () => {
    const guest = await ensureGuestSession();
    const g = guestFromResponse(guest);
    saveGuestSession(g);
    setGuestSession(g);
    setAccountMode("guest");
    setAccessToken(null);
    const st = await fetchGuestVpnStatus(g);
    setStatus(st);
    setView("dashboard");
  }, []);

  const enterMemberMode = useCallback(
    async (token: string, linkOrderId?: string) => {
      setAccessToken(token);
      setAccountMode("member");
      if (linkOrderId && supabase) {
        try {
          await linkGuestOrder(token, linkOrderId);
        } catch (e) {
          console.warn("link-order", e);
        }
      }
      const st = await fetchVpnStatus(token);
      setStatus(st);
      if (!st) {
        // Member signed in but no VPN order yet — still show dashboard with banner.
      }
      setView(shouldShowSecuritySetup() ? "setup" : "dashboard");
    },
    [],
  );

  useEffect(() => {
    if (!supabaseConfigured || !supabase) return;

    void (async () => {
      try {
        const stored = await loadSessionSecure();
        if (stored) {
          const parsed = JSON.parse(stored) as {
            access_token?: string;
            refresh_token?: string;
          };
          if (parsed.access_token && parsed.refresh_token) {
            const { data, error: sessionError } = await supabase.auth.setSession({
              access_token: parsed.access_token,
              refresh_token: parsed.refresh_token,
            });
            if (!sessionError && data.session?.access_token) {
              await enterMemberMode(data.session.access_token);
              return;
            }
          }
        }

        const { data } = await supabase.auth.getSession();
        if (data.session?.access_token) {
          await enterMemberMode(data.session.access_token);
          return;
        }

        const localGuest = loadGuestSession();
        if (localGuest) {
          try {
            const st = await fetchGuestVpnStatus(localGuest);
            setGuestSession(localGuest);
            setAccountMode("guest");
            setStatus(st);
            setView("dashboard");
            return;
          } catch {
            clearGuestSession();
          }
        }

        await enterGuestMode();
      } catch (e) {
        setView("dashboard");
        throw e;
      }
    })().catch(() => setView("dashboard"));

    const { data: sub } = supabase.auth.onAuthStateChange((_event, session) => {
      if (session?.access_token) {
        setAccessToken(session.access_token);
        void saveSessionSecure(JSON.stringify(session));
      }
    });

    return () => sub.subscription.unsubscribe();
  }, [enterGuestMode, enterMemberMode]);

  useEffect(() => {
    if (view !== "dashboard") return;
    const tick = () => {
      if (guestSession && accountMode === "guest") {
        setCountdown(formatCountdown(guestSession.expiresAt));
      }
    };
    tick();
    const id = window.setInterval(tick, 30_000);
    return () => window.clearInterval(id);
  }, [view, guestSession, accountMode]);

  useEffect(() => {
    if (view !== "dashboard") return;
    const poll = window.setInterval(() => {
      void refreshStatus().catch(() => undefined);
    }, 45_000);
    return () => window.clearInterval(poll);
  }, [view, refreshStatus]);

  const refreshProfileAndStatus = useCallback(
    async (node?: string) => {
      if (accountMode === "member" && accessToken) {
        try {
          await refreshVpnProfile(accessToken, { node });
        } catch {
          /* member may have no plan yet */
        }
      }
      return refreshStatus();
    },
    [accountMode, accessToken, refreshStatus],
  );

  const signOutToGuest = useCallback(async () => {
    if (supabase) await supabase.auth.signOut();
    await clearSessionSecure();
    clearGuestSession();
    await enterGuestMode();
  }, [enterGuestMode]);

  const signUpMember = useCallback(
    async (email: string, password: string, pendingGuestOrderId?: string) => {
      if (!supabase) return;
      const { data, error: signError } = await supabase.auth.signUp({
        email,
        password,
      });
      if (signError) throw signError;
      if (!data.session?.access_token) {
        throw new Error(
          "Account created — open your email, confirm the link, then sign in",
        );
      }
      await saveSessionSecure(JSON.stringify(data.session));
      clearGuestSession();
      setGuestSession(null);
      await enterMemberMode(data.session.access_token, pendingGuestOrderId);
    },
    [enterMemberMode],
  );

  const signInMember = useCallback(
    async (email: string, password: string, pendingGuestOrderId?: string) => {
      if (!supabase) return;
      const { data, error: signError } = await supabase.auth.signInWithPassword({
        email,
        password,
      });
      if (signError) throw signError;
      if (!data.session?.access_token) throw new Error("Sign-in failed");
      await saveSessionSecure(JSON.stringify(data.session));
      clearGuestSession();
      setGuestSession(null);
      await enterMemberMode(data.session.access_token, pendingGuestOrderId);
    },
    [enterMemberMode],
  );

  const memberWithoutPlan =
    accountMode === "member" &&
    (status == null || !status.eligibility.canConnect);

  return {
    supabaseConfigured,
    view,
    setView,
    accountMode,
    authMode,
    setAuthMode,
    guestSession,
    accessToken,
    status,
    countdown,
    guestExpired,
    memberWithoutPlan,
    refreshStatus,
    refreshProfileAndStatus,
    enterGuestMode,
    signOutToGuest,
    signUpMember,
    signInMember,
  };
}
