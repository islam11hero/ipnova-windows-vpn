import { lazy, Suspense } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronRight, RefreshCw } from "lucide-react";

import {
  DEFAULT_COUNTRY_ID,
  getCountry,
  type Country,
} from "../data/countries";
import type { VpnStatus } from "../lib/api";
import { formatBytes, formatExpire } from "../lib/api";
import { VPN_CONNECT_MODES } from "../data/connect-modes";
import type { VpnPreflight } from "../lib/vpn";
import type { VpnConnectMode } from "../lib/windows-security";
import { isTauriRuntime } from "../lib/platform";
import { BackgroundGlow } from "./BackgroundGlow";
import { ConnectButton } from "./ConnectButton";
import { CountryList } from "./CountryList";
import { LaunchOfferCard } from "./LaunchOfferCard";
import { PaywallBanner } from "./PaywallBanner";

const SettingsPanel = lazy(() =>
  import("./SettingsPanel").then((m) => ({ default: m.SettingsPanel })),
);
const PlansPage = lazy(() =>
  import("./PlansPage").then((m) => ({ default: m.PlansPage })),
);
import { AdminElevationBanner } from "./AdminElevationBanner";
import { translateApiError } from "../lib/errors";
import type { ElevationNotice } from "../lib/elevation-notice";
import type { NavId } from "./Sidebar";

type Props = {
  nav: NavId;
  status: VpnStatus | null;
  connected: boolean;
  busy: boolean;
  error: string | null;
  connectWarning?: string | null;
  elevationNotice?: ElevationNotice | null;
  recoveryNotice?: string | null;
  runtimeMessage?: string | null;
  exitIp?: string | null;
  preflight?: VpnPreflight | null;
  onReconnect?: () => void;
  onRetryWinhttp?: () => void;
  onApplyWcmUac?: () => void;
  onAutoRepair?: () => void;
  connectMode: VpnConnectMode;
  selectedCountryId: string;
  guestExpired: boolean;
  isGuest?: boolean;
  guestCountdown?: string;
  onOpenServers: () => void;
  onCountry: (c: Country) => void;
  onConnectMode: (m: VpnConnectMode) => void;
  onConnect: () => void;
  onRefresh: () => void;
  onGoPlans: () => void;
  onSignIn: () => void;
  onSignUp: () => void;
  onRestartGuestTrial?: () => void;
};

