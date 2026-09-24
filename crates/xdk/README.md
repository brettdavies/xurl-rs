# xdk-rs

[![Crates.io](https://img.shields.io/crates/v/xdk-rs.svg)](https://crates.io/crates/xdk-rs)
[![docs.rs](https://img.shields.io/docsrs/xdk-rs)](https://docs.rs/xdk-rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](#license)

Async Rust client for the X (Twitter) API v2: credentials held in code, one typed call per endpoint, OAuth1, OAuth2
PKCE, bearer tokens, media upload, and streaming. `xdk` is the name X uses for its own SDKs; this is an independent
project, not affiliated with, endorsed by, or maintained by X. The `xr` command-line tool
([`xurl-rs`](https://crates.io/crates/xurl-rs)) is built on this crate.

## Quick start

The package is `xdk-rs`; the library it installs is `xdk`:

```bash
cargo add xdk-rs
cargo add tokio --features macros,rt-multi-thread
```

Those two commands leave this in `Cargo.toml`. `#[tokio::main]` needs both tokio features:

```toml
[dependencies]
xdk-rs = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

A complete program: the app-only bearer token from the [X developer portal](https://developer.x.com/en/portal/dashboard)
in `XURL_BEARER_TOKEN`, one search, the text printed.

```rust,no_run
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

A bearer token is app-only: it reads public data (search, post and user lookups) and cannot act as a user, so `get_me`,
`create_post`, likes, follows, and direct messages answer 401 or 403 under it. Anything user-scoped needs a client built
from an OAuth2 credential, described under [Authentication](#authentication).

Every request draws credits at the rates on X's [pricing page](https://docs.x.com/x-api/getting-started/pricing), and
the app must be enrolled as described in the repository's
[Before you start](https://github.com/brettdavies/xurl-rs#before-you-start-an-x-app) section; without that step, reads
fail with `client-not-enrolled` even when the credential is valid.

## Authentication

The client holds credentials directly and picks the scheme per call from the endpoint's accepted set, in OAuth2, OAuth1,
bearer order. Four paths:

- **Bearer (app-only)**: `Client::builder().bearer(token)`. Public reads and search.
- **OAuth2 user context**: an `OAuth2Credential` carrying the app's client ID and secret, the access token, and the
  optional refresh token and expiry. The client refreshes an expired token on the next call and hands the rotated pair
  to the `OnTokenRefreshed` hook registered with `on_token_refreshed`. X rotates the refresh token on every refresh, so
  the hook is where the new pair gets persisted; a hook that returns an error fails the call that triggered the refresh.
- **OAuth1 HMAC-SHA1**: an `OAuth1Credential` with the consumer pair and the user's access pair, for legacy v1.1
  endpoints and some v2 write paths.
- **The token store**: `Client::new(&Config, Auth)` or `Client::from_env()` builds a client over the `~/.xurl` store the
  `xr` CLI writes, so a program can reuse a sign-in done with `xr auth oauth2` with nothing exported; `from_env` also
  reads `CLIENT_ID`, `CLIENT_SECRET`, `REDIRECT_URI`, `AUTH_URL`, `TOKEN_URL`, `API_BASE_URL`, `INFO_URL`, and
  `XURL_BEARER_TOKEN` when they are set. `TokenStore::refresh_hook_for` is the reference implementation of the refresh
  hook.

```rust,no_run
use std::time::{Duration, SystemTime};

use xdk::api::Client;
use xdk::auth::OAuth2Credential;

#[tokio::main]
async fn main() -> xdk::Result<()> {
    let var = |name: &str| std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set"));
    let credential = OAuth2Credential {
        client_id: var("CLIENT_ID"),
        client_secret: var("CLIENT_SECRET"),
        access_token: var("ACCESS_TOKEN"),
        refresh_token: std::env::var("REFRESH_TOKEN").ok(),
        expires_at: Some(SystemTime::now() + Duration::from_secs(7200)),
    };
    let client = Client::builder().oauth2(credential).build()?;

    let me = client.get_me().send().await?;
    println!("signed in as @{}", me.data.username);
    Ok(())
}
```

## Errors and rate limits

Every call returns `xdk::Result<T>`, whose error is the `#[non_exhaustive]` `xdk::Error`. Beyond `Display`, an error
answers four questions: `kind()` names its category as a stable string, `exit_code()` maps it to the process exit code
the `xr` CLI uses, `next_action()` names the one thing a caller can do about it as a closed `NextAction` (sign in,
register an app, enroll the app, and so on), and `docs_url()` points at the page that explains it when one exists. A 429
carries no next action; `Client::last_rate_limit()` returns the `x-rate-limit-*` window from the most recent response
that reported one, which is what a retry loop waits on.

```rust,no_run
use xdk::api::Client;

#[tokio::main]
async fn main() {
    let client = Client::builder().bearer("app-only-token").build().expect("client");
    match client.search_posts("rust", 10).send().await {
        Ok(posts) => println!("{} posts", posts.data.len()),
        Err(err) => {
            eprintln!("{}: {err}", err.kind());
            if let Some(action) = err.next_action() {
                eprintln!("next: {action:?}");
            }
            if let Some(url) = err.docs_url() {
                eprintln!("see {url}");
            }
            std::process::exit(err.exit_code());
        }
    }
}
```

## Testing

Two ways to exercise code built on this crate without spending credits.

**The `testing` feature** ships `xdk::testing::MockX`, an in-process mock of the API. `MockX::start().await` binds a
local server seeded with the fixture responses the crate's own response types are validated against, so a read against
it deserializes into the same types a live call would, and every response carries a rate-limit window. `app_client()`
and `user_client()` return clients pointed at it, `stub` overrides a route or rehearses a failure, and `requests()`
lists what arrived. Enable it in a test profile only, so a release build pulls none of its dependencies:

```toml
[dev-dependencies]
xdk-rs = { version = "0.1", features = ["testing"] }
```

The module's docs.rs page carries a complete program, and the `offline_search` example runs one end to end with no X
app, no credential, and no network, from a clone of the repository:

```bash
cargo run -p xdk-rs --example offline_search --features testing
```

**X's playground** is X's own local server that simulates the API v2 with seeded users and posts, for proving the wire
protocol end to end rather than for unit tests. Two limits: it needs a Go toolchain, and its parameter vocabulary
predates X's post-vocabulary rename (spec 2.168), so a call that sends the current expansion names comes back as an
invalid-request error.

```bash
go install github.com/xdevplatform/playground/cmd/playground@latest
export PATH="$PATH:$(go env GOPATH)/bin"
playground start --port 8089
```

Point a client at it with `Client::builder().bearer("test_token").base_url("http://localhost:8089")`; a bearer reaches
its app-only endpoints, and user-context calls stop at the auth matrix before a request leaves the machine, the same
refusal they get without credentials against the real API.

## What the crate publishes

Every published module is one an embedder has a reason to call:

- `api`: the client and its builder, one `Call` per shortcut, the `MediaUpload` builder behind `Client::upload_media`,
  the typed responses, the raw-request path for endpoints without a shortcut, and the last rate-limit window a response
  reported. A `tracing` subscriber on `api::VOCABULARY_TARGET` receives one `DEBUG` event for each key a typed response
  carried in X's legacy post vocabulary and the library read under its current name, with the fields `legacy`,
  `normalized`, `value_type`, `value_len`, and `collision`, and no values.
- `auth`: credentials held in code, the refresh hook that receives a rotated token pair, the store-backed `Auth`, and
  the OAuth2 sign-in flows.
- `config`: base URL, timeouts, and the environment overrides a client built from the environment reads.
- `error`: `Error`, `Result`, the machine-readable `NextAction`, and the exit codes a CLI built on this crate maps them
  to.
- `store`: `TokenStore`, the on-disk credential store the CLI shares, and the reference implementation of the refresh
  hook.

Items marked `#[doc(hidden)]` are seams the `xr` binary reaches across the crate boundary; they stay callable but are
not part of this surface.

To see which keys X still sends in its legacy post vocabulary, install any `tracing` subscriber filtered to the target,
here with [`tracing-subscriber`](https://docs.rs/tracing-subscriber):

```rust,no_run
use tracing_subscriber::filter::Targets;
use tracing_subscriber::prelude::*;

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(Targets::new().with_target(xdk::api::VOCABULARY_TARGET, tracing::Level::DEBUG))
    .init();
```

Each legacy key then prints once per response, while the typed value reads under its current name:

```text
DEBUG xdk::vocabulary: legacy="edit_history_tweet_ids" normalized="edit_history_post_ids" value_type="array" value_len=1 collision=false
```

## Cargo features

- `rustls` (default): TLS through [rustls](https://docs.rs/rustls) with the platform's certificate verifier; no system
  TLS library is linked.
- `native-tls`: TLS through the operating system's library (OpenSSL on Linux, Secure Transport on macOS, SChannel on
  Windows) via [native-tls](https://docs.rs/native-tls). To use it alone, turn the default off: `xdk-rs = { version =
  "0.1", default-features = false, features = ["native-tls"] }`. With both backends enabled, reqwest picks `native-tls`.
- `testing`: an in-process mock of the API seeded from the crate's fixtures, for tests that must not spend credits; see
  [Testing](#testing).

A build with neither TLS feature fails at compile time with a message naming both, rather than at the first `https`
request.

## Versioning

The crate is `0.x`, so Cargo reads the middle number as the breaking position: a breaking change moves it (`0.1.x` to
`0.2.0`), and an addition or a fix moves the last number (`0.1.0` to `0.1.1`), which a `^0.1` requirement picks up. Two
rules make the breaking releases livable:

- **Every breaking entry in the [changelog](https://github.com/brettdavies/xurl-rs/blob/main/crates/xdk/CHANGELOG.md)
  carries a before/after snippet**, not only a description, so the developer who adopted at one `0.y` and upgrades two
  later types the new form straight from the changelog. A release that breaks something and ships no snippet does not go
  out.
- **An MSRV bump never ships in the last number.** `rust-version` is declared once, in the workspace's
  `[workspace.package]`, and both crates inherit it, so a bump moves the floor of `xdk-rs` and `xurl-rs` together: the
  middle number for `xdk-rs` and a minor for `xurl-rs`, independent version lines notwithstanding.

The two crates version and tag independently: the library on `xdk-rs-vX.Y.Z`, the CLI on `vX.Y.Z`. Breaking changes the
CLI takes across its majors are written up under
[`docs/migrating`](https://github.com/brettdavies/xurl-rs/tree/main/docs/migrating).

## Relationship to xr and xurl

`xr` ([`xurl-rs`](https://crates.io/crates/xurl-rs)) is the command-line tool built on this crate and an independent
Rust port of [`xdevplatform/xurl`](https://github.com/xdevplatform/xurl), X's own Go CLI. The two crates share the
`~/.xurl` token store, so a sign-in done with `xr auth oauth2` is usable from a program through `Client::from_env()`.
The repository's [README](https://github.com/brettdavies/xurl-rs#readme) routes between the two.

## License

Licensed under either of [Apache License, Version 2.0](https://github.com/brettdavies/xurl-rs/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/brettdavies/xurl-rs/blob/main/LICENSE-MIT) at your option.
