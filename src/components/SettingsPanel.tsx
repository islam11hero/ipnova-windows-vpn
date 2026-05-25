import { useEffect, useState } from "react";

import { AdminElevationBanner } from "./AdminElevationBanner";
import { VPN_CONNECT_MODES } from "../data/connect-modes";
import { WINDOWS_TROUBLESHOOTING_STEPS } from "../data/windows-troubleshooting";
import { isTauriRuntime } from "../lib/platform";
import {
  addDefenderExclusions,
  applyMachineWideProxy,
  applyWcmFix,
  applyWcmFixUac,
  elevationFromWcmFix,
  getAppPermissionsStatus,
  copyDiagnosticsBundle,
  copyWdsiHashesForSubmission,
  getDefenderStatus,
  getProxyDiagnostics,
  getProxyScenarios,
  getSupportLogInfo,
  runAutoRepair,
  getWcmRemediationScript,
  openSupportLogsFolder,
  retryWinhttpAdmin,
  runTroubleshootingChecks,
  type DefenderStatus,
  type ProxyDiagnostics,
  type AppPermissionsStatus,
  type AutoRepairReport,
  type ElevationNotice,
  type ProxyScenarioReport,
  type TroubleshootingCheck,
  type VpnConnectMode,
} from "../lib/windows-security";

type Props = {
  busy: boolean;
  connectMode: VpnConnectMode;
  onConnectMode: (m: VpnConnectMode) => void;
  onRestartGuestTrial?: () => void;
};

