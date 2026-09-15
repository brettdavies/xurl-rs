# Get xurl-rs listed on X's Tools and Libraries page

Created: 2026-09-04

## Outcome

`https://docs.x.com/tools-and-libraries` carries a `Rust` tab under Community libraries that lists xurl-rs, and the X
developer platform team knows the project exists. Success is the merged docs PR; a documented, reviewed decline is the
acceptable failure.

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
  2022-11-29, six open issues with no maintainer reply since 2022, about 2,300 downloads in the last 90 days, not
  archived.
- **xurl-rs presentation gaps.** The README never states that the project is not affiliated with X. GitHub reports the
  license as Apache-2.0 only, while `Cargo.toml` declares `MIT OR Apache-2.0`: GitHub's detector names the first
  license file it recognizes and cannot express an OR, so the sidebar stays that way by decision and the README's
  dual-license badge carries the truth. crates.io (3.1.0), docs.rs, the Homebrew tap, and GitHub Releases all resolve.

## Decisions

| Decision             | Choice                                                                                           | Reasoning and alternative                                                                                                                                                                                                        |
| -------------------- | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Section              | New `Rust` tab under Community libraries, placed after `Ruby` and before `Other`                 | Chosen over the requested "Other tools" table once its rows proved first-party; a reviewer would move or close a third-party row there. A tab gives Rust a Description column the "Other" tab lacks.                             |
| `twitter-v2` row     | Propose removal in the same PR, with the dates stated neutrally and an explicit offer to keep it | Removing an incumbent in the PR that adds your own tool reads as self-serving, and the crate still has users. The evidence supports retirement; the offer keeps the PR mergeable either way. Fallback: both rows in the new tab. |
| Link target          | The GitHub repository                                                                            | Every row on the page links a repository (one links ReadTheDocs); the README is the one page with every install channel. crates.io is the alternative and is named in the description instead.                                   |
| Name and description | `xurl-rs`, described as a Rust port of xurl                                                      | The name is the crate and formula name, and the port relationship is the fastest way for a reader to place it. Never "official" or "X's".                                                                                        |
| Channels             | The docs PR plus outreach on X, no forum post                                                    | Chosen over the forum after the unanswered precedent. The forum stays as a last-resort contingency because it is the channel X names.                                                                                            |

## Phase 1: Make the repo listing-ready

The repository changes a reviewer clicks through to are owned by U1 through U4 of
`docs/plans/2026-09-09-1528-fix-pre-attention-cleanup-plan.md`: the README non-affiliation line and "Relationship to xurl"
section, the security policy, the contributing guide, and the issue forms. Land those before Phase 2, and tag 3.2.0
carrying that plan's Phase B (U10, U6, U7, U12a, U5, and U11 unless it slips) before Phase 2 opens, with the first
skill-bundle pass beside the tag, so the binary a reader installs on listing day carries the first-run fixes. The
description sentence for the listing is final once the README section is merged; Phase 2 copies it.

## Phase 2: The docs PR

Target `xdevplatform/docs`, fork under `brettdavies`, branch `community-libraries-rust`, base `main`.

- **Edit** `tools-and-libraries.mdx` only. Insert a `Rust` tab after the `Ruby` tab and delete the `**Rust**` row from
  the `Other` tab. The tab mirrors the existing ones exactly (two-column table, left-aligned, terse descriptions).
  Directional shape:

```mdx
  <Tab title="Rust">
| Library | Description |
|:--------|:------------|
| [xurl-rs](https://github.com/brettdavies/xurl-rs) | Rust port of xurl: CLI and library with JSON output and schema discovery |
  </Tab>
```

- **Verify locally** with the Mintlify CLI (`npx mint dev` from the fork root) that the tab renders, since a fork gets
  no preview deployment. Confirm the `Other` tab still renders with the Rust row gone.
- **PR body.** Short, no template exists. Four parts: what changed; why a tab (Rust had no description column and now
  has a maintained entry); the `twitter-v2` dates, stated as dates with a one-line offer to restore the row if X prefers
  to keep it; and an acknowledgement that xurl-rs is community-maintained, matching the page's note. Author it in
  `/tmp/`, scrub with `/unslop`, submit with `--body-file`.
- **Title.** `docs: add a Rust tab to Community libraries` or the repo's plain-English style (`Add Rust tab with xurl-rs
  to Community libraries`); the repo uses both.
- **CLA.** Sign Twitter's Contributor License Agreement when the cla-assistant check asks, as an individual unless an
  employer holds rights to your open-source work, in which case get that permission first. The grant is a non-exclusive,
  irrevocable copyright and patent license over text you submit to X's repositories and issue trackers; it does not
  reach xurl-rs, which is linked, not submitted.
- **Deliverable.** Open PR URL, recorded here.

## Phase 3: X outreach

- **Day 0 (same day as the PR).** One public post from your account: a one-line pitch for xurl-rs, the PR link, tagging
  `@XDevelopers`. Post it with `xr post` under OAuth 2.0 user context: the outreach doubles as a demonstration. It is a
  live, billed call: the posting app must be registered, on the pay-per-use package, and in the Production environment,
  the same portal prerequisites the README's before-you-start block names. Keep it to one post; no thread.
- **Day 10.** If the PR and the post are both silent, one nudge: a PR comment mentioning `@tcaldwell-x` (page owner) and
  `@santiagomed` (merges docs PRs), asking whether placement or wording needs to change. Stay on GitHub for the nudge; a
  second public post reads as pressure.
- **Day 20.** If still silent, the last-resort channel: a short post in the developer forum's "Libraries, SDKs, and
  sample code" category linking the PR. It is the channel X documents, so it belongs in the record even if it goes
  unanswered.
- **Day 40.** Stop. Leave the PR open, keep the fork branch, and revisit when the page next changes (a new commit on
  `tools-and-libraries.mdx` means someone is looking).

## Contingencies

| If                                                  | Then                                                                                                                 |
| --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Reviewer wants `twitter-v2` kept                    | Amend: both rows in the `Rust` tab, xurl-rs first.                                                                   |
| Reviewer prefers a link-only row in the `Other` tab | Accept; a bare link beats no listing. Keep the description in the repo.                                              |
| Reviewer asks for a dedicated page                  | Decline politely; `/tools/*` pages are first-party. The tab row is the ask.                                          |
| PR closed without comment                           | Ask once in a comment what would make it acceptable; then Day 20 forum post; then stop.                              |
| A docs maintainer asks about the port relationship  | Point at the README section from Phase 1 and `KNOWN_DIFFERENCES.md`; do not argue feature comparisons in the thread. |

## Out of scope

- A dedicated `/tools/xurl-rs` page, or edits to the official `/tools/xurl` page.
- Renaming the project or the binary.
- Listing in the X Ads API tools page.
- Any change to what xurl-rs does beyond the pre-attention cleanup plan; the listing describes 3.2.0, the release
  tagged before Phase 2.

## Sources

- Page: `https://docs.x.com/tools-and-libraries` (append `.md` for the rendered markdown).
- Source: `https://github.com/xdevplatform/docs/blob/main/tools-and-libraries.mdx`.
- Forum category: `https://devcommunity.x.com/t/about-the-libraries-sdks-and-sample-code-category/139317`.
- Unanswered precedent: `https://devcommunity.x.com/t/listing-official-ballerina-connector-for-x/229669`.
- Incumbent crate: `https://github.com/jpopesculian/twitter-v2-rs`, `https://crates.io/crates/twitter-v2`.
- This project: `https://github.com/brettdavies/xurl-rs`, `https://crates.io/crates/xurl-rs`, `KNOWN_DIFFERENCES.md`.
