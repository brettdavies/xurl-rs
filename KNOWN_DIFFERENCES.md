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

| Scenario | Go exit code | Rust exit code | Rust is more correct |
|---|---|---|---|
| 404 with JSON body (no literal "404" in body) | 1 (general) | 4 (not found) | Yes |
| 401 with JSON body (no literal "401" in body) | 1 (general) | 2 (auth required) | Yes |
| 429 with JSON body (no literal "429" in body) | 1 (general) | 3 (rate limited) | Yes |

This is an intentional improvement introduced in the library ergonomics work (R8).
