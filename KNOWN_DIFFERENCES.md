# Known Differences from Go xurl

## Webhooks (intentionally deferred)

The Go `xurl` includes ~194 lines of webhook/ngrok code (`cli/webhook.go`) that supports:

- Webhook registration and listing
- Local listener with ngrok tunneling

This feature is **intentionally not ported** because:

1. It requires an external `ngrok` binary and account, making it a niche workflow.
2. The X API Account Activity API (which webhooks serve) has been largely superseded by the v2 filtered stream and
   compliance endpoints.
3. It adds a significant dependency surface (ngrok process management, tunnel lifecycle) for a rarely-used feature.

If you need webhook support, continue using the Go `xurl` binary for that workflow.

## Exit code mapping for HTTP errors (intentional improvement)

The Go version maps HTTP errors to exit codes by string-matching the error message body (e.g., checking if the body
contains "404"). This fails for API responses where the JSON body doesn't contain the literal status code string,
causing 404 responses to return `EXIT_GENERAL_ERROR` (1) instead of `EXIT_NOT_FOUND` (4).

The Rust version uses structured pattern matching on `xdk::Error::Api { status, .. }`, which correctly maps HTTP status
codes to exit codes regardless of response body content. This means some edge-case exit codes differ:

| Scenario                                      | Go exit code | Rust exit code     | Rust is more correct |
| --------------------------------------------- | ------------ | ------------------ | -------------------- |
| 404 with JSON body (no literal "404" in body) | 1 (general)  | 4 (not found)      | Yes                  |
| 401 with JSON body (no literal "401" in body) | 1 (general)  | 77 (auth required) | Yes                  |
| 429 with JSON body (no literal "429" in body) | 1 (general)  | 3 (rate limited)   | Yes                  |

The structural mapping is intentional.

## A bare word is a command, not an endpoint (intentional improvement)

Go `xurl` accepts any positional and sends it as the endpoint (`cli/root.go`), so a mistyped command name becomes a
request against a path that does not exist and the caller reads an API error instead of a spelling correction.

The Rust version classifies the positional after the parse and before anything is loaded or sent. A token that reads as
a command name and matches none of them is a usage error at exit `2` with reason `unknown-command`, carrying the
offending word in `command` and the nearest real name in `suggestion` when one is close enough:

| Invocation       | Go behavior                     | Rust behavior                                         |
| ---------------- | ------------------------------- | ----------------------------------------------------- |
| `xr whoam`       | Request to `/whoam`, API error  | Exit 2, `unknown-command`, suggests `whoami`          |
| `xr zzzzzz`      | Request to `/zzzzzz`, API error | Exit 2, `unknown-command`, no suggestion              |
| `xr example.com` | Request to `/example.com`       | Exit 1, `validation` — a URL, and not an absolute one |
| `xr`             | Usage error                     | Exit 0, root help on stdout                           |

A help or version flag does not change that outcome: `xr whoam --help`, `xr whoam -h`, and `xr whoam --version` fail
exactly as `xr whoam` does.

A positional that starts with `http://`, `https://`, or `/` is still a raw request, and so is any invocation carrying a
raw-only flag (`-X`, `-H`, `-d`, `-F`, `-u`, `--auth`, `-t`, `-s`), which no command reads. A raw request has no help
page of its own, so `xr /2/users/me --help` prints the root help.

## Every stream the spec declares is streamed (intentional improvement)

Go `xurl` streams a raw request without `-s` only when its path is on a fixed list of eight: the filtered search stream,
the sample and sample10 streams, and the firehose stream with its four language variants. Any other path is read as one
buffered response, so nothing from a long-lived stream missing from the list prints as it arrives.

The Rust version streams every path whose operation the vendored X OpenAPI spec marks `x-twitter-streaming`, and the
build fails if a spec refresh leaves none marked. That set covers Go's eight plus seven more:

| Path                          | Go without `-s` | Rust without `-s` |
| ----------------------------- | --------------- | ----------------- |
| `/2/activity/stream`          | Buffered        | Streamed          |
| `/2/likes/firehose/stream`    | Buffered        | Streamed          |
| `/2/likes/sample10/stream`    | Buffered        | Streamed          |
| `/2/tweets/label/stream`      | Buffered        | Streamed          |
| `/2/tweets/compliance/stream` | Buffered        | Streamed          |
| `/2/likes/compliance/stream`  | Buffered        | Streamed          |
| `/2/users/compliance/stream`  | Buffered        | Streamed          |

`-s` still forces streaming on any path in both.

## OAuth1 signatures use RFC 5849 percent-encoding (intentional improvement)

