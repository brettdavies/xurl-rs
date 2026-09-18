# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-09-18

### Breaking changes

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

- Close the published surface, make TLS a feature, and add an embedder consumer crate (#175)
- Document the crate from a doctested README, add examples, and a testing mock (#176)
- Give the library its own release path and make every channel workspace-aware (#177)
- Upload media through a builder, keep Error small, and state the line between the crates (#178)
- Add broadcast chat moderator shortcuts and the xr broadcasts commands (#182)
- Walk the registries for the surfaces a new family can miss (#190)
- Answer every declared endpoint from the testing mock (#191)

### Changed

- Name every endpoint through the build script's declaration (#185)
