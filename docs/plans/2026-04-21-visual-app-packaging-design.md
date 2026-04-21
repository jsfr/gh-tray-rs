# Visual App Packaging Design — macOS `.app` Cask + Windows Start Menu

**Date:** 2026-04-21
**Status:** Approved, ready for implementation planning

## Goal

Let users launch `gh-tray` as a proper visual app on both platforms:

- **macOS:** `gh-tray.app` in `/Applications`, double-clickable from Finder and Spotlight, with the expected menu-bar-only behaviour (no Dock icon). Installed via a Homebrew Cask.
- **Windows:** `gh-tray.exe` with an embedded icon, reachable from the Start menu via a Scoop-managed shortcut.

## Decisions

- **macOS:** Cask replaces the current Formula. CLI access preserved via Cask's `binary` stanza.
- **`.app` bundling:** `cargo-bundle` (actively maintained, last release 2026-04-18). Metadata in `Cargo.toml`'s `[package.metadata.bundle]` section.
- **Bundle identifier:** `io.github.jsfr.gh-tray` (github.io convention).
- **Menu-bar-only:** `LSUIElement=true` injected via `osx_info_plist_exts` (prevents Dock-icon flash during startup). Existing runtime `ActivationPolicy::Accessory` stays as a fallback for `cargo run` dev usage.
- **Windows:** `winres` crate in `build.rs` embeds `assets/icon.ico` into `gh-tray.exe` at compile time. Scoop manifest gains a `shortcuts` entry for the Start menu.
- **No code signing / notarization.** Users get Gatekeeper warnings on first launch; bypass instructions documented in the Cask's `caveats` block and the README.
- **Icon source of truth:** single `assets/icon-1024.png`. `cargo-bundle` auto-generates `.icns` from it. `assets/icon.ico` is pre-generated (ImageMagick) and committed for `winres` to pick up.

## File Layout

```
gh-tray-rs/
├── assets/
│   ├── Inter-Bold.ttf              existing
│   ├── icon-1024.png               new — source icon
│   ├── icon.ico                    new — Windows resource
│   └── macos/
│       └── Info.plist.frag         new — LSUIElement + anything else we inject
├── build.rs                        new — Windows .ico embed via winres
├── Cargo.toml                      updated — [package.metadata.bundle], winres build-dep
├── Casks/
│   └── gh-tray.rb                  new — replaces Formula/gh-tray.rb
├── Formula/                        DELETED
├── bucket/gh-tray.json             updated — adds "shortcuts"
└── scripts/
    ├── update_cask.py              new — replaces update_formula.py
    ├── update_formula.py           DELETED
    ├── update-packaging.sh         updated — invokes update_cask.py
    └── tests/
        ├── test_update_cask.py     new — replaces test_update_formula.py
        ├── test_update_formula.py  DELETED
        └── test_update_packaging.sh  updated fixtures
```

## `Cargo.toml` additions

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

## `assets/macos/Info.plist.frag`

```xml
<key>LSUIElement</key>
<true/>
```

Appended inside the generated Info.plist's `<dict>` by `cargo-bundle`.

## `build.rs`

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

## `Casks/gh-tray.rb`

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

Placeholders rewritten on each release by `scripts/update_cask.py`.

Install flow:
```
brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install --cask jsfr/gh-tray-rs/gh-tray
```

## Scoop Manifest — `bucket/gh-tray.json`

Add top-level:

```json
"shortcuts": [
  ["gh-tray.exe", "gh-tray"]
]
```

The `architecture`, `checkver`, `autoupdate` blocks stay as-is. `shortcuts` is static — the updater script does not rewrite it.

Install flow unchanged:
```
scoop bucket add gh-tray-rs https://github.com/jsfr/gh-tray-rs
scoop install gh-tray-rs/gh-tray
```

## Updater Script — `scripts/update_cask.py`

Stateful line pass over the Cask file:

- Rewrite the single `  version "X"` line.
- Rewrite the `sha256 arm:   "…",` line and the paired `         intel: "…"` line.
- URL is a template referencing `#{version}` — no rewrite needed.

Tests at `scripts/tests/test_update_cask.py` follow the same pattern as the existing `test_update_formula.py`.

`scripts/update-packaging.sh` swaps:
```diff
- python3 scripts/update_formula.py "$VERSION" "$SHA_MAC_ARM" "$SHA_MAC_INTEL" "$BASE_URL" Formula/gh-tray.rb
+ python3 scripts/update_cask.py    "$VERSION" "$SHA_MAC_ARM" "$SHA_MAC_INTEL" Casks/gh-tray.rb
```

(Dropping `$BASE_URL` from the args — the Cask URL is a template so the base is hard-coded in the Cask file itself.)

## Release Workflow — `.github/workflows/release.yml`

macOS matrix jobs:

- Replace `cargo build --release --target <triple>` with `cargo install cargo-bundle --locked --version 0.10` (cacheable) then `cargo bundle --release --target <triple>`.
- Replace the archive step's `tar` source directory:
  ```diff
  - tar -czf "${{ matrix.archive }}" -C "target/${{ matrix.target }}/release" "${{ matrix.binary }}"
  + tar -czf "${{ matrix.archive }}" -C "target/${{ matrix.target }}/release/bundle/osx" "gh-tray.app"
  ```

Windows matrix job: unchanged. `cargo build --release` runs `build.rs`, which invokes `winres` and embeds the icon. The existing `Compress-Archive` step still works.

`update-packaging` job: `git add bucket/gh-tray.json Casks/gh-tray.rb` (replacing the Formula path).

## README Changes

Rewrite the macOS install block:

```sh
brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install --cask jsfr/gh-tray-rs/gh-tray
```

Add a brief "Migrating from Formula install" note:

```
If you previously installed gh-tray via the Formula:

    brew uninstall gh-tray
    brew untap  jsfr/gh-tray-rs
    brew tap    jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
    brew install --cask jsfr/gh-tray-rs/gh-tray
```

And a "Gatekeeper on first launch" subsection mirroring the Cask caveats.

## Release Cadence

Bump version to `0.0.4` after the work lands, create and push tag `v0.0.4`. The workflow exercises the new bundling path end-to-end: builds `.app`, tarballs it, publishes release assets, and auto-rewrites Cask + Scoop manifest on `main`.

## Out of Scope

- Code signing + notarization (deferred; needs paid Apple Developer Program).
- Universal macOS binary (keep two per-arch archives).
- Chocolatey / WinGet / MSI installers.
- Auto-updater framework integration (e.g., Sparkle).
- Updating `auto_launch` crate usage to register the `.app` path instead of the raw binary — the existing `current_exe()` call already resolves to the binary inside the bundle at runtime, which the LaunchAgent plist will invoke directly. Works as-is; revisit only if users report the auto-start toggle misbehaving under the bundle.
- Polished icon artwork — the committed placeholder is replaceable later without pipeline changes.
