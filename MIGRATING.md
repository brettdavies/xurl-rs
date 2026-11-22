# Migrating xurl-rs

Per-version migration guides for library consumers (`xdk::*`, and `xurl::*` before 4.0.0) and `xr` CLI users. Each
major version, and each minor that takes a break, ships its own guide under `docs/migrating/`. Start with the file for
the version you're moving to; if you're jumping several versions, read each file in version order.

## Guides

| Target version                                                                        | Guide                      | Highlights                                                                                                                                                                                                                                                                                                                                      |
| ------------------------------------------------------------------------------------- | -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`v2.0.0`](https://github.com/brettdavies/xurl-rs/blob/main/docs/migrating/v2.0.0.md) | `docs/migrating/v2.0.0.md` | Client-side auth-method enforcement; `RequestOptions.endpoint: String` -> `target: RequestTarget`; `AuthMethodMismatch` error variant; `EXIT_AUTH_MISMATCH = 2`; `block_user` / `unblock_user` shortcuts removed                                                                                                                                |
| [`v3.0.0`](https://github.com/brettdavies/xurl-rs/blob/main/docs/migrating/v3.0.0.md) | `docs/migrating/v3.0.0.md` | X API spec 2.168 post vocabulary; `Tweet` / `TweetPublicMetrics` / `ReferencedTweet` / `RetweetedResult` -> `Post` / `PostPublicMetrics` / `ReferencedPost` / `RepostedResult`; `referenced_posts`, `repost_count`, `post_count`, `includes.posts`; `xr validate --schema post` / `posts`; `xr usage credits` added                             |
| [`v4.0.0`](https://github.com/brettdavies/xurl-rs/blob/main/docs/migrating/v4.0.0.md) | `docs/migrating/v4.0.0.md` | The library is `xdk-rs` (`use xdk::`); `ApiClient` -> `Client::builder()` and async `Call<T>` shortcuts; `XurlError` -> `xdk::Error`; presentation modules left the library; `xurl-rs` is the CLI only, and nothing about installing or running `xr` changed                                                                                    |
| [`v4.5.0`](https://github.com/brettdavies/xurl-rs/blob/main/docs/migrating/v4.5.0.md) | `docs/migrating/v4.5.0.md` | `broadcasts scheduled remove` -> `cancel`, `sources rename` -> `update --name`, `scheduled live`, `--manual-publish`, and `chat send`/`remove`/`mute`/`unmute` removed; an all-digit account is a user ID; validation refusals carry `rule` and a sentence `message`; `xr schema` takes command words and prints the resolved `data` by default |
| [`v4.8.0`](https://github.com/brettdavies/xurl-rs/blob/main/docs/migrating/v4.8.0.md) | `docs/migrating/v4.8.0.md` | A stream that Ctrl+C ends writes `reason: "sigint"` in its `cancelled` document, where 4.7 wrote `sigterm`; SIGTERM still writes `sigterm`                                                                                                                                                                                                      |
| [`v4.9.0`](https://github.com/brettdavies/xurl-rs/blob/main/docs/migrating/v4.9.0.md) | `docs/migrating/v4.9.0.md` | A token endpoint answer is a refusal only when its JSON `error` is one of the six RFC 6749 codes: a 400 or a 401 with no such code is `invalid-request` or `api-error` at exit 1 with `fix-input`, where 4.8 gave `auth-required` at 77; the token request follows no redirect                                                                  |

## Convention

When a new major version ships, add the corresponding file under `docs/migrating/vX.0.0.md` and append a row to the
table above; a minor that takes a break does the same as `vX.Y.0.md`. Other releases that introduce migration steps
can live here too, and the table makes the linkage explicit so readers always know which guide applies to their jump.
