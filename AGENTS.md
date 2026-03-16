# xurl-rs

A fast, ergonomic CLI for the X (Twitter) API. Rust port of [xurl](https://github.com/xdevplatform/xurl).

## Binary & Library

- Binary: `xr` (installed via `cargo install xurl-rs`)
- Library: `xurl` (import as `use xurl::...`)
- Package: `xurl-rs` (crates.io)

## Quality Bar

- Clippy clean, edition 2024 (`cargo clippy -- -D warnings`)
- Formatted with rustfmt (`cargo fmt --check`)
- No unwrap() in production code
- Comprehensive tests (`cargo test` — unit + integration + differential conformance)
- Zero broken tests policy

## Architecture

- `src/api/` — HTTP client, endpoints, shortcuts (28 commands), media upload
- `src/auth/` — OAuth1 (HMAC-SHA1 per RFC 5849), OAuth2 (PKCE), Bearer token
- `src/cli/` — clap-based CLI with commands/mod.rs handler layer
- `src/config/` — Environment variable based configuration
- `src/store/` — YAML token store (~/.xurl), multi-app support
- `src/output.rs` — OutputConfig for text/json/jsonl formatting
- `src/error.rs` — XurlError with thiserror

## Known Differences from Go Original

See [KNOWN_DIFFERENCES.md](KNOWN_DIFFERENCES.md) for intentional deviations.
