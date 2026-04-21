#!/usr/bin/env bash
# Update the Scoop manifest and Homebrew cask for a new release.
#
# Usage: update-packaging.sh <version> <sha_win> <sha_mac_arm> <sha_mac_intel>
#   <version>  bare version, no v prefix (e.g. 0.0.3)
#   <sha_*>    sha256 hex digests of the corresponding release archives
#
# Rewrites bucket/gh-tray.json and Casks/gh-tray.rb in place.

set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <version> <sha_win> <sha_mac_arm> <sha_mac_intel>" >&2
  exit 2
fi

VERSION="$1"
SHA_WIN="$2"
SHA_MAC_ARM="$3"
SHA_MAC_INTEL="$4"
BASE_URL="https://github.com/jsfr/gh-tray-rs/releases/download/v${VERSION}"

tmp="$(mktemp)"
jq \
  --arg v "$VERSION" \
  --arg url "${BASE_URL}/gh-tray-x86_64-pc-windows-msvc.zip" \
  --arg h "$SHA_WIN" \
  '.version = $v
   | .architecture."64bit".url = $url
   | .architecture."64bit".hash = $h' \
  bucket/gh-tray.json > "$tmp"
mv "$tmp" bucket/gh-tray.json

python3 scripts/update_cask.py \
  "$VERSION" "$SHA_MAC_ARM" "$SHA_MAC_INTEL" Casks/gh-tray.rb
