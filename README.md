# xurl-rs

[![xdk-rs on crates.io](https://img.shields.io/crates/v/xdk-rs.svg?label=xdk-rs)](https://crates.io/crates/xdk-rs)
[![xurl-rs on crates.io](https://img.shields.io/crates/v/xurl-rs.svg?label=xurl-rs)](https://crates.io/crates/xurl-rs)
[![CI](https://github.com/brettdavies/xurl-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/brettdavies/xurl-rs/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](#license)

Two crates for the X (Twitter) API v2, in one repository:

- **`xdk-rs`**, an async Rust client library: credentials in code, one typed call per endpoint, OAuth1, OAuth2 PKCE,
  bearer tokens, media upload, streaming.
  [The library](#the-library-xdk-rs) · [crates/xdk/README.md](crates/xdk/README.md) · [docs.rs](https://docs.rs/xdk-rs)
- **`xurl-rs`**, the `xr` command-line tool built on it: a Rust port of X's own `xurl` with shell completions,
  machine-readable output, and structured exit codes for agents.
  [The CLI](#the-cli-xr) · [crates/xurl-cli/README.md](crates/xurl-cli/README.md)

An independent project, not affiliated with, endorsed by, or maintained by X.

## The library: xdk-rs

The package is `xdk-rs`; the library it installs is `xdk`. Add it with the tokio features `#[tokio::main]` needs:

```bash
cargo add xdk-rs
cargo add tokio --features macros,rt-multi-thread
```

A complete program: the app-only bearer token from the
[X developer portal](https://developer.x.com/en/portal/dashboard) in `XURL_BEARER_TOKEN`, one search, the text printed.

```rust
use xdk::api::Client;

#[tokio::main]
async fn main() -> xdk::Result<()> {
    let token = std::env::var("XURL_BEARER_TOKEN").expect("XURL_BEARER_TOKEN holds the app-only bearer token");
    let client = Client::builder().bearer(token).build()?;

    let posts = client.search_posts("rust", 10).send().await?;
    for post in &posts.data {
        println!("{}: {}", post.id, post.text);
    }
    Ok(())
}
```

A bearer token reads public data and cannot act as a user; user-scoped calls need an OAuth2 credential. The library
README covers [authentication](crates/xdk/README.md#authentication), [errors and rate limits](crates/xdk/README.md#errors-and-rate-limits),
the [published modules](crates/xdk/README.md#what-the-crate-publishes), and the
[Cargo features](crates/xdk/README.md#cargo-features) that select the TLS backend. The same program is the crate's
docs.rs landing page, where it is compiled as a doctest.

## The CLI: xr

`xr` calls the X API from a shell or an agent: curl-style raw requests, shortcut commands over the common endpoints,
every auth flow, chunked media upload, streaming, seven output formats, and a typed error envelope with structured exit
codes. It installs with `brew install brettdavies/tap/xurl-rs` or `cargo install xurl-rs`.

- [Install](crates/xurl-cli/README.md#install), [Quick start](crates/xurl-cli/README.md#quick-start), and the
  [command reference](crates/xurl-cli/README.md#commands)
- [Authentication](crates/xurl-cli/README.md#authentication): OAuth2, OAuth1, bearer, and multi-app management
- [Agent-native features](crates/xurl-cli/README.md#agent-native-features): the skill bundle, response schemas,
  machine-readable output, quiet and non-interactive modes, and structured exit codes
- [Shell completions](crates/xurl-cli/README.md#shell-completions)
- [Relationship to xurl](crates/xurl-cli/README.md#relationship-to-xurl), with
  [KNOWN_DIFFERENCES.md](KNOWN_DIFFERENCES.md) naming each deliberate divergence from the Go tool

## Before you start: an X app

Both crates call the X API v2 with an app of your own, so set one up first:

- Create an app in the [X developer portal](https://developer.x.com/en/portal/dashboard) with **Read and Write**
  permission.
- Set it up as a **confidential client**, so X issues a client secret alongside the client ID.
- For OAuth2 sign-in, register the redirect URI the tool listens on; `xr auth oauth2` uses
  `http://localhost:8080/callback`.
- A bearer token alone is enough for public reads and search, from either crate.

The X API is pay-per-use: every request draws credits at the rates on X's
[pricing page](https://docs.x.com/x-api/getting-started/pricing). The crates are free; the calls they make are not.

### X Platform Enrollment

If sign-in succeeds but reads such as `xr whoami` or `get_me` fail with an error body containing `client-forbidden` or
`client-not-enrolled`, the X platform fix is to move the app into the `Pay-per-use` package and use the `Production`
environment in the developer console. This is an X platform enrollment issue, not a problem in the local callback
listener or the credential.

The working recipe in the X developer console:

1. Go to `Apps` -> `Manage apps`.
2. Open the app.
3. Use `Move to package`.
4. Choose `Pay-per-use`.
5. Move the app to the `Production` environment.

Without that enrollment step, `/2/*` reads can fail even when the OAuth callback and tokens are valid.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup, the branch and PR flow, the error contract, and a recipe for
exercising `xr` against X's API Playground without an account. Release procedures live in [RELEASES.md](RELEASES.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
