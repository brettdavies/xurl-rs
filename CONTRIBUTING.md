# Contributing to xurl-rs

The repository holds two crates: `xdk-rs`, the async X API client library (`crates/xdk`), and `xurl-rs`, the `xr`
command-line tool built on it (`crates/xurl-cli`) and an independent port of
[`xdevplatform/xurl`](https://github.com/xdevplatform/xurl). Bug reports, feature requests, and code are welcome for
either. A solo maintainer cannot promise merge windows; real PRs land. Which crate a change belongs in is decided by
[Where a change goes](AGENTS.md#where-a-change-goes).

## Dev setup

```bash
git clone https://github.com/brettdavies/xurl-rs && cd xurl-rs
cargo build                                # binary at ./target/debug/xr
cargo test                                 # unit + integration
git config core.hooksPath scripts/hooks    # activate the pre-push battery
```

That one command activates both hooks. `pre-commit` is staged-file-scoped and fast: `rustfmt --check` on staged `.rs`,
`actionlint` on a staged workflow, `markdownlint-cli2` on staged `.md`, `shellcheck` on staged shell. Both hooks share
`scripts/hooks/_lib.sh`, which owns how each tool runs while each hook owns which files it runs on.

`pre-push` mirrors CI over the repo: `cargo fmt`, `cargo clippy` with warnings denied, `cargo test --workspace`, the
MSRV check, the doc build, the examples build, the credential-free example run against the `testing` mock, the TLS
feature cells (`native-tls` alone compiles; no backend fails on the guard), `cargo deny check`, `shellcheck`, a
Windows compatibility scan, `markdownlint-cli2`, and `actionlint`. Two more steps run only when their tool is
installed and say so otherwise: the docs.rs build (`cargo +nightly doc` with `--cfg docsrs` and every feature) and
the feature powerset (`cargo hack`).

`pre-push` scopes each step to what the push actually changes, so a docs-only push skips the Rust battery entirely and
finishes in seconds. The scoping fails open: an unrecognized path runs everything, and running the hook by hand sweeps
the whole repo. Every step that is skipped says so on its own line, so a skip never reads as a pass.

Four CI gates have no hook counterpart, because each needs a clean checkout, a released baseline, or a release build:
completions freshness (`./scripts/generate-completions.sh --check`), the package check (`cargo publish --dry-run
--workspace`), the public-API semver gate (`cargo semver-checks` against the last `xdk-rs-v*` tag), and the
agent-native audit with the release binary's size ceiling (`anc audit` on `target/release/xr`). The feature matrix's
`--all-features` and `--features testing` test runs and its `rustls`-alone cell are CI-only too. Run those yourself
when a change touches the CLI surface, the library API, the feature set, or the release profile. The live-API checks
stay manual by
design: `cargo test -- --ignored` runs the TLS handshake probe and, with `XURL_LIVE_SMOKE=1`, the wire-vocabulary
smoke; `crates/xurl-cli/tests/conformance/` compares against the Go `xurl` only when that binary is on `PATH`; and
`benches/benchmark.sh` times `xr` against the Go `xurl` with hyperfine.

## Branch and PR flow

Branch from `dev` as `feat/<slug>` or `fix/<slug>`, then PR back to `dev`. `main` is what consumers install and receives
code only through a release PR. [`RELEASES.md`](RELEASES.md) is the full runbook.

- **Commits and PR titles** follow [Conventional Commits](https://www.conventionalcommits.org/). The PR title becomes
  the squash-merge subject and lands in `CHANGELOG.md` at release time, so write it as the history entry you want.
- **PR bodies** fill [`.github/pull_request_template.md`](.github/pull_request_template.md). Its `## Changelog` section
  is the source of truth for `CHANGELOG.md`, which is generated and never hand-edited.
- **Engineering docs** (`docs/plans/`, `docs/research/`, `docs/reviews/`, `CONCEPTS.md`) commit straight to `dev` and
  are blocked from `main`. Everything a consumer reads, markdown included, goes through the branch and PR flow.
- **Quality bar, testing, and architecture** live in [`AGENTS.md`](AGENTS.md). Tests never touch the real home
  directory; build stores on an explicit path under a `tempfile::TempDir`, which
  `crates/xurl-cli/tests/store_isolation_guard.rs` enforces.

## Error contract

Text output is written for humans and the structured formats for agents, and the two need not match word for word: text
carries prose and a help pointer, structured output carries stable fields to branch on. Every structured error carries a
kebab-case `reason` from a closed set, an `exit_code`, a human `message`, the offending value when there is one, and a
`next_step` object `{action, command | template, docs}`, where `command` is runnable verbatim by a non-TTY caller and
`template` carries angle-bracket placeholders only the caller can fill. Prefer additive envelope changes: add keys
rather than renaming or retyping existing ones, and regenerate `schema/output.schema.json` when the envelope changes.

## Filing issues

Use a [bug report](https://github.com/brettdavies/xurl-rs/issues/new?template=bug-report.yml), a
[feature request](https://github.com/brettdavies/xurl-rs/issues/new?template=feature-request.yml), or a
[skill-bundle report](https://github.com/brettdavies/xurl-rs/issues/new?template=skill-bundle.yml). X API platform
questions (rate limits, enrollment, pricing) belong in the [X developer community](https://devcommunity.x.com/), and
questions about the original Go tool belong on [its own tracker](https://github.com/xdevplatform/xurl/issues).
Open-ended discussion goes in [Discussions](https://github.com/brettdavies/xurl-rs/discussions).

## Try it without an X account

X ships a local server that simulates the API v2, seeded with users and posts, so `xr` can be exercised end to end
without credentials or spend.

```bash
go install github.com/xdevplatform/playground/cmd/playground@latest
export PATH="$PATH:$(go env GOPATH)/bin"
playground start --port 8089     # 8080 is the default, and the OAuth2 callback port

# In another shell:
API_BASE_URL=http://localhost:8089 XURL_BEARER_TOKEN=test_token xr search "coding"
```

A bearer reaches the app-only endpoints, so `xr search` and `xr usage` both answer from the seeded state. User-context
commands such as `xr whoami` stop at the auth matrix before a request leaves the machine, which is the same refusal they
give without credentials against the real API. The playground's parameter vocabulary predates X's post-vocabulary
rename, so a command that sends the newer expansion names comes back from it as an invalid-request error.
