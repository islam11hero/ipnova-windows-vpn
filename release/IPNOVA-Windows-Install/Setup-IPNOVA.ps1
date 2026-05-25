# IPNOVA VPN setup (called from Install IPNOVA.bat) — ASCII only for Windows PowerShell 5.1
param(
    [switch]$Uninstall,
    [switch]$Silent
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Windows.Forms
$Product = "IPNOVA VPN"
$InstallRoot = Join-Path $env:LOCALAPPDATA "Programs\IPNOVA VPN"
$Desktop = [Environment]::GetFolderPath("Desktop")
$StartMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$Here = $PSScriptRoot

function Get-InstallRoot {
    param([string]$Start)
    if (Test-Path (Join-Path $Start "Install IPNOVA.bat")) {
        return $Start
    }
    $parent = Split-Path $Start -Parent
    if ($parent -and (Test-Path (Join-Path $parent "Install IPNOVA.bat"))) {
        return $parent
    }
    return $Start
}

$Here = Get-InstallRoot $Here

function Write-Step($msg) {
    if (-not $Silent) { Write-Host $msg -ForegroundColor Cyan }
}

if ($Uninstall) {
    Write-Step "Removing $Product..."
    Stop-Process -Name "IPNOVA VPN", "ipnova-windows-vpn", "sing-box" -Force -ErrorAction SilentlyContinue
    Remove-Item $InstallRoot -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item (Join-Path $Desktop "$Product.lnk") -Force -ErrorAction SilentlyContinue
    Remove-Item (Join-Path $StartMenu "$Product.lnk") -Force -ErrorAction SilentlyContinue
    Write-Host "Uninstalled." -ForegroundColor Green
    exit 0
}

$bundledSetup = @(
    (Join-Path $Here "Install IPNOVA (Setup).exe"),
    (Join-Path $Here "IPNOVA VPN_0.1.0_x64-setup.exe"),
    (Join-Path $Here "setup.exe")
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if ($bundledSetup) {
    Write-Step "Running setup: $(Split-Path $bundledSetup -Leaf)"
    if ($Silent) {
        Start-Process -FilePath $bundledSetup -ArgumentList "/S" -Wait
    } else {
        Start-Process -FilePath $bundledSetup -Wait
    }
    Write-Host "Installation finished." -ForegroundColor Green
    exit 0
}

$appSrc = Join-Path $Here "app"
$hasApp = Test-Path $appSrc
if ($hasApp) {
    $exeInApp = Get-ChildItem $appSrc -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match "IPNOVA|ipnova" -and $_.Name -notmatch "sing-box" } |
        Select-Object -First 1
    if (-not $exeInApp) {
        $hasApp = $false
    }
}

if (-not $hasApp) {
    [System.Windows.Forms.MessageBox]::Show(
        "Application files are missing (no IPNOVA .exe in the app folder).`n`n" +
        "The developer must run on Windows:`nbuild-windows-installer.ps1`n`n" +
        "Then send the new ZIP that includes setup.exe or the app folder.",
        $Product,
        "OK",
        "Error"
    ) | Out-Null
    exit 1
}

Write-Step "Installing $Product to $InstallRoot ..."
New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null
Copy-Item "$appSrc\*" $InstallRoot -Recurse -Force

$resSrc = Join-Path $Here "resources"
if (Test-Path $resSrc) {
    Copy-Item $resSrc (Join-Path $InstallRoot "resources") -Recurse -Force
}

$mainExe = Get-ChildItem $InstallRoot -Filter "*.exe" -Recurse |
    Where-Object { $_.Name -match "IPNOVA|ipnova" -and $_.Name -notmatch "sing-box" } |
    Select-Object -First 1

if (-not $mainExe) {
    throw "IPNOVA executable not found after copy"
}

$Wsh = New-Object -ComObject WScript.Shell
$lnkD = $Wsh.CreateShortcut((Join-Path $Desktop "$Product.lnk"))
$lnkD.TargetPath = $mainExe.FullName
$lnkD.WorkingDirectory = $mainExe.DirectoryName
$lnkD.Save()

$lnkS = $Wsh.CreateShortcut((Join-Path $StartMenu "$Product.lnk"))
$lnkS.TargetPath = $mainExe.FullName
$lnkS.WorkingDirectory = $mainExe.DirectoryName
$lnkS.Save()

Copy-Item $PSCommandPath (Join-Path $InstallRoot "Uninstall-IPNOVA.ps1") -Force
Set-Content -Path (Join-Path $InstallRoot "Uninstall IPNOVA.bat") -Encoding ASCII -Value @"
@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%LOCALAPPDATA%\Programs\IPNOVA VPN\Uninstall-IPNOVA.ps1" -Uninstall
pause
"@

Write-Host ""
Write-Host "Installed successfully!" -ForegroundColor Green
Write-Host "  Desktop / Start Menu shortcut: $Product" -ForegroundColor White

if (-not $Silent) {
    $run = [System.Windows.Forms.MessageBox]::Show(
        "Installation complete. Launch IPNOVA VPN now?",
        $Product,
        "YesNo",
        "Question"
    )
    if ($run -eq "Yes") {
        Start-Process -FilePath $mainExe.FullName
    }
}
