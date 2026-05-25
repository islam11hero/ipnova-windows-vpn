# IPNOVA - download helper (ASCII only). Prefer: double-click "Install IPNOVA.bat"
param(
    [string]$DownloadUrl = "",
    [switch]$Silent
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Windows.Forms

$Here = if ($PSScriptRoot) { $PSScriptRoot } else { Get-Location }
$Temp = Join-Path $env:TEMP "IPNOVA-Setup"
New-Item -ItemType Directory -Force -Path $Temp | Out-Null

$urlFile = Join-Path $Here "download-url.txt"
if (-not $DownloadUrl -and (Test-Path $urlFile)) {
    $line = Get-Content $urlFile -ErrorAction SilentlyContinue |
        Where-Object { $_ -and $_ -notmatch '^\s*#' } |
        Select-Object -First 1
    if ($line) { $DownloadUrl = $line.Trim() }
}
if (-not $DownloadUrl) {
    $DownloadUrl = $env:IPNOVA_DOWNLOAD_URL
}

function Find-Installer {
    param([string]$Dir)
    Get-ChildItem -Path $Dir -Filter "*.exe" -ErrorAction SilentlyContinue |
        Where-Object {
            $_.Name -match "setup|IPNOVA" -and $_.Name -notmatch "sing-box"
        } |
        Sort-Object Length -Descending |
        Select-Object -First 1
}

$local = Find-Installer $Here
if (-not $local) {
    $local = Find-Installer (Split-Path $Here -Parent)
}

if ($local) {
    Write-Host "Starting installer: $($local.Name)" -ForegroundColor Green
    if ($Silent) {
        Start-Process -FilePath $local.FullName -ArgumentList "/S" -Wait
    } else {
        Start-Process -FilePath $local.FullName -Wait
    }
    exit 0
}

if (-not $DownloadUrl) {
    [System.Windows.Forms.MessageBox]::Show(
        "No installer .exe found next to this script.`n`n" +
        "Put a download URL in download-url.txt (one line), OR`n" +
        "run build-windows-installer.ps1 on Windows first.",
        "IPNOVA VPN",
        "OK",
        "Warning"
    ) | Out-Null
    exit 1
}

$zipPath = Join-Path $Temp "IPNOVA-download.zip"
Write-Host "Downloading..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $DownloadUrl -OutFile $zipPath -UseBasicParsing
Expand-Archive -Path $zipPath -DestinationPath $Temp -Force

$after = Find-Installer $Temp
if (-not $after) {
    $after = Get-ChildItem -Path $Temp -Recurse -Filter "*setup*.exe" -ErrorAction SilentlyContinue |
        Select-Object -First 1
}

if ($after) {
    Write-Host "Running $($after.FullName)" -ForegroundColor Green
    Start-Process -FilePath $after.FullName -Wait
    exit 0
}

$kit = Get-ChildItem -Path $Temp -Directory | Select-Object -First 1
if ($kit) {
    explorer.exe $kit.FullName
    [System.Windows.Forms.MessageBox]::Show(
        "Download complete. Open the folder and run Install IPNOVA.bat`n" +
        "or ask support for a ZIP that includes setup.exe.",
        "IPNOVA VPN",
        "OK",
        "Information"
    ) | Out-Null
}
exit 0
