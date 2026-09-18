# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-09-18

### Added

- Close the published surface, make TLS a feature, and add an embedder consumer crate (#175)
- Document the crate from a doctested README, add examples, and a testing mock (#176)
- Give the library its own release path and make every channel workspace-aware (#177)
- Upload media through a builder, keep Error small, and state the line between the crates (#178)
- Add broadcast chat moderator shortcuts and the xr broadcasts commands (#182)
- Walk the registries for the surfaces a new family can miss (#190)
- Answer every declared endpoint from the testing mock (#191)

### Breaking changes

- Split the crate into the xdk-rs library and the xurl-rs CLI (#174)
- Return the send confirmation from send_dm (#194)
- Report API refusals by their status instead of as network errors (#198)

### Changed

- Name every endpoint through the build script's declaration (#185)
