# Run once as Administrator — Defender exclusions for IPNOVA VPN + sing-box + wintun
# Usage: Right-click PowerShell → Run as administrator → .\install-defender-exclusions.ps1

$ErrorActionPreference = "Stop"

if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw "Run this script as Administrator"
}

$Root = Split-Path -Parent $PSScriptRoot
$SingBoxDir = Join-Path $Root "resources\sing-box\windows-amd64"
$SingBox = Join-Path $SingBoxDir "sing-box.exe"
$Wintun = Join-Path $SingBoxDir "wintun.dll"
$AppData = "$env:LOCALAPPDATA\com.ipnova.windows-vpn"

$Paths = @($Root, $SingBoxDir, $AppData) | Where-Object { Test-Path $_ }

foreach ($p in $Paths) {
  Add-MpPreference -ExclusionPath $p -ErrorAction SilentlyContinue
  Write-Host "Excluded path: $p"
}

if (Test-Path -LiteralPath $SingBox) {
  Add-MpPreference -ExclusionPath $SingBox -ErrorAction SilentlyContinue
  Add-MpPreference -ExclusionProcess "sing-box.exe" -ErrorAction SilentlyContinue
  Add-MpPreference -ControlledFolderAccessAllowedApplications $SingBox -ErrorAction SilentlyContinue
  Write-Host "Excluded sing-box.exe (path, process, controlled-folder access)"
} else {
  Write-Warning "sing-box.exe not found at $SingBox — download first or restore from quarantine"
}

if (Test-Path -LiteralPath $Wintun) {
  Add-MpPreference -ExclusionPath $Wintun -ErrorAction SilentlyContinue
  Write-Host "Excluded wintun.dll"
}

$tp = (Get-MpComputerStatus).IsTamperProtected
if ($tp) {
  Write-Host "Tamper Protection is ON — exclusions only apply when this script runs elevated (as now)."
}

$sac = (Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy" -Name VerifiedAndReputablePolicyState -ErrorAction SilentlyContinue).VerifiedAndReputablePolicyState
if ($sac -eq 2) {
  Write-Warning "Smart App Control is ON — sign binaries or disable SAC if sing-box is still blocked."
}

Write-Host "Done. Reboot not required. Submit hashes to https://www.microsoft.com/en-us/wdsi/filesubmission if needed."
