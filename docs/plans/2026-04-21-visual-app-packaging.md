# Visual App Packaging Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Ship `gh-tray` as a proper visual app on both platforms: macOS `.app` bundle via Homebrew Cask (replacing the Formula), Windows `.exe` with an embedded icon plus a Scoop-managed Start-menu shortcut.

**Architecture:** `cargo-bundle` reads `[package.metadata.bundle]` from `Cargo.toml` and produces `gh-tray.app` under `target/<triple>/release/bundle/osx/` on macOS. Windows compiles with a `build.rs` that uses `winres` to embed `assets/icon.ico` into `gh-tray.exe`. The macOS release archive now contains the `.app` directory; the Scoop manifest gains a `shortcuts` entry. A new `scripts/update_cask.py` replaces the Formula-oriented updater, and the release workflow installs `cargo-bundle` before archiving.

**Tech Stack:** Rust, `cargo-bundle` 0.10, `winres` 0.1, Python 3 stdlib, `jq`, GitHub Actions, Homebrew Cask DSL, Scoop manifest schema, ImageMagick (`magick`, one-time for icon generation).

**Version Control:** Jujutsu (jj). Commits use `jj commit -m "…"`. Publishing at the end uses `jj bookmark move main --to @-` + `jj git push`, then `git tag` + `git push origin <tag>` for the release tag.

**Design Reference:** `docs/plans/2026-04-21-visual-app-packaging-design.md`.

---

## Prerequisites

Install local tooling if missing:

```bash
brew install imagemagick        # magick — for icon generation (one-time)
cargo install cargo-bundle --locked --version 0.10.0  # used by Task 4 smoke test
```

Confirm:
```bash
magick --version      # >= 7.1
cargo bundle --help   # shows cargo-bundle help
jq --version          # >= 1.6
python3 --version     # >= 3.9
```

---

## Task 1: Generate placeholder icons + Info.plist fragment

Commit three new asset files: a 1024×1024 PNG (canonical source), a multi-resolution ICO (for Windows `winres`), and the Info.plist fragment (injects `LSUIElement=true`).

**Files:**
- Create: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/assets/icon-1024.png`
- Create: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/assets/icon.ico`
- Create: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/assets/macos/Info.plist.frag`

**Step 1: Generate the PNG**

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
magick -size 1024x1024 canvas:'#24292e' \
  -fill white -gravity center -font assets/Inter-Bold.ttf \
  -pointsize 520 -annotate 0 'gh' \
  assets/icon-1024.png
file assets/icon-1024.png
```
Expected: `PNG image data, 1024 x 1024, 8-bit/color RGB, non-interlaced` (or similar).

**Step 2: Generate the multi-resolution ICO**

```bash
magick assets/icon-1024.png \
  -define icon:auto-resize=256,128,64,48,32,16 \
  assets/icon.ico
file assets/icon.ico
```
Expected: `MS Windows icon resource - 6 icons`.

**Step 3: Write the Info.plist fragment**

Create `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/assets/macos/Info.plist.frag` with this EXACT content (trailing newline):

```xml
<key>LSUIElement</key>
<true/>
```

**Step 4: Commit**

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
jj commit -m "feat: add app icon assets and macOS Info.plist fragment"
```

---

## Task 2: Add `[package.metadata.bundle]` + `winres` build-dep to `Cargo.toml`

**Files:**
- Modify: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/Cargo.toml`

**Step 1: Append the bundle metadata section**

Append to `Cargo.toml` (after the existing `[profile.release]` block):

```toml

[package.metadata.bundle]
name = "gh-tray"
identifier = "io.github.jsfr.gh-tray"
icon = ["assets/icon-1024.png"]
category = "public.app-category.developer-tools"
short_description = "Cross-platform system tray app monitoring GitHub PRs"
osx_minimum_system_version = "11.0"
osx_info_plist_exts = ["assets/macos/Info.plist.frag"]

[target.'cfg(windows)'.build-dependencies]
winres = "0.1"
```

**Step 2: Add `description` to `[package]` (required by cargo-bundle)**

Inside the existing `[package]` block, after the `edition = "2024"` line, add:

