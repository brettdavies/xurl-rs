# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Add changelog check and pull-requests permission

### Changed

- Migrate to centralized reusable workflows
- Convert guard-main-docs to reusable workflow caller

### Documentation

- Update RELEASING.md for reusable workflow pipeline
- Simplify shell completions section in RELEASING.md

### Fixed

- Align ci.yml triggers with skill template
- Align guard-main-docs and protect-dev with bird

## [1.0.4] - 2026-03-16

### Fixed

- Post-release CI hardening — Trusted Publishing, SHA pinning, rustls-tls (#4)

## [1.0.3] - 2026-03-16

### Added

- Xurl-rs v1.0.3 — Rust port of xurl (#2)

### Documentation

- Initial README for xurl-rs

### Fixed

- Switch to rustls-tls and fix macOS CI runner (#3)

