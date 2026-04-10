# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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