```toml
description = "Cross-platform system tray app monitoring GitHub PRs"
```

(cargo-bundle requires the top-level `description` to be present.)

**Step 3: Refresh lockfile**

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
cargo check --quiet
```
Expected: clean (warnings OK, no errors).

**Step 4: Commit**

```bash
jj commit -m "feat: configure cargo-bundle and add winres build-dep"
```

---

## Task 3: Create `build.rs` for Windows icon embedding

**Files:**
- Create: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/build.rs`

**Step 1: Write build.rs**

Create `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/build.rs` with EXACTLY:

```rust
fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}
```

**Step 2: Verify it doesn't break the macOS build**

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
cargo check --quiet
```
Expected: clean (macOS path compiles the empty-function `build.rs`, winres isn't pulled in).

**Step 3: Commit**

```bash
jj commit -m "feat: embed icon in Windows exe via winres"
```

---

## Task 4: Local smoke test — `cargo bundle` produces a valid `.app`

Sanity-check that cargo-bundle works before wiring it into CI.

**Step 1: Build the bundle**

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
cargo bundle --release
```
Expected: output ends with something like `Bundling gh-tray.app (target/release/bundle/osx/gh-tray.app)` and exits 0.

**Step 2: Inspect the bundle**

```bash
ls target/release/bundle/osx/gh-tray.app/Contents/
cat target/release/bundle/osx/gh-tray.app/Contents/Info.plist | grep -A1 -E 'LSUIElement|CFBundleIdentifier'
```
Expected from the first command: `Info.plist  MacOS  Resources`.
Expected from the second command: both `<key>LSUIElement</key><true/>` and `<key>CFBundleIdentifier</key><string>io.github.jsfr.gh-tray</string>` appear.

**Step 3: Verify the binary exists and runs**

```bash
./target/release/bundle/osx/gh-tray.app/Contents/MacOS/gh-tray --help | head -3
```
Expected: `GitHub PR monitor in your system tray` (the clap help output).

**Step 4: Clean up**

```bash
cargo clean
```

No commit — this task is verification only. If any step fails, investigate cargo-bundle output before proceeding.

---

## Task 5: Create `Casks/gh-tray.rb`, delete `Formula/gh-tray.rb`

**Files:**
- Create: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/Casks/gh-tray.rb`
- Delete: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/Formula/gh-tray.rb`

**Step 1: Write the Cask**

Create `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/Casks/gh-tray.rb` with this EXACT content (trailing newline):

```ruby
cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.3"
  sha256 arm:   "0000000000000000000000000000000000000000000000000000000000000000",
         intel: "0000000000000000000000000000000000000000000000000000000000000000"

  url "https://github.com/jsfr/gh-tray-rs/releases/download/v#{version}/gh-tray-#{arch}-apple-darwin.tar.gz"
  name "gh-tray"
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"

  app "gh-tray.app"
  binary "#{appdir}/gh-tray.app/Contents/MacOS/gh-tray"

  caveats <<~EOS
    On first launch, macOS may refuse to open gh-tray because the app is not signed.
    To bypass Gatekeeper, either:

      * Right-click gh-tray.app in Finder → Open → Open
      * Or run: xattr -d com.apple.quarantine "#{appdir}/gh-tray.app"
  EOS

  zap trash: [
    "~/Library/LaunchAgents/io.github.jsfr.gh-tray.plist",
    "~/Library/Preferences/io.github.jsfr.gh-tray.plist",
  ]
end
```

Note: the two `sha256` lines must align the opening quote — `arm:   "` has three spaces after the colon, `intel: "` has one space, so both `"` chars sit in the same column. The updater regex relies on preserving this.

**Step 2: Delete the Formula**

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
rm Formula/gh-tray.rb
rmdir Formula
```

**Step 3: Commit**

```bash
jj commit -m "feat: replace homebrew formula with cask for .app distribution"
```

---

## Task 6: TDD — `scripts/update_cask.py`, delete `update_formula.py` + old test

A Python3 stdlib-only script that rewrites the Cask file in place: updates `version "X"` and the two-line `sha256 arm:/intel:` stanza.

### Step 1: Write the failing test

Create `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/scripts/tests/test_update_cask.py`:

```python
"""Test for update_cask.py. Run: python3 scripts/tests/test_update_cask.py"""
from pathlib import Path
import subprocess
import sys
import tempfile

