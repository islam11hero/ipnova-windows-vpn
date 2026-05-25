import { generateDeviceHash } from "./device-hash";
import { translateApiError } from "./errors";
import {
  clearGuestSession,
  loadGuestSession,
  saveGuestSession,
} from "./guest-session";
import { isBrowserDevMode, isTauriRuntime } from "./platform";

/** Same-origin `/api` in Vite dev (proxy) avoids CORS; full URL in Tauri/production. */
export function apiBaseUrl(): string {
  if (import.meta.env.DEV && isBrowserDevMode() && !isTauriRuntime()) {
    return "";
  }
  return (
    import.meta.env.VITE_API_BASE_URL?.trim() ||
    "http://127.0.0.1:3000"
  ).replace(/\/$/, "");
}

export type VpnEligibility = {
  canConnect: boolean;
  reason?: string;
  reasonAr?: string;
};

export type VpnStatus = {
  orderId: string;
  planName: string;
  marzbanUsername: string;
  status: string;
  usedTraffic: number;
  dataLimit: number;
  expireDate: number | null;
  remainingBytes: number | null;
  usagePercent: number | null;
  eligibility: VpnEligibility;
  telemetryLive: boolean;
  telemetryError?: string;
};

export type VpnProfile = {
  orderId: string;
  marzbanUsername: string;
  config: Record<string, unknown>;
  expiresAt: number | null;
  eligibility: VpnEligibility;
};

type ApiErrorBody = {
  success?: boolean;
  error?: string;
  reason?: string;
  reasonAr?: string;
  status?: VpnStatus;
  profile?: VpnProfile;
  guest?: GuestSessionResponse;
};

async function readApiError(res: Response): Promise<string> {
  try {
    const body = (await res.json()) as ApiErrorBody;
    return translateApiError(
      body.error ?? body.reason ?? body.reasonAr ?? `HTTP ${res.status}`,
    );
  } catch {
    return `HTTP ${res.status}`;
  }
}

async function apiFetch<T>(
  path: string,
  accessToken: string,
  init?: RequestInit,
): Promise<T> {
  let res: Response;
  try {
    res = await fetch(`${apiBaseUrl()}${path}`, {
      ...init,
      headers: {
        Authorization: `Bearer ${accessToken}`,
        Accept: "application/json",
        ...(init?.headers ?? {}),
      },
    });
  } catch {
    throw new Error(
      "Cannot reach server — check internet and API URL in settings",
    );
  }

  const body = (await res.json()) as ApiErrorBody;

  if (!res.ok) {
    throw new Error(
      translateApiError(
        body.error ?? body.reason ?? body.reasonAr ?? `HTTP ${res.status}`,
      ),
    );
  }

  return body as T;
}

export async function fetchVpnStatus(
  accessToken: string,
): Promise<VpnStatus | null> {
  try {
    const data = await apiFetch<{ status: VpnStatus }>(
      "/api/client/vpn/status",
      accessToken,
    );
    return data.status;
  } catch (e) {
    const msg = e instanceof Error ? e.message : "";
    if (
      msg.includes("No active VPN") ||
      msg.includes("No active plan") ||
      msg.includes("404")
    ) {
      return null;
    }
    throw e;
  }
}

function profileQuery(node?: string): string {
  if (!node?.trim()) return "";
  return `?node=${encodeURIComponent(node.trim())}`;
}

export async function fetchVpnProfile(
  accessToken: string,
  opts?: { node?: string },
): Promise<VpnProfile> {
  const data = await apiFetch<{ profile: VpnProfile }>(
    `/api/client/vpn/profile${profileQuery(opts?.node)}`,
    accessToken,
  );
  return data.profile;
}

export async function refreshVpnProfile(
  accessToken: string,
  opts?: { node?: string },
): Promise<VpnProfile> {
  const data = await apiFetch<{ profile: VpnProfile }>(
    `/api/client/vpn/refresh${profileQuery(opts?.node)}`,
    accessToken,
    { method: "POST" },
  );
  return data.profile;
}

