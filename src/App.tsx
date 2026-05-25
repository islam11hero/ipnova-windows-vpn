import { useState } from "react";

import { AuthPage } from "./components/AuthPage";
import { Dashboard } from "./components/Dashboard";
import { GuestTrialBar } from "./components/GuestTrialBar";
import { SecuritySetup } from "./components/SecuritySetup";
import { Sidebar, type NavId } from "./components/Sidebar";
import { DEFAULT_COUNTRY_ID, type Country } from "./data/countries";
import { useAccount } from "./hooks/useAccount";
import { useVpnSession } from "./hooks/useVpnSession";
import { isBrowserDevMode } from "./lib/platform";
import { supabase } from "./lib/supabase";
import { getVpnRuntimeStatus } from "./lib/vpn";
import {
  applyWcmFixUac,
  elevationFromWcmFix,
  retryWinhttpAdmin,
  runAutoRepair,
} from "./lib/windows-security";

export default function App() {
  const account = useAccount();
  const [selectedCountryId, setSelectedCountryId] = useState(DEFAULT_COUNTRY_ID);
  const vpn = useVpnSession({ ...account, selectedNodeId: selectedCountryId });
  const [nav, setNav] = useState<NavId>("home");

  async function handleAuth(mode: "signin" | "signup", email: string, password: string) {
    await vpn.runBusy(async () => {
      vpn.setError(null);
      const pendingGuestOrderId = account.guestSession?.orderId;
      try {
        if (mode === "signup") {
          await account.signUpMember(email, password, pendingGuestOrderId);
        } else {
          await account.signInMember(email, password, pendingGuestOrderId);
        }
      } catch (err) {
      const msg = err instanceof Error ? err.message : "Authentication failed";
      if (msg.includes("Invalid login credentials")) {
        vpn.setError("Incorrect email or password");
      } else if (msg.includes("Email not confirmed")) {
        vpn.setError("Confirm your email from the link we sent, then sign in");
      } else {
        vpn.setError(msg);
      }
      }
    });
  }

  async function handleRetryWinhttp() {
    await vpn.runBusy(async () => {
      const r = await retryWinhttpAdmin();
      if (r.ok) {
        vpn.setError(null);
        vpn.setElevationNotice(null);
        const runtime = await getVpnRuntimeStatus();
        vpn.applyRuntimeStatus(runtime);
      } else {
        if (r.elevation?.required) {
          vpn.setElevationNotice(r.elevation);
        }
        vpn.setError(r.message);
      }
    });
  }

  async function handleApplyWcmUac() {
    await vpn.runBusy(async () => {
      const r = await applyWcmFixUac();
      if (r.ok) {
        vpn.setError(null);
        vpn.setElevationNotice(null);
      } else if (r.needs_admin) {
        const notice = elevationFromWcmFix(r);
        if (notice) vpn.setElevationNotice(notice);
      }
      vpn.setRecoveryNotice(r.message);
    });
  }

  async function handleAutoRepair() {
    await vpn.runBusy(async () => {
      const r = await runAutoRepair("full");
      if (r.ok) {
        vpn.setError(null);
      }
      vpn.setRecoveryNotice(r.summary_en);
      if (r.elevation?.required) {
        vpn.setElevationNotice(r.elevation);
      } else if (!r.needs_admin) {
        vpn.setElevationNotice(null);
      }
      if (r.recommend_reconnect) {
        vpn.setConnectWarning(
          "After repair, tap Connect to apply the VPN proxy again.",
        );
      }
      const runtime = await getVpnRuntimeStatus();
      vpn.applyRuntimeStatus(runtime);
      const pre = await import("./lib/vpn").then((m) => m.vpnPreflight());
      vpn.setPreflight(pre);
    });
  }

  async function handleRefresh() {
    await vpn.runBusy(async () => {
      vpn.setError(null);
      try {
        await account.refreshProfileAndStatus(selectedCountryId);
      } catch (err) {
        vpn.setError(err instanceof Error ? err.message : "Refresh failed");
      }
    });
  }

  async function handleRestartGuestTrial() {
    if (!supabase) return;
    vpn.setError(null);
    try {
      await account.signOutToGuest();
      setNav("home");
    } catch (err) {
      vpn.setError(err instanceof Error ? err.message : "Could not restart trial");
    }
  }

  async function handleContinueAsGuest() {
    vpn.setError(null);
    try {
      await account.signOutToGuest();
      setNav("home");
    } catch (err) {
      vpn.setError(err instanceof Error ? err.message : "Could not switch to trial");
    }
  }

  function handleCountrySelect(country: Country) {
    if (!country.available) return;
    setSelectedCountryId(country.id);
    setNav("home");
  }

  if (!account.supabaseConfigured) {
    return (
      <div className="shell shell--center">
        <p className="subtitle">
          Copy `.env.example` to `.env` and fill in the variables.
        </p>
      </div>
    );
  }

  if (account.view === "booting") {
    return (
      <div className="shell shell--center boot-screen">
        <div className="boot-spinner" />
        <p>Starting your free day…</p>
      </div>
    );
  }

  if (account.view === "auth") {
    return (
      <AuthPage
        initialMode={account.authMode}
        busy={vpn.busy}
        error={vpn.error}
        onSubmit={handleAuth}
        onBack={() => account.setView("dashboard")}
      />
    );
  }

  if (account.view === "setup") {
    return (
      <div className="shell shell--center">
        <div className="setup-wrap">
          <SecuritySetup onDone={() => account.setView("dashboard")} />
        </div>
      </div>
    );
  }

  return (
    <div className={`shell ${isBrowserDevMode() ? "shell--preview" : ""}`}>
      {isBrowserDevMode() ? (
        <div className="dev-banner">
          Preview mode — API via dev proxy. Real VPN + system proxy on Windows only.
        </div>
      ) : null}

      <div className="shell-body">
        {account.accountMode === "guest" &&
        account.guestSession &&
        !account.guestExpired ? (
          <GuestTrialBar
            countdown={account.countdown || "—"}
            onBuy={() => setNav("plans")}
            onSignUp={() => {
              account.setAuthMode("signup");
              account.setView("auth");
            }}
          />
        ) : null}

        {account.memberWithoutPlan ? (
          <div className="guest-trial-bar guest-trial-bar--warn">
            <div className="guest-trial-bar__text">
              <strong>Signed in without a VPN plan</strong>
              <span>Buy a plan or continue with the free day</span>
            </div>
            <div className="guest-trial-bar__actions">
              <button
                type="button"
                className="btn btn--accent btn--sm"
                onClick={() => setNav("plans")}
              >
                Plans
              </button>
              <button
                type="button"
                className="btn btn--ghost btn--sm"
                onClick={() => void handleContinueAsGuest()}
                disabled={vpn.busy}
              >
                Free day
              </button>
            </div>
          </div>
        ) : null}

        <div className="shell-row">
          <Sidebar
            active={nav}
            onNavigate={setNav}
            planName={account.status?.planName}
            isGuest={account.accountMode === "guest"}
            hasSubscription={account.status?.eligibility.canConnect === true}
            onUpgrade={() => setNav("plans")}
          />

          <Dashboard
            nav={nav}
            status={account.status}
            connected={vpn.connected}
            busy={vpn.busy}
            error={vpn.error}
            connectWarning={vpn.connectWarning}
            elevationNotice={vpn.elevationNotice}
            recoveryNotice={vpn.recoveryNotice}
            runtimeMessage={vpn.runtimeMessage}
            exitIp={vpn.exitIp}
            preflight={vpn.preflight}
            onReconnect={() => void vpn.handleConnectToggle(() => setNav("plans"))}
            onRetryWinhttp={() => void handleRetryWinhttp()}
            onApplyWcmUac={() => void handleApplyWcmUac()}
            onAutoRepair={() => void handleAutoRepair()}
            connectMode={vpn.connectMode}
            selectedCountryId={selectedCountryId}
            guestExpired={account.guestExpired}
            isGuest={account.accountMode === "guest"}
            guestCountdown={account.countdown}
            onOpenServers={() => setNav("servers")}
            onCountry={handleCountrySelect}
            onConnectMode={vpn.setConnectMode}
            onConnect={() => void vpn.handleConnectToggle(() => setNav("plans"))}
            onRefresh={() => void handleRefresh()}
            onGoPlans={() => setNav("plans")}
            onSignIn={() => {
              account.setAuthMode("signin");
              account.setView("auth");
            }}
            onSignUp={() => {
              account.setAuthMode("signup");
              account.setView("auth");
            }}
            onRestartGuestTrial={
              isBrowserDevMode() ? () => void handleRestartGuestTrial() : undefined
            }
          />
        </div>
      </div>
    </div>
  );
}
