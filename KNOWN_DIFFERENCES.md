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

The Rust version uses structured pattern matching on `XurlError::Api { status, .. }`, which correctly maps HTTP status
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
