---
title: Get xurl-rs and xdk-rs listed on X's Tools and Libraries page
type: docs
status: active
date: 2026-09-04
---

# Get xurl-rs and xdk-rs listed on X's Tools and Libraries page

## Outcome

`https://docs.x.com/tools-and-libraries` carries a `Rust` tab under Community libraries that lists both crates this
repository publishes — `xdk-rs`, the client library, and `xurl-rs`, the `xr` CLI — and the X developer platform team
knows the project exists. Success is the merged docs PR; a documented, reviewed decline is the acceptable failure.

This plan owns the submission. The adoption-grade crate plan's U13 owns the publish, which is done, and the
post-submission checkpoint.

## What the evidence says

- **The page and its owner.** Source is `tools-and-libraries.mdx` in `xdevplatform/docs`, a Mintlify site. One X
  engineer (`tcaldwell-x`) made nine of the last ten edits to the file; `santiagomed` merges the other staff PRs. The
  repo has no `CONTRIBUTING`, no PR template, and no CI on pull requests (its only workflow is a scheduled OpenAPI
  scrape). It has 21 forks. Every PR must pass Twitter's Contributor License Agreement, the Apache Individual CLA with
  Twitter as licensee, enforced across all X repositories by the cla-assistant app; one signature covers every future
  contribution to any of them.
- **Outside PRs are unproven, not rejected.** Of the last 100 closed PRs, 90 came from the Mintlify web-editor bot and
  10 from X staff. No outside human has opened one.
- **"Other tools" is first-party.** Its three rows are the OpenAPI spec, twitter-text, and the Embed Generator, all
  X-owned. Third-party tools live under "Community libraries", which already lists a CLI (`twarc`, under Python) and
  carries the note "Community libraries are not maintained by X". Rust has no tab today; it is one bare-link row in the
  "Other" tab: `twitter-v2`.
- **The forum channel is dead.** The Libraries category's description invites announcements of libraries "that could be
  included in or linked from" the docs, but the one such request found (a Ballerina connector, November 2024) drew zero
  replies in 22 months and was never listed.
- **`twitter-v2` is stale but still used.** `jpopesculian/twitter-v2-rs`: last release 0.1.8 on 2022-10-25, last commit
  2022-11-29, seven open issues with no maintainer reply since 2022, about 2,200 downloads in the last 90 days, not
  archived.
- **The repository publishes two crates.** `xdk-rs` 0.1.0 is the async client library, imported as `xdk`; `xurl-rs`
  4.0.0 is the `xr` CLI built on it. Both are on crates.io as of 2026-09-18, and each has its own README, changelog, and
  tag line. A listing that names only one of them misses either the thing a library reader wants or the thing that
  carries the `xurl` lineage.
- **Presentation is ready.** Every README states that the project is independent and not affiliated with X, and
  `docs.rs/xdk-rs`, the Homebrew tap, and GitHub Releases all resolve. GitHub reports the license as Apache-2.0 only,
  while the manifests declare `MIT OR Apache-2.0`: GitHub's detector names the first license file it recognizes and
  cannot express an OR, so the sidebar stays that way by decision and the README's dual-license badge carries the truth.

## Decisions

| Decision         | Choice                                                                                 | Reasoning and alternative                                                                                                                                                                                                        |
| ---------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Section          | New `Rust` tab under Community libraries, placed after `Ruby` and before `Other`       | Chosen over the "Other tools" table once its rows proved first-party; a reviewer would move or close a third-party row there. A tab gives Rust a Description column the "Other" tab lacks, which two entries need more than one. |
| Entries          | Two rows: `xdk-rs` first, then `xurl-rs`                                               | The section is Community *libraries*, so the library leads; the CLI follows because it is what a reader coming from X's own `xurl` page is looking for. Listing one crate would leave the other undiscoverable from this page.   |
| `twitter-v2` row | Relocate it into the new `Rust` tab and delete the `**Rust**` row from the `Other` tab | Removing an incumbent in the PR that adds your own tool reads as self-serving, and the crate still has users. Relocating keeps every existing link working, retires the bare-link row, and makes the PR purely additive.         |
| Link target      | The GitHub repository, deep-linked to the crate directory where one exists             | Every row on the page links a repository (one links ReadTheDocs). `xdk-rs` points at `crates/xdk`, `xurl-rs` at the repository root. crates.io and docs.rs are the alternatives and are named in the descriptions instead.       |
| Names            | `xdk-rs` and `xurl-rs`, the crates.io names                                            | Each is the package name a reader installs, and `xr` alone would not be findable. Never "official" or "X's"; the descriptions say port and independent.                                                                          |
| Channels         | The docs PR plus outreach on X, no forum post                                          | Chosen over the forum after the unanswered precedent. The forum stays as a last-resort contingency because it is the channel X names.                                                                                            |

