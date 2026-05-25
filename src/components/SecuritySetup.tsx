import { useEffect, useState } from "react";

import {
  addDefenderExclusions,
  copyWdsiHashesForSubmission,
  getDefenderStatus,
  getWindowsSecurityStatus,
  markSecuritySetupDone,
  openDefenderSettings,
  type DefenderStatus,
  type WindowsSecurityStatus,
} from "../lib/windows-security";

type Props = {
  onDone: () => void;
};

export function SecuritySetup({ onDone }: Props) {
  const [status, setStatus] = useState<WindowsSecurityStatus | null>(null);
  const [defender, setDefender] = useState<DefenderStatus | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function refreshDefender() {
    const d = await getDefenderStatus();
    setDefender(d);
    return d;
  }

  useEffect(() => {
    void getWindowsSecurityStatus().then(setStatus).catch(() => undefined);
    void refreshDefender().catch(() => undefined);
  }, []);

  async function handleDefender() {
    setBusy(true);
    setMessage(null);
    try {
      const result = await addDefenderExclusions();
      setMessage(result.message);
      await refreshDefender();
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Exclusion request failed");
    } finally {
      setBusy(false);
    }
  }

  async function handleCopyHashes() {
    setBusy(true);
    try {
      await copyWdsiHashesForSubmission();
      setMessage("SHA256 hashes copied — paste at Microsoft WDSI (Software developer)");
    } catch {
      setMessage("Could not copy hashes");
    } finally {
      setBusy(false);
    }
  }

  const needsExclusion =
    defender != null && defender.singbox_exists && !defender.singbox_excluded;

  return (
    <div className="setup-card">
      <h2>Windows setup (one time)</h2>
      <p className="subtitle">
        Reduce Defender and SmartScreen friction before you connect.
      </p>

      <ul className="setup-list">
        <li>
          <strong>SmartScreen on install:</strong> if you see &quot;Windows
          protected your PC&quot;, choose <em>More info</em> then{" "}
          <em>Run anyway</em>.
        </li>
        <li>
          <strong>Defender:</strong> add exclusions for sing-box and wintun (one
          UAC prompt). Required if Connect fails or the .exe disappears.
        </li>
        <li>
          <strong>Smart App Control:</strong> if enabled, unsigned binaries may
          be blocked — sign the app or turn SAC off in Windows Security.
        </li>
        <li>
          <strong>Connection:</strong> System proxy works without admin. TUN
          asks UAC for sing-box only.
        </li>
      </ul>

      {defender ? (
        <ul className="proxy-diag-list setup-defender-status">
          <li>
            sing-box:{" "}
            {!defender.singbox_exists
              ? "missing"
              : defender.singbox_excluded
                ? "excluded ✓"
                : "not excluded"}
          </li>
          <li>Smart App Control: {defender.smart_app_control_state}</li>
          <li>
            Tamper Protection:{" "}
            {defender.tamper_protection_enabled ? "on (UAC required)" : "off"}
          </li>
          {defender.recommendations_en.map((r) => (
            <li key={r} className="proxy-diag-warn">
              {r}
            </li>
          ))}
        </ul>
      ) : null}

      <div className="setup-actions">
        <button
          type="button"
          className="btn btn--primary"
          onClick={() => void handleDefender()}
          disabled={busy}
        >
          {needsExclusion ? "Add Defender exclusion (required)" : "Add Defender exclusion"}
        </button>
        <button
          type="button"
          className="btn btn--ghost"
          onClick={() => void handleCopyHashes()}
          disabled={busy}
        >
          Copy hashes for Microsoft
        </button>
        <button
          type="button"
          className="btn btn--ghost"
          onClick={() => void openDefenderSettings()}
        >
          Open security settings
        </button>
      </div>

      {message ? <p className="hint">{message}</p> : null}

      {status ? (
        <p className="hint">
          Running as admin: {status.is_admin ? "yes" : "no"} — you do not need
          to run the app as admin when using system proxy.
        </p>
      ) : null}

      <button
        type="button"
        className="btn btn--accent setup-done"
        onClick={() => {
          markSecuritySetupDone();
          onDone();
        }}
      >
        Continue
      </button>
    </div>
  );
}
