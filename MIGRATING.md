# Migrating to xurl 2.0.0

xurl 2.0.0 lands client-side enforcement of which X API auth method each endpoint accepts. The validator refuses
mismatched `--auth` invocations before the HTTP round-trip with a typed envelope. To enable the validator, the library's
`RequestOptions` shape changes; that is the breaking surface this guide covers.

If you only use the `xr` CLI, the only behavior change you should care about is the new exit code and JSON envelope
shape — skip to [CLI behavior changes](#cli-behavior-changes).

If you depend on `xurl` as a Rust library on crates.io, read every section.

## Why the break

xurl 1.x stored every API call's path as a rendered string on `RequestOptions.endpoint`. The validator needs the
**unrendered template** (`/2/users/{id}/likes`, not `/2/users/12345/likes`) so it can look the endpoint up against the
auth matrix generated from X's OpenAPI spec.

Threading the template alongside the rendered string would have left two strings that could silently drift. The 2.0.0
release replaces them with one typed value.

## `RequestOptions` shape change

Before (1.x):

```rust
use xurl::api::RequestOptions;

let req = RequestOptions {
    method: "POST".to_string(),
    endpoint: format!("/2/users/{user_id}/likes"),
    data: serde_json::to_string(&body)?,
    auth_type: "oauth1".to_string(),
    ..Default::default()
};
client.send_request(&req)?;
```

After (2.0):

```rust
use std::collections::HashMap;
use xurl::api::{RequestOptions, RequestTarget};

let req = RequestOptions {
    method: "POST".to_string(),
    target: RequestTarget::Template {
        // spec template — uses X's parameter names ({id}), not your local
        // variable name ({user_id}).
        path: "/2/users/{id}/likes".to_string(),
        path_params: HashMap::from([
            ("id".to_string(), user_id.to_string()),
        ]),
        // For paginated endpoints, push cursor as a query param.
        query: vec![],
    },
    data: serde_json::to_string(&body)?,
    auth_type: "oauth1".to_string(),
    ..Default::default()
};
client.send_request(&req)?;
```

### Spec parameter names

The template path **must** use X's spec parameter names. xurl-rs's internal variable names frequently differ from the
spec's. Cross-reference table (sample — see `vendor/x-api-openapi.json` for the full list):

| xurl-rs local | Spec parameter      | Where it appears                              |
| ------------- | ------------------- | --------------------------------------------- |
| `user_id`     | `{id}`              | most `/2/users/{id}/*` endpoints              |
| `post_id`     | `{tweet_id}`        | `/2/users/{id}/likes/{tweet_id}`              |
| `post_id`     | `{source_tweet_id}` | `/2/users/{id}/retweets/{source_tweet_id}`    |
| `media_id`    | `{id}`              | `/2/media/upload/{id}/append` and `/finalize` |

If you guess wrong, `cargo test` fails the `auth_matrix_coverage` check with the offending `(method, path)` named. The
check runs on every test run; CI will not let a wrong template ship to crates.io.

### Query parameters

In 1.x, query parameters were appended to the rendered `endpoint` string. In 2.0, push them onto `query`:

```rust
RequestTarget::Template {
    path: "/2/users/{id}/tweets".to_string(),
    path_params: HashMap::from([("id".to_string(), user_id.to_string())]),
    query: vec![
        ("max_results".to_string(), "10".to_string()),
        ("pagination_token".to_string(), cursor.to_string()),
    ],
}
```

Order is preserved; URLs round-trip stably for snapshot tests.

### Raw mode

If you were passing a full URL to `RequestOptions.endpoint` (e.g. forwarding from `xr <URL>` raw mode), use the `RawUrl`
variant:

```rust
RequestTarget::RawUrl("https://api.x.com/2/users/me".to_string())
```

The validator skips `RawUrl` targets per the same R18 escape hatch the CLI's `xr <URL>` mode uses. You accept the
contract by reaching for raw mode.

`RawUrl` enforces a scheme allowlist: only `http://` and `https://` addresses pass. Other schemes (`file://`, `ftp://`,
etc.) return `XurlError::InvalidUrl` rather than reaching `reqwest`.

## `get_auth_header_public` signature change

The companion library-public auth helper signature changes from `(&mut self, method: &str, url: &str, auth_type: &str,
username: &str)` to `(&mut self, options: &RequestOptions)`. Construct a `RequestOptions` with the relevant fields and
pass it.

Before:

```rust
let header = client.get_auth_header_public(
    "GET",
    "https://api.x.com/2/users/me",
    "oauth2",
    "alice",
)?;
```

After:

```rust
let opts = RequestOptions {
    method: "GET".to_string(),
    target: RequestTarget::Template {
        path: "/2/users/me".to_string(),
        path_params: HashMap::new(),
        query: vec![],
    },
    auth_type: "oauth2".to_string(),
    username: "alice".to_string(),
    ..Default::default()
};
let header = client.get_auth_header_public(&opts)?;
```

This signature change unlocks the endpoint-aware intersection in the auto-detect path (next section).

## New error variant: `XurlError::AuthMethodMismatch`

The validator returns a new variant when an explicit `--auth` choice does not match the endpoint's accepted schemes, OR
when auto-detect's intersection of stored credentials and endpoint-accepted schemes is empty.

```rust
XurlError::AuthMethodMismatch {
    endpoint: String,                       // path template
    method: String,                         // HTTP method, uppercase
    requested: Option<String>,              // None when auto-detect fired
    supported: Vec<String>,                 // schemes the endpoint accepts
    available_in_app: Option<Vec<String>>,  // None when --auth was explicit;
                                            // Some(..) for empty-intersection
}
```

If you pattern-match on `XurlError`, add an arm for the new variant. `XurlError::kind()` returns
`"auth-method-mismatch"` as a stable closed-set string for downstream tooling. `XurlError::exit_code()` returns `2`
(`EX_USAGE`) — distinct from `EXIT_AUTH_REQUIRED` (`77`), which still surfaces when no credentials are stored at all.

## CLI behavior changes

Even if you only use the `xr` CLI, two observable behaviors change:

### New exit code on mismatched `--auth`

```text
$ xr media upload x.jpg --app bird-prod --auth app
Error: Bearer (app) auth is not accepted at POST /2/media/upload. Use --auth oauth1 or --auth oauth2.
$ echo $?
2
```

In 1.x, this same invocation would have produced X's 403 with `reason: network-error`. In 2.0, the validator catches it
locally with exit code 2 and `reason: auth-method-mismatch`.

### Auto-detect prefers OAuth2 over OAuth1 when both are stored

If an app holds both OAuth1 and OAuth2 credentials and the endpoint accepts both, `--auth` auto-detect picks OAuth2
first. In 1.x, auto-detect walked OAuth2 → OAuth1 → Bearer in fixed order regardless of endpoint acceptance; in 2.0 the
walk is intersected with the endpoint's accepted schemes first.

Notable consequence: an OAuth2 token that lacks a required scope still surfaces X's 403 through the existing
`network-error` envelope — client-side scope checking is not part of 2.0 (it ships in a later release).

### Two shortcuts removed

`xr` 1.x exposed `block_user` and `unblock_user` shortcuts. Neither path is documented in X's current OpenAPI spec, so
the auth matrix has no entry to validate against and the matrix coverage check would fail. Both shortcuts are removed in
2.0.

If you need the behavior, use raw mode:

```bash
xr POST /2/users/$source/blocking -d '{"target_user_id":"...","auth":"oauth1"}'
xr DELETE /2/users/$source/blocking/$target --auth oauth1
```

R19's permissive matrix-miss path lets raw-mode calls through without validation. X arbitrates whether the call actually
works.

## Validator behavior reference

For library callers wiring custom request flows:

- `RequestTarget::Template` with a `(method, path)` pair in the auth matrix is validated. Explicit `--auth` mismatches
  return `AuthMethodMismatch`. Empty-intersection on auto-detect returns `AuthMethodMismatch` when the app holds at
  least one credential.
- `RequestTarget::Template` with a `(method, path)` pair **not** in the auth matrix is permissive — the request goes out
  and X arbitrates (R19). Unknown endpoints from spec drift fall through the same path.
- `RequestTarget::RawUrl` skips validation entirely (R18) but still enforces the scheme allowlist (`InvalidUrl` for
  non-HTTP[S]).
- `path_params` values containing `/`, `?`, `#`, or `%` return `InvalidPathParam` before any HTTP call — these
  characters would change the URL's path semantics under percent-encoding.

## Further reading

- Brainstorm: `docs/brainstorms/2026-06-04-001-auth-method-enforcement-requirements.md`
- Plan: `docs/plans/2026-06-04-001-feat-auth-method-enforcement-plan.md`
- Vendored OpenAPI spec: `vendor/x-api-openapi.json` (and `vendor/README.md` for provenance)