INPUT_CASK = '''cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.2"
  sha256 arm:   "0000000000000000000000000000000000000000000000000000000000000000",
         intel: "0000000000000000000000000000000000000000000000000000000000000000"

  url "https://github.com/jsfr/gh-tray-rs/releases/download/v#{version}/gh-tray-#{arch}-apple-darwin.tar.gz"
  name "gh-tray"
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"

  app "gh-tray.app"
end
'''

EXPECTED_CASK = '''cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "1.2.3"
  sha256 arm:   "aaaa111111111111111111111111111111111111111111111111111111111111",
         intel: "bbbb222222222222222222222222222222222222222222222222222222222222"

  url "https://github.com/jsfr/gh-tray-rs/releases/download/v#{version}/gh-tray-#{arch}-apple-darwin.tar.gz"
  name "gh-tray"
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"

  app "gh-tray.app"
end
'''

def main():
    repo = Path(__file__).resolve().parents[2]
    script = repo / "scripts" / "update_cask.py"
    with tempfile.NamedTemporaryFile("w", suffix=".rb", delete=False) as f:
        f.write(INPUT_CASK)
        cask_path = f.name
    try:
        subprocess.run(
            [sys.executable, str(script),
             "1.2.3",
             "aaaa111111111111111111111111111111111111111111111111111111111111",
             "bbbb222222222222222222222222222222222222222222222222222222222222",
             cask_path],
            check=True,
        )
        got = Path(cask_path).read_text()
        if got != EXPECTED_CASK:
            print("MISMATCH", file=sys.stderr)
            print("--- expected ---"); print(EXPECTED_CASK)
            print("--- got ---"); print(got)
            sys.exit(1)
        print("OK")
    finally:
        Path(cask_path).unlink(missing_ok=True)

if __name__ == "__main__":
    main()
```

### Step 2: Verify test fails

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
python3 scripts/tests/test_update_cask.py
```
Expected: non-zero exit ("No such file" on the not-yet-existing script).

### Step 3: Write the implementation

Create `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/scripts/update_cask.py`:

```python
#!/usr/bin/env python3
"""Rewrite version + sha256 arm:/intel: in a Homebrew Cask file, in place.

Usage: update_cask.py <version> <sha_arm> <sha_intel> <cask_path>
"""
import re
import sys
from pathlib import Path


def rewrite(text: str, version: str, sha_arm: str, sha_intel: str) -> str:
    out = []
    after_arm = False
    version_done = False
    for line in text.splitlines(keepends=True):
        if not version_done and re.match(r'^\s*version\s+"[^"]*"\s*$', line):
            out.append(re.sub(r'"[^"]*"', f'"{version}"', line, count=1))
            version_done = True
            continue
        m = re.match(r'^(\s*sha256\s+arm:\s+)"[^"]*"(,\s*)$', line)
        if m:
            out.append(f'{m.group(1)}"{sha_arm}"{m.group(2)}')
            after_arm = True
            continue
        if after_arm:
            m2 = re.match(r'^(\s*intel:\s+)"[^"]*"(\s*)$', line)
            if m2:
                out.append(f'{m2.group(1)}"{sha_intel}"{m2.group(2)}')
                after_arm = False
                continue
            after_arm = False
        out.append(line)
    return "".join(out)


def main(argv: list[str]) -> int:
    if len(argv) != 5:
        print("usage: update_cask.py <version> <sha_arm> <sha_intel> <cask_path>", file=sys.stderr)
        return 2
    _, version, sha_arm, sha_intel, path = argv
    p = Path(path)
    p.write_text(rewrite(p.read_text(), version, sha_arm, sha_intel))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
```

### Step 4: Verify test passes

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
python3 scripts/tests/test_update_cask.py
```
Expected: `OK`

### Step 5: Delete the old formula updater + its test

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
rm scripts/update_formula.py scripts/tests/test_update_formula.py
```

