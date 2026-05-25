# IPNOVA VPN — Windows download package

## Requirements

- Windows 10 (1809+) or Windows 11, 64-bit  
- WebView2 runtime (preinstalled on most systems)

## Install

1. Extract `IPNOVA-VPN-0.1.0-win64.zip`
2. Run the **NSIS setup** (`.exe`) or **MSI** installer
3. Launch IPNOVA VPN → sign in or guest trial → Connect

## Build (developers)

On Windows with Node 20 + Rust:

```powershell
.\scripts\build-release.ps1
```

Output: `release\dist\IPNOVA-VPN-0.1.0-win64.zip`

See `README-AR.md` for Arabic instructions and troubleshooting links.
