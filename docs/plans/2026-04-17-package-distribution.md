# Package Distribution (Scoop + Homebrew) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Let users install `gh-tray` via `scoop install gh-tray-rs/gh-tray` on Windows and `brew install jsfr/gh-tray-rs/gh-tray` on macOS, with this single repo hosting both the Scoop bucket manifest and the Homebrew formula, auto-updated by the release workflow.

**Architecture:** The release workflow builds per-target archives (`.tar.gz` on macOS, `.zip` on Windows) with SHA256 sidecar files, uploads them as release assets, then runs a new `update-packaging` job that invokes a small updater script. The script rewrites `bucket/gh-tray.json` with `jq` and `Formula/gh-tray.rb` with a Python stateful-line rewriter, then pushes the updated files to `main` — matching the existing CHANGELOG auto-commit pattern.

**Tech Stack:** Rust (existing), GitHub Actions, `jq`, Python 3 stdlib, Scoop, Homebrew.

**Version Control:** This repo uses Jujutsu (jj). All commit commands below use `jj`. Publishing at the end: `jj bookmark move main --to @-` then `jj git push`.

**Design Reference:** `docs/plans/2026-04-17-package-distribution-design.md`.

---

## Prerequisites

Verify tools available locally (all used only during tests/dev):

```bash
jq --version          # >= 1.6 expected
python3 --version     # >= 3.9 expected
```

If missing: `brew install jq` (python3 ships with macOS).

---

## Task 1: Add MIT LICENSE file

**Files:**
- Create: `LICENSE`

**Step 1: Write the file**

Content:

