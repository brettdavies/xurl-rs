# Migrating xurl-rs

Per-version migration guides for `xurl::*` library consumers and `xr` CLI users. Each major version ships its own guide
under `docs/migrating/`. Start with the file for the version you're moving to; if you're jumping multiple majors, read
each file in version order.

## Guides

| Target version                       | Guide                      | Highlights                                                                                                                                                                                                       |
| ------------------------------------ | -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`v2.0.0`](docs/migrating/v2.0.0.md) | `docs/migrating/v2.0.0.md` | Client-side auth-method enforcement; `RequestOptions.endpoint: String` -> `target: RequestTarget`; `AuthMethodMismatch` error variant; `EXIT_AUTH_MISMATCH = 2`; `block_user` / `unblock_user` shortcuts removed |

## Convention

When a new major version ships, add the corresponding file under `docs/migrating/vX.0.0.md` and append a row to the
table above. Minor and patch releases that introduce migration steps may also live here as `vX.Y.0.md`; the table makes
the linkage explicit so readers always know which guide applies to their jump.
