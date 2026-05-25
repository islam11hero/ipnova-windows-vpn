# Proof-of-concept: fetch sing-box JSON from vpnnovo API and save to disk.
param(
  [Parameter(Mandatory = $true)]
  [string]$AccessToken,
  [string]$ApiBase = "http://127.0.0.1:3000",
  [string]$OutFile = ".\sing-box-config.json"
)

$ErrorActionPreference = "Stop"
$headers = @{ Authorization = "Bearer $AccessToken"; Accept = "application/json" }
$uri = "$($ApiBase.TrimEnd('/'))/api/client/vpn/profile"

Write-Host "GET $uri"
$res = Invoke-RestMethod -Uri $uri -Headers $headers -Method Get
if (-not $res.success) {
  throw $res.error
}

$res.profile.config | ConvertTo-Json -Depth 100 | Set-Content -Path $OutFile -Encoding UTF8
Write-Host "Saved sing-box config to $OutFile"
