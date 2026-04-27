# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.7] - 2026-04-27

### CI

- codesign bundled .app with hardened runtime

## [0.0.6] - 2026-04-27

### Fixed

- skip re-exec when launched from .app bundle to preserve NSApplication context

## [0.0.5] - 2026-04-27

### Fixed

- strip quarantine attribute on cask install for macOS Tahoe
- re-exec via /usr/bin/env to avoid Tahoe ImageIO SIGBUS from adhoc-signed parents

## [0.0.4] - 2026-04-21

### Added

- add app icon assets and macOS Info.plist fragment
- configure cargo-bundle and add winres build-dep
- embed icon in Windows exe via winres
- replace homebrew formula with cask for .app distribution
- add cask updater script, remove formula updater
- add start menu shortcut to scoop manifest

### CI

- bundle macOS .app via cargo-bundle in release workflow
- commit cask path in update-packaging job

### Changed

- point packaging updater at cask

### Fixed

- regenerate icon as 8-bit and pre-build .icns for cargo-bundle

## [0.0.3] - 2026-04-20

### Added

- add scoop bucket manifest
- add homebrew formula
- add homebrew formula updater script
- add packaging updater script

### CI

- archive release binaries with sha256 sidecar
- auto-update scoop and homebrew manifests on release

## [0.0.2] - 2026-04-10

### Fixed

- hide from Dock on macOS and create tray icon after event loop init
- improve tray icon text centering and size using font metrics

## [0.0.1] - 2026-04-09

### Added

- initial project scaffold with dependencies
- add domain types (PullRequest, PullRequestGroup, CheckStatus, ReviewStatus)
- add configuration loading with JSON + env var overrides
- add GitHub client with GraphQL query and JSON parsing
- add demo mode with fake PR data
- add cross-platform dark/light theme detection
- add tray icon rendering and native menu building
- add logging setup with optional file output
- wire up main event loop with tray icon, polling, hotkey, and auto-launch

### CI

- add GitHub Actions workflows and project docs
- drop Ubuntu from CI matrix (app targets macOS/Windows only)
- add git-cliff changelog generation to release workflow

### Fixed

- unsafe extern block for Rust 2024 edition, CI fail-fast disabled


