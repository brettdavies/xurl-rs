# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Close the published surface, expose the rate-limit window, and add an embedder consumer crate
- Select the TLS backend by feature and compile every configuration in CI
- Document the crate from a doctested README and tighten the lint posture
- Add runnable examples and a testing feature with an in-process mock of the API
- Give the library its own release path and make every channel workspace-aware

### Breaking changes

- Split the crate into the xdk-rs library and the xurl-rs CLI
