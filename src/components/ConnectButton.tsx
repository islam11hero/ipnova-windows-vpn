import { Power } from "lucide-react";

type Props = {
  connected: boolean;
  busy: boolean;
  disabled?: boolean;
  onClick: () => void;
};

export function ConnectButton({ connected, busy, disabled, onClick }: Props) {
  return (
    <button
      type="button"
      className={`connect-btn ${connected ? "connect-btn--on" : ""} ${busy ? "connect-btn--busy" : ""}`}
      onClick={onClick}
      disabled={disabled || busy}
      aria-label={connected ? "Disconnect" : "Connect"}
    >
      <Power size={28} strokeWidth={2} />
      <span className="connect-btn__label">
        {busy ? "Connecting…" : connected ? "Disconnect" : "Connect"}
      </span>
    </button>
  );
}
