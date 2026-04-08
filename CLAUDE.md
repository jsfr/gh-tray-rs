# CLAUDE.md

## Version Control

This repository uses **Jujutsu (jj)**, not git. Always use `jj` commands.

## Build Commands

All commands use `just`:

- `just build` — build (debug)
- `just build-release` — build (release)
- `just run` — run the app
- `just demo` — run in demo mode
- `just fmt` — format with rustfmt
- `just check` — check formatting + clippy
- `just test` — run tests

## Architecture

Cross-platform system tray app monitoring GitHub PRs. Uses native OS menus.

### Source Files

- `src/main.rs` — entry point, event loop, polling thread
- `src/types.rs` — domain types
- `src/config.rs` — config loading
- `src/github.rs` — gh CLI wrapper + GraphQL
- `src/demo.rs` — demo mode
- `src/tray.rs` — tray icon + menu building
- `src/theme.rs` — dark/light mode detection
- `src/logging.rs` — tracing setup

## Commit Style

Uses Conventional Commits (`feat:`, `fix:`, `refactor:`, `ci:`, `chore:`, `docs:`).
