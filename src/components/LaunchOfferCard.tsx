import { motion } from "framer-motion";
import { ExternalLink, UserPlus } from "lucide-react";

import { LAUNCH_OFFER, planCheckoutUrl } from "../data/pricing";

type Props = {
  variant?: "guest" | "expired" | "upgrade";
  countdown?: string;
  onPlans?: () => void;
  onSignUp: () => void;
};

export function LaunchOfferCard({
  variant = "guest",
  countdown,
  onPlans,
  onSignUp,
}: Props) {
  const title =
    variant === "expired"
      ? "Trial ended"
      : variant === "upgrade"
        ? "Upgrade"
        : "After your trial";

  const subtitle =
    variant === "guest" && countdown
      ? `${countdown} left on free trial`
      : "3 months · first payment only";

  return (
    <motion.section
      className="launch-offer"
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
    >
      <div className="launch-offer__body">
        <div className="launch-offer__copy">
          <h3 className="launch-offer__title">{title}</h3>
          <p className="launch-offer__sub">{subtitle}</p>
        </div>

        <div className="launch-offer__price-block">
          <div className="launch-offer__price">
            <span className="launch-offer__currency">€</span>
            <span className="launch-offer__amount">
              {LAUNCH_OFFER.priceEur.toFixed(2)}
            </span>
          </div>
          <span className="launch-offer__duration">3 months</span>
        </div>
      </div>

      <div className="launch-offer__actions">
        <a
          className="btn btn--accent launch-offer__cta-primary"
          href={planCheckoutUrl(LAUNCH_OFFER)}
          target="_blank"
          rel="noreferrer"
        >
          Subscribe
          <ExternalLink size={15} />
        </a>
        <button type="button" className="btn btn--ghost" onClick={onSignUp}>
          <UserPlus size={15} />
          Sign up
        </button>
        {onPlans ? (
          <button type="button" className="btn btn--ghost" onClick={onPlans}>
            Plans
          </button>
        ) : null}
      </div>
    </motion.section>
  );
}
