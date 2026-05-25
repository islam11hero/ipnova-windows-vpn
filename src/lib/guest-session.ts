export type GuestSession = {
  orderId: string;
  deviceHash: string;
  marzbanUsername: string;
  expiresAt: number;
  planName: string;
  expired: boolean;
};

const STORAGE_KEY = "ipnova.guest_session";

export function loadGuestSession(): GuestSession | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    return JSON.parse(raw) as GuestSession;
  } catch {
    return null;
  }
}

export function saveGuestSession(session: GuestSession): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(session));
}

export function clearGuestSession(): void {
  localStorage.removeItem(STORAGE_KEY);
}

export function isGuestExpired(session: GuestSession): boolean {
  if (session.expired) return true;
  return session.expiresAt * 1000 <= Date.now();
}

export function formatCountdown(expiresAtSec: number): string {
  const ms = expiresAtSec * 1000 - Date.now();
  if (ms <= 0) return "Expired";
  const h = Math.floor(ms / 3_600_000);
  const m = Math.floor((ms % 3_600_000) / 60_000);
  if (h > 0) return `${h}h ${m}m`;
  return `${m} min`;
}