### Step 6: Commit

```bash
jj commit -m "feat: add cask updater script, remove formula updater"
```

---

## Task 7: Update `scripts/update-packaging.sh` + integration test

### Step 1: Update the shell script

Modify `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/scripts/update-packaging.sh`. Replace the final `python3` invocation line. Current content of that line:

```bash
python3 scripts/update_formula.py \
  "$VERSION" "$SHA_MAC_ARM" "$SHA_MAC_INTEL" "$BASE_URL" Formula/gh-tray.rb
```

Replace with:

```bash
python3 scripts/update_cask.py \
  "$VERSION" "$SHA_MAC_ARM" "$SHA_MAC_INTEL" Casks/gh-tray.rb
```

(The Cask URL is a template referencing `#{version}`, so the updater drops the `$BASE_URL` arg.)

### Step 2: Update the integration test

Modify `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/scripts/tests/test_update_packaging.sh`:

- Replace every occurrence of `Formula/gh-tray.rb` with `Casks/gh-tray.rb`.
- Replace every occurrence of `scripts/update_formula.py` with `scripts/update_cask.py`.
- Replace the five `grep -q` formula assertions with these Cask-shape assertions:

```bash
# Cask assertions
grep -q 'version "1.2.3"' "$TMP/Casks/gh-tray.rb"
grep -q 'sha256 arm:   "aaaa111111111111111111111111111111111111111111111111111111111111",' "$TMP/Casks/gh-tray.rb"
grep -q '         intel: "bbbb222222222222222222222222222222222222222222222222222222222222"' "$TMP/Casks/gh-tray.rb"
grep -q '/v#{version}/gh-tray-#{arch}-apple-darwin.tar.gz' "$TMP/Casks/gh-tray.rb"
```

