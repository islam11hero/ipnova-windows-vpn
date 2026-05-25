# IPNOVA VPN — Release

## Ready now (this machine)

| Artifact | Path |
|----------|------|
| **Download kit** (sing-box, docs, build scripts) | `release/dist/IPNOVA-VPN-0.1.0-download-kit.zip` |
| **Logo** | `release/brand/ipnova-logo-512.png` |
| **sing-box** (Windows amd64) | `resources/sing-box/windows-amd64/` |

## Full installer (.exe / .msi)

### Option A — GitHub + WiFi download (recommended)

1. Push repo to GitHub (see `docs/GITHUB-WIFI-AR.md`)
2. `git tag v0.1.0 && git push origin v0.1.0`
3. Customer downloads from **Releases** → `IPNOVA-VPN-Setup.exe`

### Option B — Build on your Windows PC

```powershell
.\scripts\INSTALL-ON-WINDOWS.ps1
```

Or: `.\scripts\build-release.ps1` → `release\dist\IPNOVA-VPN-0.1.0-win64.zip`

## Prepare kit only (macOS / Linux / Windows)

```bash
npm run release:kit
# or
powershell -File scripts\prepare-download-kit.ps1
```
