function getBasicCanvasSample(): string {
  if (typeof document === "undefined") return "ssr";
  try {
    const canvas = document.createElement("canvas");
    canvas.width = 200;
    canvas.height = 40;
    const ctx = canvas.getContext("2d");
    if (!ctx) return "no-ctx";
    ctx.textBaseline = "top";
    ctx.font = "14px 'Segoe UI'";
    ctx.fillStyle = "#0f172a";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "#3b82f6";
    ctx.fillText("ipnova-desktop", 2, 2);
    return canvas.toDataURL();
  } catch {
    return "canvas-blocked";
  }
}

async function digestHex(input: string): Promise<string> {
  const data = new TextEncoder().encode(input);
  const digest = await crypto.subtle.digest("SHA-256", data);
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("")
    .slice(0, 32);
}

async function randomFallbackHash(): Promise<string> {
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  const seed = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  return digestHex(`ipnova-fallback|${seed}|${Date.now()}`);
}

export async function generateDeviceHash(): Promise<string> {
  if (typeof navigator === "undefined" || typeof window === "undefined") {
    return randomFallbackHash();
  }
  if (!crypto?.subtle) {
    return randomFallbackHash();
  }
  const parts = [
    navigator.userAgent,
    navigator.language,
    String(screen.width),
    String(screen.height),
    String(window.devicePixelRatio ?? 1),
    getBasicCanvasSample(),
  ];
  return digestHex(parts.join("|"));
}