(The URL line does NOT get rewritten — it's a template. So the assertion just confirms the literal template survived.)

### Step 3: Verify test passes

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
bash scripts/tests/test_update_packaging.sh
```
Expected: `OK`

### Step 4: Commit

```bash
jj commit -m "refactor: point packaging updater at cask"
```

---

## Task 8: Update `bucket/gh-tray.json` — add `shortcuts`

**Files:**
- Modify: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/bucket/gh-tray.json`

### Step 1: Insert the `shortcuts` field

After the existing `"architecture"` block (at top level, sibling of `architecture`), add:

```json
  "shortcuts": [
    ["gh-tray.exe", "gh-tray"]
  ],
```

The final structure:

```json
{
  "version": "...",
  "description": "...",
  "homepage": "...",
  "license": "...",
  "architecture": { "64bit": { ... } },
  "shortcuts": [["gh-tray.exe", "gh-tray"]],
  "checkver": { ... },
  "autoupdate": { ... }
}
```

### Step 2: Validate

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
jq -e '.shortcuts == [["gh-tray.exe", "gh-tray"]]' bucket/gh-tray.json && echo OK
```
Expected: `true` on stdout, then `OK`.

### Step 3: Commit

```bash
jj commit -m "feat: add start menu shortcut to scoop manifest"
```

---

## Task 9: Update `.github/workflows/release.yml` — macOS uses cargo-bundle

**Files:**
- Modify: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/.github/workflows/release.yml`

### Step 1: Replace the `build:` job

Replace the entire `build:` job block with:

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
            archive: gh-tray-aarch64-apple-darwin.tar.gz
          - os: macos-latest
            target: x86_64-apple-darwin
            archive: gh-tray-x86_64-apple-darwin.tar.gz
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cargo-bundle (macOS)
        if: runner.os == 'macOS'
        run: cargo install cargo-bundle --locked --version 0.10.0

      - name: Build (Windows)
        if: runner.os == 'Windows'
        run: cargo build --release --target ${{ matrix.target }}

      - name: Build + Bundle (macOS)
        if: runner.os == 'macOS'
        run: cargo bundle --release --target ${{ matrix.target }}

      - name: Package (macOS)
        if: runner.os == 'macOS'
        run: |
          tar -czf "${{ matrix.archive }}" -C "target/${{ matrix.target }}/release/bundle/osx" "gh-tray.app"
          shasum -a 256 "${{ matrix.archive }}" | awk '{print $1}' | tr -d '\n' > "${{ matrix.archive }}.sha256"

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

Differences from the current job:
- macOS matrix entries no longer have `binary:` (the `.app` wrapping moots it).
- New step: `Install cargo-bundle (macOS)`.
- Split build step: `Build (Windows)` via `cargo build`, `Build + Bundle (macOS)` via `cargo bundle`.
- macOS `Package` tars from `target/<triple>/release/bundle/osx` with `gh-tray.app`.

### Step 2: Validate YAML

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
python3 -c "import yaml; d=yaml.safe_load(open('.github/workflows/release.yml')); assert set(d['jobs'].keys()) == {'build','release','update-packaging'}; print('OK')"
```
Expected: `OK`

### Step 3: Commit

```bash
jj commit -m "ci: bundle macOS .app via cargo-bundle in release workflow"
```

---

## Task 10: Update `.github/workflows/release.yml` — `update-packaging` commits Cask

### Step 1: Update the `Commit and push` step in `update-packaging`

In the `update-packaging` job (bottom of release.yml), replace the `git add` line:

```diff
-          git add bucket/gh-tray.json Formula/gh-tray.rb
+          git add bucket/gh-tray.json Casks/gh-tray.rb
```

(Only that one line changes in this job.)

### Step 2: Validate YAML

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" && echo OK
```
Expected: `OK`

### Step 3: Commit

```bash
jj commit -m "ci: commit cask path in update-packaging job"
```

---

## Task 11: Update README

**Files:**
- Modify: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/README.md`

### Step 1: Replace the `### macOS (Homebrew)` block

Replace the existing macOS Homebrew block (current content: `brew tap ... && brew install jsfr/gh-tray-rs/gh-tray`) with:

````markdown
### macOS (Homebrew)

```sh
brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install --cask jsfr/gh-tray-rs/gh-tray
```

#### First launch on macOS

The `.app` is not code-signed. macOS will block it on first launch with a "cannot verify developer" warning. To bypass:

- Right-click `gh-tray.app` in Finder → Open → Open, **or**
- Run: `xattr -d com.apple.quarantine /Applications/gh-tray.app`

#### Migrating from a prior Formula install

If you installed gh-tray before v0.0.4:

```sh
brew uninstall gh-tray
brew untap  jsfr/gh-tray-rs
brew tap    jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install --cask jsfr/gh-tray-rs/gh-tray
```
````

Leave the Windows Scoop block unchanged.

### Step 2: Verify the file still renders (fences balanced)

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
grep -c '^```' README.md
```
Expected: an even number (each fence has an opener and closer).

### Step 3: Commit

```bash
jj commit -m "docs: update install instructions for cask migration"
```

---

## Task 12: Bump version to 0.0.4

**Files:**
- Modify: `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/Cargo.toml`

### Step 1: Edit

Change `version = "0.0.3"` → `version = "0.0.4"` in `[package]`.

### Step 2: Refresh lockfile

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
cargo check --quiet
```
Expected: clean.

### Step 3: Also bump the Cask's `version` line

Edit `/Users/jens/Repos/github.com/jsfr/gh-tray-rs/Casks/gh-tray.rb`: change `version "0.0.3"` to `version "0.0.4"`. (Hash placeholders stay as zeros — the release workflow rewrites them.)

Also update `bucket/gh-tray.json`: change `"version": "0.0.3"` to `"version": "0.0.4"` and update both URLs that reference `v0.0.3` to `v0.0.4`.

### Step 4: Commit

```bash
jj commit -m "chore: bump version to 0.0.4"
```

---

## Task 13: Advance main and push

### Step 1: Move main

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
jj bookmark move main --to @-
```

### Step 2: Push

```bash
jj git push
```
Expected: `Move forward bookmark main from …`.

---

## Task 14: Tag `v0.0.4` and push

This triggers the release workflow end-to-end.

### Step 1: Get the tip commit SHA on main

```bash
cd /Users/jens/Repos/github.com/jsfr/gh-tray-rs
git rev-parse main
```
Note the SHA printed.

### Step 2: Create and push the tag

```bash
git tag v0.0.4 <SHA-from-step-1>
git push origin v0.0.4
```

### Step 3: Watch the workflow

```bash
gh run list --repo jsfr/gh-tray-rs --workflow release.yml --limit 1
gh run watch <run-id> --repo jsfr/gh-tray-rs --exit-status
```
Expected: all three jobs (`build`, `release`, `update-packaging`) green.

### Step 4: Verify the release

```bash
gh release view v0.0.4 --repo jsfr/gh-tray-rs --json assets --jq '.assets[].name'
```
Expected: 6 lines — three archives (`.tar.gz` x2, `.zip` x1) + three sidecars.

### Step 5: Verify main auto-updated

```bash
jj git fetch
jj log --no-pager -n 3 -r 'main::@-' -T 'description.first_line() ++ "\n"'
```
Expected: the top commit is `chore: update scoop manifest and homebrew formula for v0.0.4`.

### Step 6: Spot-check the auto-rewritten Cask

```bash
curl -sL https://raw.githubusercontent.com/jsfr/gh-tray-rs/main/Casks/gh-tray.rb | head -10
```
Expected: `version "0.0.4"`, real 64-char hex SHAs (not zeros), URL template unchanged.

---

## Task 15: Install the Cask locally and verify the `.app`

### Step 1: Fresh tap + Cask install

```bash
brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs  # if not tapped
brew install --cask jsfr/gh-tray-rs/gh-tray
```

### Step 2: Confirm `.app` landed in /Applications

```bash
ls -d /Applications/gh-tray.app
defaults read /Applications/gh-tray.app/Contents/Info LSUIElement
defaults read /Applications/gh-tray.app/Contents/Info CFBundleIdentifier
```
Expected: directory exists, `LSUIElement = 1`, `CFBundleIdentifier = io.github.jsfr.gh-tray`.

### Step 3: Confirm CLI symlink works

```bash
which gh-tray
gh-tray --help | head -3
```
Expected: `/opt/homebrew/bin/gh-tray`, help output.

### Step 4: Launch the `.app`

```bash
xattr -d com.apple.quarantine /Applications/gh-tray.app 2>/dev/null || true
open /Applications/gh-tray.app
```
Expected: menu-bar icon appears (no Dock icon). If auth isn't set up yet (no `gh auth login`), the app will silently exit — check tray briefly, or run `gh-tray --demo` from terminal to smoke-test the visual path.

No commit needed for this task — verification only.

---

## Validation checklist

- [ ] Workflow run for v0.0.4 completed with all 3 jobs green.
- [ ] Release page shows 6 assets.
- [ ] `main` has the auto-update commit for v0.0.4 with real SHAs in `Casks/gh-tray.rb`.
- [ ] `/Applications/gh-tray.app/Contents/Info.plist` contains `LSUIElement=true` and `io.github.jsfr.gh-tray`.
- [ ] `gh-tray` command still on PATH after Cask install.
- [ ] `.app` opens to a menu-bar icon (no Dock icon).
- [ ] Scoop manifest on `main` has `"shortcuts": [["gh-tray.exe", "gh-tray"]]`.

---

## Notes

- Icon artwork is intentionally minimal (`#24292e` square with "gh" text). Replace later by regenerating with `magick` and re-committing — no pipeline changes needed.
- `auto_launch` crate still registers `current_exe()`, which resolves to the binary inside the bundle at runtime. LaunchAgent plist will point at `/Applications/gh-tray.app/Contents/MacOS/gh-tray`, which launches the full bundle via the runtime's own path. If users report auto-start misbehaving under the bundle, revisit.
- cargo-bundle runs `cargo build` internally; the macOS job no longer needs a separate `cargo build --release` step.
- `winres` is pulled in only for Windows builds via the `[target.'cfg(windows)'.build-dependencies]` gate — it doesn't affect macOS compile time.
- The `.app` archive is a directory tree inside a `.tar.gz`. Homebrew Cask handles this natively (`app "gh-tray.app"` stanza). Users downloading the archive directly can `tar -xzf …` and drag the resulting `.app` to `/Applications`.
