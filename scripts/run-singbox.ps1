# Run sing-box with config from fetch-profile.ps1 (requires Administrator for TUN).
param(
  [string]$Config = ".\sing-box-config.json",
  [string]$SingBoxDir = "..\resources\sing-box\windows-amd64"
)

$ErrorActionPreference = "Stop"
$exe = Join-Path (Resolve-Path $SingBoxDir) "sing-box.exe"
if (-not (Test-Path $exe)) {
  throw "Run scripts\download-singbox.ps1 first"
}
if (-not (Test-Path $Config)) {
  throw "Missing config — run scripts\fetch-profile.ps1 -AccessToken <jwt>"
}

$dir = Split-Path -Parent (Resolve-Path $Config)
Push-Location $dir
try {
  & $exe run -c (Resolve-Path $Config) -D $dir
} finally {
  Pop-Location
}
