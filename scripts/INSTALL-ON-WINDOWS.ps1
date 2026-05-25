# IPNOVA VPN - Build and install on THIS Windows PC (run once as developer)
# Right-click -> Run with PowerShell   OR:
# powershell -ExecutionPolicy Bypass -File ".\scripts\INSTALL-ON-WINDOWS.ps1"

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  IPNOVA VPN - Windows install builder" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Project: $Root" -ForegroundColor Gray
Write-Host ""

function Test-Tool($name) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        Write-Host "MISSING: $name" -ForegroundColor Red
        return $false
    }
    Write-Host "OK: $name" -ForegroundColor Green
    return $true
}

$ok = $true
$ok = (Test-Tool "node") -and $ok
$ok = (Test-Tool "npm") -and $ok
$ok = (Test-Tool "cargo") -and $ok

if (-not $ok) {
    Write-Host ""
    Write-Host "Install first:" -ForegroundColor Yellow
    Write-Host "  1) Node.js 20 LTS  https://nodejs.org" -ForegroundColor White
    Write-Host "  2) Rust            https://rustup.rs" -ForegroundColor White
    Write-Host "  3) Restart PC, run this script again" -ForegroundColor White
    Write-Host ""
    Write-Host "Also install: Visual Studio Build Tools (C++) or 'Desktop development with C++'" -ForegroundColor Yellow
    pause
    exit 1
}

Write-Host ""
Write-Host "[1/5] Download sing-box..." -ForegroundColor Cyan
& "$Root\scripts\download-singbox.ps1"
New-Item -ItemType Directory -Force -Path "$Root\src-tauri\resources\sing-box\windows-amd64" | Out-Null
Copy-Item "$Root\resources\sing-box\windows-amd64\*" "$Root\src-tauri\resources\sing-box\windows-amd64\" -Force

Write-Host "[2/5] npm ci (may take a few minutes)..." -ForegroundColor Cyan
npm ci

Write-Host "[3/5] Build web UI..." -ForegroundColor Cyan
npm run build

Write-Host "[4/5] Build Windows app (10-20 min first time)..." -ForegroundColor Cyan
npm run tauri:build

Write-Host "[5/5] Find installer..." -ForegroundColor Cyan
$setup = Get-ChildItem "$Root\src-tauri\target\release\bundle" -Recurse -Filter "*setup*.exe" -ErrorAction SilentlyContinue |
    Select-Object -First 1

if (-not $setup) {
    $setup = Get-ChildItem "$Root\src-tauri\target\release" -Recurse -Filter "*.exe" -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match "IPNOVA" -and $_.Name -notmatch "sing-box" } |
        Select-Object -First 1
}

if (-not $setup) {
    Write-Host "Build finished but installer .exe not found." -ForegroundColor Red
    Write-Host "Check: src-tauri\target\release\bundle\" -ForegroundColor Yellow
    pause
    exit 1
}

$dest = Join-Path $env:USERPROFILE "Desktop\IPNOVA-VPN-Setup.exe"
Copy-Item $setup.FullName $dest -Force

Write-Host ""
Write-Host "SUCCESS!" -ForegroundColor Green
Write-Host "Installer copied to:" -ForegroundColor White
Write-Host "  $dest" -ForegroundColor Cyan
Write-Host ""
Write-Host "Double-click IPNOVA-VPN-Setup.exe on your Desktop to install the app." -ForegroundColor Yellow
Write-Host ""

$run = Read-Host "Run installer now? (Y/N)"
if ($run -eq "Y" -or $run -eq "y") {
    Start-Process -FilePath $dest -Wait
}

pause
