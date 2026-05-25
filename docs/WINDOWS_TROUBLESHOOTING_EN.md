# Windows connection checklist (≈5 minutes)

Use this when a client says «Connected» but the browser or apps do not use the VPN.

## 1. API & account

- vpnnovo running (`http://127.0.0.1:3000` or production URL in `.env`)
- Guest free day active, or paid member with Marzban user
- In the app: quota and expiry visible on the dashboard

## 2. sing-box

- Run `scripts/download-singbox.ps1` on the PC
- If `sing-box.exe` disappears: **Windows Security → Protection history** → allow / exclusion
- In IPNOVA **Settings → Run live checks** → `sing-box can execute` must be **pass**

## 3. Connect

- Prefer **System proxy** (no admin)
- If another corporate proxy is set, confirm the replace dialog
- Wait for **Exit IP** or a WinHTTP warning (yellow bar)

## 4. Browser

- Edge/Chrome: use system proxy (default when IPNOVA sets WinINet)
- Firefox: **Settings → Network → Use system proxy settings**, or switch to **TUN**
- Edge GPO `ProxyMode=direct` overrides Windows — fix IT policy or use TUN

## 5. Still broken?

1. **Settings → Check proxy & WinHTTP** — WinINet and WinHTTP on `127.0.0.1:2080`
2. Win11 24H2 Wi‑Fi issues → **Apply 24H2 fix** or **Copy admin script**
3. Other VPN running → disconnect Nord/WireGuard/OpenVPN, etc.
4. **Copy diagnostics** → send JSON to support

## Advanced (shared PC)

- **Proxy for all users (admin)**: sets HKLM + `ProxySettingsPerUser=0` — run IPNOVA as Administrator first

## What system proxy does not cover

- Games / UDP / QUIC-heavy apps → use **TUN** mode (UAC once)
- Full DNS for every app → TUN + sing-box DNS rules
