import { useState } from "react";
import { motion } from "framer-motion";
import { Lock, Mail, Shield, UserPlus } from "lucide-react";

type Props = {
  initialMode?: "signin" | "signup";
  busy: boolean;
  error: string | null;
  onSubmit: (mode: "signin" | "signup", email: string, password: string) => void;
  onBack: () => void;
};

export function AuthPage({
  initialMode = "signup",
  busy,
  error,
  onSubmit,
  onBack,
}: Props) {
  const [mode, setMode] = useState<"signin" | "signup">(initialMode);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    onSubmit(mode, email.trim(), password);
  }

  return (
    <div className="login-shell">
      <motion.div
        className="login-card login-card--wide"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
      >
        <div className="login-card__hero">
          <motion.div
            className="login-orb"
            animate={{ scale: [1, 1.08, 1], opacity: [0.5, 0.85, 0.5] }}
            transition={{ duration: 3, repeat: Infinity }}
          />
          <Shield size={44} className="login-card__shield" />
        </div>
        <h1>{mode === "signup" ? "Create account" : "Sign in"}</h1>
        <p className="subtitle">
          {mode === "signup"
            ? "Save your subscription after the free day"
            : "Access your paid account"}
        </p>

        <div className="auth-tabs">
          <button
            type="button"
            className={mode === "signup" ? "auth-tabs__on" : ""}
            onClick={() => setMode("signup")}
          >
            <UserPlus size={14} /> Sign up
          </button>
          <button
            type="button"
            className={mode === "signin" ? "auth-tabs__on" : ""}
            onClick={() => setMode("signin")}
          >
            Sign in
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          <label className="field">
            <Mail size={14} /> Email
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
              autoComplete="email"
            />
          </label>
          <label className="field">
            <Lock size={14} /> Password
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              minLength={6}
              autoComplete={
                mode === "signup" ? "new-password" : "current-password"
              }
            />
          </label>
          {error ? <div className="error">{error}</div> : null}
          <button type="submit" className="btn btn--primary" disabled={busy}>
            {busy
              ? "Please wait…"
              : mode === "signup"
                ? "Create account"
                : "Sign in"}
          </button>
        </form>
        <button type="button" className="btn btn--ghost auth-back" onClick={onBack}>
          Back to app
        </button>
      </motion.div>
    </div>
  );
}
