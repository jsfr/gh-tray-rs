# Package Distribution Design — Scoop + Homebrew

**Date:** 2026-04-17
**Status:** Approved, ready for implementation planning

## Goal

Let users install `gh-tray` via Scoop on Windows and Homebrew on macOS, with this
single repo (`jsfr/gh-tray-rs`) serving as the source for both package managers.

## Decisions

- **Single repo** hosts both the Scoop manifest and the Homebrew formula.
  Users tap Homebrew via the long form because the repo name does not follow
  the `homebrew-*` convention.
- **Fully automated** updates: the release workflow computes SHA256s and
  commits updated manifest + formula directly to `main`, matching the
  existing CHANGELOG auto-commit pattern.
- **Pre-built binaries** distributed as release assets — not a source build.
- **macOS split binaries** with `on_arm` / `on_intel` branches in the formula
  (no universal binary).
- **Archive formats:** `.tar.gz` for macOS (Homebrew convention), `.zip` for
  Windows (Scoop convention). Each with a sidecar `.sha256` file.
- **License:** MIT. A `LICENSE` file is added alongside this work.

## File Layout

```
gh-tray-rs/
├── bucket/
│   └── gh-tray.json              # Scoop manifest
├── Formula/
│   └── gh-tray.rb                # Homebrew formula
├── scripts/
│   ├── update-packaging.sh       # orchestrates manifest + formula update
│   └── update_formula.py         # rewrites the .rb file
├── .github/workflows/
│   └── release.yml               # extended with update-packaging job
├── LICENSE                       # new — MIT
└── README.md                     # new Installation section
```

## Release Artifacts

| Target | Archive | Checksum |
|---|---|---|
| `aarch64-apple-darwin` | `gh-tray-aarch64-apple-darwin.tar.gz` | `.tar.gz.sha256` sidecar |
| `x86_64-apple-darwin` | `gh-tray-x86_64-apple-darwin.tar.gz` | `.tar.gz.sha256` sidecar |
| `x86_64-pc-windows-msvc` | `gh-tray-x86_64-pc-windows-msvc.zip` | `.zip.sha256` sidecar |

Each matrix build step archives the binary and writes the checksum beside it:

- macOS: `tar -czf <name>.tar.gz -C target/<target>/release gh-tray`, then
  `shasum -a 256 <name>.tar.gz | awk '{print $1}' > <name>.tar.gz.sha256`.
- Windows: `Compress-Archive` + `Get-FileHash -Algorithm SHA256`, emit a bare
  hex digest into `<name>.zip.sha256`.

Both archive and sidecar are uploaded as release assets via the existing
`files: gh-tray-*/*` glob.

## Scoop Manifest — `bucket/gh-tray.json`

```json
{
  "version": "0.0.2",
  "description": "Cross-platform system tray app monitoring GitHub PRs",
  "homepage": "https://github.com/jsfr/gh-tray-rs",
  "license": "MIT",
  "architecture": {
    "64bit": {
      "url": "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-x86_64-pc-windows-msvc.zip",
      "hash": "<sha256>",
      "bin": "gh-tray.exe"
    }
  },
  "checkver": {
    "github": "https://github.com/jsfr/gh-tray-rs"
  },
  "autoupdate": {
    "architecture": {
      "64bit": {
        "url": "https://github.com/jsfr/gh-tray-rs/releases/download/v$version/gh-tray-x86_64-pc-windows-msvc.zip"
      }
    }
  }
}
```

Install flow:

```
scoop bucket add gh-tray-rs https://github.com/jsfr/gh-tray-rs
scoop install gh-tray-rs/gh-tray
```

## Homebrew Formula — `Formula/gh-tray.rb`

```ruby
class GhTray < Formula
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"
  version "0.0.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-aarch64-apple-darwin.tar.gz"
      sha256 "<sha256>"
    end
    on_intel do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-x86_64-apple-darwin.tar.gz"
      sha256 "<sha256>"
    end
  end

  def install
    bin.install "gh-tray"
  end

  test do
    system bin/"gh-tray", "--help"
  end
end
```

Install flow:

```
brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install jsfr/gh-tray-rs/gh-tray
```

## Updater Script — `scripts/update-packaging.sh`

Takes version + three SHA256s; rewrites the manifest with `jq` and the formula
via a Python helper.

```bash
#!/usr/bin/env bash
set -euo pipefail
VERSION="$1"            # bare, no v prefix (e.g. 0.0.2)
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

python3 scripts/update_formula.py \
  "$VERSION" "$SHA_MAC_ARM" "$SHA_MAC_INTEL" "$BASE_URL" Formula/gh-tray.rb
```

`scripts/update_formula.py` — stateful line pass:

- Update the top-level `version "..."` line.
- Track whether we are inside an `on_arm do` or `on_intel do` block.
- Within each block, rewrite the `url "..."` and `sha256 "..."` lines using the
  matching arch values.

## Release Workflow Changes — `.github/workflows/release.yml`

1. **Build matrix job:** append archive + sha256 steps per-OS; upload archive
   and `.sha256` as artifacts (still per-target name).
2. **Release job:** unchanged — `files: gh-tray-*/*` already picks everything
   up.
3. **New `update-packaging` job** (`needs: release`, `runs-on: ubuntu-latest`):
   - `actions/checkout@v4` at `ref: main` with write perms.
   - `actions/download-artifact@v4` to pull all three per-target artifact
     folders.
   - Extract SHA256s from the sidecar files.
   - Run `scripts/update-packaging.sh "${GITHUB_REF_NAME#v}" "$SHA_WIN" "$SHA_MAC_ARM" "$SHA_MAC_INTEL"`.
   - `git add bucket/gh-tray.json Formula/gh-tray.rb`,
     `git commit -m "chore: update scoop manifest and homebrew formula for ${GITHUB_REF_NAME}"`,
     `git push origin HEAD:main` (same pattern as existing CHANGELOG push).

## README Changes

Add an `## Installation` section with one fenced code block per OS:

- **macOS (Homebrew):** `brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs && brew install jsfr/gh-tray-rs/gh-tray`
- **Windows (Scoop):** `scoop bucket add gh-tray-rs https://github.com/jsfr/gh-tray-rs && scoop install gh-tray-rs/gh-tray`

## Out of Scope

- Source-based Homebrew build (`depends_on "rust" => :build`).
- Chocolatey, WinGet, Cask, or other package managers.
- Running `brew audit --strict` / `scoop checkver` as CI gates (can be added
  later if desired).
- Universal macOS binary via `lipo`.
- Migration of past releases — packaging starts at the next tagged release
  after this work lands.
