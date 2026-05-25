#!/usr/bin/env bash
# macOS/Linux: frontend + sing-box + icons (Rust build must run on Windows)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="${1:-0.1.0}"
SINGBOX_VER="1.11.7"

echo "=== IPNOVA prepare download kit v${VERSION} ==="

npm ci
npm run build

if [[ -f "release/brand/ipnova-logo-512.png" ]]; then
  npx tauri icon "release/brand/ipnova-logo-512.png"
fi

OUT="resources/sing-box/windows-amd64"
mkdir -p "$OUT"
if [[ ! -f "$OUT/sing-box.exe" ]]; then
  echo "Downloading sing-box ${SINGBOX_VER}..."
  TMP="$(mktemp -d)"
  curl -fsSL -o "$TMP/sb.zip" \
    "https://github.com/SagerNet/sing-box/releases/download/v${SINGBOX_VER}/sing-box-${SINGBOX_VER}-windows-amd64.zip"
  unzip -qo "$TMP/sb.zip" -d "$TMP"
  find "$TMP" -name "sing-box.exe" -exec cp {} "$OUT/" \;
  curl -fsSL -o "$TMP/wt.zip" "https://www.wintun.net/builds/wintun-0.14.1.zip"
  unzip -qo "$TMP/wt.zip" -d "$TMP/wt"
  find "$TMP/wt" -name "wintun.dll" | head -1 | xargs -I{} cp {} "$OUT/"
  rm -rf "$TMP"
fi

KIT="release/dist/IPNOVA-VPN-${VERSION}-download-kit"
rm -rf "$KIT"
mkdir -p "$KIT/brand" "$KIT/scripts" "$KIT/docs" "$KIT/sing-box"

cp release/README-AR.md release/README-EN.md "$KIT/"
cp release/brand/*.png "$KIT/brand/" 2>/dev/null || true
cp scripts/build-release.ps1 scripts/prepare-download-kit.ps1 scripts/download-singbox.ps1 \
  scripts/install-defender-exclusions.ps1 scripts/smoke-check.ps1 "$KIT/scripts/"
cp docs/WINDOWS_*.md "$KIT/docs/" 2>/dev/null || true
cp -R "$OUT"/* "$KIT/sing-box/"

cat > "$KIT/BUILD-ON-WINDOWS.txt" <<EOF
IPNOVA VPN v${VERSION} — download kit

This folder contains everything except the Windows .exe installer.
On a Windows 10/11 PC with Node 20 + Rust:

  1. Copy the full project (this repo) to the PC
  2. Ensure resources/sing-box/windows-amd64/ matches sing-box/ here
  3. Run: powershell -ExecutionPolicy Bypass -File scripts\\build-release.ps1

Output: release\\dist\\IPNOVA-VPN-${VERSION}-win64.zip (MSI + NSIS + docs)
EOF

ZIP="release/dist/IPNOVA-VPN-${VERSION}-download-kit.zip"
rm -f "$ZIP"
(cd "$(dirname "$KIT")" && zip -rq "$(basename "$ZIP")" "$(basename "$KIT")")

echo "Done: $ZIP"
ls -lh "$ZIP"
