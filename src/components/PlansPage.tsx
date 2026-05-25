import { motion } from "framer-motion";
import { Check, ExternalLink, Zap } from "lucide-react";

import {
  LAUNCH_OFFER,
  MARKET_NOTE,
  planCheckoutUrl,
  PRICING_PLANS,
} from "../data/pricing";
import { LaunchOfferCard } from "./LaunchOfferCard";

type Props = {
  onCreateAccount: () => void;
  onSignIn: () => void;
};

export function PlansPage({ onCreateAccount, onSignIn }: Props) {
  return (
    <motion.div
      className="panel-view panel-view--plans"
      initial={{ opacity: 0, y: 12 }}
      animate={{ opacity: 1, y: 0 }}
    >
      <div className="plans-hero">
        <Zap size={28} className="plans-hero__icon" />
        <h2>Maximum data transfer speed</h2>
        <p>
          Servers optimized for Spain — no streaming or gaming categories. Focus
          on speed and stability only.
        </p>
      </div>

      <LaunchOfferCard variant="upgrade" onSignUp={onCreateAccount} />

      <p className="plans-section-label">All plans</p>

      <div className="plans-grid">
        {PRICING_PLANS.filter((p) => p.id !== LAUNCH_OFFER.id).map((plan, i) => (
          <motion.article
            key={plan.id}
            className={`plan-card ${plan.highlight ? "plan-card--highlight" : ""}`}
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.08 }}
          >
            {plan.badge ? (
              <span className="plan-card__badge">{plan.badge}</span>
            ) : null}
            <h3>{plan.name}</h3>
            <p className="plan-card__duration">{plan.durationLabel}</p>
            <div className="plan-card__price">
              <span className="plan-card__amount">€{plan.priceEur.toFixed(2)}</span>
              {plan.compareAtEur ? (
                <span className="plan-card__compare">
                  was €{plan.compareAtEur.toFixed(2)}
                </span>
              ) : null}
            </div>
            <p className="plan-card__per-month">
              ≈ €{plan.pricePerMonth.toFixed(2)} / month
            </p>
            <ul className="plan-card__features">
              {plan.features.map((f) => (
                <li key={f}>
                  <Check size={14} /> {f}
                </li>
              ))}
            </ul>
            <a
              className="btn btn--primary plan-card__cta"
              href={planCheckoutUrl(plan)}
              target="_blank"
              rel="noreferrer"
            >
              Buy now
              <ExternalLink size={14} />
            </a>
          </motion.article>
        ))}
      </div>

      <p className="hint plans-note">{MARKET_NOTE}</p>

      <div className="plans-auth-row">
        <button type="button" className="btn btn--ghost" onClick={onSignIn}>
          I have an account
        </button>
        <button type="button" className="btn btn--primary" onClick={onCreateAccount}>
          Create free account
        </button>
      </div>
    </motion.div>
  );
}
