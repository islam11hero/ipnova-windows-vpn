import type { ElevationNotice } from "../lib/elevation-notice";
import {
  elevationPrimaryMessage,
  elevationSecondaryMessage,
} from "../lib/elevation-notice";

type Props = {
  notice: ElevationNotice;
  busy?: boolean;
  onRetryWinhttp?: () => void;
  onApplyWcmUac?: () => void;
  onAutoRepair?: () => void;
};

export function AdminElevationBanner({
  notice,
  busy,
  onRetryWinhttp,
  onApplyWcmUac,
  onAutoRepair,
}: Props) {
  if (!notice.required) return null;

  const secondary = elevationSecondaryMessage(notice);
  const action = notice.suggested_action;

  return (
    <div className="main-toast warn admin-notice" role="status">
      <div className="admin-notice__text">
        <p className="admin-notice__ar">{elevationPrimaryMessage(notice)}</p>
        {secondary ? (
          <p className="admin-notice__en hint">{secondary}</p>
        ) : null}
      </div>
      <div className="admin-notice__actions">
        {action === "uac_winhttp" && onRetryWinhttp ? (
          <button
            type="button"
            className="btn btn--accent btn--sm"
            disabled={busy}
            onClick={() => void onRetryWinhttp()}
          >
            WinHTTP (UAC)
          </button>
        ) : null}
        {action === "uac_wcm_2026" && onApplyWcmUac ? (
          <button
            type="button"
            className="btn btn--accent btn--sm"
            disabled={busy}
            onClick={() => void onApplyWcmUac()}
          >
            إصلاح 2026 (UAC)
          </button>
        ) : null}
        {(action === "auto_repair_full" || action === "run_as_admin") &&
        onAutoRepair ? (
          <button
            type="button"
            className="btn btn--accent btn--sm"
            disabled={busy}
            onClick={() => void onAutoRepair()}
          >
            Auto-Repair
          </button>
        ) : null}
        {action === "uac_wcm_2026" && onAutoRepair ? (
          <button
            type="button"
            className="btn btn--ghost btn--sm"
            disabled={busy}
            onClick={() => void onAutoRepair()}
          >
            Auto-Repair
          </button>
        ) : null}
      </div>
    </div>
  );
}
