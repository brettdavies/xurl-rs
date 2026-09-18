# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-09-18

### Breaking changes

- Change the error type to `xdk::Error`, re-exported with `xdk::Result` at the crate root and marked `#[non_exhaustive]`, so a match outside the crate needs a wildcard arm. Display strings are lowercase fragments with no prefix and no trailing period, every variant carries an explicit exit code, and the `EnvelopeAlreadyEmitted` variant is gone because it was the binary's control flow rather than anything an embedder can act on. by @brettdavies in [#171](https://github.com/brettdavies/xurl-rs/pull/171)

  ```rust
  // Before
  use xurl::error::{XurlError, exit_code_for_error};

  match client.get_me(&opts) {
      Ok(me) => println!("{}", me.data.username),
      Err(XurlError::Auth(msg)) => {
          eprintln!("{msg}");
          std::process::exit(exit_code_for_error(&XurlError::Auth(msg)));
      }
      Err(err) => return Err(err),
  }

  // After
  use xdk::Error;

  match client.get_me().send().await {
      Ok(me) => println!("{}", me.data.username),
      Err(err @ Error::Auth(_)) => {
          eprintln!("{}: {err}", err.kind());
          std::process::exit(err.exit_code());
      }
      Err(err) => return Err(err),
  }
  ```