export function SettingsPanel({
  busy,
  connectMode,
  onConnectMode,
  onRestartGuestTrial,
}: Props) {
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [proxyDiag, setProxyDiag] = useState<ProxyDiagnostics | null>(null);
  const [scenarioReport, setScenarioReport] =
    useState<ProxyScenarioReport | null>(null);
  const [repairReport, setRepairReport] = useState<AutoRepairReport | null>(null);
  const [defenderStatus, setDefenderStatus] = useState<DefenderStatus | null>(
    null,
  );
  const [liveChecks, setLiveChecks] = useState<TroubleshootingCheck[] | null>(
    null,
  );
  const [supportLogTail, setSupportLogTail] = useState<string | null>(null);
  const [permissions, setPermissions] = useState<AppPermissionsStatus | null>(
    null,
  );
  const [adminNotice, setAdminNotice] = useState<ElevationNotice | null>(null);

  useEffect(() => {
    if (!isTauriRuntime()) return;
    void getAppPermissionsStatus()
      .then(setPermissions)
      .catch(() => undefined);
  }, []);

  return (
    <>
      <details className="settings-advanced" open={isTauriRuntime()}>
        <summary className="settings-advanced__summary">
          Permissions & 2026 stack
        </summary>
        <div className="settings-advanced__body">
          <div className="settings-actions-row">
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy || !isTauriRuntime()}
              onClick={() => {
                void getAppPermissionsStatus()
                  .then(setPermissions)
                  .catch(() => setPermissions(null));
              }}
            >
              Refresh status
            </button>
            {permissions?.wcm_stack.has_dependency_conflict ||
            permissions?.wcm_stack.modern_stack ? (
              <button
                type="button"
                className="btn btn--accent btn--sm"
                disabled={busy || !isTauriRuntime()}
                title="WinHttpAutoProxySvc + WcmSvc/WlanSvc (UAC)"
                onClick={() => {
                  void applyWcmFixUac()
                    .then((r) => {
                      setStatusMessage(r.message);
                      return getAppPermissionsStatus();
                    })
                    .then(setPermissions)
                    .catch((e) =>
                      setStatusMessage(
                        e instanceof Error ? e.message : "2026 fix failed",
                      ),
                    );
                }}
              >
                Apply 2026 fix (UAC)
              </button>
            ) : null}
          </div>
          {permissions ? (
            <>
              <p className="permissions-summary">
                Admin:{" "}
                <strong>{permissions.is_admin ? "yes" : "no"}</strong>
                {permissions.windows_build != null
                  ? ` · Build ${permissions.windows_build}`
                  : ""}
              </p>
              <ul className="proxy-diag-list permissions-caps">
                {permissions.capabilities.map((c) => (
                  <li key={c.id}>
                    <strong>{c.title_en}</strong>
                    {c.available_now ? " ✓" : ""}
                    {c.needs_admin && !permissions.is_admin ? " · admin" : ""}
                    {c.needs_uac && !c.available_now ? " · UAC" : ""}
                    <br />
                    <span className="permissions-hint">{c.hint_en}</span>
                  </li>
                ))}
              </ul>
              <p className="permissions-wcm-title">2026 Wi‑Fi / proxy stack</p>
              <ul className="proxy-diag-list">
                <li>
                  WinHttpAutoProxySvc:{" "}
                  {permissions.wcm_stack.winhttp_autoproxy_start}
                  {permissions.wcm_stack.winhttp_autoproxy_running
                    ? " · running"
                    : " · stopped"}
                </li>
                <li>
                  WcmSvc:{" "}
                  {permissions.wcm_stack.wcmsvc_running ? "running" : "stopped"}
                </li>
                <li>
                  WlanSvc:{" "}
                  {permissions.wcm_stack.wlansvc_running ? "running" : "stopped"}
                </li>
                {permissions.wcm_stack.has_dependency_conflict ? (
                  <li className="proxy-diag-warn">
                    Dependency conflict — use Apply 2026 fix (UAC)
                  </li>
                ) : null}
                <li>{permissions.wcm_stack.recommendation_en}</li>
              </ul>
            </>
          ) : (
            <p className="hint">Tap Refresh to see admin and 2026 stack status.</p>
          )}
          {adminNotice?.required ? (
            <AdminElevationBanner
              notice={adminNotice}
              busy={busy}
              onRetryWinhttp={() => {
                void retryWinhttpAdmin()
                  .then((r) => {
                    setStatusMessage(r.message);
                    if (r.elevation?.required) setAdminNotice(r.elevation);
                    else if (r.ok) setAdminNotice(null);
                  })
                  .catch((e) =>
                    setStatusMessage(
                      e instanceof Error ? e.message : "WinHTTP retry failed",
                    ),
                  );
              }}
              onApplyWcmUac={() => {
                void applyWcmFixUac()
                  .then((r) => {
                    setStatusMessage(r.message);
                    if (r.needs_admin && !r.ok) {
                      const n = elevationFromWcmFix(r);
                      if (n) setAdminNotice(n);
                    } else if (r.ok) setAdminNotice(null);
                  })
                  .catch((e) =>
                    setStatusMessage(
                      e instanceof Error ? e.message : "2026 fix failed",
                    ),
                  );
              }}
              onAutoRepair={() => {
                void runAutoRepair("full")
                  .then((r) => {
                    setRepairReport(r);
                    setStatusMessage(r.summary_en);
                    setAdminNotice(r.elevation?.required ? r.elevation : null);
                  })
                  .catch((e) =>
                    setStatusMessage(
                      e instanceof Error ? e.message : "Auto-Repair failed",
                    ),
                  );
              }}
            />
          ) : null}
        </div>
      </details>

      <section className="settings-section">
        <p className="settings-heading">Connection mode</p>
        <div className="mode-grid">
          {VPN_CONNECT_MODES.map((m) => (
            <button
              key={m.id}
              type="button"
              className={`mode-chip ${connectMode === m.id ? "mode-chip--on" : ""}`}
              onClick={() => onConnectMode(m.id)}
              title={m.hint}
            >
              <span className="mode-chip__label">{m.label}</span>
              <small className="mode-chip__hint">{m.hint}</small>
            </button>
          ))}
        </div>
      </section>

      <details className="settings-advanced">
        <summary className="settings-advanced__summary">
          Connection help
        </summary>
        <div className="settings-advanced__body">
          <ol className="troubleshoot-steps">
            {WINDOWS_TROUBLESHOOTING_STEPS.map((step, i) => (
              <li key={step.id}>
                <strong>
                  {i + 1}. {step.title}
                </strong>
                <span>{step.hint}</span>
              </li>
            ))}
          </ol>
          <div className="settings-actions-row">
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy || !isTauriRuntime()}
              onClick={() => {
                setStatusMessage(null);
                void runTroubleshootingChecks()
                  .then(setLiveChecks)
                  .catch(() => setLiveChecks(null));
              }}
            >
              Run live checks
            </button>
            {isTauriRuntime() ? (
              <button
                type="button"
                className="btn btn--ghost btn--sm"
                disabled={busy}
                title="Requires Run as administrator"
                onClick={() => {
                  void applyMachineWideProxy()
                    .then(setStatusMessage)
                    .catch((e) =>
                      setStatusMessage(
                        e instanceof Error ? e.message : "Failed",
                      ),
                    );
                }}
              >
                Proxy for all users (admin)
              </button>
            ) : null}
          </div>
          {liveChecks?.length ? (
            <ul className="proxy-diag-list troubleshoot-live">
              {liveChecks.map((c) => (
                <li key={c.id} className={`troubleshoot-live--${c.status}`}>
                  <span className="troubleshoot-live__badge">{c.status}</span>
                  <span>
                    <strong>{c.title}</strong> — {c.detail}
                  </span>
                </li>
              ))}
            </ul>
          ) : null}
        </div>
      </details>

      <details className="settings-advanced">
        <summary className="settings-advanced__summary">
          Windows Defender
        </summary>
        <div className="settings-advanced__body">
          <div className="settings-actions-row">
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy || !isTauriRuntime()}
              onClick={() => {
                setStatusMessage(null);
                void getDefenderStatus()
                  .then(setDefenderStatus)
                  .catch(() => undefined);
              }}
            >
              Check Defender
            </button>
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy || !isTauriRuntime()}
              onClick={() => {
                void addDefenderExclusions()
                  .then((r) => {
                    setStatusMessage(r.message);
                    return getDefenderStatus();
                  })
                  .then(setDefenderStatus)
                  .catch((e) =>
                    setStatusMessage(
                      e instanceof Error ? e.message : "Exclusion failed",
                    ),
                  );
              }}
            >
              Add exclusion
            </button>
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy || !isTauriRuntime()}
              onClick={() => {
                void copyWdsiHashesForSubmission()
                  .then(() =>
                    setStatusMessage(
                      "SHA256 hashes copied for Microsoft WDSI submission",
                    ),
                  )
                  .catch(() => setStatusMessage("Could not copy hashes"));
              }}
            >
              Copy hashes (WDSI)
            </button>
          </div>
          {defenderStatus ? (
            <ul className="proxy-diag-list">
              <li>
                sing-box:{" "}
                {!defenderStatus.singbox_exists
                  ? "missing — run download-singbox.ps1"
                  : defenderStatus.singbox_excluded
                    ? "excluded ✓"
                    : "not excluded"}
              </li>
              <li>
                Smart App Control: {defenderStatus.smart_app_control_state}
              </li>
              <li>
                Tamper Protection:{" "}
                {defenderStatus.tamper_protection_enabled ? "on" : "off"}
              </li>
              {defenderStatus.recommendations_en.map((r) => (
                <li key={r} className="proxy-diag-warn">
                  {r}
                </li>
              ))}
            </ul>
          ) : null}
        </div>
      </details>

      <details className="settings-advanced">
        <summary className="settings-advanced__summary">
          Proxy diagnostics
        </summary>
        <div className="settings-advanced__body">
          <div className="settings-actions-row">
            <button
              type="button"
              className="btn btn--accent btn--sm"
              disabled={busy || !isTauriRuntime()}
              title="Orphan proxy, PAC, 2026 Wi‑Fi/WinHTTP stack, DNS flush"
              onClick={() => {
                setStatusMessage(null);
                void runAutoRepair("full")
                  .then((r) => {
                    setRepairReport(r);
                    setStatusMessage(r.summary_en);
                    setAdminNotice(r.elevation?.required ? r.elevation : null);
                    return Promise.all([
                      getProxyDiagnostics(),
                      getProxyScenarios(),
                    ]);
                  })
                  .then(([diag, scenarios]) => {
                    if (diag) setProxyDiag(diag);
                    if (scenarios) setScenarioReport(scenarios);
                  })
                  .catch((e) =>
                    setStatusMessage(
                      e instanceof Error ? e.message : "Auto-Repair failed",
                    ),
                  );
              }}
            >
              Auto-Repair
            </button>
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy}
              onClick={() => {
                setStatusMessage(null);
                void Promise.all([getProxyDiagnostics(), getProxyScenarios()])
                  .then(([diag, scenarios]) => {
                    setProxyDiag(diag);
                    setScenarioReport(scenarios);
                  })
                  .catch(() => undefined);
              }}
            >
              Check proxy & WinHTTP
            </button>
            {proxyDiag && !proxyDiag.winhttp_ok ? (
              <button
                type="button"
                className="btn btn--accent btn--sm"
                disabled={busy || !isTauriRuntime()}
                title="Requires UAC — applies WinHTTP for Windows Update and system apps"
                onClick={() => {
                  setStatusMessage(null);
                  void retryWinhttpAdmin()
                    .then((r) => {
                      setStatusMessage(r.message);
                      return getProxyDiagnostics();
                    })
                    .then(setProxyDiag)
                    .catch((e) =>
                      setStatusMessage(
                        e instanceof Error ? e.message : "WinHTTP retry failed",
                      ),
                    );
                }}
              >
                Retry WinHTTP (admin)
              </button>
            ) : null}
            {proxyDiag?.wcmsvc_dependency_issue ? (
              <>
                <button
                  type="button"
                  className="btn btn--ghost btn--sm"
                  disabled={busy}
                  onClick={() => {
                    setStatusMessage(null);
                    void applyWcmFix()
                      .then(setStatusMessage)
                      .catch((e) =>
                        setStatusMessage(
                          e instanceof Error ? e.message : "Fix failed",
                        ),
                      );
                  }}
                >
                  Apply 24H2 fix
                </button>
                <button
                  type="button"
                  className="btn btn--ghost btn--sm"
                  disabled={busy}
                  onClick={() => {
                    void getWcmRemediationScript()
                      .then((script) => navigator.clipboard.writeText(script))
                      .then(() =>
                        setStatusMessage(
                          "Admin script copied — run in PowerShell as Administrator",
                        ),
                      )
                      .catch(() =>
                        setStatusMessage(
                          "Could not copy — select script manually",
                        ),
                      );
                  }}
                >
                  Copy admin script
                </button>
              </>
            ) : null}
          </div>
          {proxyDiag ? (
            <ul className="proxy-diag-list">
              <li>
                WinINet:{" "}
                {proxyDiag.wininet_ok
                  ? "on (2080) ✓"
                  : proxyDiag.ie_proxy_enabled
                    ? `on (${proxyDiag.ie_proxy_server || "—"})`
                    : "off"}
              </li>
              <li>
                WinHTTP:{" "}
                {proxyDiag.winhttp_ok
                  ? "on (2080) ✓"
                  : proxyDiag.winhttp_line || "off"}
              </li>
              {proxyDiag.pac_url || proxyDiag.auto_detect ? (
                <li className="proxy-diag-warn">
                  PAC/WPAD:{" "}
                  {proxyDiag.pac_url
                    ? proxyDiag.pac_url
                    : "auto-detect enabled"}
                </li>
              ) : null}
              {proxyDiag.port_2080_in_use ? (
                <li className="proxy-diag-warn">
                  Port 2080: in use
                  {proxyDiag.port_2080_pid
                    ? ` (PID ${proxyDiag.port_2080_pid}${proxyDiag.port_2080_process ? ` · ${proxyDiag.port_2080_process}` : ""})`
                    : ""}
                </li>
              ) : null}
              {proxyDiag.edge_proxy_policy ? (
                <li className="proxy-diag-warn">{proxyDiag.edge_proxy_policy}</li>
              ) : null}
              {proxyDiag.firefox_detected ? (
                <li className="proxy-diag-warn">
                  Firefox: use system proxy or TUN mode
                </li>
              ) : null}
              {proxyDiag.other_vpn_hints?.length ? (
                <li className="proxy-diag-warn">
                  Other VPN: {proxyDiag.other_vpn_hints.join("; ")}
                </li>
              ) : null}
              {proxyDiag.wcmsvc_dependency_issue ? (
                <li className="proxy-diag-warn">
                  Win11 24H2: WinHttpAutoProxySvc / WcmSvc conflict
                </li>
              ) : null}
              {proxyDiag.recommendations_en.map((r) => (
                <li key={r}>{r}</li>
              ))}
            </ul>
          ) : null}
          {repairReport ? (
            <div className="proxy-scenarios">
              <p className="proxy-scenarios__summary">
                Auto-Repair ({repairReport.mode}) —{" "}
                {repairReport.ok ? "completed" : "partial"}
                {repairReport.issues_2026_found.length
                  ? ` · ${repairReport.issues_2026_found.length} 2026-era issue(s) detected`
                  : ""}
              </p>
              <ul className="proxy-diag-list proxy-scenarios__list">
                {repairReport.steps.map((s) => (
                  <li
                    key={s.id}
                    className={
                      s.status === "fail" || s.status === "warn"
                        ? "proxy-diag-warn"
                        : undefined
                    }
                  >
                    <strong>{s.title}</strong> ({s.status}) — {s.detail}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
          {scenarioReport ? (
            <div className="proxy-scenarios">
              <p className="proxy-scenarios__summary">
                Scenarios:{" "}
                {scenarioReport.ready_for_connect
                  ? "ready to connect"
                  : "blocked"}{" "}
                ·{" "}
                {scenarioReport.connected_healthy
                  ? "VPN proxy healthy"
                  : "check WinINet/WinHTTP after Connect"}
              </p>
              <ul className="proxy-diag-list proxy-scenarios__list">
                {scenarioReport.scenarios.map((s) => (
                  <li
                    key={s.id}
                    className={
                      s.status === "fail" || s.blocks_connect
                        ? "proxy-diag-warn"
                        : s.status === "warn"
                          ? "proxy-diag-warn"
                          : undefined
                    }
                  >
                    <strong>{s.title}</strong> ({s.status}) — {s.detail}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </div>
      </details>

      <details className="settings-advanced">
        <summary className="settings-advanced__summary">Support logs</summary>
        <div className="settings-advanced__body">
          <div className="settings-actions-row">
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy}
              onClick={() => {
                void getSupportLogInfo()
                  .then((info) => {
                    setSupportLogTail(
                      info.tail || "(empty — connect once to generate logs)",
                    );
                  })
                  .catch(() => setSupportLogTail("Could not read log"));
              }}
            >
              View log tail
            </button>
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy}
              onClick={() => void openSupportLogsFolder()}
            >
              Open log folder
            </button>
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy}
              onClick={() => {
                void copyDiagnosticsBundle()
                  .then(() =>
                    setStatusMessage("Diagnostics JSON copied to clipboard"),
                  )
                  .catch(() => setStatusMessage("Could not copy diagnostics"));
              }}
            >
              Copy diagnostics
            </button>
          </div>
          {supportLogTail ? (
            <pre className="support-log-preview">{supportLogTail}</pre>
          ) : null}
        </div>
      </details>

      {statusMessage ? <p className="hint settings-status-hint">{statusMessage}</p> : null}

      {onRestartGuestTrial ? (
        <details className="settings-advanced">
          <summary className="settings-advanced__summary">Developer</summary>
          <div className="settings-advanced__body">
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              onClick={onRestartGuestTrial}
              disabled={busy}
            >
              Reset free day (clear session)
            </button>
          </div>
        </details>
      ) : null}
    </>
  );
}
