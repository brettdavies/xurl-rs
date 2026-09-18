<!-- changelog-router: this file routes to the per-crate changelogs and carries no release history of its
     own. A tool that generates a changelog refuses to write here; see scripts/generate-changelog.py. -->

# Changelog

This repository publishes two crates, and each keeps its own changelog beside its own manifest. There is no combined
history here, because the two crates version and release independently and a merged list would imply a shared release
line that does not exist.

| Crate                                         | Changelog                                                      | Tags            | What it is                                    |
| --------------------------------------------- | -------------------------------------------------------------- | --------------- | --------------------------------------------- |
| [`xurl-rs`](https://crates.io/crates/xurl-rs) | [`crates/xurl-cli/CHANGELOG.md`](crates/xurl-cli/CHANGELOG.md) | `vX.Y.Z`        | the `xr` command-line tool                    |
| [`xdk-rs`](https://crates.io/crates/xdk-rs)   | [`crates/xdk/CHANGELOG.md`](crates/xdk/CHANGELOG.md)           | `xdk-rs-vX.Y.Z` | the X API client library, imported as `xdk::` |

Reading them apart is the point. A release of one says nothing about the other: `xr` is unchanged across a library
minor that breaks its API, and the library is untouched by a CLI release that only moves flags around.

## Which one you want

- Installing or running `xr`, or reading what changed in a command, its output, or an exit code: the
  [CLI changelog](crates/xurl-cli/CHANGELOG.md).
- Depending on `xdk-rs` from a Rust program, or upgrading across one of its minors: the
  [library changelog](crates/xdk/CHANGELOG.md). Every breaking entry there carries a before and after snippet, which is
  a release gate rather than a courtesy; see [`crates/xdk/README.md`](crates/xdk/README.md) § Versioning.
- Moving a program from the `xurl` library target of `xurl-rs` 3.x to `xdk-rs`:
  [`docs/migrating/v4.0.0.md`](docs/migrating/v4.0.0.md) carries the rename table and a before and after for each moved
  API.

## How the crate changelogs are written

Neither is hand-edited. Each is generated from the section of every merged pull request that addresses that crate,
`## Changelog (xurl-rs)` or `## Changelog (xdk-rs)`, and cut on the release branch. A wording fix goes to the pull
request body and the file is regenerated, so the two never drift from the record they are built from.

This file is the exception: it is prose, written by hand, and holds no version sections. The generator refuses to write
here.