## Phase 1: Make the repo listing-ready

**Complete.** The repository changes a reviewer clicks through to were owned by U1 through U4 of
`docs/plans/2026-09-09-1528-fix-pre-attention-cleanup-plan.md` — the non-affiliation line and the relationship section,
the security policy, the contributing guide, and the issue forms — and all four are merged. Two releases went out behind
them rather than the single 3.2.0 this phase asked for: 3.2.0 on 2026-09-14 carrying the first-run fixes, and 4.0.0 on
2026-09-18 carrying the workspace split, alongside `xdk-rs` 0.1.0. The skill-bundle pass landed as `xurl-rs-skill` #13
and was re-verified against the binary by #17.

The description sentences for the listing are final: each crate's README first paragraph is the source.

## Phase 2: The docs PR

Target `xdevplatform/docs`, fork under `brettdavies`, branch `community-libraries-rust`, base `main`.

- **Edit** `tools-and-libraries.mdx` only. Insert a `Rust` tab after the `Ruby` tab and delete the `**Rust**` row from
  the `Other` tab, whose single entry moves into the new tab. The tab mirrors the existing ones (two-column table,
  left-aligned). "Other tools" is not touched. The two new descriptions run to two lines where every other row fits
  one; they stay at full length by decision on 2026-09-22. As submitted in `b2b2d39`:

```mdx
  <Tab title="Rust">
| Library | Description |
|:--------|:------------|
| [xdk-rs](https://github.com/brettdavies/xurl-rs/tree/main/crates/xdk) | Async v2 client: OAuth 1.0a, OAuth 2.0 PKCE with refresh, typed responses, chunked media upload, streaming |
| [xurl-rs](https://github.com/brettdavies/xurl-rs) | Rust port of xurl (`xr`): the same raw requests and shortcuts, with machine-readable output and structured exit codes |
| [twitter-v2](https://github.com/jpopesculian/twitter-v2-rs) | Async client library |
  </Tab>
```

- **Verify locally** with the Mintlify CLI (`npx mint dev` from the fork root), since a fork gets no preview
  deployment. Verified 2026-09-22 in headless Chromium: the `Rust` tab shows the three rows with their links, and the
  `Other` tab keeps its four remaining rows. The page's `require is not defined` error also occurs on unmodified
  upstream. npm's 7-day release-age cooldown on this machine rejects a pinned fresh `mint` version; run it unpinned.
- **PR body.** Short, no template exists. Four parts: what changed; why a tab (Rust had no description column and now
  has two maintained entries that need one); what moved, naming the relocation of `twitter-v2` as a relocation so no
  reviewer reads it as a removal; and an acknowledgement that both crates are community-maintained and unaffiliated,
  matching the page's note, plus an offer to fall back to the first contingency below. It states no `twitter-v2`
  figures. The body as sent is on #447.
- **Title.** `docs: add a Rust tab to Community libraries`, the `docs:` form the page owner uses.
- **CLA.** Signed 2026-09-22; the `license/cla` check on #447 passes. It is Twitter's Contributor License Agreement,
  and one signature covers every future contribution to X's repositories. The grant is a non-exclusive,
  irrevocable copyright and patent license over text you submit to X's repositories and issue trackers; it does not
  reach either crate, which are linked, not submitted.
- **Re-verify before opening.** Done on 2026-09-22, the day the PR opened: the page source and the `twitter-v2`
  repository and crate were re-read, and the `twitter-v2` figures in "What the evidence says" are from that read.
- **Deliverable.** Open PR URL, recorded here: https://github.com/xdevplatform/docs/pull/447, opened 2026-09-22 from
  `brettdavies/docs:community-libraries-rust` at `b2b2d39`.

## Phase 3: X outreach

