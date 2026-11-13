# Contributing to xurl-rs

The repository holds two crates: `xdk-rs`, the async X API client library (`crates/xdk`), and `xurl-rs`, the `xr`
command-line tool built on it (`crates/xurl-cli`) and an independent port of
[`xdevplatform/xurl`](https://github.com/xdevplatform/xurl). Bug reports, feature requests, and code are welcome for
either. A solo maintainer cannot promise merge windows; real PRs land. Which crate a change belongs in is decided by
[Where a change goes](AGENTS.md#where-a-change-goes).

## Dev setup

```bash
git clone https://github.com/brettdavies/xurl-rs && cd xurl-rs
scripts/dev-setup.sh                       # install the tools CI and the hooks use
git config core.hooksPath scripts/hooks    # activate the pre-push battery
```

`cargo build` then puts the binary at `./target/debug/xr`. `cargo nextest run --workspace` runs the unit and integration
tests and stops at the first failure; add `--profile ci` to run every test the way CI does, rerunning a failed one once.
`cargo test --doc --workspace` runs the doc tests, which nextest does not.

`scripts/dev-setup.sh` needs `cargo` and reads `.github/tool-versions`, the one list of tool versions, which CI's
install steps read too. It installs each missing or older tool, Rust tools through `cargo binstall` and the rest through
Homebrew, and prints one line per tool. It never uses `sudo`, and a rerun changes nothing. When an install fails, it
exits 1 naming `dev-setup/<tool>` and the command to run by hand. `--dry-run` reports what it would install, and
`--root DIR` puts the Rust tools under `DIR/bin`. To change a version, edit the list:
`crates/xurl-cli/tests/dev_setup_tests.rs` fails for a `ci.yml` step that installs a tool any other way.

The `git config` line activates both hooks. `pre-commit` is fast and runs a check only when a file it reads is staged:
`cargo fmt -- --check` over every crate when a `.rs` file is staged, `actionlint` on a staged workflow,
`markdownlint-cli2` on staged `.md`, `shellcheck` on staged shell. It builds nothing, so a markdown-only commit runs
`markdownlint-cli2` alone, even for a document a test reads; those tests run at push. Both hooks share
`scripts/hooks/_lib.sh`, which owns how each tool runs while each hook owns which files it runs on.

`pre-push` mirrors CI over the repo: `cargo fmt`, `cargo clippy` with warnings denied, `cargo nextest run --workspace`
and `cargo test --doc --workspace`, the MSRV check, the doc build, the examples build, the credential-free example run
against the `testing` mock, the TLS feature cells (`native-tls` alone compiles; no backend fails on the guard), the
docs.rs feature set (`xdk-rs` documented with every feature), `cargo deny check`, `shellcheck`, a Windows compatibility
scan, `markdownlint-cli2`, and `actionlint`. The feature powerset (`cargo hack`) runs only when the tool is installed
and says so otherwise. nextest is the exception: without it, or with one older than `.config/nextest.toml` requires,
`pre-push` stops at once with `pre-push/missing-tool` and the `scripts/dev-setup.sh` command, and when nextest cannot
parse `.config/nextest.toml`, with `pre-push/nextest-config`.

`pre-push` scopes each step to what the push actually changes, so a docs-only push skips the Rust battery entirely and
finishes in seconds. A document counts as docs only when it is markdown outside `crates/`, not a README, and named by
no Rust source: tests read `AGENTS.md` and the PR template, so a push touching one of those runs the battery. The
gitignored `.context/`, the license texts, the issue forms, the rulesets, and the Dependabot config count as docs too,
unless a Rust source names the file or a directory above it; workflows do not. `scripts/cargo-inputs.sh` makes that
call, and CI's `Change scope` job applies the same filter to a pull request, skipping every Rust job on a docs-only PR
while lint still runs. The scoping fails open: an unrecognized path runs everything, and running the hook by hand
sweeps the whole repo. Every step that is skipped says so on its own line, so a skip never reads as a pass.

Five CI gates have no hook counterpart, because each needs a clean checkout, a released baseline, a release build, or a
Windows machine: completions freshness (`./scripts/generate-completions.sh --check`), the package check (`cargo publish
--dry-run --workspace`), the public-API semver gate (`cargo semver-checks` against the last `xdk-rs-v*` tag), the
agent-native audit (`anc audit` on a dev build of `xr`, or on a release PR on `target/release/xr` under the release
binary's size ceiling), and `Windows program target` (`cargo test -p xurl-rs --test webhooks_program_tests` on a Windows
runner). The feature matrix's `--all-features` and `--features testing` test runs and its `rustls`-alone cell are
CI-only too. Run those yourself when a change touches the CLI surface, the library API, the feature set, or the release
profile. The live-API checks stay manual by design: `cargo nextest run --workspace --run-ignored only` runs the TLS
handshake probe and, with `XURL_LIVE_SMOKE=1`, the wire-vocabulary smoke; and `benches/benchmark.sh` times `xr` against
the Go `xurl` with hyperfine.

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

## Adding a command

A command family touches about nine files across both crates, and the test suite, not a checklist, is what catches a
family that is only half-registered. [Adding a command family](AGENTS.md#adding-a-command-family) in `AGENTS.md` names
every surface and the walk that guards it. When one of those walks fails, its message names the missing entry, the
likely cause, and the file to edit, so a red test is the recipe's next step rather than a puzzle.

## Error contract

Text output is written for humans and the structured formats for agents, and the two need not match word for word: text
carries prose and a help pointer, structured output carries stable fields to branch on. Every structured error carries a
kebab-case `reason` from a closed set, an `exit_code`, a human `message`, and the offending value when there is one. A
reason with a step the caller can take also carries a `next_step` object `{action, command | template, docs}`, where
`command` is runnable verbatim by a non-TTY caller and `template` carries angle-bracket placeholders only the caller can
fill. The CLI README's "Which Errors Carry a Step" names the reasons that carry one and the ones that do not; a new
error picks an existing action where one fits before it ships without a step. Prefer additive envelope changes: add keys
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
