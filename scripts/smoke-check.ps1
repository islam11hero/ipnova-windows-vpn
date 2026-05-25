# Quick local smoke check (run on Windows after build).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

$required = @(
  "src-tauri\src\vpn\mod.rs",
  "src-tauri\src\vpn\connect.rs",
  "src-tauri\src\vpn\status.rs",
  "src-tauri\src\vpn\logs.rs",
  "src-tauri\src\wininet_registry.rs",
  "src\hooks\useAccount.ts",
  "src\hooks\useVpnSession.ts"
)

foreach ($rel in $required) {
  $path = Join-Path $root $rel
  if (-not (Test-Path $path)) {
    throw "Missing: $rel"
  }
}

Write-Host "Smoke check passed."
