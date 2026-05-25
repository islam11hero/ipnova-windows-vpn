import { motion } from "framer-motion";
import { AlertTriangle, Sparkles } from "lucide-react";

type Props = {
  onPlans: () => void;
  onSignIn: () => void;
  onSignUp: () => void;
};

export function PaywallBanner({ onPlans, onSignIn, onSignUp }: Props) {
  return (
    <motion.div
      className="paywall-banner"
      initial={{ opacity: 0, y: -8 }}
      animate={{ opacity: 1, y: 0 }}
    >
      <AlertTriangle size={22} />
      <div className="paywall-banner__text">
        <strong>Free day ended</strong>
        <span>
          Create an account or buy a plan — <b>3 months for €17.99</b> launch
          price
        </span>
      </div>
      <div className="paywall-banner__actions">
        <button type="button" className="btn btn--accent" onClick={onPlans}>
          <Sparkles size={14} /> View offers
        </button>
        <button type="button" className="btn btn--ghost" onClick={onSignUp}>
          Sign up
        </button>
        <button type="button" className="btn btn--ghost" onClick={onSignIn}>
          Sign in
        </button>
      </div>
    </motion.div>
  );
}
