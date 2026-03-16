# Known Differences from Go xurl

## Webhooks (intentionally deferred)

The Go `xurl` includes ~194 lines of webhook/ngrok code (`cli/webhook.go`) that supports:

- Webhook registration and listing
- Local listener with ngrok tunneling

This feature is **intentionally not ported** because:

1. It requires an external `ngrok` binary and account, making it a niche workflow.
2. The X API Account Activity API (which webhooks serve) has been largely superseded by the v2 filtered stream and compliance endpoints.
3. It adds a significant dependency surface (ngrok process management, tunnel lifecycle) for a rarely-used feature.

If you need webhook support, continue using the Go `xurl` binary for that workflow.
