#!/usr/bin/env bash
# Integration test for scripts/update-packaging.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/bucket" "$TMP/Formula" "$TMP/scripts"
cp "$ROOT/bucket/gh-tray.json" "$TMP/bucket/gh-tray.json"
cp "$ROOT/Formula/gh-tray.rb"  "$TMP/Formula/gh-tray.rb"
cp "$ROOT/scripts/update_formula.py" "$TMP/scripts/update_formula.py"
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

# Formula assertions
grep -q 'version "1.2.3"' "$TMP/Formula/gh-tray.rb"
grep -q 'v1.2.3/gh-tray-aarch64-apple-darwin.tar.gz' "$TMP/Formula/gh-tray.rb"
grep -q 'v1.2.3/gh-tray-x86_64-apple-darwin.tar.gz' "$TMP/Formula/gh-tray.rb"
grep -q 'sha256 "aaaa111111111111111111111111111111111111111111111111111111111111"' "$TMP/Formula/gh-tray.rb"
grep -q 'sha256 "bbbb222222222222222222222222222222222222222222222222222222222222"' "$TMP/Formula/gh-tray.rb"
echo "OK"
