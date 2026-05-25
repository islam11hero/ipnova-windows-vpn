#!/usr/bin/env bash
# macOS/Linux helper — same as fetch-profile.ps1 for API smoke test
set -euo pipefail

TOKEN="${1:?Usage: $0 <supabase_access_token> [api_base]}"
API_BASE="${2:-http://127.0.0.1:3000}"
OUT="${3:-./sing-box-config.json}"

curl -fsS \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Accept: application/json" \
  "${API_BASE%/}/api/client/vpn/profile" \
  | jq -r '.profile.config' > "${OUT}"

echo "Saved ${OUT}"
