---
title: Command Families - Plan
type: feat
date: 2026-10-26
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# Command Families - Plan

**Target repo:** brettdavies/xurl-rs. Every path is in it. **Base:** `dev`.

## Goal Capsule

- **Objective:** a person or agent reaches every X API operation that `endpoint-decisions.json` marks `planned` through
  an `xr` command that checks its inputs offline, answers a dry run, returns a typed and schema-described response, and
  fails with an agent-readable error, instead of through raw mode.
- **Means:** twenty family PRs, each built through AGENTS.md § "Adding a command family", stacked on `dev` (KTD1, KTD2).
- **Authority:** R-IDs win on behavior and KTDs on mechanism; a unit overrides neither. AGENTS.md owns the family recipe
  and its walks (§ "Adding a command family"), the command grammar (§ "Command grammar: flags vs subcommands"), and the
  crate boundary (§ "Where a change goes"); this plan cites them and adds only what each family needs.
- **Execution profile:** inline in one session, no subagents or worktrees, one PR per family on a stacked chain, long
  jobs in the background.
- **Stop conditions:** a test or the session would send a request to X; `--dry-run` would send a request; a family would
  need an OAuth2 scope that sign-in does not request; a unit gated on an open question (OQ1, OQ2) is next while the
  question is unanswered, in which case the next ungated unit is built instead.
- **Who ships:** each PR targets the PR below it, the bottom one `dev`; Brett reviews the command names and merges, and
  a green run of the stack lands with `gh stack merge`.

---

## Product Contract

### Summary

Twenty families bring the 76 planned operations into `xr`: new top-level commands for reads and actions on posts and
users, new families for the other API namespaces, new verbs on `xr broadcasts` and `xr media`, and the caller's own
endpoints behind `xr followers` and `xr following`. Each PR removes its family's entry from `endpoint-decisions.json`.
The Command Surface table under the Planning Contract names every command.

### Problem Frame

After the spec refresh to upstream 2.172, `endpoint-decisions.json` records a decision for every operation that has no
shortcut command. Twenty groups were decided as `planned` on 2026-10-26, group by group, each with its scope in the
entry's `reason`. Until they ship, a caller reaches those operations only through raw mode, which checks nothing
offline, describes no response schema, carries no command help or examples, and gives a dry run nothing to say but the
method and URL. An agent working from `xr examples`, `xr schema`, or the skill bundle cannot see them at all.

### Requirements

**Command surface**

- R1. Each family ships the commands the Command Surface table names for it, and removes its `planned` entry from
  `endpoint-decisions.json` in the same PR.
- R2. Command words follow AGENTS.md's command grammar: a flag never selects an endpoint, with the single exception KD3
  adds.
- R3. Each command prints X's response as a typed struct that derives `JsonSchema`, listed by `xr schema` and accepted
  by `xr validate`.

**Safety and offline checks**

- R4. Every command answers `--dry-run` per the dry-run contract in AGENTS.md § "Output formats": nothing sent, nothing
  stored, inputs checked, the credential named.
- R5. A verb that deletes or cancels something, or puts the caller's account in front of others (going live,
  publishing), asks for confirmation, which `--force` skips.
- R6. An input the spec bounds (a length, a count, an id shape, an enum, a required time window) is checked offline and
  refused with a validator code that has a text-mode sentence in `crates/xurl-cli/src/cli/output/validation.rs`.

**Agent contract**

- R7. Failures use the existing reasons and `next_step` actions; a family adds a reason or an action only when none
  fits, with its row in the README's "Which Errors Carry a Step" table.
- R8. No family adds an OAuth2 scope: sign-in already requests every scope these endpoints declare, and the scope walks
  in `crates/xdk/tests/auth_tests.rs` keep it so.

**Testing**

- R9. No test reaches X: every new endpoint has a mock route answering from a fixture whose shape the spec validates.

### Key Decisions

- KD1. **One PR per family.** (session-settled: user-directed — chosen over landing the families together: Brett, "those
  should be implemented as one pr per family".) Governs R1.
- KD2. **The twenty families and their scope are the planned entries' `reason` text.** (session-settled: user-directed —
  chosen over `raw-only` or `not-supported` for each group, decided group by group on 2026-10-26.) Governs R1.