```
MIT License

Copyright (c) 2026 Jens Fredskov

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

**Step 2: Commit**

```bash
jj commit -m "docs: add MIT LICENSE"
```

---

## Task 2: Scaffold `bucket/gh-tray.json`

The committed manifest is a "seed" that the updater rewrites on each future release. For the initial commit the hash can be a 64-char placeholder — no archive exists for v0.0.2 yet (that release shipped raw binaries), and the next tag (v0.0.3 or later) is what the updater will fix.

**Files:**
- Create: `bucket/gh-tray.json`

**Step 1: Write the manifest**

```json
{
  "version": "0.0.2",
  "description": "Cross-platform system tray app monitoring GitHub PRs",
  "homepage": "https://github.com/jsfr/gh-tray-rs",
  "license": "MIT",
  "architecture": {
    "64bit": {
      "url": "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-x86_64-pc-windows-msvc.zip",
      "hash": "0000000000000000000000000000000000000000000000000000000000000000",
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

**Step 2: Validate JSON**

Run: `jq empty bucket/gh-tray.json && echo OK`
Expected: `OK`

**Step 3: Commit**

```bash
jj commit -m "feat: add scoop bucket manifest"
```

---

## Task 3: Scaffold `Formula/gh-tray.rb`

Same "seed" reasoning as Task 2. Placeholder hashes; the updater overwrites them on the next tagged release.

**Files:**
- Create: `Formula/gh-tray.rb`

**Step 1: Write the formula**

```ruby
class GhTray < Formula
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"
  version "0.0.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
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

**Step 2: Commit**

```bash
jj commit -m "feat: add homebrew formula"
```

---

## Task 4: TDD — `scripts/update_formula.py`

A Python3 stdlib-only script that rewrites `Formula/gh-tray.rb` in place. Signature:

```
python3 scripts/update_formula.py <version> <sha_arm> <sha_intel> <base_url> <formula_path>
```

It performs a stateful line pass:
- Update the single top-level `  version "..."` line.
- Track whether the current line is inside an `on_arm do` or `on_intel do` block (depth by indentation / nesting via `do`/`end` bookkeeping).
- Within each block, rewrite the `url "..."` and `sha256 "..."` lines with the block-appropriate archive URL and hash.

### Step 1: Write the failing test

**File:** `scripts/tests/test_update_formula.py`

```python
"""Test for update_formula.py. Run: python3 scripts/tests/test_update_formula.py"""
from pathlib import Path
import subprocess
import sys
import tempfile

INPUT_FORMULA = '''class GhTray < Formula
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"
  version "0.0.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "gh-tray"
  end

  test do
    system bin/"gh-tray", "--help"
  end
end
'''

EXPECTED_FORMULA = '''class GhTray < Formula
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"
  version "1.2.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v1.2.3/gh-tray-aarch64-apple-darwin.tar.gz"
      sha256 "aaaa111111111111111111111111111111111111111111111111111111111111"
    end
    on_intel do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v1.2.3/gh-tray-x86_64-apple-darwin.tar.gz"
      sha256 "bbbb222222222222222222222222222222222222222222222222222222222222"
    end
  end

  def install
    bin.install "gh-tray"
  end

  test do
    system bin/"gh-tray", "--help"
  end
end
'''

def main():
    repo = Path(__file__).resolve().parents[2]
    script = repo / "scripts" / "update_formula.py"
    with tempfile.NamedTemporaryFile("w", suffix=".rb", delete=False) as f:
        f.write(INPUT_FORMULA)
        formula_path = f.name
    try:
        subprocess.run(
            [sys.executable, str(script),
             "1.2.3",
             "aaaa111111111111111111111111111111111111111111111111111111111111",
             "bbbb222222222222222222222222222222222222222222222222222222222222",
             "https://github.com/jsfr/gh-tray-rs/releases/download/v1.2.3",
             formula_path],
            check=True,
        )
        got = Path(formula_path).read_text()
        if got != EXPECTED_FORMULA:
            print("MISMATCH", file=sys.stderr)
            print("--- expected ---"); print(EXPECTED_FORMULA)
            print("--- got ---"); print(got)
            sys.exit(1)
        print("OK")
    finally:
        Path(formula_path).unlink(missing_ok=True)

if __name__ == "__main__":
    main()
```

### Step 2: Run test to verify it fails

Run: `python3 scripts/tests/test_update_formula.py`
Expected: exit code != 0 (script file doesn't exist yet).

### Step 3: Write minimal implementation

**File:** `scripts/update_formula.py`

```python
#!/usr/bin/env python3
"""Rewrite version + per-arch url/sha256 in a Homebrew formula, in place.

Usage: update_formula.py <version> <sha_arm> <sha_intel> <base_url> <formula_path>
"""
import re
import sys
from pathlib import Path


def rewrite(text: str, version: str, sha_arm: str, sha_intel: str, base_url: str) -> str:
    out = []
    block = None  # None | "arm" | "intel"
    version_done = False
    for line in text.splitlines(keepends=True):
        stripped = line.strip()
        if not version_done and re.match(r'^\s*version\s+"[^"]*"\s*$', line):
            out.append(re.sub(r'"[^"]*"', f'"{version}"', line, count=1))
            version_done = True
            continue
        if stripped.startswith("on_arm do"):
            block = "arm"
        elif stripped.startswith("on_intel do"):
            block = "intel"
        elif stripped == "end" and block is not None:
            block = None
        if block and re.match(r'^\s*url\s+"[^"]*"\s*$', line):
            suffix = "aarch64-apple-darwin.tar.gz" if block == "arm" else "x86_64-apple-darwin.tar.gz"
            out.append(re.sub(r'"[^"]*"', f'"{base_url}/gh-tray-{suffix}"', line, count=1))
            continue
        if block and re.match(r'^\s*sha256\s+"[^"]*"\s*$', line):
            sha = sha_arm if block == "arm" else sha_intel
            out.append(re.sub(r'"[^"]*"', f'"{sha}"', line, count=1))
            continue
        out.append(line)
    return "".join(out)


def main(argv: list[str]) -> int:
    if len(argv) != 6:
        print("usage: update_formula.py <version> <sha_arm> <sha_intel> <base_url> <formula_path>", file=sys.stderr)
        return 2
    _, version, sha_arm, sha_intel, base_url, path = argv
    p = Path(path)
    p.write_text(rewrite(p.read_text(), version, sha_arm, sha_intel, base_url))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
```

### Step 4: Run test to verify it passes

Run: `python3 scripts/tests/test_update_formula.py`
Expected: `OK`

### Step 5: Commit

```bash
jj commit -m "feat: add homebrew formula updater script"
```

---

## Task 5: TDD — `scripts/update-packaging.sh`

Integration-level test: runs the shell script against fixture copies of the manifest + formula, diffs against expected outputs.

### Step 1: Write the failing test

**File:** `scripts/tests/test_update_packaging.sh`

```bash
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
```

Make executable: `chmod +x scripts/tests/test_update_packaging.sh`

### Step 2: Run test to verify it fails

Run: `bash scripts/tests/test_update_packaging.sh`
Expected: exit code != 0 (update-packaging.sh doesn't exist yet).

### Step 3: Write minimal implementation

**File:** `scripts/update-packaging.sh`

```bash
#!/usr/bin/env bash
# Update the Scoop manifest and Homebrew formula for a new release.
#
# Usage: update-packaging.sh <version> <sha_win> <sha_mac_arm> <sha_mac_intel>
#   <version>  bare version, no v prefix (e.g. 0.0.3)
#   <sha_*>    sha256 hex digests of the corresponding release archives
#
# Rewrites bucket/gh-tray.json and Formula/gh-tray.rb in place.

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

python3 scripts/update_formula.py \
  "$VERSION" "$SHA_MAC_ARM" "$SHA_MAC_INTEL" "$BASE_URL" Formula/gh-tray.rb
```

Make executable: `chmod +x scripts/update-packaging.sh`

### Step 4: Run test to verify it passes

Run: `bash scripts/tests/test_update_packaging.sh`
Expected: `OK`

### Step 5: Commit

```bash
jj commit -m "feat: add packaging updater script"
```

---

## Task 6: Release workflow — archive build matrix output

Modify `.github/workflows/release.yml` so each matrix job produces a `.tar.gz` (macOS) or `.zip` (Windows), plus a `.sha256` sidecar containing the bare hex digest, and uploads both.

**Files:**
- Modify: `.github/workflows/release.yml`

### Step 1: Rewrite the `build` job

Replace the entire `build` job stanza with:

```yaml
  build:
    strategy:
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            binary: gh-tray.exe
            archive: gh-tray-x86_64-pc-windows-msvc.zip
          - os: macos-latest
            target: aarch64-apple-darwin
            binary: gh-tray
            archive: gh-tray-aarch64-apple-darwin.tar.gz
          - os: macos-latest
            target: x86_64-apple-darwin
            binary: gh-tray
            archive: gh-tray-x86_64-apple-darwin.tar.gz
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}

      - name: Package (macOS)
        if: runner.os == 'macOS'
        run: |
          tar -czf "${{ matrix.archive }}" -C "target/${{ matrix.target }}/release" "${{ matrix.binary }}"
          shasum -a 256 "${{ matrix.archive }}" | awk '{print $1}' > "${{ matrix.archive }}.sha256"

      - name: Package (Windows)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          Compress-Archive -Path "target/${{ matrix.target }}/release/${{ matrix.binary }}" -DestinationPath "${{ matrix.archive }}"
          $hash = (Get-FileHash -Algorithm SHA256 "${{ matrix.archive }}").Hash.ToLower()
          Set-Content -Path "${{ matrix.archive }}.sha256" -Value $hash -NoNewline

      - uses: actions/upload-artifact@v4
        with:
          name: gh-tray-${{ matrix.target }}
          path: |
            ${{ matrix.archive }}
            ${{ matrix.archive }}.sha256
```

### Step 2: Lint the workflow file

Run: `python3 -c "import yaml, sys; yaml.safe_load(open('.github/workflows/release.yml'))" && echo OK`
Expected: `OK` (confirms YAML parses).

### Step 3: Commit

```bash
jj commit -m "ci: archive release binaries with sha256 sidecar"
```

---

## Task 7: Release workflow — add `update-packaging` job

Append a new job that runs after `release`, downloads the artifacts, extracts the three SHA256s, runs the updater, and pushes the updated files to `main`.

**Files:**
- Modify: `.github/workflows/release.yml`

### Step 1: Append the new job

After the existing `release` job, add:

```yaml
  update-packaging:
    needs: release
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
        with:
          ref: main
          fetch-depth: 0

      - uses: actions/download-artifact@v4

      - name: Read SHA256s
        id: sha
        run: |
          echo "win=$(cat gh-tray-x86_64-pc-windows-msvc/gh-tray-x86_64-pc-windows-msvc.zip.sha256)" >> "$GITHUB_OUTPUT"
          echo "arm=$(cat gh-tray-aarch64-apple-darwin/gh-tray-aarch64-apple-darwin.tar.gz.sha256)" >> "$GITHUB_OUTPUT"
          echo "intel=$(cat gh-tray-x86_64-apple-darwin/gh-tray-x86_64-apple-darwin.tar.gz.sha256)" >> "$GITHUB_OUTPUT"

      - name: Update Scoop manifest and Homebrew formula
        env:
          VERSION: ${{ github.ref_name }}
        run: |
          ./scripts/update-packaging.sh "${VERSION#v}" "${{ steps.sha.outputs.win }}" "${{ steps.sha.outputs.arm }}" "${{ steps.sha.outputs.intel }}"

      - name: Commit and push
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add bucket/gh-tray.json Formula/gh-tray.rb
          git diff --cached --quiet || git commit -m "chore: update scoop manifest and homebrew formula for ${{ github.ref_name }}"
          git push origin HEAD:main
```

### Step 2: Also upload sha256 sidecars as release assets

In the existing `release` job's "Create release" step, the `files: gh-tray-*/*` glob already picks up the sidecars. Confirm by re-reading the step — no change needed, but verify.

Run: `grep -A1 'Create release' .github/workflows/release.yml | grep 'gh-tray-\*/\*'`
Expected: matches one line (glob already present).

### Step 3: Lint the workflow file

Run: `python3 -c "import yaml, sys; yaml.safe_load(open('.github/workflows/release.yml'))" && echo OK`
Expected: `OK`

### Step 4: Commit

```bash
jj commit -m "ci: auto-update scoop and homebrew manifests on release"
```

---

## Task 8: README — Installation section

**Files:**
- Modify: `README.md` (create if it does not exist)

### Step 1: Check whether README.md exists

Run: `test -f README.md && echo exists || echo missing`

- If `missing` — create a minimal README with Installation as the first content section.
- If `exists` — read it, insert an `## Installation` section near the top (after title/description, before any Build/Dev sections).

### Step 2: Installation section content

```markdown
## Installation

### macOS (Homebrew)

```sh
brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install jsfr/gh-tray-rs/gh-tray
```

### Windows (Scoop)

```powershell
scoop bucket add gh-tray-rs https://github.com/jsfr/gh-tray-rs
scoop install gh-tray-rs/gh-tray
```
```

(Nested code fences: outer block uses four backticks, inner uses three.)

### Step 3: Commit

```bash
jj commit -m "docs: add installation instructions for scoop and homebrew"
```

---

## Task 9: End-to-end dry run

Verify the updater produces well-formed outputs when run against the committed seed files with realistic inputs.

### Step 1: Run

```bash
cp bucket/gh-tray.json bucket/gh-tray.json.bak
cp Formula/gh-tray.rb Formula/gh-tray.rb.bak

./scripts/update-packaging.sh \
  "9.9.9" \
  "1111111111111111111111111111111111111111111111111111111111111111" \
  "2222222222222222222222222222222222222222222222222222222222222222" \
  "3333333333333333333333333333333333333333333333333333333333333333"
```

### Step 2: Verify manifest

Run: `jq -e '.version == "9.9.9" and .architecture."64bit".hash == "1111111111111111111111111111111111111111111111111111111111111111"' bucket/gh-tray.json`
Expected: `true`

### Step 3: Verify formula

Run:
```bash
grep -c 'version "9.9.9"' Formula/gh-tray.rb
grep -c 'sha256 "2222222222222222222222222222222222222222222222222222222222222222"' Formula/gh-tray.rb
grep -c 'sha256 "3333333333333333333333333333333333333333333333333333333333333333"' Formula/gh-tray.rb
grep -c 'v9.9.9/gh-tray-aarch64-apple-darwin.tar.gz' Formula/gh-tray.rb
grep -c 'v9.9.9/gh-tray-x86_64-apple-darwin.tar.gz' Formula/gh-tray.rb
```
Expected: each prints `1`.

### Step 4: Restore seed files

```bash
mv bucket/gh-tray.json.bak bucket/gh-tray.json
mv Formula/gh-tray.rb.bak Formula/gh-tray.rb
```

Run: `jj st --no-pager`
Expected: working copy has no changes (files restored to committed state).

No commit needed for this task.

---

## Task 10: Advance `main` and push

### Step 1: Move main bookmark

```bash
jj bookmark move main --to @-
```

### Step 2: Push

```bash
jj git push
```

Expected: remote `main` advances to include all commits from Tasks 1–8.

### Step 3: Verify

Run: `jj log --no-pager -r '@- | @--' -T 'change_id.short() ++ " " ++ description.first_line() ++ "\n"'`
Expected: the two most recent commits show the CI/docs changes from this plan.

---

## Validation checklist (post-merge)

- [ ] Cut a test tag (`jj git push --remote origin --change @- -b refs/tags/v0.0.3-rc1` or via GitHub UI) and confirm the release workflow runs all three jobs green: `build`, `release`, `update-packaging`.
- [ ] Release page shows six assets: three archives + three `.sha256` sidecars.
- [ ] A `chore: update scoop manifest and homebrew formula for v0.0.3-rc1` commit lands on `main`.
- [ ] On macOS: `brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs && brew install jsfr/gh-tray-rs/gh-tray` installs `gh-tray` onto PATH.
- [ ] On Windows: `scoop bucket add gh-tray-rs https://github.com/jsfr/gh-tray-rs && scoop install gh-tray-rs/gh-tray` installs `gh-tray.exe` onto PATH.

---

## Notes

- `bucket/gh-tray.json` and `Formula/gh-tray.rb` initially ship with placeholder hashes (v0.0.2 had no archive uploads). The first real release after this plan lands will rewrite them.
- If the Homebrew formula later grows complexity (shell completions, service plist, etc.), revisit the stateful line-pass updater — regex-style rewrites of structured Ruby become fragile past a certain size.
- `scripts/tests/*` can be wired into CI later (`just test-scripts`) if desired; not in scope here.
