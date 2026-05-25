# يُشغّل على Windows بعد build-release.ps1 لإنشاء مجلد واحد للعميل
param([string]$Version = "0.1.0")

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Out = Join-Path $Root "release\dist\IPNOVA-للعميل-ضغطة-واحدة"
New-Item -ItemType Directory -Force -Path $Out | Out-Null

$scripts = @(
    "IPNOVA-اضغط-للتثبيت.bat",
    "IPNOVA-Setup.hta",
    "Install-IPNOVA.ps1",
    "download-url.example.txt"
)
foreach ($s in $scripts) {
    Copy-Item (Join-Path $Root "scripts\$s") $Out -Force
}

# نسخ المثبّت إن وُجد
$bundle = Join-Path $Root "src-tauri\target\release\bundle"
if (Test-Path $bundle) {
    Get-ChildItem $bundle -Recurse -Include "*setup*.exe","*.msi" | ForEach-Object {
        Copy-Item $_.FullName $Out -Force
        Write-Host "Added installer: $($_.Name)"
    }
}

Rename-Item (Join-Path $Out "download-url.example.txt") "download-url.txt" -Force -ErrorAction SilentlyContinue
@"
ضع في download-url.txt رابط ZIP أو setup.exe إن لم يُرفق المثبّت هنا.
للعميل: ضغطة واحدة = IPNOVA-اضغط-للتثبيت.bat أو IPNOVA-Setup.hta
"@ | Set-Content (Join-Path $Out "اقرأني.txt") -Encoding UTF8

$zip = Join-Path $Root "release\dist\IPNOVA-للعميل-ضغطة-واحدة.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $Out "*") -DestinationPath $zip -Force
Write-Host "Client package: $zip" -ForegroundColor Green
Write-Host "Send ONE file to customer: IPNOVA-اضغط-للتثبيت.bat (with setup.exe in same folder)" -ForegroundColor Cyan
Write-Host "Or send the whole ZIP after filling download-url.txt" -ForegroundColor Cyan
