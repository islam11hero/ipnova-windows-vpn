# Builds app + customer ZIP (flat root: Install IPNOVA.bat at top level)
# Usage (Windows 10/11): powershell -ExecutionPolicy Bypass -File .\scripts\build-windows-installer.ps1

param(
    [string]$Version = "0.1.0",
    [switch]$SkipNpmCi,
    [switch]$SkipSingbox,
    [switch]$SkipTauriBuild
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

Write-Host "=== IPNOVA: Build + Windows installer ZIP v$Version ===" -ForegroundColor Cyan

foreach ($cmd in @("node", "npm", "cargo")) {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
        throw "Missing $cmd — install Node 20 + Rust from https://rustup.rs"
    }
}

if (-not $SkipSingbox) {
    & "$Root\scripts\download-singbox.ps1"
}
Copy-Item "$Root\resources\sing-box\windows-amd64\*" "$Root\src-tauri\resources\sing-box\windows-amd64\" -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path "$Root\src-tauri\resources\sing-box\windows-amd64" | Out-Null

if (-not $SkipTauriBuild) {
    if (-not $SkipNpmCi) {
        npm ci
        npm run build
        npm run tauri:build
    }
}

$exe = Get-ChildItem -Path "$Root\src-tauri\target\release" -Recurse -Filter "*.exe" -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match "IPNOVA|ipnova" -and $_.Name -notmatch "sing-box" } |
    Select-Object -First 1

if (-not $exe) {
    $exe = Get-ChildItem "$Root\src-tauri\target\release\bundle\nsis" -Recurse -Filter "*setup*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
}

$pkg = Join-Path $Root "release\IPNOVA-Windows-Install"
$appDir = Join-Path $pkg "app"
Remove-Item $pkg -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $appDir, (Join-Path $pkg "resources\sing-box\windows-amd64") | Out-Null

if ($exe -and $exe.DirectoryName -match "bundle") {
    # NSIS build: copy whole bundle folder
    $bundleDir = $exe.Directory.Parent.FullName
    Copy-Item "$bundleDir\*" $appDir -Recurse -Force
    Copy-Item $exe.FullName (Join-Path $pkg "Install IPNOVA (Setup).exe") -Force
} elseif ($exe) {
    Copy-Item $exe.FullName (Join-Path $appDir $exe.Name) -Force
    Get-ChildItem $exe.Directory -Filter "*.dll" -ErrorAction SilentlyContinue | Copy-Item -Destination $appDir -Force
} else {
    throw "Build finished but no IPNOVA .exe found under src-tauri\target\release"
}

Copy-Item "$Root\resources\sing-box\windows-amd64\*" (Join-Path $pkg "resources\sing-box\windows-amd64\") -Force
Copy-Item "$Root\scripts\Setup-IPNOVA.ps1" $pkg -Force
Copy-Item "$Root\scripts\Install IPNOVA.bat" $pkg -Force
Copy-Item "$Root\release\brand\ipnova-logo-512.png" $pkg -Force -ErrorAction SilentlyContinue
Copy-Item "$Root\release\README-AR.md" $pkg -Force -ErrorAction SilentlyContinue

@"
IPNOVA VPN v$Version
================
1. Double-click: Install IPNOVA.bat
2. Follow the steps (Install / Launch)
3. Sign in and press Connect

Requires Windows 10 (1809+) or 11, 64-bit.
"@ | Set-Content (Join-Path $pkg "README.txt") -Encoding UTF8

$zip = Join-Path $Root "release\dist\IPNOVA-Windows-Install-v$Version.zip"
New-Item -ItemType Directory -Force -Path (Split-Path $zip) | Out-Null
if (Test-Path $zip) { Remove-Item $zip -Force }
# Flat ZIP: files at root (avoids IPNOVA-Windows-Install\IPNOVA-Windows-Install\)
Push-Location $pkg
try {
    Compress-Archive -Path * -DestinationPath $zip -Force
} finally {
    Pop-Location
}
# UTF-8 BOM helps PowerShell 5.1 read Setup-IPNOVA.ps1
$setupPs1 = Join-Path $pkg "Setup-IPNOVA.ps1"
if (Test-Path $setupPs1) {
    $content = Get-Content $setupPs1 -Raw -Encoding UTF8
    [System.IO.File]::WriteAllText($setupPs1, $content, (New-Object System.Text.UTF8Encoding $true))
    Remove-Item $zip -Force
    Push-Location $pkg
    try { Compress-Archive -Path * -DestinationPath $zip -Force } finally { Pop-Location }
}

Write-Host ""
Write-Host "SUCCESS — send this file to customers:" -ForegroundColor Green
Write-Host "  $zip" -ForegroundColor White
Write-Host "Inside ZIP: Install IPNOVA.bat (one double-click)" -ForegroundColor Cyan