Go `xurl` percent-encodes OAuth1 parameters with `url.QueryEscape` (`auth/auth.go`), a query-string encoder: a space
becomes `+`, `~` becomes `%7E`, and `*` stays bare. RFC 5849 section 3.6 leaves only the RFC 3986 unreserved characters
bare (letters, digits, `-`, `.`, `_`, `~`) and writes every other byte as `%XX`. X documents the RFC's encoding for the
signature it recomputes, so an OAuth1 request whose query or body value carries one of those characters signs a string
that differs from the one X builds.

The Rust version encodes the signature base string, the signing key, and the `Authorization` header parameters as the
RFC says. It reproduces the `oauth_signature` of X's published example,
[Creating a signature](https://docs.x.com/resources/fundamentals/authentication/oauth-1-0a/creating-a-signature), whose
`status` value contains spaces, a `+`, a comma, and a `!`:

| Value | Go `encode` | Rust `encode` |
| ----- | ----------- | ------------- |
| `a b` | `a+b`       | `a%20b`       |
| `~`   | `%7E`       | `~`           |
| `*`   | `*`         | `%2A`         |

A request with none of those characters in a signed value produces the same signature in both.

## A wait on media processing has a deadline (intentional improvement)

Go `xurl` waits on media processing in a loop with no deadline (`api/media.go`, `WaitForProcessing`): it polls until the
status reads `succeeded` or `failed`. A status that carries no `processing_info`, which is what an image reports, reads
as neither, so that wait never ends, and a job X never finishes is polled for as long as the process lives. Its `--wait`
is a boolean flag.

The Rust version always ends the wait:

| Situation                           | Go behavior                     | Rust behavior                                            |
| ----------------------------------- | ------------------------------- | -------------------------------------------------------- |
| Status carries no `processing_info` | Polls once a second without end | Returns the status after one call                        |
| Job still running                   | Polls without a deadline        | Exit 1, reason `processing-timeout`, at the deadline     |
| `--wait`, `--wait=true`             | Waits                           | Waits up to 60 seconds                                   |
| `--wait=<SECS>`                     | Accepts only `0` or `1`         | Waits up to that many seconds; `0` does not wait         |
| `--wait=false`                      | Does not wait                   | Does not wait                                            |
| `--wait 60` (value after a space)   | Not the flag's value            | Exit 2, with a tip pointing at `--wait=60`               |

Which uploads are waited for differs too. Go `xurl` decides by category alone: one whose name contains `video` or
`gif` (`mediaNeedsProcessing`). The Rust version waits for a video category, and for any other upload whose FINALIZE
answer carries a `processing_info` that is not yet final, which is the signal X documents for media that needs
processing. X answers FINALIZE for an animated GIF with `processing_info.state` `pending`, so both tools wait for one. An
image X reports as ready costs no status call here, whatever its category.

`media upload` still waits by default and `media status` still reads the status once unless `--wait` is given. The
timeout leaves the upload intact: its envelope carries `media_id` and a `resume-wait` step whose `command`, `xr media
status <media_id> --wait=<secs>`, waits again for twice as long.

## A waited `media upload` answers one document (intentional improvement)

Go `xurl` prints the FINALIZE response and, when it waited, the final status response after it (`api/media.go`,
`ExecuteMediaUpload`): two JSON documents on stdout, so a caller that parses the output once fails on the second and
`.data.id` reads twice.

The Rust version prints one document. It is FINALIZE's, with `processing_info` replaced by the final status's, or
removed when that status carries none, and with any other field only the status carried added:

```json
{
  "data": {
    "id": "1880028106020515840",
    "media_key": "7_1880028106020515840",
    "expires_after_secs": 86400,
    "processing_info": { "state": "succeeded", "progress_percent": 100 }
  }
}
```

An upload that did not wait prints FINALIZE's document unchanged, and so does one whose wait failed or timed out, with
the error on stderr. Under `--verbose`, INIT's response is printed ahead of it in text output only.

## The caller's id is asked for once per login (intentional improvement)

A command that acts as the caller (`like`, `follow`, `timeline`, `bookmarks`, and the rest) needs the caller's id in its
path. Go `xurl` asks `/2/users/me` for it on every run (`cli/shortcuts.go`, `resolveMyUserID`), or
`/2/users/by/username/<name>` under `--username`. Each is a billed user read.

The Rust version stores the id beside the login the first time `/2/users/me` answers under it: at sign-in, at the
refresh of a login that has none, on `xr whoami`, or on the first command that needs it. Later commands read it from the
store and send one request where Go sends two. The field is `user_id` on the OAuth2 token and on the OAuth1 access pair
in `~/.xurl/auth.yml`. Go `xurl` does not read it, and drops it when it rewrites a login, after which `xr` asks once
more.

The id is written only from an answer given under that same credential, and a credential that replaces another starts
without one, so it cannot name a different account. Another user's handle is looked up on every run in both tools.
