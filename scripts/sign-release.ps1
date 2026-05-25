# Signs installer + sing-box for SmartScreen reputation (requires OV/OV code signing cert in cert store)
param(
  [Parameter(Mandatory = $true)]
  [string]$Thumbprint,
  [string]$TimestampUrl = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"

function Sign-File($Path) {
  if (-not (Test-Path $Path)) {
    Write-Warning "Skip missing: $Path"
    return
  }
  Write-Host "Signing $Path"
  signtool sign /fd SHA256 /sha1 $Thumbprint /tr $TimestampUrl /td SHA256 $Path
}

$Root = Split-Path -Parent $PSScriptRoot
$Singbox = Join-Path $Root "resources\sing-box\windows-amd64\sing-box.exe"

Sign-File $Singbox

$Bundles = Get-ChildItem -Path (Join-Path $Root "src-tauri\target\release\bundle") -Recurse -Include "*.exe","*.msi" -ErrorAction SilentlyContinue
foreach ($b in $Bundles) {
  Sign-File $b.FullName
}

Write-Host @"

Next steps for fewer SmartScreen warnings:
1. Use the SAME thumbprint for every release (reputation is per file hash).
2. Distribute the same signed build widely — reputation builds over weeks.
3. Submit false positives: https://www.microsoft.com/en-us/wdsi/filesubmission
4. Optional: Microsoft Store avoids SmartScreen entirely.

Set in tauri.conf.json bundle.windows:
  certificateThumbprint, digestAlgorithm sha256, timestampUrl
"@
