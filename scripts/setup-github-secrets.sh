#!/usr/bin/env bash
# Push VITE_* from local .env to GitHub Actions secrets (repo: islam11hero/ipnova-windows-vpn)
# Requires: gh auth login   OR   export GH_TOKEN=ghp_...

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="${1:-$ROOT/.env}"
REPO="${GITHUB_REPO:-islam11hero/ipnova-windows-vpn}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing $ENV_FILE — copy .env.example and fill VITE_* values first."
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "Install GitHub CLI: https://cli.github.com/"
  exit 1
fi

unset GITHUB_TOKEN
if ! gh auth status -h github.com >/dev/null 2>&1; then
  echo "Run: gh auth login"
  exit 1
fi

set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

for name in VITE_API_BASE_URL VITE_SITE_URL VITE_SUPABASE_URL VITE_SUPABASE_ANON_KEY; do
  val="${!name:-}"
  if [[ -z "$val" ]] || [[ "$val" == *"YOUR_PROJECT"* ]] || [[ "$val" == *"your-anon"* ]]; then
    echo "Skip $name (empty or placeholder in $ENV_FILE)"
    continue
  fi
  echo "Setting secret $name on $REPO ..."
  printf '%s' "$val" | gh secret set "$name" --repo "$REPO"
done

echo ""
echo "Done. Re-run Release workflow or push tag v0.1.0 to rebuild with secrets."