- KD3. **`xr followers` and `xr following` without `--of` use the caller's own endpoints, and AGENTS.md's grammar gains
  the exception that a flag naming another user may pick that user's endpoint over the caller's own.** (session-settled:
  user-directed — chosen over a separate command for the caller's lists; recorded in the `me-follow-lists` reason.)
  Governs R2.

### Scope Boundaries

- Only the `planned` entries. Operations decided `raw-only` or `not-supported` stay as they are.
- Text output stays colored JSON for these commands, as for every command but dry runs (Brett, 2026-10-26).
- Shell completions keep regenerating in each PR (Brett kept PR-time completions on 2026-10-26).
- Reading the filtered stream stays raw mode (`xr -s /2/tweets/search/stream`); the stream family manages rules only.
- Considered and not built: a JSON passthrough for an article's `content_state`. The text format of KTD9 covers
  drafting; a caller who needs entities or inline styles can send a raw request, and a request for it in practice would
  change the call.
- Considered and not built: `--of` on `xr lists list`, `followed`, and `memberships`. The `lists` reason asks for the
  caller's own lists; the endpoints take any user, and the flag is one addition when someone asks.
- Considered and not built: `--exclude` on `xr user-posts` and the other optional filters the spec offers but no planned
  reason names. Each is one flag to add when someone needs it.

#### Deferred to Follow-Up Work

- The CI-scoping workstream (run only the jobs a branch needs) follows the families (Brett, 2026-10-26).
- The skill bundle documents the new commands with the next bundle pass, after a release ships them.

### Open Questions

Both questions are answered; each answer is what the units below build.

- OQ1 (U17, U18, U19). **Single-id lookup paths.** `GET /2/users/{id}`, `GET /2/spaces/{id}`, and
  `GET /2/media/{media_key}` answer one object, while their batch siblings (`/2/users`, `/2/spaces`, `/2/media`) answer
  a list. One command spanning both would give one command two response schemas, and a second command per namespace
  (`get` beside `lookup`) duplicates what the batch command does with one id. **Answer (Brett chose "batch only"):**
  each namespace gets one batch command that takes 1 to 100 ids, and the three single-id paths are `raw-only` entries
  (`user-by-id`, `media-by-key`, `space-by-id`) with that reason.
- OQ2 (U20). **Stream rules and `--dry-run`.** The `stream-rules` reason put X's `dry_run` parameter "behind
  `--dry-run`", but `xr`'s dry-run contract sends nothing on any command, and X's `dry_run=true` is a request X answers.
  **Answer (the recommendation Brett okayed):** `--dry-run` keeps sending nothing, and `--validate-only` on
  `xr stream rules add` and `remove` sends the change with `dry_run=true`, so X checks it and saves nothing. The name
  follows the `validate_only` request field in Google's AIP-163. A removal X only checks destroys nothing, so it skips
  the confirmation, and `--dry-run --validate-only` sends nothing.

---

## Planning Contract

### Key Technical Decisions

- KTD1. **The twenty PRs stack on `dev`, in the Unit Index order, and land in batches with `gh stack merge`.** The
  families share `help.golden`, the completions, the schema registry, the validate alias table, the examples page,
  `SHORTCUT_TEMPLATES`, and the KNOWN_DIFFERENCES list of commands Go lacks, so independent PRs off `dev` would
  conflict pairwise, and every rebase would rerun the full CI matrix. A stack builds each PR on the one below, and an
  atomic stack merge restacks nothing. About five unmerged PRs stay open at a time: a review fix low in a deeper stack
  would restack, and rerun CI on, every PR above it.
- KTD2. **Reads and actions on posts and users get top-level words; other namespaces get families.** This is AGENTS.md's
  grammar applied: a standalone read or action in the core domain gets its own word, as `followers`, `like`, and
  `bookmarks` do, and a noun that owns several operations becomes a family, as `bookmark-folders` and `calls` do.
  Existing families (`broadcasts`, `media`) gain verbs rather than gaining a sibling family.
- KTD3. **A batch lookup command takes 1 to 100 ids or handles as positional arguments**, refused offline past 100 (R6).
  Where both an id and a handle form exist (`xr users`), an argument that starts with `@` is a handle and an all-digit
  one is an id, and a mix is refused, since X answers each form from a different endpoint with the same response shape.
  Applies to U4, U17, U18, U19.
- KTD4. **New input checks live in `crates/xdk/src/api/shortcuts.rs` beside the existing validators, each returning a
  kebab-case code, and each code gets its sentence in `crates/xurl-cli/src/cli/output/validation.rs`.** The guard test
  there fails for a code without a sentence. New shared checks: a batch size (1 to 100), a time window (RFC 3339 start
  before end) for the analytics commands, and one id check per id shape the spec gives a pattern for.
- KTD5. **Deleting, cancelling, going live, and publishing go through `send_confirmed`**, as `delete`,
  `webhooks remove`, `broadcasts sources remove`, and `calls scheduled cancel` do (R5). Reversible actions (`hide`,
  `unfollow`, `unpin`, `mute`, removing a list member) do not ask.
- KTD6. **Responses reuse `Post`, `User`, and `DmEvent` wherever X answers with those objects, and add one struct per
  new resource in `crates/xdk/src/api/response/types.rs`**, named for the resource rather than the operation (a list is
  `List`, a Space is `Space`). `crates/xdk/tests/spec_types_validation.rs` holds every field to the spec.
- KTD7. **Each fixture is written from the operation's response schema in `crates/xdk/vendor/x-api-openapi.json`**, with
  the field names the spec gives, since no request to X may produce one (R9). `crates/xdk/tests/spec_validation.rs`
  checks each fixture's shape against the spec.
- KTD8. **Commands pick no auth scheme.** The generated auth matrix does, so `xr stream rules` goes out with the app's
  bearer token (the only scheme those endpoints take) and the user-context endpoints with the login, with no per-command
  code.
- KTD9. **`xr articles draft` reads the body as text from a file or stdin and the library turns it into X's
  `content_state` blocks**: a blank line ends a block, and a block starting `#`, `##`, `###`, `-`, `1.`, or `>` is a
  header, list item, or quote. The conversion is request building, so it lives in the library (AGENTS.md § "Where a
  change goes").
- KTD10. **`xr notes` uses X's verbs**: `evaluate` for the endpoint the reason calls "rate", since
  `POST /2/notes/evaluate` scores a draft note's text and rates no existing note. The `test_mode` parameter X requires
  is `--test`, false by default.

### Command Surface

Every command the plan adds or changes. "Confirms" marks the verbs KTD5 gates. A path marked OQ1 or OQ2 depends on that
question's answer.

| Unit | Command                                                                 | Operation                                                                                      | Confirms |
| ---- | ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | -------- |
| U1   | `xr followers` (no `--of`)                                              | `GET /2/users/me/followers`                                                                    |          |
| U1   | `xr following` (no `--of`)                                              | `GET /2/users/me/following`                                                                    |          |
| U2   | `xr hide <REPLY>`                                                       | `PUT /2/tweets/{tweet_id}/hidden` (`hidden: true`)                                             |          |
| U2   | `xr unhide <REPLY>`                                                     | `PUT /2/tweets/{tweet_id}/hidden` (`hidden: false`)                                            |          |
| U3   | `xr likers <POST>`                                                      | `GET /2/tweets/{id}/liking_users`                                                              |          |
| U3   | `xr reposters <POST>`                                                   | `GET /2/tweets/{id}/retweeted_by`                                                              |          |
| U3   | `xr quotes <POST>`                                                      | `GET /2/tweets/{id}/quote_tweets`                                                              |          |
| U4   | `xr read-posts <POST>...`                                               | `GET /2/tweets?ids=`                                                                           |          |
| U5   | `xr user-posts <USER>`                                                  | `GET /2/users/{id}/tweets`                                                                     |          |
| U6   | `xr search-users <QUERY>`                                               | `GET /2/users/search`                                                                          |          |
| U7   | `xr search-counts <QUERY>`                                              | `GET /2/tweets/counts/recent`                                                                  |          |
| U8   | `xr analytics <POST>... --start --end`                                  | `GET /2/tweets/analytics`                                                                      |          |
| U9   | `xr trends place <WOEID>`                                               | `GET /2/trends/by/woeid/{woeid}`                                                               |          |
| U9   | `xr trends personal`                                                    | `GET /2/users/personalized_trends`                                                             |          |
| U10  | `xr news search <QUERY>`                                                | `GET /2/news/search`                                                                           |          |
| U10  | `xr news get <ID>`                                                      | `GET /2/news/{id}`                                                                             |          |
| U11  | `xr communities search <QUERY>`                                         | `GET /2/communities/search`                                                                    |          |
| U11  | `xr communities get <ID>`                                               | `GET /2/communities/{id}`                                                                      |          |
| U12  | `xr conversations start <USER>... --text`                               | `POST /2/dm_conversations`                                                                     |          |
| U12  | `xr conversations send <CONVERSATION> <TEXT>`                           | `POST /2/dm_conversations/{id}/messages`                                                       |          |
| U12  | `xr conversations with <USER>`                                          | `GET /2/dm_conversations/with/{participant_id}/dm_events`                                      |          |
| U12  | `xr conversations history <CONVERSATION>`                               | `GET /2/dm_conversations/{id}/dm_events`                                                       |          |
| U12  | `xr dm-events get <EVENT>`                                              | `GET /2/dm_events/{event_id}`                                                                  |          |
| U12  | `xr dm-events remove <EVENT>`                                           | `DELETE /2/dm_events/{event_id}`                                                               | yes      |
| U12  | `xr dm-block <USER>`, `xr dm-unblock <USER>`                            | `POST /2/users/{id}/dm/block`, `/dm/unblock`                                                   |          |
| U13  | `xr lists list`, `followed`, `memberships`, `pinned`                    | `GET /2/users/{id}/owned_lists`, `/followed_lists`, `/list_memberships`, `/pinned_lists`       |          |
| U13  | `xr lists get <LIST>`, `posts`, `followers`                             | `GET /2/lists/{id}`, `/tweets`, `/followers`                                                   |          |
| U13  | `xr lists add <NAME>`, `update <LIST>`                                  | `POST /2/lists`, `PUT /2/lists/{id}`                                                           |          |
| U13  | `xr lists remove <LIST>`                                                | `DELETE /2/lists/{id}`                                                                         | yes      |
| U13  | `xr lists members list`, `add`, `remove`                                | `GET`, `POST /2/lists/{id}/members`, `DELETE .../members/{user_id}`                            |          |
| U13  | `xr lists follow`, `unfollow`, `pin`, `unpin`                           | `POST`/`DELETE /2/users/{id}/followed_lists`, `/pinned_lists`                                  |          |
| U14  | `xr broadcasts list`, `get <ID>`                                        | `GET /2/broadcasts`, `/2/broadcasts/{id}`                                                      |          |
| U14  | `xr broadcasts scheduled list`, `get`, `add`, `update`                  | `GET`/`POST /2/broadcasts/scheduled`, `GET`/`PUT .../{id}`                                     |          |
| U14  | `xr broadcasts scheduled remove`, `live`                                | `DELETE .../scheduled/{id}`, `POST .../scheduled/{id}/live`                                    | yes      |
| U14  | `xr broadcasts chat history`, `send`, `mute`, `unmute`                  | `GET`/`POST /2/broadcasts/{id}/chat`, `POST .../chat/mutes`, `DELETE .../chat/mutes/{user_id}` |          |
| U14  | `xr broadcasts chat remove <BROADCAST> <MESSAGE>`                       | `DELETE /2/broadcasts/{id}/chat/{message_id}`                                                  | yes      |
| U15  | `xr notes write`, `evaluate`, `eligible`, `written`                     | `POST /2/notes`, `POST /2/notes/evaluate`, `GET /2/notes/search/*`                             |          |
| U15  | `xr notes remove <NOTE>`                                                | `DELETE /2/notes/{id}`                                                                         | yes      |
| U16  | `xr articles draft --title --body-file`                                 | `POST /2/articles/draft`                                                                       |          |
| U16  | `xr articles publish <ARTICLE>`                                         | `POST /2/articles/{article_id}/publish`                                                        | yes      |
| U17  | `xr users <ID or @HANDLE>...`                                           | `GET /2/users?ids=` or `GET /2/users/by?usernames=` (OQ1)                                      |          |
| U18  | `xr media get <KEY>...`, `analytics <KEY>... --start --end`             | `GET /2/media?media_keys=` (OQ1), `GET /2/media/analytics`                                     |          |
| U19  | `xr spaces search`, `get <ID>...`, `by-creator <USER>...`, `posts <ID>` | `GET /2/spaces/search`, `/2/spaces?ids=` (OQ1), `/by/creator_ids`, `/{id}/tweets`              |          |
| U20  | `xr stream rules list`, `add`, `counts`                                 | `GET`/`POST /2/tweets/search/stream/rules`, `GET .../rules/counts` (OQ2)                       |          |
| U20  | `xr stream rules remove <ID>...`                                        | `POST /2/tweets/search/stream/rules` (`delete`)                                                | yes      |

### Assumptions

Unconfirmed bets, made without a scoping confirmation; each is cheap to change in its unit's PR before it merges.

- The command words in the Command Surface table that no `reason` names are proposals for Brett's review: `likers`,
  `reposters`, `quotes`, `read-posts`, `user-posts`, `search-users`, `search-counts`, `analytics`, `hide`, `unhide`,
  `conversations`, `dm-events`, `dm-block`, `dm-unblock`, `trends place` and `trends personal`, and the `lists` verbs.
- `xr spaces by-creator` and `xr conversations start` take handles and resolve them with one `GET /2/users/by` request,
  as `xr dm` resolves its one recipient.
- `xr broadcasts scheduled update` requires `--start` and `--end`, because X requires both on every update; it does not
  read the current values first.
- Times on the command line are RFC 3339; the broadcasts endpoints' millisecond strings are produced from them in the
  library.
- `xr stream rules add` takes one rule per call with an optional `--tag`; adding several rules at once is a raw request.

### Sequencing

Units build in Unit Index order. The order puts the smallest families first so the stack's early PRs can merge while
later ones are built, puts the units gated on OQ1 and OQ2 last so the answers can arrive meanwhile, and builds U8 before
U18 because U18 reuses U8's time-window check.

---

## Implementation Units

| U-ID | Family           | Main files beyond the recipe                             | Depends on |
| ---- | ---------------- | -------------------------------------------------------- | ---------- |
| U1   | me-follow-lists  | `crates/xurl-cli/src/cli/commands/graph.rs`, `AGENTS.md` | none       |
| U2   | hidden-replies   | `crates/xurl-cli/src/cli/commands/engagement.rs`         | U1         |
| U3   | post-engagement  | `crates/xurl-cli/src/cli/commands/reads.rs`              | U2         |
| U4   | posts-lookup     | `crates/xurl-cli/src/cli/commands/reads.rs`              | U3         |
| U5   | user-posts       | `crates/xurl-cli/src/cli/commands/reads.rs`              | U4         |
| U6   | users-search     | `crates/xurl-cli/src/cli/commands/reads.rs`              | U5         |
| U7   | post-counts      | `crates/xurl-cli/src/cli/commands/reads.rs`              | U6         |
| U8   | post-analytics   | `crates/xurl-cli/src/cli/commands/reads.rs`              | U7         |
| U9   | trends           | `crates/xurl-cli/src/cli/commands/trends.rs` (new)       | U8         |
| U10  | news             | `crates/xurl-cli/src/cli/commands/news.rs` (new)         | U9         |
| U11  | communities      | `crates/xurl-cli/src/cli/commands/communities.rs` (new)  | U10        |
| U12  | dm-conversations | `crates/xurl-cli/src/cli/commands/dms.rs`                | U11        |
| U13  | lists            | `crates/xurl-cli/src/cli/commands/lists.rs` (new)        | U12        |
| U14  | broadcasts       | `crates/xurl-cli/src/cli/commands/broadcasts.rs`         | U13        |
| U15  | community-notes  | `crates/xurl-cli/src/cli/commands/notes.rs` (new)        | U14        |
| U16  | articles         | `crates/xurl-cli/src/cli/commands/articles.rs` (new)     | U15        |
| U17  | users-lookup     | `crates/xurl-cli/src/cli/commands/reads.rs`              | U16, OQ1   |
| U18  | media-lookup     | `crates/xurl-cli/src/cli/commands/media.rs`              | U17, OQ1   |
| U19  | spaces           | `crates/xurl-cli/src/cli/commands/spaces.rs` (new)       | U18, OQ1   |
| U20  | stream-rules     | `crates/xurl-cli/src/cli/commands/stream_rules.rs` (new) | U19, OQ2   |

**The recipe, in every unit.** Each unit touches the surfaces AGENTS.md § "Adding a command family" lists, for each of
its operations: a `SHORTCUT_TEMPLATES` row, a shortcut, a typed response (KTD6), a fixture with its `spec_*` test and
`FIXTURE_ENDPOINTS` row (KTD7), a mock route, the clap command with its help fixture, the dispatch arm, the schema
registry row and regenerated schema, the validate alias, the examples line, and the completions. Each also removes its
entry from `endpoint-decisions.json`, adds its commands to the KNOWN_DIFFERENCES list of commands Go lacks, and
documents them in `crates/xurl-cli/README.md`. Each unit's **Files** field names only what the recipe does not imply,
and each unit's tests live in a family test file under `crates/xurl-cli/tests/` (named in the unit) plus the golden
cases its validators need. The walks the recipe names, the `endpoint_decisions` tests, and `dry_run_guard` check every
unit without new test code; the scenarios below are the family's own behavior.

### U1. me-follow-lists

- **Goal:** `xr followers` and `xr following` without `--of` read the caller's own lists from `/2/users/me/*`, with no
  id lookup first.
- **Requirements:** R1, R2 (KD3), R4.
- **Dependencies:** none.
- **Files:** `crates/xurl-cli/src/cli/commands/graph.rs`, `AGENTS.md` § "Command grammar: flags vs subcommands",
  `crates/xurl-cli/tests/cli_tests.rs` (beside the existing `followers` tests).
- **Approach:** without `--of`, the two commands call new shortcuts over the `/me` paths and skip `resolve_my_user_id`;
  with `--of` they keep today's path. AGENTS.md's grammar gains KD3's exception in one sentence.
- **Patterns to follow:** the existing `followers --of` arm and `list_for_user`.
- **Test scenarios:**
  - `xr followers` against the mock requests `GET /2/users/me/followers` and no `/2/users/me` lookup.
  - `xr following --of @jack` still requests the id lookup and then `GET /2/users/{id}/following`.
  - `xr --dry-run followers` names the OAuth2 credential and sends nothing.
  - Pagination: `--cursor T` reaches the `/me` path as `pagination_token=T`.
- **Verification:** the `me-follow-lists` entry is gone and `endpoint_decisions` passes; the two help fixtures move only
  where the text describes the endpoint.

### U2. hidden-replies

- **Goal:** `xr hide <REPLY>` and `xr unhide <REPLY>` hide or show a reply to the caller's post.
- **Requirements:** R1, R4, R5 (not gated, KTD5).
- **Dependencies:** U1.
- **Files:** `crates/xurl-cli/src/cli/commands/engagement.rs`, `crates/xurl-cli/tests/hidden_replies_tests.rs`.
- **Approach:** one shortcut takes the reply id and the hidden flag; the two commands differ only in the body. A post
  URL resolves to its id as other post commands do.
- **Patterns to follow:** `like` and `unlike` (`act_from_me_on_post` is not needed: the path holds the reply, not the
  caller).
- **Test scenarios:**
  - `xr hide 123` sends `PUT /2/tweets/123/hidden` with `{"hidden":true}` and prints `{"data":{"hidden":true}}`.
  - `xr unhide https://x.com/a/status/123` sends `{"hidden":false}` to the same path.
  - `xr hide ""` is refused offline with `empty-post-id`, and its dry run says so.
- **Verification:** both help fixtures exist; the dry-run golden for an empty id moves into place.

### U3. post-engagement

- **Goal:** `xr likers`, `xr reposters`, and `xr quotes` list who liked or reposted a post and which posts quote it.
- **Requirements:** R1, R3, R4.
- **Dependencies:** U2.
- **Files:** `crates/xurl-cli/src/cli/commands/reads.rs`, `crates/xurl-cli/tests/post_engagement_tests.rs`.
- **Approach:** three paged reads; `likers` and `reposters` answer `User` lists, `quotes` a `Post` list (KTD6). `-n` and
  `--cursor` behave as on `followers`.
- **Patterns to follow:** `followers` for paged user lists; `search` for paged post lists.
- **Test scenarios:**
  - `xr likers 123 -n 50` requests `GET /2/tweets/123/liking_users?max_results=50` and prints the users.
  - `xr quotes 123 --cursor T` sends `pagination_token=T`.
  - With only `XURL_BEARER_TOKEN`, `xr reposters 123` goes out with the bearer token, and `xr likers 123` is refused
    with `auth-method-mismatch`, since `liking_users` takes no app token.
- **Verification:** three help fixtures; `xr examples` invokes one of them.

### U4. posts-lookup

- **Goal:** `xr read-posts <POST>...` reads up to 100 posts by id in one request. The word sits beside `read` and away
  from `post`, since `xr post 123` publishes a post.
- **Requirements:** R1, R3, R6 (KTD3, KTD4).
- **Dependencies:** U3.
- **Files:** `crates/xurl-cli/src/cli/commands/reads.rs`, `crates/xurl-cli/tests/posts_lookup_tests.rs`.
- **Approach:** introduces KTD4's batch-size check. The response keeps X's `errors` array, where X reports ids it could
  not return beside the posts it could.
- **Patterns to follow:** `read` for a post id or URL per argument.
- **Test scenarios:**
  - `xr read-posts 1 2 3` requests `GET /2/tweets?ids=1,2,3` and prints three posts.
  - 101 ids are refused offline with the batch-size code; 100 are accepted.
  - A mock answer carrying both `data` and `errors` prints both, at exit 0.
- **Verification:** the batch-size code has a sentence and a dry-run golden.

### U5. user-posts

- **Goal:** `xr user-posts <USER>` lists a user's own posts.
- **Requirements:** R1, R3, R4.
- **Dependencies:** U4.
- **Files:** `crates/xurl-cli/src/cli/commands/reads.rs`, `crates/xurl-cli/tests/user_posts_tests.rs`.
- **Approach:** a paged read through `list_for_user` with the named user, so the handle resolves once and the dry run
  names both requests' credential.
- **Patterns to follow:** `followers --of`.
- **Test scenarios:**
  - `xr user-posts @jack` requests the handle lookup, then `GET /2/users/{id}/tweets`.
  - `xr user-posts ""` is refused offline with `empty-username`.
  - `xr --dry-run user-posts @jack` sends nothing and reports the credential.
- **Verification:** the help fixture and the examples line exist.

### U6. users-search

- **Goal:** `xr search-users <QUERY>` finds accounts by keyword.
- **Requirements:** R1, R3.
- **Dependencies:** U5.
- **Files:** `crates/xurl-cli/src/cli/commands/reads.rs`, `crates/xurl-cli/tests/users_search_tests.rs`.
- **Approach:** a paged read answering a `User` list; the endpoint pages with `next_token`, which `--cursor` fills.
- **Patterns to follow:** `search`.
- **Test scenarios:**
  - `xr search-users rust -n 20` requests `GET /2/users/search?query=rust&max_results=20`.
  - With only a bearer token, the command is refused with `auth-method-mismatch` (the endpoint takes user context only).
  - An empty query is refused offline.
- **Verification:** the help fixture exists.

### U7. post-counts

- **Goal:** `xr search-counts <QUERY>` reports how many recent posts match a query per bucket, before a search is paid
  for.
- **Requirements:** R1, R3, R6.
- **Dependencies:** U6.
- **Files:** `crates/xurl-cli/src/cli/commands/reads.rs`, `crates/xurl-cli/tests/post_counts_tests.rs`.
- **Approach:** `--granularity` takes the spec's enum (`minute`, `hour`, `day`); optional `--start` and `--end` use
  KTD4's window check when both are given. A new `PostCounts` response carries the buckets and `meta.total_tweet_count`.
- **Patterns to follow:** `search` for the query argument.
- **Test scenarios:**
  - `xr search-counts "from:jack" --granularity day` requests
    `GET /2/tweets/counts/recent?query=from:jack&granularity=day`.
  - `--granularity week` is refused by clap with the accepted values listed.
  - The bearer token alone serves it.
- **Verification:** the response schema lists the bucket fields.

### U8. post-analytics

- **Goal:** `xr analytics <POST>... --start <TIME> --end <TIME>` reports impressions and engagement for the caller's
  posts.
- **Requirements:** R1, R3, R6 (KTD4).
- **Dependencies:** U7.
- **Files:** `crates/xurl-cli/src/cli/commands/reads.rs`, `crates/xurl-cli/tests/post_analytics_tests.rs`.
- **Approach:** introduces the time-window check KTD4 names: both bounds RFC 3339, start before end. `--granularity`
  takes `hourly`, `daily`, `weekly`, or `total`. A new response type follows the spec's analytics schema.
- **Patterns to follow:** `calls scheduled add --start` for RFC 3339 input.
- **Test scenarios:**
  - `xr analytics 1 2 --start 2026-10-01T00:00:00Z --end 2026-10-02T00:00:00Z` requests both ids with the two times.
  - An end before the start, and a time without a zone, are refused offline with the window check's code.
  - Omitting `--start` is refused by clap.
- **Verification:** the window code has a sentence; U18 reuses the check unchanged.

### U9. trends

- **Goal:** `xr trends place <WOEID>` and `xr trends personal` list trends for a place and the caller's personalized
  trends.
- **Requirements:** R1, R3, R6.
- **Dependencies:** U8.
- **Files:** `crates/xurl-cli/src/cli/commands/trends.rs` (new), `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/trends_tests.rs`.
- **Approach:** a family of two verbs; `place` takes `--max` for `max_trends`. Two response types, since the two
  endpoints describe a trend differently.
- **Patterns to follow:** `bookmark-folders` for a small family.
- **Test scenarios:**
  - `xr trends place 1 --max 10` requests `GET /2/trends/by/woeid/1?max_trends=10`.
  - A WOEID that is not a positive integer is refused offline.
  - `xr trends personal` with only a bearer token is refused with `auth-method-mismatch`.
- **Verification:** the family's help and both verbs' help fixtures exist.

### U10. news

- **Goal:** `xr news search <QUERY>` and `xr news get <ID>` search news stories and read one.
- **Requirements:** R1, R3.
- **Dependencies:** U9.
- **Files:** `crates/xurl-cli/src/cli/commands/news.rs` (new), `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/news_tests.rs`.
- **Approach:** `search` takes `-n` and `--max-age-hours`; a `NewsStory` response serves both verbs.
- **Patterns to follow:** U9.
- **Test scenarios:**
  - `xr news search ai --max-age-hours 24` requests both parameters.
  - `xr news get 123` requests `GET /2/news/123`.
  - An id the spec's pattern rejects is refused offline.
- **Verification:** help fixtures for the family and both verbs.

### U11. communities

- **Goal:** `xr communities search <QUERY>` and `xr communities get <ID>` search communities and read one.
- **Requirements:** R1, R3.
- **Dependencies:** U10.
- **Files:** `crates/xurl-cli/src/cli/commands/communities.rs` (new), `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/communities_tests.rs`.
- **Approach:** `search` takes user context only and `get` also takes a bearer token, which the auth matrix handles
  (KTD8). A `Community` response serves both.
- **Patterns to follow:** U10.
- **Test scenarios:**
  - `xr communities search rust` requests `GET /2/communities/search?query=rust`.
  - `xr communities get 123` with only a bearer token goes out with it; `search` with only a bearer token is refused.
  - An id the spec's pattern rejects is refused offline.
- **Verification:** help fixtures for the family and both verbs.

### U12. dm-conversations

- **Goal:** start a group conversation, send into a conversation, read one conversation's history, read or delete one
  message, and block or unblock a user's direct messages.
- **Requirements:** R1, R3, R5, R6.
- **Dependencies:** U11.
- **Files:** `crates/xurl-cli/src/cli/commands/dms.rs`, `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/dm_conversations_tests.rs`.
- **Approach:**
  1. `xr conversations` (`start`, `send`, `with`, `history`) over `/2/dm_conversations/*`; `start` resolves its handles
     with one lookup and sends `conversation_type: "Group"` with at most 49 participants, checked offline.
  2. `xr dm-events` (`get`, `remove`) over `/2/dm_events/{id}`; `remove` confirms (KTD5).
  3. `xr dm-block` and `xr dm-unblock` resolve the user and put that user's id in the path (`act_on_user`), since the
     path names whom to block.
- **Patterns to follow:** `dm` for text and recipients; `validate_dm_body`.
- **Test scenarios:**
  - `xr conversations start @a @b --text hi` requests one handle lookup, then `POST /2/dm_conversations` with both ids
    and the message.
  - 50 participants are refused offline.
  - `xr conversations history 123-456` requests `GET /2/dm_conversations/123-456/dm_events`.
  - `xr dm-events remove 9 --no-interactive` without `--force` exits 1 with `confirmation-required`; with `--force` it
    sends the `DELETE`.
  - `xr dm-block @a` resolves the handle and sends `POST /2/users/{id}/dm/block`.
- **Verification:** the families' help fixtures and the dry-run goldens for the empty-text and participant-count codes
  exist.

### U13. lists

- **Goal:** an `xr lists` family: create, update, and delete lists; list, add, and remove members; a list's posts and
  followers; the caller's owned, followed, and pinned lists and memberships; follow, unfollow, pin, and unpin.
- **Requirements:** R1, R3, R5, R6.
- **Dependencies:** U12.
- **Files:** `crates/xurl-cli/src/cli/commands/lists.rs` (new), `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/lists_tests.rs`.
- **Approach:**
  1. Reads: `list`, `followed`, `memberships`, `pinned`, `get`, `posts`, `followers`, and `members list`.
  2. Writes: `add <NAME> [--description] [--private]`, `update <LIST> [--name] [--description] [--private|--public]`,
     `remove <LIST>` (confirms), `members add|remove <LIST> <USER>`, `follow`, `unfollow`, `pin`, `unpin`.
  3. Offline checks from the spec: name at most 25 characters, description at most 100, a list id's pattern, and an
     `update` that changes nothing refused like `no-call-changes`.
  4. A `List` response serves the reads; the writes answer X's small result objects.
- **Patterns to follow:** `calls scheduled` for nested verbs and a change-nothing refusal; `broadcasts moderators` for
  `members`; `act_from_me` for follow and pin.
- **Test scenarios:**
  - `xr lists add Rustaceans --private` sends `POST /2/lists` with `name` and `private: true`.
  - A 26-character name is refused offline with its code.
  - `xr lists update 1` with no field is refused offline.
  - `xr lists remove 1 --no-interactive` asks for confirmation; `--force` sends `DELETE /2/lists/1`.
  - `xr lists members add 1 @jack` resolves the handle and sends `{"user_id": ...}` to `/2/lists/1/members`.
  - `xr lists follow 1` sends `POST /2/users/{me}/followed_lists` with `list_id`.
  - `xr lists list` requests the caller's `/2/users/{id}/owned_lists`, using the stored account id.
- **Verification:** help fixtures for the family, its verbs, and `members`; the largest unit, so its PR description
  lists every verb.

### U14. broadcasts

- **Goal:** extend `xr broadcasts` with the caller's broadcasts, scheduled broadcasts, and broadcast chat.
- **Requirements:** R1, R3, R5, R6.
- **Dependencies:** U13.
- **Files:** `crates/xurl-cli/src/cli/commands/broadcasts.rs`, `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/broadcasts_tests.rs`.
- **Approach:**
  1. `list` and `get <ID>` at the family's top.
  2. `scheduled` (`list`, `get`, `add --source --start [--end] [--title] [--description] ...`,
     `update <ID> --start --end ...`, `remove` and `live`, both confirming).
  3. `chat` (`history`, `send <ID> <TEXT> [--reply-to]`, `remove <ID> <MESSAGE>` confirming,
     `mute <ID> <USER> [--until] [--message]`, `unmute`).
  4. RFC 3339 times become the millisecond strings X takes, in the library; chat text is at most 140 characters, checked
     offline.
- **Patterns to follow:** `broadcasts sources` for the source id; `calls scheduled` for times and confirmations.
- **Test scenarios:**
  - `xr broadcasts scheduled add --source S --start 2026-11-02T16:00:00Z` sends `source_id` and `scheduled_start_ms` as
    milliseconds.
  - `xr broadcasts scheduled live 1 --no-interactive` asks for confirmation; `--force` sends
    `POST .../scheduled/1/live`.
  - A 141-character chat message is refused offline.
  - `xr broadcasts chat mute 1 @a --until 2026-11-02T17:00:00Z` resolves the handle and sends `user_id` and `end_at_ms`.
- **Verification:** help fixtures for `scheduled`, `chat`, and every verb.

### U15. community-notes

- **Goal:** an `xr notes` family over X's AI Note Writer API: write, evaluate, and delete notes, find posts eligible for
  one, and list the notes the caller wrote.
- **Requirements:** R1, R3, R5, R7.
- **Dependencies:** U14.
- **Files:** `crates/xurl-cli/src/cli/commands/notes.rs` (new), `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/notes_tests.rs`.
- **Approach:** verbs per KTD10.
  `write <POST> --text --classification [--tag ...] [--trustworthy-sources] [--media-note] [--test]` takes the spec's
  enums; `remove` confirms. An app not enrolled in the program gets X's 403, which carries the existing `enroll-app`
  step (R7).
- **Patterns to follow:** `calls scheduled add` for enum flags.
- **Test scenarios:**
  - `xr notes write 1 --text t --classification not_misleading --test` sends `test_mode: true` and the `info` object.
  - `xr notes eligible` sends `test_mode=false` by default.
  - A mock 403 naming enrollment prints `forbidden` with the `enroll-app` step.
  - `xr notes remove 1 --no-interactive` asks for confirmation.
- **Verification:** the family's help fixtures exist.

### U16. articles

- **Goal:** `xr articles draft` and `xr articles publish` write and publish long-form X Articles.
- **Requirements:** R1, R3, R5.
- **Dependencies:** U15.
- **Files:** `crates/xurl-cli/src/cli/commands/articles.rs` (new), `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/articles_tests.rs`, and the conversion's unit tests in the library (KTD9).
- **Approach:** `draft --title <TITLE> --body-file <PATH|-> [--cover <MEDIA_ID>]` converts the text per KTD9;
  `publish <ARTICLE>` confirms (KTD5).
- **Patterns to follow:** `media upload` for reading a file; `webhooks remove` for a confirmed action.
- **Test scenarios:**
  - A body of a heading, a paragraph, and two list items becomes four blocks of types `header-one`, `unstyled`, and
    `unordered-list-item`.
  - `--body-file -` reads stdin.
  - An empty title is refused offline.
  - `xr articles publish 1 --no-interactive` asks for confirmation; `--force` sends `POST /2/articles/1/publish`.
- **Verification:** the library conversion has unit tests for every block type; help fixtures exist.

### U17. users-lookup

- **Goal:** `xr users <ID or @HANDLE>...` looks up 1 to 100 users by id or by handle in one request.
- **Requirements:** R1, R3, R6 (KTD3).
- **Dependencies:** U16, and OQ1's answer.
- **Files:** `crates/xurl-cli/src/cli/commands/reads.rs`, `crates/xurl-cli/tests/users_lookup_tests.rs`,
  `endpoint-decisions.json` (the `GET /2/users/{id}` entry, per OQ1).
- **Approach:** all-digit arguments go to `/2/users?ids=`, `@` arguments to `/2/users/by?usernames=`; a mix is refused
  offline.
- **Patterns to follow:** U4.
- **Test scenarios:**
  - `xr users 1 2` requests `GET /2/users?ids=1,2`.
  - `xr users @a @b` requests `GET /2/users/by?usernames=a,b`.
  - `xr users 1 @a` is refused offline with a code saying ids and handles do not mix.
- **Verification:** per OQ1, the single-id path has a command or a `raw-only` decision, and `endpoint_decisions` passes.

### U18. media-lookup

- **Goal:** `xr media get <KEY>...` reads uploads by media key, and `xr media analytics` reports their metrics.
- **Requirements:** R1, R3, R6.
- **Dependencies:** U17, U8 (the window check), and OQ1's answer.
- **Files:** `crates/xurl-cli/src/cli/commands/media.rs`, `crates/xurl-cli/tests/media_lookup_tests.rs`,
  `endpoint-decisions.json` (the `GET /2/media/{media_key}` entry, per OQ1).
- **Approach:** `get` takes 1 to 100 media keys (`<number>_<number>`), checked offline; `analytics` takes keys,
  `--start`, `--end`, and `--granularity` (`hourly`, `daily`, `total`).
- **Patterns to follow:** U8 and U4.
- **Test scenarios:**
  - `xr media get 3_1 3_2` requests `GET /2/media?media_keys=3_1,3_2`.
  - A key without the underscore form is refused offline.
  - `xr media analytics 3_1 --start ... --end ...` reuses U8's window check.
- **Verification:** the `media` family help lists the new verbs; per OQ1 the single-key path is decided.

### U19. spaces

- **Goal:** an `xr spaces` family for discovery: search, by id, by creator, and a Space's posts.
- **Requirements:** R1, R3, R6.
- **Dependencies:** U18, and OQ1's answer.
- **Files:** `crates/xurl-cli/src/cli/commands/spaces.rs` (new), `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/spaces_tests.rs`, `endpoint-decisions.json` (the `GET /2/spaces/{id}` entry, per OQ1).
- **Approach:** `search <QUERY> [--state live|scheduled|all]`, `get <ID>...`, `by-creator <USER>...` (handles resolved
  in one lookup), and `posts <ID>`; a `Space` response serves the first three.
- **Patterns to follow:** U17 for batches and handles.
- **Test scenarios:**
  - `xr spaces search rust --state live` sends both parameters.
  - `xr spaces by-creator @a @b` requests one handle lookup, then `/2/spaces/by/creator_ids?user_ids=...`.
  - `xr spaces posts 1` requests `GET /2/spaces/1/tweets`.
- **Verification:** help fixtures for the family and its verbs.

### U20. stream-rules

- **Goal:** `xr stream rules` lists, adds, removes, and counts the filtered stream's rules.
- **Requirements:** R1, R3, R4, R5.
- **Dependencies:** U19, and OQ2's answer.
- **Files:** `crates/xurl-cli/src/cli/commands/stream_rules.rs` (new), `crates/xurl-cli/src/cli/family_help.rs`,
  `crates/xurl-cli/tests/stream_rules_tests.rs`.
- **Approach:** a `stream` family with a `rules` subfamily. `add <VALUE> [--tag]` and `remove <ID>...` both post to the
  rules endpoint, with `add` and `delete` bodies; `remove` confirms. Per OQ2, `--validate-only` on `add` and `remove`
  sends `dry_run=true`, a removal sent that way skips the confirmation, and `--dry-run` sends nothing. Every request
  goes out with the bearer token (KTD8).
- **Patterns to follow:** `webhooks stream-links` for the app-auth stream surface.
- **Test scenarios:**
  - `xr stream rules add "from:jack" --tag j` sends `{"add":[{"value":"from:jack","tag":"j"}]}`.
  - `xr stream rules remove 1 2 --force` sends `{"delete":{"ids":["1","2"]}}`.
  - With only an OAuth2 login and no bearer token, the commands are refused with `auth-method-mismatch` and its step.
  - `xr --dry-run stream rules add x` sends nothing.
- **Verification:** help fixtures for `stream`, `stream rules`, and every verb.

---

## Verification Contract

| Gate                    | When             | Command                                                                                                                                                                                                                                                                                    | Proves                              |
| ----------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------- |
| Family tests            | each unit        | `cargo test -p xurl-rs --test <family>_tests` and the library's spec and mock tests                                                                                                                                                                                                        | the family's behavior               |
| Recipe walks            | each unit        | `cargo test -p xdk-rs --test endpoint_decisions --test mock_endpoint_coverage --test spec_validation --test spec_types_validation` and `cargo test -p xurl-rs --test schema_tests --test golden_tests --test dispatch_guard --test dry_run_guard --test agentic_tests --test recipe_guard` | every surface is in place           |
| Goldens                 | each unit        | `XURL_GOLDEN_BLESS=1 cargo test -p xurl-rs --test golden_tests`, then a review that only the family's fixtures and the root help moved                                                                                                                                                     | no other output moved               |
| Schemas and completions | each unit        | `scripts/generate-response-schemas.sh`, `scripts/generate-completions.sh`                                                                                                                                                                                                                  | committed artifacts match the build |
| Full suite and lint     | before each push | `cargo test --workspace --no-fail-fast`, `cargo clippy --workspace --all-targets -- -D warnings`                                                                                                                                                                                           | nothing else broke                  |
| Local CI mirror         | each push        | `scripts/hooks/pre-push` (the hook)                                                                                                                                                                                                                                                        | the gates CI runs                   |
| CI                      | each PR          | the rollup read, every check `SUCCESS` or a conditional `SKIPPED`                                                                                                                                                                                                                          | the PR is mergeable                 |

No gate sends a request to X; a manual check runs a debug `xr` with `XURL_TOKEN_STORE` at a scratch path and
`API_BASE_URL` at the mock or a closed port.

## Definition of Done

- Each unit: its commands answer as its scenarios say against the mock; its `endpoint-decisions.json` entry is gone; its
  validator codes have sentences; README, KNOWN_DIFFERENCES, examples, schemas, completions, and help fixtures are
  current; its PR's `## Changelog (xurl-rs)` lists the commands under Added, and `## Changelog (xdk-rs)` lists the new
  shortcuts and response types under Added.
- The plan: every `planned` entry is gone from `endpoint-decisions.json`, or, for OQ1's paths, replaced by the decision
  Brett chose.
- No code from an abandoned approach remains in any PR's diff.
