/** Static steps shown in Settings; live checks run via Tauri (`runTroubleshootingChecks`). */
export const WINDOWS_TROUBLESHOOTING_STEPS = [
  {
    id: "singbox",
    title: "sing-box on disk",
    hint: "Run scripts/download-singbox.ps1. If the .exe disappears, add a Defender exclusion (Settings → Windows Defender).",
  },
  {
    id: "port_2080",
    title: "Port 2080 free",
    hint: "Close other VPN apps. Settings → Run live checks shows which process uses the port.",
  },
  {
    id: "pac_wpad",
    title: "PAC / corporate proxy",
    hint: "If you use a company PAC script, Connect will replace it temporarily. WinINet GPO may block changes — ask IT or use TUN mode.",
  },
  {
    id: "wininet",
    title: "Browser proxy (WinINet)",
    hint: "Edge and Chrome use system proxy. After Connect, open https://api.ipify.org or check Exit IP in the app.",
  },
  {
    id: "winhttp",
    title: "WinHTTP (Windows services)",
    hint: "If WinHTTP stays off, browsers may still work. Reconnect once; on Win11 24H2 try «Apply 24H2 fix» in Proxy diagnostics.",
  },
  {
    id: "other_vpn",
    title: "Other VPN software",
    hint: "Disconnect OpenVPN, NordVPN, WireGuard, etc. before system proxy mode.",
  },
  {
    id: "foreign_proxy",
    title: "Corporate / third-party proxy",
    hint: "IPNOVA replaces it with 127.0.0.1:2080 while connected and restores on disconnect.",
  },
  {
    id: "wininet_gpo",
    title: "WinINet group policy",
    hint: "IT GPO may block proxy changes — ask IT or use TUN mode.",
  },
  {
    id: "edge_policy",
    title: "Edge browser policy",
    hint: "ProxyMode=fixed_servers or direct in GPO overrides system proxy.",
  },
  {
    id: "wcm_24h2",
    title: "Windows 11 24H2 Wi‑Fi",
    hint: "Apply 24H2 fix in Proxy diagnostics if Wi‑Fi drops after VPN.",
  },
  {
    id: "firefox",
    title: "Firefox",
    hint: "Enable «Use system proxy settings» or use TUN mode.",
  },
  {
    id: "backup",
    title: "Proxy backup",
    hint: "proxy-backup.json is written on Connect; Disconnect restores your previous proxy.",
  },
  {
    id: "auto_repair",
    title: "Auto-Repair",
    hint: "Settings → Proxy diagnostics → Auto-Repair fixes orphan proxy, PAC, 2026 Wi‑Fi/WinHTTP, and DNS cache.",
  },
  {
    id: "support",
    title: "Still failing?",
    hint: "Settings → Copy diagnostics and send to support (includes proxy + Defender state).",
  },
] as const;
