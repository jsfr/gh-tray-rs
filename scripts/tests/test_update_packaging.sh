#!/usr/bin/env bash
# Integration test for scripts/update-packaging.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/bucket" "$TMP/Casks" "$TMP/scripts"
cp "$ROOT/bucket/gh-tray.json" "$TMP/bucket/gh-tray.json"
cp "$ROOT/Casks/gh-tray.rb"  "$TMP/Casks/gh-tray.rb"
cp "$ROOT/scripts/update_cask.py" "$TMP/scripts/update_cask.py"
cp "$ROOT/scripts/update-packaging.sh" "$TMP/scripts/update-packaging.sh"

(
  cd "$TMP"
  ./scripts/update-packaging.sh \
    "1.2.3" \
    "cccc333333333333333333333333333333333333333333333333333333333333" \
    "aaaa111111111111111111111111111111111111111111111111111111111111" \
    "bbbb222222222222222222222222222222222222222222222222222222222222"
)

# Scoop manifest assertions
jq -e '.version == "1.2.3"' "$TMP/bucket/gh-tray.json" >/dev/null
jq -e '.architecture."64bit".hash == "cccc333333333333333333333333333333333333333333333333333333333333"' "$TMP/bucket/gh-tray.json" >/dev/null
jq -e '.architecture."64bit".url | endswith("/v1.2.3/gh-tray-x86_64-pc-windows-msvc.zip")' "$TMP/bucket/gh-tray.json" >/dev/null

# Cask assertions
grep -q 'version "1.2.3"' "$TMP/Casks/gh-tray.rb"
grep -q 'sha256 arm:   "aaaa111111111111111111111111111111111111111111111111111111111111",' "$TMP/Casks/gh-tray.rb"
grep -q '         intel: "bbbb222222222222222222222222222222222222222222222222222222222222"' "$TMP/Casks/gh-tray.rb"
grep -q '/v#{version}/gh-tray-#{arch}-apple-darwin.tar.gz' "$TMP/Casks/gh-tray.rb"
echo "OK"