- **Day 0 (same day as the PR).** One public post from your account: a one-line pitch naming both crates, the PR link,
  tagging `@XDevelopers`. Post it with `xr post` under OAuth 2.0 user context: the outreach doubles as a demonstration.
  It is a live, billed call: the posting app must be registered, on the pay-per-use package, and in the Production
  environment, the same portal prerequisites the README's before-you-start block names. Keep it to one post; no thread.
- **Day 10.** If the PR and the post are both silent, one nudge: a PR comment mentioning `@tcaldwell-x` (page owner) and
  `@santiagomed` (merges docs PRs), asking whether placement or wording needs to change. Stay on GitHub for the nudge; a
  second public post reads as pressure.
- **Day 20.** If still silent, the last-resort channel: a short post in the developer forum's "Libraries, SDKs, and
  sample code" category linking the PR. It is the channel X documents, so it belongs in the record even if it goes
  unanswered.
- **Day 40.** Stop. Leave the PR open, keep the fork branch, and revisit when the page next changes (a new commit on
  `tools-and-libraries.mdx` means someone is looking).

## Contingencies

| If                                                  | Then                                                                                                             |
| --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| Reviewer objects to moving `twitter-v2`             | Amend: leave the `Other` tab's Rust row as it is and list only the two new crates in the `Rust` tab.             |
| Reviewer prefers a link-only row in the `Other` tab | Accept; a bare link beats no listing, and both crates fit one row. Keep the descriptions in the repo.            |
| Reviewer asks for a dedicated page                  | Decline politely; `/tools/*` pages are first-party. The tab row is the ask.                                      |
| PR closed without comment                           | Ask once in a comment what would make it acceptable; then Day 20 forum post; then stop.                          |
| A docs maintainer asks about the port relationship  | Point at `crates/xurl-cli/README.md` and `KNOWN_DIFFERENCES.md`; do not argue feature comparisons in the thread. |

## Out of scope

- A dedicated `/tools/xurl-rs` page, or edits to the official `/tools/xurl` page.
- A row in the "Other tools" table, which holds only X-owned entries.
- Renaming either crate or the binary.
- Listing in the X Ads API tools page.
- Any further change to what either crate does. The listing describes `xdk-rs` 0.1.0 and `xurl-rs` 4.0.0, both released
  2026-09-18.

## Sources

- Page: `https://docs.x.com/tools-and-libraries` (append `.md` for the rendered markdown).
- Source: `https://github.com/xdevplatform/docs/blob/main/tools-and-libraries.mdx`.
- Forum category: `https://devcommunity.x.com/t/about-the-libraries-sdks-and-sample-code-category/139317`.
- Unanswered precedent: `https://devcommunity.x.com/t/listing-official-ballerina-connector-for-x/229669`.
- Incumbent crate: `https://github.com/jpopesculian/twitter-v2-rs`, `https://crates.io/crates/twitter-v2`.
- This project: `https://github.com/brettdavies/xurl-rs`, `https://crates.io/crates/xdk-rs`,
  `https://crates.io/crates/xurl-rs`, `https://docs.rs/xdk-rs`, `KNOWN_DIFFERENCES.md`.

## Reconciliation

(against `xurl-rs` `origin/dev` @ `5cff0ee`, 2026-09-22)

| Phase   | State     | Note                                                                                                 |
| ------- | --------- | ---------------------------------------------------------------------------------------------------- |
| Phase 1 | landed    | U1-U4 merged; 3.2.0 and 4.0.0 released; the skill-bundle pass landed as `xurl-rs-skill` #13 and #17. |
| Phase 2 | in-review | xdevplatform/docs#447 open since 2026-09-22; CLA signed and `license/cla` green; awaiting X review.  |
| Phase 3 | not-built | Day 0 post undecided; nudge 2026-10-02, forum 2026-10-12, stop 2026-11-01 if #447 stays silent.      |

The evidence this plan rests on was re-read on 2026-09-22, the day the PR opened, and still holds:
`tools-and-libraries.mdx` last changed on 2026-07-25, "Other tools" still carries only its three X-owned rows, and Rust
is still one bare-link `twitter-v2` row in the `Other` tab. The PR body states no `twitter-v2` figures.

What changed since the plan was written is on this side, not X's. The repository now publishes two crates instead of
one, so the listing names both; the release the listing describes is 4.0.0 rather than the 3.2.0 this plan expected to
be the last one before the PR; and the `twitter-v2` row is relocated into the new tab rather than proposed for removal.