export async function linkGuestOrder(
  accessToken: string,
  orderId: string,
): Promise<void> {
  let res: Response;
  try {
    res = await fetch(`${apiBaseUrl()}/api/client/link-order`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${accessToken}`,
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify({ orderId }),
    });
  } catch {
    throw new Error("Could not link trial to account — check server connection");
  }
  if (!res.ok) {
    throw new Error(await readApiError(res));
  }
}

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  const value = bytes / 1024 ** i;
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${units[i]}`;
}

export type GuestSessionResponse = {
  orderId: string;
  deviceHash: string;
  marzbanUsername: string;
  expiresAt: number;
  planName: string;
  isGuest: true;
  expired: boolean;
};

function guestHeaders(guest: { orderId: string; deviceHash: string }) {
  return {
    "X-Guest-Order-Id": guest.orderId,
    "X-Guest-Device-Hash": guest.deviceHash,
  };
}

async function guestFetch<T>(
  path: string,
  guest: { orderId: string; deviceHash: string },
  init?: RequestInit,
): Promise<T> {
  let res: Response;
  try {
    res = await fetch(`${apiBaseUrl()}${path}`, {
      ...init,
      headers: {
        Accept: "application/json",
        ...guestHeaders(guest),
        ...(init?.headers ?? {}),
      },
    });
  } catch {
    throw new Error("Cannot reach server — check vpnnovo is running");
  }

  const body = (await res.json()) as ApiErrorBody;

  if (!res.ok) {
    const err = new Error(
      translateApiError(
        body.error ?? body.reason ?? body.reasonAr ?? `HTTP ${res.status}`,
      ),
    );
    if (res.status === 403) {
      (err as Error & { code?: string }).code = "GUEST_SESSION_INVALID";
    }
    throw err;
  }

  return body as T;
}

export async function startGuestTrial(
  deviceHash: string,
): Promise<GuestSessionResponse> {
  let res: Response;
  try {
    res = await fetch(`${apiBaseUrl()}/api/client/vpn/guest-start`, {
      method: "POST",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: JSON.stringify({ deviceHash }),
    });
  } catch {
    throw new Error("Cannot reach server to start free trial");
  }
  const body = (await res.json()) as ApiErrorBody;
  if (!res.ok || !body.guest) {
    throw new Error(
      translateApiError(
        body.error ?? body.reason ?? body.reasonAr ?? "Could not start free trial",
      ),
    );
  }
  return body.guest;
}

export async function fetchGuestVpnStatus(guest: {
  orderId: string;
  deviceHash: string;
}): Promise<VpnStatus> {
  const data = await guestFetch<{ status: VpnStatus }>(
    "/api/client/vpn/guest/status",
    guest,
  );
  return data.status;
}

export async function fetchGuestVpnProfile(
  guest: {
    orderId: string;
    deviceHash: string;
  },
  opts?: { node?: string },
): Promise<VpnProfile> {
  const data = await guestFetch<{ profile: VpnProfile }>(
    `/api/client/vpn/guest/profile${profileQuery(opts?.node)}`,
    guest,
  );
  return data.profile;
}

/** Ensures a valid guest session (re-provisions if local session is stale). */
export async function ensureGuestSession(): Promise<GuestSessionResponse> {
  const local = loadGuestSession();
  if (local) {
    try {
      await fetchGuestVpnStatus(local);
      return {
        orderId: local.orderId,
        deviceHash: local.deviceHash,
        marzbanUsername: local.marzbanUsername,
        expiresAt: local.expiresAt,
        planName: local.planName,
        isGuest: true,
        expired: false,
      };
    } catch (e) {
      const code = (e as Error & { code?: string }).code;
      if (code !== "GUEST_SESSION_INVALID") throw e;
      clearGuestSession();
    }
  }
  const deviceHash = await generateDeviceHash();
  const guest = await startGuestTrial(deviceHash);
  saveGuestSession(guest);
  return guest;
}

export function formatExpire(expireSec: number | null): string {
  if (!expireSec) return "No expiry date";
  return new Date(expireSec * 1000).toLocaleDateString("en", {
    dateStyle: "medium",
  });
}