- Change the library to async throughout: the client's methods, token exchange and refresh, media upload, and the OAuth2 callback listener are `async fn`s on the caller's runtime, and constructing a client returns `Result`. Shortcuts no longer take a `&CallOptions` by reference at the call site. by @brettdavies in [#172](https://github.com/brettdavies/xurl-rs/pull/172)

  ```rust
  // Before: blocking, options passed by reference
  let posts = client.search_posts("rust", 10, &opts)?;

  // After: async on the caller's runtime
  let posts = client.search_posts("rust", 10).send().await?;
  ```
- Change `stream_request` to return a `StreamLines`, which implements `futures_core::Stream<Item = Result<String>>` and offers `next_line().await`. The caller drives the loop and decides when to stop, and dropping the stream closes the connection.

  ```rust
  // Before
  for line in client.stream_request(&opts)? {
      println!("{}", line?);
  }

  // After
  let mut lines = client.stream_request(&opts).await?;
  while let Some(line) = lines.next_line().await? {
      println!("{line}");
  }
  ```
- Change the browser sign-in to `Client::oauth2_flow(username, cancel, opener)`, which takes a cancellation token and an opener closure and returns the username it signed in. The library performs no terminal output and launches no browser, and it registers no signal handler, so cancellation is the caller's to own.

  ```rust
  // Before
  let username = auth.oauth2_flow(&out, &mut stderr, open::that, None)?;

  // After
  use tokio_util::sync::CancellationToken;

  let username = client
      .oauth2_flow("", CancellationToken::new(), |url| open::that(url))
      .await?;
  ```
- Change `ApiClient` to `Client`, drop the `&CallOptions` parameter from every shortcut, and make `CallOptions` private. A shortcut returns `Call<T>`, finished by `send().await`, which carries the per-call `auth`, `username`, `trace`, `no_auth`, `timeout`, `pagination_token`, and `header` settings that used to live on `CallOptions`. by @brettdavies in [#173](https://github.com/brettdavies/xurl-rs/pull/173)

  ```rust
  // Before
  use xurl::api::{ApiClient, CallOptions};

  let mut opts = CallOptions::default();
  opts.username = Some("brettdavies".to_string());
  let posts = client.search_posts("rust", 10, &opts)?;

  // After
  use xdk::api::Client;

  let posts = client
      .search_posts("rust", 10)
      .username("brettdavies")
      .send()
      .await?;
  ```
- Change `Client::auth()` to an `async fn` returning a `Result`, because a client built from credentials in code holds no token store to hand back.

  ```rust
  // Before: an infallible accessor
  let auth = client.auth();

  // After
  let auth = client.auth().await?;
  ```
- Add `xdk-rs` as the library: the `xurl` library target no longer ships in `xurl-rs`, and every module it exported lives under `xdk::`. The client is async and every shortcut is a builder finished by `.send().await`. `docs/migrating/v4.0.0.md` carries the full rename table. by @brettdavies in [#174](https://github.com/brettdavies/xurl-rs/pull/174)

  ```rust
  // Before: Cargo.toml had xurl-rs = "3"
  use xurl::api::{ApiClient, CallOptions};
  use xurl::auth::Auth;
  use xurl::config::Config;

  let cfg = Config::new();
  let mut client = ApiClient::new(&cfg, Auth::new(&cfg));
  let posts = client.search_posts("rust", 10, &CallOptions::default())?;

  // After: Cargo.toml has xdk-rs = "0.1"
  use xdk::api::Client;
  use xdk::auth::Auth;
  use xdk::config::Config;

  let cfg = Config::new();
  let client = Client::new(&cfg, Auth::new(&cfg))?;
  let posts = client.search_posts("rust", 10).send().await?;
  ```
- Change the published surface to what the documentation names. The OAuth1 signing primitives, the OAuth2 callback listener, the pending sign-in state, the token-store snapshot, and the raw-URL media helpers are no longer part of the documented API, and `Call::auth_wire` is `Call::auth_wire_unchecked` because it bypasses the auth-method check. by @brettdavies in [#175](https://github.com/brettdavies/xurl-rs/pull/175)

  ```rust
  // Before
  let call = client.search_posts("rust", 10).auth_wire("oauth2");

  // After
  let call = client.search_posts("rust", 10).auth_wire_unchecked("oauth2");
  ```
- Change `Error::AuthMethodMismatch` to carry a boxed `AuthMismatch` struct, which brings `xdk::Error` to 48 bytes so a function returning `xdk::Result` no longer trips clippy's `result_large_err`. by @brettdavies in [#178](https://github.com/brettdavies/xurl-rs/pull/178)

  ```rust
  // Before
  return Err(Error::AuthMethodMismatch {
      requested: "oauth2".into(),
      required: "oauth1".into(),
  });

  // After
  return Err(Error::from(AuthMismatch {
      requested: "oauth2".into(),
      required: "oauth1".into(),
  }));
  ```
- Remove the unread `pagination_token` field from `RequestOptions` and `execute_media_upload` from the documented API. A list shortcut's cursor is `Call::pagination_token`, and `Client::upload_media` is the embedder path.

  ```rust
  // Before
  let mut opts = RequestOptions::default();
  opts.pagination_token = Some(cursor);

  // After
  let page = client.get_bookmarks().pagination_token(cursor).send().await?;
  ```
- Change `Client::send_dm` to return `Call<ApiResponse<DmSentResult>>` instead of `Call<ApiResponse<DmEvent>>`. The API's reply never carried the fields `DmEvent` declared, so the call failed deserialization; read `dm_event_id` where you read `id`, and `dm_conversation_id` for the conversation. by @brettdavies in [#194](https://github.com/brettdavies/xurl-rs/pull/194)

  ```rust
  // Before
  let sent = client.send_dm(participant_id, "hello").send().await?;
  println!("{}", sent.data.id);

  // After
  let sent = client.send_dm(participant_id, "hello").send().await?;
  println!("{} in {}", sent.data.dm_event_id, sent.data.dm_conversation_id);
  ```
- Change `Error::kind()` to name an API refusal by its status: 403 reports `forbidden`, 400 and 422 report `invalid-request`, 5xx reports `server-error`, and any other status reports `api-error`. `network-error` now names only a request that got no answer, and `Error::exit_code()` returns 5 for a transport failure instead of a code read from digits in the request URL. by @brettdavies in [#198](https://github.com/brettdavies/xurl-rs/pull/198)

  ```rust
  // Before: every status but 401, 404 and 429 arrived as a transport failure
  match err.kind() {
      "network-error" => retry_after_backoff(),
      other => report(other),
  }

  // After: a refusal names itself, and network-error means no answer came back
  match err.kind() {
      "forbidden" => check_app_permissions(),
      "invalid-request" => fix_request(),
      "server-error" | "api-error" => retry_after_backoff(),
      "network-error" => retry_after_backoff(),
      other => report(other),
  }
  ```

### Added

- Add `Error::docs_url()`, which names the documentation page for an enrollment refusal, a credential failure, or a rate limit. by @brettdavies in [#171](https://github.com/brettdavies/xurl-rs/pull/171)
- Add `Client::builder()`, so a consumer can build a client from a bearer token, an OAuth2 token pair, or OAuth1 credentials held in code, with no token store and no environment variable. by @brettdavies in [#173](https://github.com/brettdavies/xurl-rs/pull/173)

  ```rust
  let client = Client::builder()
      .bearer(std::env::var("BEARER")?)
      .build()?;
  ```
- Add the `OnTokenRefreshed` hook: the client hands every rotated OAuth2 pair to it, `TokenStore` implements it for its default app, and `TokenStore::refresh_hook_for(app)` scopes it to a named app.
- Add `Client::last_rate_limit()` and the `RateLimit` type, reporting the `x-rate-limit-*` window from the most recent response. by @brettdavies in [#175](https://github.com/brettdavies/xurl-rs/pull/175)
- Add the `rustls` (default) and `native-tls` Cargo features, which select the TLS backend. A build with neither fails at compile time with the fix named rather than at the first request.
- Add a doctested README as the crate's docs.rs landing page, opening with a complete bearer-token program and covering authentication, errors, rate limits, testing, and the TLS features. by @brettdavies in [#176](https://github.com/brettdavies/xurl-rs/pull/176)
- Add `examples/`: read a post, search, authenticate with the refresh hook, post with media, stream, and a credential-free `offline_search`.
- Add the `testing` feature and `xdk::testing::MockX`, an in-process mock of the API seeded from the crate's own fixtures, so a consumer's tests need no credentials and spend nothing.
- Add `Client::upload_media(path)`, a `MediaUpload` builder that infers the media type and category from the file, takes the same per-call `auth`, `username`, `trace`, and `header` settings a `Call` does, awaits a video's processing, and returns a `MediaUploadOutcome` whose `media_id()` feeds `create_post`. by @brettdavies in [#178](https://github.com/brettdavies/xurl-rs/pull/178)
- Add `Client::get_chat_moderators`, `add_chat_moderator`, and `remove_chat_moderator`, with a `ChatModeratorsResult` response type and a `chat_moderators` fixture in the `testing` mock. by @brettdavies in [#182](https://github.com/brettdavies/xurl-rs/pull/182)
- Add routes for every declared shortcut endpoint to the `testing` mock (`MockX`): the user-graph reads, the undo verbs, the media upload phases, and usage credits. by @brettdavies in [#191](https://github.com/brettdavies/xurl-rs/pull/191)
- Add `crates/xdk/CHANGELOG.md`, generated on the release branch like the binary's. by @brettdavies in [#202](https://github.com/brettdavies/xurl-rs/pull/202)

### Changed

- Change the client to a cheap-`Clone`, `&self` handle that shares one HTTP client and one credential lock, so a token refresh cannot be lost to a dropped request or duplicated by a concurrent one. by @brettdavies in [#172](https://github.com/brettdavies/xurl-rs/pull/172)
- Change `ClientBuilder`, `Call`, `Auth`, and `TokenStore` to implement `Debug` with every secret redacted. by @brettdavies in [#176](https://github.com/brettdavies/xurl-rs/pull/176)
- Change `Client::from_env()` to build over the `~/.xurl` store without `CLIENT_ID` exported; the store's default app supplies the client id. by @brettdavies in [#178](https://github.com/brettdavies/xurl-rs/pull/178)
- Change the OAuth2 scopes requested at enrollment to include `broadcast.read` and `broadcast.write`; a token issued before this release must be re-enrolled before the broadcast shortcuts work. by @brettdavies in [#182](https://github.com/brettdavies/xurl-rs/pull/182)

### Fixed

- Fix credential writes made during a legacy-store migration or a twurlrc import bypassing the store lock, and make every store write durable across a crash, unique per writer, and safe on a symlinked store path. by @brettdavies in [#171](https://github.com/brettdavies/xurl-rs/pull/171)
- Fix a token exchange, refresh, or username lookup hanging forever when the HTTP client builder failed: the client is built once, and a build failure is an error rather than a fallback with no timeout. by @brettdavies in [#172](https://github.com/brettdavies/xurl-rs/pull/172)
- Fix a token refresh whose response omits `refresh_token` dropping the existing one; the stored refresh token is kept. by @brettdavies in [#177](https://github.com/brettdavies/xurl-rs/pull/177)
- Fix the headless OAuth2 sign-in writing its pending state through a symlink at the pending path; it refuses instead.
- Fix `Debug` output of the token store's app and token types to redact every credential.
- Fix warnings raised while the token store loads, such as a rejected `REDIRECT_URI` or a legacy-store migration, reaching stderr again.
- Fix `Config` and `EnvOverrides` printing the client secret and bearer token under `Debug`. by @brettdavies in [#178](https://github.com/brettdavies/xurl-rs/pull/178)
- Fix credential-store writes so a crash or a concurrent writer can no longer leave the store truncated or drop a rotated refresh token: saves are atomic, `0600` from creation, and serialized by an OS file lock beside the store. by @brettdavies in [#180](https://github.com/brettdavies/xurl-rs/pull/180)
