# Prepares sing-box + icons + frontend (no Rust). Run before build-release on a dev machine.
# Usage: powershell -ExecutionPolicy Bypass -File .\scripts\prepare-download-kit.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

& (Join-Path $Root "scripts\download-singbox.ps1")
npm ci
npm run build
npx tauri icon "release\brand\ipnova-logo-512.png"
Write-Host "Kit ready. On Windows run: .\scripts\build-release.ps1" -ForegroundColor Green
