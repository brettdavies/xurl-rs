# Vendored X API OpenAPI Spec

This directory contains a checked-in copy of X's public OpenAPI spec, used at
build time to generate the auth-method matrix in `src/api/auth_matrix.rs` (see
`build.rs`).

## Provenance

| Field              | Value                                  |
| ------------------ | -------------------------------------- |
| Upstream URL       | https://api.x.com/2/openapi.json |
| Spec `info.version` | 2.168                          |
| Path count         | 156                                    |
| File size          | 894290 bytes                            |
| SHA256             | `b0a471fc4e27c8824e2b6c32954c75557656d2661945b151e0e6ece16ab6ac36` |
| Refreshed (UTC)    | 2026-09-02                             |

## Refresh

Run from the repo root before each release cycle:

```bash
scripts/refresh-x-openapi.sh
```

The script downloads the current spec, validates it as JSON, replaces this
directory's copy, and rewrites this README. CI drift-check
(`.github/workflows/spec-drift.yml`) flags divergence between runs and posts
either a job summary, a PR comment, or a tracked issue depending on the
trigger.

## Why vendor?

A checked-in spec gives reproducible builds (Homebrew bottle CI, offline
builds), an auditable supply chain (the spec is greppable from source), and CI
that does not need to reach `api.x.com` on every push. Manual refresh is the
trade-off we accept; the drift-check workflow shortens time-to-notice.

See `docs/brainstorms/2026-06-04-001-auth-method-enforcement-requirements.md`
and `docs/plans/2026-06-04-001-feat-auth-method-enforcement-plan.md` for the
full rationale.
