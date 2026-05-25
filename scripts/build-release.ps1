# IPNOVA VPN — build Windows installers + release ZIP (run on Windows 10/11 x64)
# Usage: powershell -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
# Optional: -Version 0.1.0 -SkipSingbox

param(
    [string]$Version = "0.1.0",
    [switch]$SkipSingbox,
    [switch]$SkipNpmCi
)

# SkipSingbox: sing-box already downloaded in CI

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Require-Command($name) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        throw "Required command not found: $name"
    }
}

Write-Host "=== IPNOVA VPN release build v$Version ===" -ForegroundColor Cyan

Require-Command node
Require-Command npm
Require-Command cargo

if (-not $SkipNpmCi) {
    Write-Host "npm ci ..."
    npm ci
}

if (-not $SkipSingbox) {
    Write-Host "Downloading sing-box + wintun ..."
    & (Join-Path $Root "scripts\download-singbox.ps1")
}

$singbox = Join-Path $Root "resources\sing-box\windows-amd64\sing-box.exe"
if (-not (Test-Path $singbox)) {
    throw "Missing $singbox — run scripts\download-singbox.ps1"
}

$tauriRes = Join-Path $Root "src-tauri\resources\sing-box\windows-amd64"
New-Item -ItemType Directory -Force -Path $tauriRes | Out-Null
Copy-Item (Join-Path $Root "resources\sing-box\windows-amd64\*") $tauriRes -Force
Write-Host "Copied sing-box into src-tauri/resources for Tauri bundle"

Write-Host "Building frontend ..."
npm run build

Write-Host "Building Tauri (MSI + NSIS) — may take several minutes ..."
npm run tauri:build

$bundleRoot = Join-Path $Root "src-tauri\target\release\bundle"
if (-not (Test-Path $bundleRoot)) {
    throw "Bundle folder not found: $bundleRoot"
}

$outDir = Join-Path $Root "release\dist\IPNOVA-VPN-$Version-win64"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

$installers = @()
foreach ($pattern in @("*.msi", "*.exe", "*.nsis.zip")) {
    Get-ChildItem -Path $bundleRoot -Recurse -Filter $pattern -ErrorAction SilentlyContinue | ForEach-Object {
        Copy-Item $_.FullName $outDir -Force
        $installers += $_.Name
    }
}

# Portable sing-box folder (for support / manual copy)
$portable = Join-Path $outDir "sing-box"
New-Item -ItemType Directory -Force -Path $portable | Out-Null
Copy-Item (Join-Path $Root "resources\sing-box\windows-amd64\*") $portable -Force

# Docs & scripts for end users
$extras = @(
    "release\README-AR.md",
    "release\README-EN.md",
    "docs\WINDOWS_PROXY_AR.md",
    "docs\WINDOWS_SECURITY_AR.md",
    "docs\WINDOWS_TROUBLESHOOTING_EN.md",
    "scripts\install-defender-exclusions.ps1",
    "release\brand\ipnova-logo-512.png"
)
foreach ($rel in $extras) {
    $src = Join-Path $Root ($rel -replace '\\', [IO.Path]::DirectorySeparatorChar)
    if (Test-Path $src) {
        $dest = Join-Path $outDir (Split-Path $rel -Leaf)
        Copy-Item $src $dest -Force
    }
}

$zipPath = Join-Path $Root "release\dist\IPNOVA-VPN-$Version-win64.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path (Join-Path $outDir "*") -DestinationPath $zipPath -Force

$manifest = @{
    product     = "IPNOVA VPN"
    version     = $Version
    built_at    = (Get-Date).ToUniversalTime().ToString("o")
    platform    = "windows-x64"
    installers  = $installers
    zip         = (Split-Path $zipPath -Leaf)
} | ConvertTo-Json -Depth 4
$manifest | Set-Content (Join-Path $outDir "manifest.json") -Encoding UTF8
$manifest | Set-Content (Join-Path $Root "release\dist\manifest-$Version.json") -Encoding UTF8

Write-Host ""
Write-Host "Done." -ForegroundColor Green
Write-Host "  Folder: $outDir"
Write-Host "  ZIP:    $zipPath"
if ($installers.Count -eq 0) {
    Write-Warning "No MSI/NSIS found under $bundleRoot — check Rust/Tauri build logs."
}
