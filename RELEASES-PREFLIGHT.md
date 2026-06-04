# Pre-release verification: `xurl-rs`

Operational pre-flight checklist. Runs **before** step 1 of
[`RELEASES.md` § Releasing dev to main](./RELEASES.md#releasing-dev-to-main). Gates the cut of the `release/v<version>`
branch, not the daily dev integration. Each box is an explicit go/no-go. If any item is unchecked or red, hold the
release.

CI (fmt, clippy, test, cargo-deny, Windows-compat, package-check) catches mechanical regressions inside this repo. This
checklist covers what CI structurally can't:

- Behavioral drift against the live X (Twitter) API. CI runs unit and integration tests against mocked HTTP, not the
  real endpoints, so API-contract regressions land silently until a user hits them.
- Distribution paths that only exercise on real artifacts (cross-compile binaries, `cargo install` from a clean machine,
  `cargo-binstall` against a published GitHub Release, `brew install` once the Homebrew dispatch finishes).
- Token-store correctness on a fresh machine (the YAML at `~/.xurl` is not exercised by the in-repo tests in the same
  way it is on first run).
- TLS stack correctness on Windows (the rustls + rustls-platform-verifier path differs from the dynamic-linker stack on
  Linux/macOS and CI's `ci / Windows check` only covers compile-time correctness).

## Establish the surface

Everything below assumes you know what's changing. Run this first.

```bash
LAST_TAG=$(git tag --sort=-version:refname | head -n 1)
git log "$LAST_TAG..dev" --oneline                              # commits going out
git diff "$LAST_TAG..dev" --stat                                # file-level scope
git diff "$LAST_TAG..dev" -- src/api/ src/auth/ src/cli/        # surface area: HTTP, auth, CLI
git log "$LAST_TAG..dev" --grep '^[a-z]\+!:' --oneline          # Conventional-Commits breaking markers
```

Every `!:` commit drives the major-version decision and gets a row in the release's `### Breaking changes` section.

## Checklist

### API-contract surface

xurl-rs is a thin client over the live X API. The contract that ships is the union of the 29 shortcut commands, the
generic `xr request` path, and the library re-exports in `src/lib.rs`.

- [ ] `xr help` lists the same shortcut commands as the previous release plus any net additions / removals. Diff
  `$LAST_TAG`'s `xr help` against `dev`'s and confirm every removed or renamed command has a `!:` commit and a `###
  Changed` (or `### Breaking changes`) bullet in the release changelog.
- [ ] `xr schema` (typed response introspection, added in v1.1.0) still emits a parseable JSON shape; downstream agents
  feature-detect from this. Diff the shape against `$LAST_TAG`'s output and surface any field rename / removal as a
  breaking row.
- [ ] Public library surface (`xurl_rs::*`): run `cargo public-api diff` if available, or `git diff "$LAST_TAG..dev" --
  src/lib.rs src/api/mod.rs` and confirm every removed / renamed export is captured as a breaking row. Library consumers
  feature-detect on type names.

### Real-world smoke (live X API)

The in-repo tests mock the HTTP layer. The four auth paths and the three output formats only exercise end-to-end on the
live API. Pick fresh targets each release.

- [ ] OAuth1 path: `xr <some-shortcut>` against an account configured with OAuth1 in `~/.xurl`. Confirms HMAC-SHA1
  signing didn't regress.
- [ ] OAuth2 PKCE path: `xr auth` against a fresh `XDG_CONFIG_HOME` (or temporarily renamed `~/.xurl`), then exercise a
  write-scoped shortcut. Confirms the PKCE flow and refresh-token rotation still work.
- [ ] OAuth2 headless (`--no-browser`) path: same exercise on a host without a graphical browser. Confirms the
  copy-paste-the-URL flow still works.
- [ ] Bearer token (env var, one-shot): `XURL_BEARER_TOKEN=… xr <read-only-shortcut> --auth app`. Confirms
  `Auth::get_bearer_token_header` honors the env var without a persisted store entry, the pattern stateless containers
  and agents pipe a bearer through.
- [ ] Bearer token (stored, two-step): `xr auth app --bearer-token "$(…)"` then `xr <read-only-shortcut> --auth app`.
  Confirms the persisted bearer in `~/.xurl` still loads for callers that opt into the store-backed path.
- [ ] Media upload: `xr media-upload <image>`. Confirms the chunked-upload state machine still works (`INIT` → `APPEND`
  → `FINALIZE` → poll `STATUS` until `processing_info.state` is `succeeded`).
- [ ] Output formats: `--output text`, `--output json`, `--output jsonl` for one streaming and one non-streaming
  endpoint. Confirms the `OutputConfig` plumbing didn't regress, particularly for jsonl on streaming endpoints.
- [ ] Error paths: drive at least one 401 (revoked token), one 429 (rate-limited), and one 4xx-with-error-body response.
  Confirms `XurlError` mapping still produces useful messages, not raw upstream JSON dumps.

### Distribution and install paths

The release builds cross-compiled binaries and the homebrew tap dispatches downstream. None of this runs in `cargo
test`.

- [ ] Last green run of `release.yml` (on this branch or a sibling) cross-compiled all five targets listed in
  `RELEASES.md` § Tagging and publishing. If the workflow has changed since, dry-run with `cargo build --release
  --target <target>` for each.
- [ ] In a clean container or fresh machine: download a prior release archive (`xurl-rs-<target>.tar.gz` or `.zip` for
  Windows), run `xr --version` and one read-only shortcut. Confirms the archive layout (binary + completions + licenses)
  still works without the project's toolchain.
- [ ] `cargo install xurl-rs --version <new>` from a clean environment resolves and runs once the crates.io publish
  completes (post-tag check, see below).
- [ ] Token-store roundtrip on a fresh machine: `xr auth` produces a `~/.xurl` YAML, subsequent `xr` invocations
  authenticate from it without re-prompting, and `xr auth status` correctly identifies multi-app entries.

### Release mechanics sanity

These items duplicate steps in `RELEASES.md` deliberately: easy to skip, expensive to recover from. Confirm explicitly.

- [ ] `Cargo.toml` `version` bumped to the new tag value (`check-version` in `release.yml` enforces this; catch early).
- [ ] `Cargo.lock` regenerated via `cargo update -p xurl-rs`, committed.
- [ ] Rebuild locally, confirm `xr --version` prints the new tag value.
- [ ] Every PR merged since `$LAST_TAG` has a non-empty `## Changelog` section. Spot-check via `gh pr list --base dev
  --state merged --search "merged:>$(git log -1 --format=%aI $LAST_TAG)"` then `gh pr view <num> --json body`.
- [ ] `rust-toolchain.toml` last bumped ≥7 days ago (supply-chain quarantine). If a bump landed inside the window, hold
  or revert it before tagging.
- [ ] No unmerged dependency advisories from `cargo deny check advisories`. The full local pre-push check
  (`scripts/hooks/pre-push`) mirrors CI; run it explicitly before pushing the release branch.
- [ ] Leak check: `git diff origin/main..HEAD --name-only | grep -E
  '^(docs/plans|docs/brainstorms|docs/ideation|docs/reviews|docs/solutions|\.context)'` returns nothing. If cherry-picks
  pulled in guarded paths via rename detection, resolve per `RELEASES.md` § Cherry-pick conflicts on guarded paths.
- [ ] `CHANGELOG.md` versioned section has no `[Unreleased]` placeholder and matches the bumped `Cargo.toml` version.

### Post-tag verification

Run immediately after the tag push triggers `release.yml`.

- [ ] `release.yml` green end-to-end. `gh run watch <id> --exit-status` then verify with `gh run view <id> --json
  conclusion --jq .conclusion` — the watcher exit code alone is not authoritative.
- [ ] Homebrew-tap `update-formula` dispatch completed (check `gh run list -R brettdavies/homebrew-tap`), then
  `finalize-release.yml` ran back here and flipped the GitHub Release `make_latest: true`.
- [ ] `crates.io` shows the new version published. `cargo install xurl-rs --version <new>` from a clean environment
  resolves and runs. The crate's `xurl_rs` library re-exports surface (`use xurl_rs::*`) compiles in a downstream toy
  crate.
- [ ] `brew update && brew install brettdavies/tap/xurl-rs` on a fresh prefix resolves the new bottle and `xr --version`
  reports the new tag. Confirms the homebrew-tap end of the cross-repo dispatch chain landed cleanly.
- [ ] `cargo binstall xurl-rs` (without `--version`) resolves to the new tag and installs the matching prebuilt binary.
  Confirms the GitHub Release asset layout matches binstall's expectations.
- [ ] Backport `main` → `dev` per `RELEASES.md` § After publish, then `git push origin dev`.

## Related docs

- [`RELEASES.md`](./RELEASES.md): operational runbook this checklist gates.
- [`RELEASES-RATIONALE.md`](./RELEASES-RATIONALE.md): release-flow rationale.
- [`AGENTS.md`](./AGENTS.md): project structure, auth paths, output formats.
