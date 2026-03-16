# Changelog

All notable changes to this project will be documented in this file.

## [1.0.3] - 2026-03-16

### Added

- Full xurl-rs implementation — Rust port of Go xurl CLI
- Shell autocomplete for bash, zsh, fish, powershell, elvish
- Agentic coding flags — `--output json`, `--quiet`, `--no-interactive`, exit codes
- Wire `--output`/`--quiet`/`--no-interactive` through all handlers

### Fixed

- Align test imports with implementation after red/green team merge
- Address code review findings (Default, JSON escaping, UTF-8 safety, docs)
- Config tests use serial_test to prevent env var race conditions

### Changed

- Remove dead_code allows on exit code constants — all now used
