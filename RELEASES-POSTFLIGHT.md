# Post-release verification: `xurl-rs`

Operational post-flight checklist. Runs **after** the `release/v<version> → main` PR merges and you push the tag (`git
push origin vX.Y.Z`) per [`RELEASES.md` § Tagging and publishing](./RELEASES.md#tagging-and-publishing). Verifies that
the tag-triggered pipeline landed cleanly across `release.yml` → `homebrew-tap` → `finalize-release.yml`, and that the
published artifacts resolve on the public distribution channels.

Companion to [`RELEASES-PREFLIGHT.md`](./RELEASES-PREFLIGHT.md), which gates the release-branch cut. Both docs follow
the same go/no-go shape: every box is explicit, an unchecked or red item holds the next release (or motivates a hotfix).

## Quick start: run the automated gates

```bash
scripts/release-postflight.sh all
```

The script (`scripts/release-postflight.sh`) covers the automatable post-tag gates: `release.yml` end-to-end,
homebrew-tap dispatch, `finalize-release.yml` callback, GitHub Release `make_latest` flip, crates.io publish
verification. Install-on-fresh-machine smokes (`cargo install`, `brew install`, `cargo binstall`) are documented but not
driven from the script — running them on the local dev machine pollutes its toolchain and doesn't actually exercise the
fresh-machine semantics. Drive those on a throwaway container or a sibling machine.

Sub-commands let you re-run one verification in isolation:

| Sub-command   | What it checks                                                                                                   | Source of truth                           |
| ------------- | ---------------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| `release`     | `release.yml` on the tag push: `gh run view ... --json conclusion` is `"success"`                                | `gh run view`                             |
| `tap`         | `brettdavies/homebrew-tap` `update-formula` (repository_dispatch) + `Publish bottles` (workflow_run) ran SUCCESS | `gh run list -R brettdavies/homebrew-tap` |
| `finalize`    | `finalize-release.yml` callback ran in this repo (cross-repo dispatch loop closed)                               | `gh run list -e repository_dispatch`      |
| `make-latest` | GitHub Release `vX.Y.Z` is non-draft, non-prerelease, and `releases/latest` resolves to it                       | `gh api /releases/latest`                 |
| `crates`      | `crates.io` shows `xurl-rs vX.Y.Z` published (`cargo search xurl-rs` returns the tag)                            | `crates.io` index API                     |
| `all`         | every above                                                                                                      | —                                         |

Flags:

- `--repo OWNER/REPO` — override the auto-detected nameWithOwner
- `--tap-repo OWNER/REPO` — override the homebrew-tap repo (default: `brettdavies/homebrew-tap`)
- `--tag vX.Y.Z` — override auto-detection (default: `v$(grep version Cargo.toml)`)

## Checklist

Run immediately after the tag push triggers `release.yml`.

- [ ] **`release.yml` green end-to-end.** `gh run watch <id> --exit-status` then verify with `gh run view <id> --json
  conclusion --jq .conclusion` — the watcher exit code alone is not authoritative (a completed watcher is not a green
  watcher). Builds 5 cross-compile targets, publishes to crates.io via OIDC Trusted Publishing, and dispatches
  `update-formula` into the homebrew-tap. Run `scripts/release-postflight.sh release` for the automated check.
- [ ] **Homebrew-tap dispatch landed.** `gh run list -R brettdavies/homebrew-tap --limit 5` should show a recent
  `update-formula` (event=repository_dispatch) and a `Publish bottles` (event=workflow_run) both SUCCESS. The bottles
  workflow auto-merges the formula bump PR and pushes a `xurl-rs: add <version> bottle.` commit to tap `main`. Run
  `scripts/release-postflight.sh tap` for the automated check.
- [ ] **`finalize-release.yml` callback ran.** After the bottles publish, the tap dispatches back to this repo and the
  callback flips the GitHub Release `make_latest: true`. Check `gh run list -e repository_dispatch --limit 3`; expect a
  `finalize-release` SUCCESS. Run `scripts/release-postflight.sh finalize` for the automated check.
- [ ] **GitHub Release marked latest.** `gh api repos/<owner>/<repo>/releases/latest --jq .tag_name` returns `vX.Y.Z`,
  not the previous tag. Confirms `finalize-release.yml` actually flipped the flag. Run `scripts/release-postflight.sh
  make-latest` for the automated check.
- [ ] **`crates.io` shows the new version published.** `cargo search xurl-rs` lists the new version. The `xurl_rs`
  library re-exports (`use xurl_rs::*`) compile in a downstream toy crate. Run `scripts/release-postflight.sh crates`
  for the automated index check; the downstream-compile smoke is human-driven on a throwaway target.
- [ ] **`cargo install xurl-rs --version <new>`** on a clean environment resolves and runs. Drive on a fresh container
  or a sibling machine so the local `~/.cargo/bin` isn't polluted. Confirms the crates.io publish landed all package
  data and `cargo install` can reconstruct the binary from source.
- [ ] **`brew update && brew install brettdavies/tap/xurl-rs`** on a fresh prefix resolves the new bottle and `xr
  --version` reports the new tag. Drive on a throwaway prefix (`HOMEBREW_PREFIX=/tmp/brew-postflight-X brew ...`).
  Confirms the homebrew-tap end of the cross-repo dispatch chain landed cleanly and the published bottle SHA matches the
  formula.
- [ ] **`cargo binstall xurl-rs`** (without `--version`) resolves to the new tag and installs the matching prebuilt
  binary. Confirms the GitHub Release asset layout (binary + completions + licenses, expected archive naming) matches
  binstall's asset-resolution rules. Drive on a clean container.
- [ ] **Backport `main` → `dev`.** Per the documented learning at
  `docs/solutions/workflow-issues/post-release-backport-prevents-diff-b-false-positives-2026-05-07.md`: sync the
  release-only files (`CHANGELOG.md`, `cliff.toml`, any release-prep prose) from `main` to `dev` via a direct commit on
  `dev`. Keeps the next release's PREFLIGHT `diff-B` step quiet so a real missed cherry-pick stands out instead of
  hiding in expected divergence noise.

  ```bash
  git switch dev && git pull
  git checkout origin/main -- CHANGELOG.md cliff.toml   # add other release-only files as they emerge
  git status --short                                    # confirm only the expected files changed
  git commit -m "docs: sync release-only files from main post-v<X.Y.Z> (backport)"
  git push
  ```

## Related docs

- [`RELEASES-PREFLIGHT.md`](./RELEASES-PREFLIGHT.md): pre-cut go/no-go checklist (runs BEFORE this one).
- [`RELEASES.md`](./RELEASES.md): operational runbook for the full release lifecycle.
- [`RELEASES-RATIONALE.md`](./RELEASES-RATIONALE.md): release-flow rationale.