export function Dashboard({
  nav,
  status,
  connected,
  busy,
  error,
  connectWarning,
  elevationNotice,
  recoveryNotice,
  runtimeMessage,
  exitIp,
  preflight,
  onReconnect,
  onRetryWinhttp,
  onApplyWcmUac,
  onAutoRepair,
  connectMode,
  selectedCountryId,
  guestExpired,
  isGuest,
  guestCountdown,
  onOpenServers,
  onCountry,
  onConnectMode,
  onConnect,
  onRefresh,
  onGoPlans,
  onSignIn,
  onSignUp,
  onRestartGuestTrial,
}: Props) {
  const country = getCountry(selectedCountryId) ?? getCountry(DEFAULT_COUNTRY_ID)!;
  const usagePercent = status?.usagePercent ?? 0;
  const canConnect =
    !guestExpired &&
    (status?.eligibility.canConnect ?? false) &&
    (!isTauriRuntime() || preflight?.ready !== false);
  const preflightBlocked =
    isTauriRuntime() && preflight != null && !preflight.ready;
  const hasPaidPlan =
    !guestExpired && status?.eligibility.canConnect === true && !isGuest;
  const showOfferCard = nav === "home" && !hasPaidPlan;

  return (
    <main className="main-panel">
      <BackgroundGlow />

      {error ? (
        <div className="main-toast error">{translateApiError(error)}</div>
      ) : null}

      {recoveryNotice ? (
        <div className="main-toast success">{recoveryNotice}</div>
      ) : null}

      {elevationNotice?.required ? (
        <AdminElevationBanner
          notice={elevationNotice}
          busy={busy}
          onRetryWinhttp={onRetryWinhttp}
          onApplyWcmUac={onApplyWcmUac}
          onAutoRepair={onAutoRepair}
        />
      ) : null}

      {connectWarning ? (
        <div className="main-toast warn runtime-interrupted">
          <span>{connectWarning}</span>
          {isTauriRuntime() &&
          connectWarning.toLowerCase().includes("winhttp") ? (
            <button
              type="button"
              className="btn btn--accent btn--sm"
              disabled={busy}
              onClick={() => void onRetryWinhttp?.()}
            >
              Retry WinHTTP
            </button>
          ) : null}
        </div>
      ) : null}

      {!connected && runtimeMessage && !guestExpired ? (
        <div className="main-toast warn runtime-interrupted">
          <span>{runtimeMessage}</span>
          {onReconnect ? (
            <button
              type="button"
              className="btn btn--accent btn--sm"
              onClick={onReconnect}
              disabled={busy || !canConnect}
            >
              Reconnect
            </button>
          ) : null}
        </div>
      ) : null}

      {preflightBlocked && preflight?.messages.length ? (
        <div className="main-toast warn runtime-interrupted">
          <span>{preflight.messages.join(" ")}</span>
          {onAutoRepair ? (
            <button
              type="button"
              className="btn btn--accent btn--sm"
              disabled={busy}
              onClick={() => void onAutoRepair()}
            >
              Auto-Repair
            </button>
          ) : null}
        </div>
      ) : null}

      {connected && exitIp ? (
        <div className="main-toast success">Exit IP: {exitIp}</div>
      ) : null}

      {guestExpired ? (
        <PaywallBanner
          onPlans={onGoPlans}
          onSignIn={onSignIn}
          onSignUp={onSignUp}
        />
      ) : null}

      <header className="main-header">
        <div className="status-line">
          <span
            className={`status-dot ${connected ? "status-dot--on" : ""}`}
            aria-hidden
          />
          <div>
            <h2 className="status-title">
              {connected ? "Connected" : "Ready to connect"}
            </h2>
            <p className="status-sub">
              {connected && exitIp
                ? `Exit IP ${exitIp}`
                : `${country.flag} ${country.name} · ${country.ping} ms`}
            </p>
          </div>
        </div>
        <button
          type="button"
          className="icon-btn"
          onClick={onRefresh}
          disabled={busy}
          title="Refresh status"
          aria-label="Refresh status"
        >
          <RefreshCw size={18} className={busy ? "spin" : ""} />
        </button>
      </header>

      {showOfferCard ? (
        <LaunchOfferCard
          variant={
            guestExpired ? "expired" : isGuest ? "guest" : "upgrade"
          }
          countdown={guestCountdown}
          onPlans={onGoPlans}
          onSignUp={onSignUp}
        />
      ) : null}

      <AnimatePresence mode="wait">
        {nav === "plans" ? (
          <Suspense fallback={<div className="panel-loading">Loading plans…</div>}>
            <PlansPage onCreateAccount={onSignUp} onSignIn={onSignIn} />
          </Suspense>
        ) : nav === "servers" ? (
          <motion.div
            key="servers"
            className="panel-view"
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 20 }}
          >
            <CountryList selectedId={selectedCountryId} onSelect={onCountry} />
            <p className="country-hint">
              Server location is assigned by your Marzban plan. Country picker is
              for display until multi-node routing is enabled.
            </p>
          </motion.div>
        ) : nav === "settings" ? (
          <motion.div
            key="settings"
            className="panel-view panel-view--settings"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <Suspense fallback={<div className="panel-loading">Loading settings…</div>}>
              <SettingsPanel
                busy={busy}
                connectMode={connectMode}
                onConnectMode={onConnectMode}
                onRestartGuestTrial={onRestartGuestTrial}
              />
            </Suspense>
          </motion.div>
        ) : (
          <motion.div
            key="home"
            className="panel-view panel-view--home"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <div className="connect-section">
              <ConnectButton
                connected={connected}
                busy={busy}
                disabled={!connected && !canConnect}
                onClick={onConnect}
              />
              <div className="mode-pills" role="group" aria-label="Connection mode">
                {VPN_CONNECT_MODES.map((m) => (
                  <button
                    key={m.id}
                    type="button"
                    className={`mode-pill ${connectMode === m.id ? "mode-pill--on" : ""}`}
                    onClick={() => onConnectMode(m.id)}
                    disabled={guestExpired || busy}
                    title={m.hint}
                  >
                    {m.label}
                  </button>
                ))}
              </div>
            </div>

            <button
              type="button"
              className="server-picker"
              onClick={onOpenServers}
            >
              <span className="server-picker__flag">{country.flag}</span>
              <span className="server-picker__text">
                <strong>{country.name}</strong>
                <small>{country.city} · node {country.id}</small>
              </span>
              <ChevronRight size={18} className="server-picker__chevron" />
            </button>
          </motion.div>
        )}
      </AnimatePresence>

      {nav !== "settings" && nav !== "plans" ? (
        <footer className="main-footer">
          <div className="usage-bars">
            <div className="usage-bar-row">
              <span>Data used</span>
              <div className="bar bar--used">
                <motion.span
                  animate={{ width: `${Math.max(usagePercent, 4)}%` }}
                  transition={{ duration: 0.8 }}
                />
              </div>
              <span className="usage-val">
                {status ? formatBytes(status.usedTraffic) : "—"}
              </span>
            </div>
            <div className="usage-bar-row">
              <span>Quota</span>
              <div className="bar bar--quota">
                <span
                  style={{
                    width: `${status?.dataLimit ? Math.min(100, usagePercent) : 40}%`,
                  }}
                />
              </div>
              <span className="usage-val">
                {status?.dataLimit
                  ? formatBytes(status.dataLimit)
                  : "Trial 1 GB"}
              </span>
            </div>
          </div>
          <div className="footer-meta">
            <span>Expires: {formatExpire(status?.expireDate ?? null)}</span>
            <span
              className={`status-pill status-pill--${status?.status ?? "unknown"}`}
            >
              {status?.status ?? "—"}
            </span>
          </div>
        </footer>
      ) : null}
    </main>
  );
}
