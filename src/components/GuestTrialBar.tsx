import { Gift } from "lucide-react";

import { LAUNCH_OFFER } from "../data/pricing";

type Props = {
  countdown: string;
  onBuy: () => void;
  onSignUp: () => void;
};

export function GuestTrialBar({ countdown, onBuy, onSignUp }: Props) {
  return (
    <div className="guest-trial-bar">
      <div className="guest-trial-bar__main">
        <Gift size={18} className="guest-trial-bar__icon" />
        <div className="guest-trial-bar__text">
          <strong>Full free day — try VPN now</strong>
          <span>
            Ends in <b>{countdown}</b> · then get{" "}
            <b>3 months for €{LAUNCH_OFFER.priceEur.toFixed(2)}</b> (save 33%)
          </span>
        </div>
      </div>
      <div className="guest-trial-bar__actions">
        <button type="button" className="btn btn--accent btn--sm" onClick={onBuy}>
          3-month offer
        </button>
        <button type="button" className="btn btn--ghost btn--sm" onClick={onSignUp}>
          Free account
        </button>
      </div>
    </div>
  );
}
