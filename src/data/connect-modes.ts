import type { VpnConnectMode } from "../lib/windows-security";

export const VPN_CONNECT_MODES: {
  id: VpnConnectMode;
  label: string;
  hint: string;
}[] = [
  {
    id: "system_proxy",
    label: "System proxy",
    hint: "Browsers + WinHTTP — no admin",
  },
  {
    id: "auto",
    label: "Smart",
    hint: "System proxy first, then TUN if needed",
  },
  { id: "tun", label: "Full TUN", hint: "All apps — UAC required" },
];
