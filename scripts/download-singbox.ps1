# Downloads sing-box + wintun for Windows amd64 into resources/sing-box/windows-amd64/
$ErrorActionPreference = "Stop"

$Version = "1.11.7"
$Root = Split-Path -Parent $PSScriptRoot
$OutDir = Join-Path $Root "resources\sing-box\windows-amd64"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$ZipUrl = "https://github.com/SagerNet/sing-box/releases/download/v$Version/sing-box-$Version-windows-amd64.zip"
$ZipPath = Join-Path $env:TEMP "sing-box-$Version-windows-amd64.zip"

Write-Host "Downloading $ZipUrl"
Invoke-WebRequest -Uri $ZipUrl -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $env:TEMP -Force

$Extracted = Get-ChildItem -Path $env:TEMP -Filter "sing-box.exe" -Recurse |
  Where-Object { $_.DirectoryName -like "*windows-amd64*" } |
  Select-Object -First 1

if (-not $Extracted) {
  throw "sing-box.exe not found in archive"
}

Copy-Item $Extracted.FullName (Join-Path $OutDir "sing-box.exe") -Force
Write-Host "Installed sing-box to $OutDir"

$WintunUrl = "https://www.wintun.net/builds/wintun-0.14.1.zip"
$WintunZip = Join-Path $env:TEMP "wintun.zip"
Invoke-WebRequest -Uri $WintunUrl -OutFile $WintunZip
Expand-Archive -Path $WintunZip -DestinationPath $env:TEMP -Force
$WintunDll = Get-ChildItem -Path $env:TEMP -Filter "wintun.dll" -Recurse |
  Where-Object { $_.DirectoryName -like "*amd64*" } |
  Select-Object -First 1

if ($WintunDll) {
  Copy-Item $WintunDll.FullName (Join-Path $OutDir "wintun.dll") -Force
  Write-Host "Installed wintun.dll"
} else {
  Write-Warning "wintun.dll not found — download manually from https://www.wintun.net/"
}
