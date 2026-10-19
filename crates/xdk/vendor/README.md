# Vendored X API OpenAPI Spec

This directory contains a checked-in copy of X's public OpenAPI spec, used at build time to generate the auth-method
matrix in `crates/xdk/src/api/auth_matrix.rs` (see `crates/xdk/build.rs`).

## Provenance

| Field               | Value                                                              |
| ------------------- | ------------------------------------------------------------------ |
| Upstream URL        | https://api.x.com/2/openapi.json                                   |
| Spec `info.version` | 2.172                                                              |
| Path count          | 186                                                                |
| File size           | 1162584 bytes                                                      |
| SHA256              | `b6f82d056ce28850a31284a3d522d5ef5c4bcfd3fea5a65ae60143b053811e7a` |
| Refreshed (UTC)     | 2026-10-19                                                         |

## Refresh

Run from the repo root before each release cycle:

```bash
scripts/refresh-x-openapi.sh
```

The script downloads the current spec, validates it as JSON, replaces this directory's copy, and rewrites this README
through `scripts/render-vendor-readme.sh`. The CI drift check (`.github/workflows/spec-drift.yml`) flags divergence
between runs and, depending on the trigger, writes a job summary, comments on the PR, or opens a refresh PR to `dev`.

## Why vendor?

A checked-in spec gives reproducible builds (Homebrew bottle CI, offline builds), an auditable supply chain (the spec is
greppable from source), and CI that does not need to reach `api.x.com` on every push. Manual refresh is the trade-off we
accept; the drift-check workflow shortens time-to-notice.
