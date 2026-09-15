# CLAUDE.md

## Version Control

This repository uses **Jujutsu (jj)**, not git. Always use `jj` commands.

The one exception is `just changelog`, which calls `git cliff` to read history.
It does not write to the repository state.

## Build Commands

All commands use `just`:

- `just build` — build (debug)
- `just build-release` — build (release)
- `just run` — run the app
- `just demo` — run in demo mode
- `just fmt` — format with rustfmt
- `just check` — check formatting + clippy
- `just test` — run tests (Rust only)
- `just changelog` — regenerate `CHANGELOG.md` with git-cliff

## Architecture

macOS menu bar app monitoring GitHub PRs. Uses native macOS menus.

macOS is the only supported platform. The code has no platform `cfg` branches:
do not add Windows or Linux paths back.

### Source Files

- `src/main.rs` — entry point, event loop, polling thread
- `src/types.rs` — domain types
- `src/config.rs` — config loading
- `src/github.rs` — gh CLI wrapper + GraphQL
- `src/demo.rs` — demo mode
- `src/tray.rs` — tray icon + menu building
- `src/theme.rs` — dark/light mode detection
- `src/clipboard.rs` — clipboard writes
- `src/modifiers.rs` — modifier key state
- `src/logging.rs` — tracing setup

## Packaging

The repository is also its own Homebrew tap. Users add it with
`brew tap jsfr/gh-tray-rs`. A change to the cask reaches users on the next
`brew update`.

- `Casks/gh-tray.rb` — Homebrew cask
- `scripts/update_cask.py` — rewrites version + sha256 in the cask for a
  release. Arguments: `<version> <sha_mac_arm> <sha_mac_intel> <cask_path>`.
- `scripts/tests/` — tests for that script. Run the files directly; `just test`
  does not include them.
- `cliff.toml` — git-cliff config

The release script rewrites only `version` and the `sha256` lines, so other
stanzas are safe to edit by hand.

To check the cask, copy it into the tap clone at
`$(brew --repository)/Library/Taps/jsfr/homebrew-gh-tray-rs` and run
`brew style --cask jsfr/gh-tray-rs/gh-tray`. Homebrew rejects casks outside a
tap, so it cannot lint the file in place.

## Commit Style

Uses Conventional Commits (`feat:`, `fix:`, `refactor:`, `ci:`, `chore:`, `docs:`).
