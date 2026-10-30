# Changelog

All notable changes to this project will be documented in this file.

## [4.5.0] - 2026-10-30

### Added

- Add `xr broadcasts mutes list | add | remove`: mute someone in every chat of your broadcasts, by handle or user ID, until `--until` or for good, and see who is muted and until when. A moderator mutes in another host's chats with `--broadcast`. by @brettdavies in [#409](https://github.com/brettdavies/xurl-rs/pull/409)
- Add `xr broadcasts remove`: delete a scheduled or ended broadcast of yours with its replay. It asks first; `--force` skips the prompt.
- Add `xr broadcasts sources region`, which names the ingest region to pass to `xr broadcasts sources add --region`.
- Add `message` and `next_step` to the `--dry-run` envelope of a command that refuses its inputs, under every structured format: `message` is the rule the input broke, as text output words it, and `next_step` is the step the real run's error names. by @brettdavies in [#437](https://github.com/brettdavies/xurl-rs/pull/437)
- Add numeric user IDs to every command that names one account, and to `--of` on `following`, `followers`, `lists list`, `lists followed`, and `lists memberships`: each takes a handle (`@jack` or `jack`) or a numeric user ID. An argument made only of digits is now a user ID, sent with no lookup; prefix an all-digit handle with `@` to keep the old meaning (`xr follow @1234`). by @brettdavies in [#439](https://github.com/brettdavies/xurl-rs/pull/439)
- Add handles to `xr calls scheduled add --invite`, `xr calls scheduled update --invite`, and `xr webhooks subscriptions remove`: each account is a handle or a numeric user ID, in any mix, and every handle is looked up in one request before the command's own.
- Add the auth schemes each command accepts to its `--auth` help, such as `this command accepts: app` on `xr webhooks replay`, with the set `--of` allows where it differs. by @brettdavies in [#445](https://github.com/brettdavies/xurl-rs/pull/445)
- Add `xr broadcasts scheduled cancel`, which replaces `xr broadcasts scheduled remove` and matches `xr calls scheduled cancel`. The old word exits 2 as an unknown command whose `suggestion` and help pointer name `cancel`. Remove `xr broadcasts scheduled live` and `--manual-publish` on `add` and `update`: X takes every schedule `xr` makes live at its start time. A schedule already created for a manual start goes live through raw mode, `xr -X POST /2/broadcasts/scheduled/<ID>/live`. by @brettdavies in [#447](https://github.com/brettdavies/xurl-rs/pull/447)
- Add `rule` to every validation refusal's JSON envelope, in the real run and the dry run: the validator's kebab-case code for the input rule that failed, such as `invalid-username`. A real run's `message` on a validation refusal changes from that code to the sentence for the rule, and a dry run's `reason` changes from the code to `validation`, the `reason` its real run carries. by @brettdavies in [#449](https://github.com/brettdavies/xurl-rs/pull/449)
- Add `value` to a validation refusal's JSON envelope, the offending input as it was given, when the rule names one: an app name, a redirect URI, an unknown scope, a missing program, or a `-H` header's position. Every validation refusal now names its rule: refusals that carried prose, such as `xr auth clear` with nothing to clear or a malformed `-H` header, carry `rule` and the sentence for it in `message`. by @brettdavies in [#450](https://github.com/brettdavies/xurl-rs/pull/450)
- Add `account` to the `not-found` envelope of a handle X knows no account for, where the refusal was `validation` with the handle in `message`.
- Add `media_id` to the refusal of a media job X failed after processing it, which is now `invalid-request` with X's detail in `message`, where it was `validation` with the message "media processing failed".
- Change a 200 X answers with `errors` and no `data` from `validation` to the reason its problem type names: `not-found` (exit 4), `forbidden`, or `invalid-request`.
- Change raw mode's refusal of a URL that is neither absolute nor an absolute path from `validation` to `invalid-url`, and of a missing URL from `validation` (exit 1) to `invalid-args` (exit 2). A failed confirmation or picker prompt is now `io`, not `validation`.
- Add command words to `xr schema`: `xr schema webhooks subscriptions count` prints that command's schema. A hyphen-joined name such as `webhooks-subscriptions-count` is now refused at exit 2 with `invalid-args`, and its `fix-input` step carries the words form to run instead. by @brettdavies in [#451](https://github.com/brettdavies/xurl-rs/pull/451)
- Add command words to `xr schema --list` and `xr schema --all`. `--list` has no `envelope` row, and in JSON, YAML, CSV, and TSV it prints one record per command with `command`, `type`, and `file`.
- Add the `unknown-command` refusal at exit 2 to `xr schema`, as `xr help` gives it, for a word that names no command; it was `validation` at exit 1.
- Add the refusal of a bare `xr schema envelope` at exit 2 with a `fix-input` step carrying `xr schema --envelope`, the form that works.
- Add the replacement of a retired command word to `xr schema`'s refusal: `xr schema broadcasts scheduled remove` suggests `cancel`, and `xr schema broadcasts-scheduled-remove` refuses with a `fix-input` step carrying `xr schema broadcasts scheduled cancel`.
- Add the commands beneath a family to its `no-typed-response` refusal: `xr schema webhooks` names `webhooks list`, `webhooks add`, and the rest.
- Add the full command path to the names of two committed schema files: `schema/responses/redirect-uri-get.schema.json` and `redirect-uri-set.schema.json` are now `auth-apps-redirect-uri-get.schema.json` and `auth-apps-redirect-uri-set.schema.json`.
- Add the resolved `data` view as the default of `xr schema <command>`: the schema of the command's `data`, every reference resolved, with `command`, `data_cardinality`, `data_type`, `common_schema`, and `data_nullable` labels. The complete schema the command printed before is now `xr schema <command> --full`. by @brettdavies in [#452](https://github.com/brettdavies/xurl-rs/pull/452)
- Add `xr schema --common` and `xr schema --path <DOT_PATH>`. `xr schema --all` now prints every command's default view; the complete schemas stay under `schema/responses/`.
- Add the `path` and `known_properties` keys to the error envelope, on an `xr schema --path` segment that names no property, with a `fix-input` template.
- Add the refusal of combined scopes such as `xr schema --list --all` at exit 2 with `invalid-args`; before, one flag silently won.
- Add a `Response schema:` block to the help of every command with a typed response, naming the `xr schema` invocation that describes its output. by @brettdavies in [#453](https://github.com/brettdavies/xurl-rs/pull/453)
- Add `response_schema` to the dry-run envelope of every command with a typed response, and a `response schema:` line to its text summary.
- Add `xr broadcasts sources update <ID> --name <NAME>`, which replaces `xr broadcasts sources rename <ID> <NAME>` and takes the label as a flag, as `xr lists update --name` does. The old word exits 2 as an unknown command whose `suggestion` and help pointer name `update`. by @brettdavies in [#454](https://github.com/brettdavies/xurl-rs/pull/454)
- Remove `xr broadcasts chat send`, `remove`, `mute`, and `unmute`, which acted inside a live broadcast's chat, and the `chat-sent` schema name of `xr validate --schema` with them. `xr broadcasts chat history` stays, and raw mode still reaches the chat endpoints. by @brettdavies in [#462](https://github.com/brettdavies/xurl-rs/pull/462)

### Changed

- Change the vendored X API spec to upstream 2.173, which adds the broadcast chat mutes, ending and deleting a broadcast, the ingest region, and the `/2/users/{id}/broadcasts` and `/2/users/{id}/sources` paths. Those last two are reachable in raw mode. by @brettdavies in [#409](https://github.com/brettdavies/xurl-rs/pull/409)
- Change the text output of a `--dry-run` that refuses its inputs to print the real run's `Try 'xr <command> --help'.` line, which `--quiet` leaves out. by @brettdavies in [#437](https://github.com/brettdavies/xurl-rs/pull/437)
- Change every help page to list the global flags under their own `Global Options` heading, so they no longer sit between a command's own options. by @brettdavies in [#441](https://github.com/brettdavies/xurl-rs/pull/441)
- Change `xr schema skill install` to refuse with the no-typed-response message at exit 1, as `xr schema skill update` does. `schema/responses/skill-install.schema.json` and `skill-install-all.schema.json` are removed. The JSON report `xr skill install` prints is unchanged. by @brettdavies in [#451](https://github.com/brettdavies/xurl-rs/pull/451)

### Fixed

- Fix `xr broadcasts sources list | get | add | update | remove` for the shape X now answers with: `data` holds the sources or the source itself, a source's fields are `region`, `stream_key`, and `created_at_ms`, `owner_id` is gone, and `remove` answers `{"deleted": true}`. by @brettdavies in [#409](https://github.com/brettdavies/xurl-rs/pull/409)
- Fix `xr --dry-run webhooks listen --forward <URL>` naming `fix-input` for a URL it refuses, where the real run names `show-help`. by @brettdavies in [#437](https://github.com/brettdavies/xurl-rs/pull/437)
- Fix `xr notes evaluate` sending a draft whose `--text` holds no source URL; it is now refused before the request, as `xr notes write` refuses it. by @brettdavies in [#438](https://github.com/brettdavies/xurl-rs/pull/438)
- Fix the help of every argument that names accounts to state the username and user ID rules its validator enforces. by @brettdavies in [#439](https://github.com/brettdavies/xurl-rs/pull/439)
- Fix `xr search-users --help` to state the query rule under Arguments, `xr webhooks add --help` to say the URL names no port, and the `webhooks replay --from-checkpoint` and `webhooks listen --checkpoint` help to name the checkpoint file's JSON shape. by @brettdavies in [#443](https://github.com/brettdavies/xurl-rs/pull/443)
- Fix the error for an unreadable checkpoint file to name the shape a checkpoint file holds.
- Fix the `--limit` and `--cursor` help to describe every command that pages, instead of naming ten and saying the rest ignore them. by @brettdavies in [#444](https://github.com/brettdavies/xurl-rs/pull/444)
- Fix `xr calls scheduled list` to take its page size from the global `--limit` when `-n` is not given; the default stays 20.
- Fix the `-n` help of `xr stream rules list` and `xr broadcasts chat history` to state the 100 results a page holds at most; it said 1000 and 200.
- Fix `xr webhooks listen` with no signing secret to carry a `next_step`: `fix-input` with a template that stores the app's client secret, or its OAuth1 credentials under `--secret oauth1`, and `register-app` when no app is registered. by @brettdavies in [#448](https://github.com/brettdavies/xurl-rs/pull/448)
- Fix `xr webhooks listen --dry-run` with no signing secret to answer with its real run's `reason` (`auth-required`), message, and step, in place of `reason: no-signing-secret`.

### Documentation

- Document how `xr` differs from Go `xurl` 1.3.4: a map of the commands and flags spelled differently (`xurl posts` is `xr user-posts`), the Go commands `xr` lacks, and sections on errors on stderr, `--dry-run` and `XURL_DRY_RUN=1`, the confirmation before `delete`, the credential check before sending, `--cursor`, post-vocabulary key names, and secrets from a file or stdin. by @brettdavies in [#434](https://github.com/brettdavies/xurl-rs/pull/434)
- Fix the account of Go `xurl`'s exit codes: it exits 1 on every failure.
- Compare `xr` with Go `xurl` 1.3.4 row by row in the CLI README, with release binary sizes and command counts measured from both tools, and name the differences a migrated script hits first in the root README. by @brettdavies in [#435](https://github.com/brettdavies/xurl-rs/pull/435)

**Full Changelog**: [v4.4.1...v4.5.0](https://github.com/brettdavies/xurl-rs/compare/v4.4.1...v4.5.0)

## [4.4.1] - 2026-10-28

### Fixed

- Fix a mistyped command followed by another word (`xr suth status`) reporting the second word as unknown and suggesting an unrelated command. The error now names the mistyped word and its nearest command (`unknown command 'suth'. Did you mean 'auth'?`), with `command`, `suggestion`, and the `show-help` `next_step` to match. by @brettdavies in [#427](https://github.com/brettdavies/xurl-rs/pull/427)
- Fix a word after a raw request's target (`xr /2/users/me extra`, `xr -X POST suth status`) being reported as a mistyped command with an unrelated suggestion. It is now refused as an unexpected argument: reason `invalid-args`, exit 2, with a `show-help` step for `xr --help`. by @brettdavies in [#428](https://github.com/brettdavies/xurl-rs/pull/428)

**Full Changelog**: [v4.4.0...v4.4.1](https://github.com/brettdavies/xurl-rs/compare/v4.4.0...v4.4.1)

## [4.4.0] - 2026-10-27

### Added

- Add the token store layout Go `xurl` 1.3.0 uses. `xr` keeps the store at `~/.xurl/auth.yml`, and the first command that opens it moves a single-file `~/.xurl` into that directory, reporting the move on stderr in text output. The two tools then share one store. The move is one-way: `xr` 4.3.0 and earlier cannot read the directory. by @brettdavies in [#315](https://github.com/brettdavies/xurl-rs/pull/315)
- Add dry runs for every read command and for raw requests: `--dry-run` answers the dry-run envelope and sends nothing, where a read sent its request. by @brettdavies in [#323](https://github.com/brettdavies/xurl-rs/pull/323)
- Add an `auth` object to the dry-run envelope: the `app` whose stored credential would be sent, the `scheme` the request would go out under, and for an OAuth2 login the `username` and `token_expired`. A dry run with no fitting credential answers `would_succeed: false` with reason `auth-required` or `auth-method-mismatch` and the exit code the real run would end on.
- Add `confirmation_required` to the dry-run envelope of `delete`, `auth clear`, and `auth apps remove`. `--dry-run` answers without `--force`, where it stopped on `confirmation-required`.
- Add a file check to `xr --dry-run media upload`: a path that is not a readable file answers `would_succeed: false` with reason `io`.
- Add a `next_step` to the 27 error reasons that carried none, so every reason names what to do next. The README's "Which Errors Carry a Step" table gives each reason a row. by @brettdavies in [#325](https://github.com/brettdavies/xurl-rs/pull/325)
- Add five actions to the closed `next_step.action` set: `retry`, `fix-input`, `report-issue`, `confirm`, and `run-command`.
- Add a `next_step` to the `skill install` and `skill update` envelopes, on a failure and on a host `--all` passes over.
- Add a help pointer after the error line in text mode for an argument `xr` refused, such as `Try 'xr post --help'.`
- Add a stored account id for each login. A command that acts as you (`like`, `follow`, `timeline`, `bookmarks`, and the others) reads it from the store and no longer calls `/2/users/me` first, or `/2/users/by/username/<name>` under `--username`. `xr` stores it at sign-in, at the refresh of a login that lacks it, on `xr whoami`, and on the first command that needs it. OAuth1 logins are covered as well. by @brettdavies in [#328](https://github.com/brettdavies/xurl-rs/pull/328)
- Add naming of a login stored without a username: `xr whoami` stores it under the username X answers with, and reports `Stored the OAuth2 login as @<name>` in text output.
- Add `xr webhooks list`, `add`, `validate`, `remove`, and `replay` for registering and managing X API webhooks. `remove` asks for confirmation (`--force` skips it), and `replay` takes its window as `--from` and `--to` in `yyyymmddhhmm` UTC. by @brettdavies in [#334](https://github.com/brettdavies/xurl-rs/pull/334)
- Add the schema names `webhooks-list`, `webhooks-add`, `webhooks-validate`, `webhooks-remove`, and `webhooks-replay` to `xr schema`, and `webhook`, `webhooks`, `webhook-validation`, and `webhook-replay` to `xr validate --schema`.
- Add `xr webhooks subscriptions count`, `list`, `add`, `check`, and `remove` for Account Activity subscriptions on a webhook. `add` and `check` act for the signed-in account; `remove` takes a numeric user id and asks for confirmation (`--force` skips it). by @brettdavies in [#336](https://github.com/brettdavies/xurl-rs/pull/336)
- Add the schema names `webhooks-subscriptions-count`, `-list`, `-add`, `-check`, and `-remove` to `xr schema`, and `subscription-count`, `subscriptions`, and `subscribed` to `xr validate --schema`.
- Add `xr webhooks stream-links list`, `add`, and `remove`, which deliver the posts your filtered-stream rules match to a webhook. `remove` asks for confirmation (`--force` skips it). by @brettdavies in [#337](https://github.com/brettdavies/xurl-rs/pull/337)
- Add the schema names `webhooks-stream-links-list`, `-add`, and `-remove` to `xr schema`, and `stream-links` and `stream-link` to `xr validate --schema`.
- Add `xr webhooks listen`, a local receiver that answers X's CRC check and prints each delivered event whose signature verifies as one JSON line. It takes `--port`, `--bind`, `--path`, `--secret oauth2|oauth1`, `--allow-unsigned`, and `--max-events`, and opens no tunnel: expose the local address at a public HTTPS URL of your own. by @brettdavies in [#338](https://github.com/brettdavies/xurl-rs/pull/338)
- Add an offline check to `xr webhooks add` that refuses a URL that is not HTTPS or that names a port. X accepts neither.
- Add `xr calls get` and `xr calls scheduled list | get | add | update | cancel | rsvp`: read an X call, and schedule, move, re-invite, cancel, and answer the calls on your calendar. Times are RFC 3339, and every bound the spec sets is checked before any request; `cancel` asks first. A sign-in now requests the `calls.read` and `calls.write` scopes these take, so a login made earlier needs `xr auth oauth2` again. by @brettdavies in [#340](https://github.com/brettdavies/xurl-rs/pull/340)
- Add `xr broadcasts sources list | get | add | rename | remove`: manage the RTMP and RTMPS ingest points you broadcast from. A source's `id` is its stream key. `remove` asks first.
- Add `xr bookmark-folders list`, `posts`, and `add`: list the folders you sort bookmarks into, list the posts in one by ID, and create one. A folder ID or name the API refuses is refused before any request. by @brettdavies in [#341](https://github.com/brettdavies/xurl-rs/pull/341)
- Add `xr whoami --cached`: it prints your account's id, username, and display name from the store with no request (`"source": "cache"`), and asks X once when the store does not hold them yet (`"source": "api"`). by @brettdavies in [#345](https://github.com/brettdavies/xurl-rs/pull/345)
- Add `oauth2_accounts` and `oauth1_account` to `xr auth status` and `xr auth apps list`: the id, username, and display name of the account each login belongs to, as the store holds them. The text form prints the id and display name beside each login.
- Add reason `address-in-use` (exit 5) to `xr webhooks listen`: a port another process holds now answers with a `fix-input` step whose `command` reruns `listen` on `--port 0`, where it answered `io` with no command. by @brettdavies in [#354](https://github.com/brettdavies/xurl-rs/pull/354)
- Add `test_command` and `test_request` to the `listening` record of `xr webhooks listen`: a signed sample delivery, as a curl invocation and as data, that proves the receiver passes a signed event through before X sends one. The sample's body carries `"xr_test_event": true`. Text output prints the curl invocation on stderr. by @brettdavies in [#355](https://github.com/brettdavies/xurl-rs/pull/355)
- Add `--forward <URL>`, `-H 'Name: Value'`, and `--forward-timeout <SECS>` to `xr webhooks listen`: each verified event is POSTed to the URL after X is answered, in delivery order, retried up to four times, and reported on stderr as a `forwarded` or `forward-failed` record. A failed record carries a `run-command` step whose template replays the event's minute. Shutdown delivers what is queued under one deadline and counts the rest in a `forward-incomplete` record. by @brettdavies in [#357](https://github.com/brettdavies/xurl-rs/pull/357)
- Add a structured answer to a panic in any `xr` command: the `internal` error (exit 1, the `report-issue` step) in the active output format, with a `backtrace` field when `RUST_BACKTRACE` asks for one. A panic ended the process with exit 134 and Rust's plain-text message.
- Add a program after `--` to `xr webhooks listen`: it runs once per verified event with the event's body on stdin and `XR_EVENT_SHA256` set, beside or instead of `--forward`, with no shell. Its stdout and stderr reach `listen`'s stderr a whole line at a time, so a record never shares a line with them. A failed or timed-out run is retried and reported like a forward, and a run that is stopped takes every process it started with it. by @brettdavies in [#358](https://github.com/brettdavies/xurl-rs/pull/358)
- Add `--checkpoint <FILE>` to `xr webhooks listen`: it records, at most once a minute, the arrival of the oldest event not yet delivered to every forward target, so a stop with a backlog stays recoverable. A failed write is reported on stderr as `checkpoint-failed` and `listen` goes on. by @brettdavies in [#360](https://github.com/brettdavies/xurl-rs/pull/360)
- Add `--from-checkpoint <FILE>` to `xr webhooks replay`: it requests the window from the checkpoint's minute to the current one, at most 24 hours, and reports `clamped` and `lost_from` beside X's answer. `--dry-run` shows the window and sends nothing.
- Add `active_follower_count` to `xr whoami`: the followers active on X in the last 24 hours, which X returns for your account when account analytics is enabled. by @brettdavies in [#373](https://github.com/brettdavies/xurl-rs/pull/373)
- Add `source` to the `xr --dry-run whoami --cached` envelope: `cache` when the store holds the whole account for the login, `api` when the real run would ask X. by @brettdavies in [#375](https://github.com/brettdavies/xurl-rs/pull/375)
- Add `xr hide <REPLY>` and `xr unhide <REPLY>` to hide a reply to your post or show it again, by id or post URL. by @brettdavies in [#384](https://github.com/brettdavies/xurl-rs/pull/384)
- Add `xr likers <POST>`, `xr reposters <POST>`, and `xr quotes <POST>` to list who liked or reposted a post and the posts that quote it, paged with `-n` and `--cursor`. by @brettdavies in [#386](https://github.com/brettdavies/xurl-rs/pull/386)
- Add `xr posts <POST_ID>...` to read up to 100 posts, by id or URL, in one request, with the ids X could not return listed under `errors`. by @brettdavies in [#387](https://github.com/brettdavies/xurl-rs/pull/387)
- Add `xr user-posts <USER>` to list the posts a user wrote, newest first, paged with `-n` and `--cursor`. by @brettdavies in [#388](https://github.com/brettdavies/xurl-rs/pull/388)
- Add `xr search-users <QUERY>` to find accounts by keyword, paged with `-n` and `--cursor`. by @brettdavies in [#389](https://github.com/brettdavies/xurl-rs/pull/389)
- Add `xr search-counts <QUERY>` to count the posts from the last seven days that match a query, per minute, hour, or day, between optional `--start` and `--end` times. by @brettdavies in [#390](https://github.com/brettdavies/xurl-rs/pull/390)
- Add `xr analytics <POST>... --start <TIME> --end <TIME>` to report impressions and engagement for up to 100 of your posts over a time window, in total or per hour, day, or week. by @brettdavies in [#391](https://github.com/brettdavies/xurl-rs/pull/391)
- Add `xr trends place <WOEID>` to list the trends for a place by its Yahoo WOEID, and `xr trends personal` to list the trends X picks for you. by @brettdavies in [#392](https://github.com/brettdavies/xurl-rs/pull/392)
- Add `xr news search <QUERY>` to search X's news stories, with `--max-age-hours` to keep recent ones, and `xr news get <STORY_ID>` to read one story and the posts it clusters. by @brettdavies in [#393](https://github.com/brettdavies/xurl-rs/pull/393)
- Add `xr communities search <QUERY>` to search X's communities and `xr communities get <COMMUNITY_ID>` to read one. by @brettdavies in [#394](https://github.com/brettdavies/xurl-rs/pull/394)
- Add `xr conversations start`, `send`, `history`, and `with` to start a group conversation, send into a conversation by id, and read a conversation's messages or your messages with one person. by @brettdavies in [#395](https://github.com/brettdavies/xurl-rs/pull/395)
- Add `xr dm-events get` and `xr dm-events remove` to read one direct message or delete one of yours, asking first unless `--force`.
- Add `xr dm-block` and `xr dm-unblock` to stop a user sending you direct messages, or let them again.
- Add an `xr lists` family: your owned, followed, member, and pinned lists; a list's posts, followers, and members; creating, changing, and deleting a list (deleting asks first unless `--force`); adding and removing members; and following and pinning a list. by @brettdavies in [#396](https://github.com/brettdavies/xurl-rs/pull/396)
- Add `xr broadcasts list` and `xr broadcasts get` to read your broadcasts. by @brettdavies in [#398](https://github.com/brettdavies/xurl-rs/pull/398)
- Add `xr broadcasts scheduled` to list, read, schedule, move, cancel, and start early a scheduled broadcast; cancelling and going live ask first unless `--force`.
- Add `xr broadcasts chat` to read a broadcast's chat, send into it, remove a message (asking first unless `--force`), and mute someone or lift the mute.
- Add `xr notes` to write Community Notes through X's AI Note Writer API, score a draft, list the posts eligible for a note and the notes you wrote, and delete a note (asking first unless `--force`); `--test` works in X's test mode. by @brettdavies in [#399](https://github.com/brettdavies/xurl-rs/pull/399)
- Add `xr articles draft` to save a draft X Article from a plain-text body, read from a file or stdin, and `xr articles publish` to publish it, asking first unless `--force`. by @brettdavies in [#400](https://github.com/brettdavies/xurl-rs/pull/400)
- Add `xr users` to look up 1 to 100 accounts in one request, by handle or by numeric id. by @brettdavies in [#401](https://github.com/brettdavies/xurl-rs/pull/401)
- Add `xr media get` to read 1 to 100 uploads by media key in one request, and `xr media analytics` to report your videos' views, plays, and watch time over a time window. by @brettdavies in [#402](https://github.com/brettdavies/xurl-rs/pull/402)
- Add `xr spaces` to find X Spaces by keyword, read up to 100 by id or by the accounts that created them in one request, and read the posts shared in a Space. by @brettdavies in [#403](https://github.com/brettdavies/xurl-rs/pull/403)
- Add `xr stream rules` to list, add, remove, and count the filtered stream's rules, with `--validate-only` to have X check a change without saving it. by @brettdavies in [#404](https://github.com/brettdavies/xurl-rs/pull/404)
- Add `--of <USERNAME>` to `xr lists list`, `xr lists followed`, and `xr lists memberships` to read another account's lists. by @brettdavies in [#407](https://github.com/brettdavies/xurl-rs/pull/407)

### Changed

- Change the vendored X API spec to the 2026-10-08 upstream content of 2.169: an activity stream event carries `created_at`, the time the activity occurred, and a filtered-stream rule deletion accepts ids as integers as well as strings. Both are reachable in raw mode as before; shortcut behavior is unchanged. by @brettdavies in [#327](https://github.com/brettdavies/xurl-rs/pull/327)
- Change a token refresh to call `/2/users/me` only when the store lacks the login's name or account id. It called it on every refresh without `--username`. by @brettdavies in [#328](https://github.com/brettdavies/xurl-rs/pull/328)
- Change the vendored X API spec to upstream 2.172, which adds the calls, stream-source, and XChat archive, unread, and message-request endpoints. The XChat ones are reachable in raw mode; no existing shortcut changes. by @brettdavies in [#340](https://github.com/brettdavies/xurl-rs/pull/340)
- Change the unknown-command guess to require a closer match: a typo of a command still gets "Did you mean ...?", but an unrelated word such as `localhost` no longer does, and its `next_step` points at `xr --help`.
- Change the YAML that `--output yaml` prints and that the token store writes to double-quote quoted strings and to quote values a YAML 1.1 reader would misread (`yes`, `no`, `on`, `off`, version numbers such as `4.3.0`, dates); the data is the same and existing token stores load unchanged. by @brettdavies in [#348](https://github.com/brettdavies/xurl-rs/pull/348)
- Change the vendored X API spec to the 2026-10-24 upstream content of 2.172, which adds starting, joining, and leaving a live call and the caller's own followers and following lists. All five are reachable in raw mode; shortcut behavior is unchanged. by @brettdavies in [#366](https://github.com/brettdavies/xurl-rs/pull/366)
- Change how a rename spreads: an account's new username and display name, learned through any login, show for every login stored for that account in every app, in `xr whoami --cached` and `xr auth status`. by @brettdavies in [#376](https://github.com/brettdavies/xurl-rs/pull/376)
- Change text output for a refused input to state the rule it broke, such as `Error: The text is empty; give at least one character.`, instead of the validator's code (`Error: empty-body`). Structured output still carries the code. by @brettdavies in [#379](https://github.com/brettdavies/xurl-rs/pull/379)
- Change text output for `--dry-run` to a summary: that nothing was sent, whether the real run would succeed, and when it would not, the exit code and why, followed by the inputs, the credential, and whether the run would ask for confirmation. Structured output is unchanged. by @brettdavies in [#380](https://github.com/brettdavies/xurl-rs/pull/380)
- Change `xr followers` and `xr following` without `--of` to read `/2/users/me/followers` and `/2/users/me/following` directly, one request with no id lookup first. by @brettdavies in [#383](https://github.com/brettdavies/xurl-rs/pull/383)

### Fixed

- Fix `xr mentions -n` and `xr likes -n` with a count below 5: the count is raised to 5, X's minimum for those endpoints, where it was sent as given and refused with a 400. by @brettdavies in [#310](https://github.com/brettdavies/xurl-rs/pull/310)
- Fix `xr post`, `xr reply`, and `xr quote` refusing a body over 280 characters. Length is left to X, so a Premium account's long post is sent, and so is a post whose URL takes the plain count past 280. `--dry-run` no longer reports `body-too-long`. by @brettdavies in [#311](https://github.com/brettdavies/xurl-rs/pull/311)
- Fix a `--<name>-file` flag storing an empty secret. An empty file or pipe is refused at exit 2 with reason `invalid-args`, and nothing is stored. by @brettdavies in [#312](https://github.com/brettdavies/xurl-rs/pull/312)
- Fix `xr auth clear` reporting success against a token store that could not be loaded. It exits 77 with reason `token-store` and the `inspect-store` step, and leaves the file alone. by @brettdavies in [#313](https://github.com/brettdavies/xurl-rs/pull/313)
- Fix zsh completion of the raw-mode `[URL]` argument: it completes as a URL where it offered file names. by @brettdavies in [#318](https://github.com/brettdavies/xurl-rs/pull/318)
- Fix `--dry-run` on a raw request, which sent the request, a `POST` or a `DELETE` included. by @brettdavies in [#323](https://github.com/brettdavies/xurl-rs/pull/323)
- Fix shortcut commands reporting `auth-required` when the only OAuth2 login is one stored without a username, which is what a sign-in whose `/2/users/me` lookup failed leaves behind. The login is used, as it was under `--auth oauth2`. by @brettdavies in [#324](https://github.com/brettdavies/xurl-rs/pull/324)
- Fix the command a `select-app` step offers when the invocation already named an app: it repeated `--app`, which `xr` refuses as a usage error. The rerun names the app once, and names the binary `xr` however it was started. by @brettdavies in [#325](https://github.com/brettdavies/xurl-rs/pull/325)
- Fix a token refresh for a login stored under a label other than its X username: the new token was saved under the X username and the label kept a dead one, so a later refresh could fail. The refreshed token replaces the old one where it is stored. by @brettdavies in [#328](https://github.com/brettdavies/xurl-rs/pull/328)
- Fix a refresh of a login stored without a username leaving the replaced token in the unnamed slot.
- Fix RUSTSEC-2026-0346, RUSTSEC-2026-0347, RUSTSEC-2026-0348, and RUSTSEC-2026-0349 in the TLS stack by building with rustls 0.23.46. by @brettdavies in [#348](https://github.com/brettdavies/xurl-rs/pull/348)
- Fix TLS certificate verification accepting a path through distrusted intermediates, by updating rustls-webpki to 0.103.16 (RUSTSEC-2026-0382). by @brettdavies in [#352](https://github.com/brettdavies/xurl-rs/pull/352)
- Fix a debug build of `xr` on Windows overflowing its main-thread stack on every command, `--help` included: the Windows binary reserves 8 MiB, the size Linux gives, where the Windows default is 1 MiB. by @brettdavies in [#358](https://github.com/brettdavies/xurl-rs/pull/358)
- Fix RUSTSEC-2026-0385 by updating ctutils, the constant-time helper under the HMAC and SHA code, to 0.4.6. by @brettdavies in [#369](https://github.com/brettdavies/xurl-rs/pull/369)
- Fix `-X`, `-H`, `-d`, `--auth`, `-u`, `-t`, `-s`, and `-F` given before a command word being ignored without a word: `xr -u work whoami` answered as the default login. Such an invocation now exits 2 with reason `invalid-args`, and for `-u`, `--auth`, or `-t` on a command that takes it, `next_step` is `fix-input` with the invocation that puts the flag after the command word. by @brettdavies in [#374](https://github.com/brettdavies/xurl-rs/pull/374)
- Fix the recovery step for `-u <login>` when the app holds no such login: it named any other signed-in app, whose own step pointed back. It now names an app that holds the login, or a sign-in that stores the login under that name in the app the rerun reads (`xr auth oauth2 <login>`, with `--app` when that app is not the default).
- Fix the step 2 command the headless sign-in prints: it carries the login name and `--app` step 1 was given, which step 2 needs to store the login under that name in that app.
- Fix `--dry-run` refusals to carry the `next_step` the real run's error names. A step that reruns the invocation keeps `--dry-run`.

### Documentation

- Document that X reports an animated GIF as `pending` at FINALIZE, so `xr media upload` waits for one as Go `xurl` does. by @brettdavies in [#316](https://github.com/brettdavies/xurl-rs/pull/316)
- Add the agent-native badge to the README; it links to `xr`'s scorecard. by @brettdavies in [#321](https://github.com/brettdavies/xurl-rs/pull/321)
- Document that the Homebrew formula installs the release's archive directly, with no bottle, and where to verify an archive against its attestation. by @brettdavies in [#331](https://github.com/brettdavies/xurl-rs/pull/331)
- Document the `xr webhooks` family in the README: the order in which a receiver, a registered URL, and a subscription have to be set up, and what `xr webhooks listen` verifies. by @brettdavies in [#339](https://github.com/brettdavies/xurl-rs/pull/339)
- Rewrite the webhooks entry in `KNOWN_DIFFERENCES.md` to give the differences from Go `xurl`'s `webhook start`.
- Document `xr webhooks listen` forwarding (`--forward`, `-H`, `--forward-timeout`, and a program after `--`), the `forwarded` and `forward-failed` records, the `Idempotency-Key` that repeats share, and restart recovery with `--checkpoint` and `xr webhooks replay --from-checkpoint`, dry run first. by @brettdavies in [#361](https://github.com/brettdavies/xurl-rs/pull/361)
- Document that shortcut commands print X's keys under the vendored spec's names, renaming a key X still sends in its legacy post vocabulary (`edit_history_tweet_ids` as `edit_history_post_ids`, `tweet_count` as `post_count`, and others), while a raw request prints X's body unchanged. by @brettdavies in [#365](https://github.com/brettdavies/xurl-rs/pull/365)
- Label `xr calls` a preview in its help and the README: X marks the Calls API as in development and not yet generally available, so its requests and the scopes it takes can change. by @brettdavies in [#366](https://github.com/brettdavies/xurl-rs/pull/366)
- Document that `xr whoami --cached` answers with what X said when the login last asked, so a rename shows on the next live `xr whoami`, and that it does not check the token, so an expired login still answers. by @brettdavies in [#375](https://github.com/brettdavies/xurl-rs/pull/375)
- Describe `-u` as the name a login is stored under (its username, or the name given to `xr auth oauth2`).
- Document how Go `xurl` treats `-u`, `--auth`, `-t`, and the other raw-only flags before a command word, beside `xr`'s refusal, in KNOWN_DIFFERENCES.md. by @brettdavies in [#382](https://github.com/brettdavies/xurl-rs/pull/382)

**Full Changelog**: [v4.3.0...v4.4.0](https://github.com/brettdavies/xurl-rs/compare/v4.3.0...v4.4.0)

## [4.3.0] - 2026-10-07

### Added

- Add `--client-secret-file`, `--consumer-secret-file`, `--access-token-file`, `--token-secret-file`, and `--bearer-token-file` to `xr auth apps add`, `auth apps update`, `auth oauth1`, and `auth app`. Each reads the secret from a file, or from stdin when the path is `-`, so the secret never appears in the process's arguments or shell history. The plain flags keep working. by @brettdavies in [#276](https://github.com/brettdavies/xurl-rs/pull/276)
- Add `--scopes` to `xr auth oauth2`: a comma-separated subset of the OAuth2 scopes to request, with `offline.access` always added. The default stays every scope. An unknown name exits with `validation` and lists the valid ones. by @brettdavies in [#277](https://github.com/brettdavies/xurl-rs/pull/277)
- Add `--wait[=<SECS>]` to `xr media upload` and `xr media status`: bare or `true` waits up to 60 seconds, a number that many seconds, and `false` or `0` not at all. by @brettdavies in [#281](https://github.com/brettdavies/xurl-rs/pull/281)
- Add the `processing-timeout` reason (exit 1) for a wait that reaches its deadline. Its envelope carries `media_id` and a `next_step` with the new `resume-wait` action, whose `command` resumes the wait for twice as long.
- Add `retry_after_secs` and `retry_at` to the `rate-limited` envelope when the 429 names its reset, with a `next_step` whose new action is `wait-and-retry`. by @brettdavies in [#282](https://github.com/brettdavies/xurl-rs/pull/282)
- Add `--wait-on-rate-limit` (`XURL_WAIT_ON_RATE_LIMIT`) to wait out a rate limit and retry once, and `--rate-limit-max-wait <SECS>` (`XURL_RATE_LIMIT_MAX_WAIT`, default 60) to bound that wait. Without the flag `xr` never retries.
- Add build-provenance and SBOM attestations to the release archives. Verify a downloaded archive with `gh attestation verify <archive> --repo brettdavies/xurl-rs --signer-workflow brettdavies/.github/.github/workflows/rust-release.yml`. by @brettdavies in [#293](https://github.com/brettdavies/xurl-rs/pull/293)
- Add `yml` as an alias of `yaml` for `--output` and `XURL_OUTPUT`. by @brettdavies in [#304](https://github.com/brettdavies/xurl-rs/pull/304)
- Add a `next_step` to every `invalid-args` envelope: `show-help` with the `xr <command> --help` the message names. by @brettdavies in [#306](https://github.com/brettdavies/xurl-rs/pull/306)
- Add a `next_step` to a `token-store` envelope when the store file exists and could not be loaded: `inspect-store` with `docs`.
- Add `docs` to the `inspect-store` step an `auth-required` envelope carries.

### Changed

- Change the `register-app` `next_step` template to `<secret-command> | xr auth apps add <name> --client-id <client-id> --client-secret-file -`, and show the piped or file form in every help example, `xr examples`, and the README. by @brettdavies in [#276](https://github.com/brettdavies/xurl-rs/pull/276)
- Change `xr media upload`'s default wait on a video from unbounded to 60 seconds. A video that processes longer exits 1 with `processing-timeout`; the upload is intact, `--wait=N` waits longer, and the envelope's `resume-wait` command (`xr media status <media_id> --wait=<secs>`) continues the wait. by @brettdavies in [#281](https://github.com/brettdavies/xurl-rs/pull/281)
- Change the error for an unsupported `--output` or `XURL_OUTPUT` value (`toml`, `xml`) from plain text to a JSON envelope with reason `invalid-args`. The exit code is still 2. by @brettdavies in [#305](https://github.com/brettdavies/xurl-rs/pull/305)

### Fixed

- Fix two `xr` processes refreshing the same expired OAuth2 login at once: the second now waits and uses the token the first saved, where both used to spend the single-use refresh token. The store's directory gains a `<store>.refresh.lock` file beside `<store>.lock`. by @brettdavies in [#279](https://github.com/brettdavies/xurl-rs/pull/279)
- Fix OAuth1 signatures for a request whose query or body value contains a space, `+`, `~`, or `*`. Parameters are now percent-encoded as RFC 5849 requires, matching the signature X computes. by @brettdavies in [#280](https://github.com/brettdavies/xurl-rs/pull/280)
- Fix `xr media status --wait` polling forever on media whose status carries no processing information, such as an image. The wait returns after one status call. by @brettdavies in [#281](https://github.com/brettdavies/xurl-rs/pull/281)
- Fix `xr media upload --wait=false`, which was rejected, so an upload can return after FINALIZE without waiting.
- Fix a raw request printing `{}` for a successful response whose body is not JSON. The body is printed as it was sent. by @brettdavies in [#282](https://github.com/brettdavies/xurl-rs/pull/282)
- Fix an error response whose body is not JSON reporting `HTTP error: <status>`. The message carries the body.
- Fix a response whose body fails to read being reported as an empty success. It exits 5 with reason `network-error`.
- Fix a malformed raw URL (for example `xr 'http://[bad'`) exiting 5 with `network-error`, or 77 with `auth-required` under `--auth oauth1`. It exits 1 with reason `invalid-url`, names the URL, and sends nothing. by @brettdavies in [#285](https://github.com/brettdavies/xurl-rs/pull/285)
- Fix a `.twurlrc` that does not parse being reported as `serialization`; it is `token-store`.
- Fix the `--dry-run` and `--limit` help text, and the shell completions generated from it, ending in an internal reference, `(U7)`. by @brettdavies in [#291](https://github.com/brettdavies/xurl-rs/pull/291)
- Fix `xr media upload` printing two JSON documents for an upload it waited on (three under `--verbose`). It prints one: FINALIZE's document with the final `processing_info`, so `.data.id` and `.data.processing_info.state` read from the same object. by @brettdavies in [#296](https://github.com/brettdavies/xurl-rs/pull/296)
- Fix `xr media upload` returning at once for a GIF or other non-video upload that X reports as still processing. It waits whenever FINALIZE's answer carries an unfinished `processing_info`, up to the same deadline as a video. by @brettdavies in [#297](https://github.com/brettdavies/xurl-rs/pull/297)
- Fix `xr auth default <app> <user>` printing two JSON documents. It prints one, whose message names the app and the user. by @brettdavies in [#299](https://github.com/brettdavies/xurl-rs/pull/299)
- Fix `xr auth default <app> <user>` moving the default app when the user is not one the app holds. The command now fails with the store unchanged.
- Fix `--output jsonl` printing an indented, multi-line document for anything that is not a stream. It prints one record per line, as `ndjson` does and as JSON Lines means. by @brettdavies in [#300](https://github.com/brettdavies/xurl-rs/pull/300)
- Fix the `search -n` and `likes -n` help, which gave 1 as the minimum; X's minimums are 10 and 5, and `search` raises a lower value to 10. by @brettdavies in [#301](https://github.com/brettdavies/xurl-rs/pull/301)
- Fix the examples under `search`, `timeline`, `mentions`, `bookmarks`, `likes`, and `dms`, and on the examples page, that piped a list into `jaq '.id'` and printed `null`. They show a filter that prints each record's id.
- Fix `search`, `timeline`, `mentions`, `bookmarks`, `likes`, `following`, `followers`, `muted`, `blocked`, and `dms` exiting 1 with reason `serialization` when nothing matched. X sends such a page without a `data` key; the command now prints an empty `data` list at exit 0. by @brettdavies in [#303](https://github.com/brettdavies/xurl-rs/pull/303)

### Documentation

- Document `cargo install --locked xurl-rs` as the cargo install command: without `--locked`, a fresh resolution that picks cc 1.6.0 fails to build aws-lc-sys when `CFLAGS` carries an optimization flag. by @brettdavies in [#272](https://github.com/brettdavies/xurl-rs/pull/272)
- Add a Stability section to the README: what `xr`'s machine contract covers, what is outside it, and what a patch, a minor, and a major can carry. by @brettdavies in [#292](https://github.com/brettdavies/xurl-rs/pull/292)
- Correct the README where it disagreed with the binary: the uploads `media upload` waits for and its one-document answer, the `bearer` and `bearer_source` fields of `auth status`, exit 5 for an unreadable file, and which errors carry a `next_step`. by @brettdavies in [#301](https://github.com/brettdavies/xurl-rs/pull/301)
- Document which error reasons carry a `next_step` and which do not, and how to recover a token store that could not be read. by @brettdavies in [#306](https://github.com/brettdavies/xurl-rs/pull/306)

**Full Changelog**: [v4.2.1...v4.3.0](https://github.com/brettdavies/xurl-rs/compare/v4.2.1...v4.3.0)

## [4.2.1] - 2026-10-04

### Changed

- Change the vendored X API spec to the 2026-10-02 upstream content of 2.169, which adds an optional `user_id` filter to listing activity subscriptions; shortcut behavior is unchanged. by @brettdavies in [#258](https://github.com/brettdavies/xurl-rs/pull/258)

**Full Changelog**: [v4.2.0...v4.2.1](https://github.com/brettdavies/xurl-rs/compare/v4.2.0...v4.2.1)

## [4.2.0] - 2026-09-30

### Added

- Add `XURL_SKILL_HOME`: it stands in for `~` in `xr skill install` and `xr skill update` destinations, so the skill bundle can install into a directory of your choosing without changing `HOME`. by @brettdavies in [#249](https://github.com/brettdavies/xurl-rs/pull/249)
- Add support for each skill host's own config-directory variable (`CLAUDE_CONFIG_DIR`, `KIRO_HOME`, `OPENCODE_CONFIG_DIR`). When one is set, that host's destination sits under it, ahead of `XURL_SKILL_HOME` and `HOME`; `xr --help` lists them.
- Add `XDG_CONFIG_HOME` to the OpenCode skill destination: when it is set and `XURL_SKILL_HOME` is not, `xr skill install opencode` and `xr skill update opencode` use `$XDG_CONFIG_HOME/opencode/skills/xurl-rs`, where OpenCode reads skills. `OPENCODE_CONFIG_DIR` still wins over both. by @brettdavies in [#250](https://github.com/brettdavies/xurl-rs/pull/250)
- Add `legacy_install_dir` to the `xr skill install` and `xr skill update` envelopes. It names a copy of the skill at a location the host still reads but `xr` no longer installs to; `skill update` removes that copy, and `skill install` leaves it in place and says so in text mode.
- Add `xr media alt-text <MEDIA_ID> <TEXT>` to set the alt text screen readers announce for an uploaded image or video; `--dry-run` checks the id and the 1000-character limit. by @brettdavies in [#252](https://github.com/brettdavies/xurl-rs/pull/252)
- Add `xr media subtitles add <VIDEO_ID> <SUBTITLES_ID> --language <CODE> [--name <NAME>]` and `xr media subtitles remove <VIDEO_ID> --language <CODE>` to attach and detach a subtitle track uploaded with `xr media upload --category subtitles`; `--category` names the video's upload category (`amplify_video` by default, or `tweet_video`).
- Add the `alt-text` and `subtitles` schemas to `xr validate`, with auto-detection, and the three commands to `xr schema`.

### Changed

- Change the vendored X API spec to 2.169: 23 new `/2/chat` operations that raw mode reaches, two filtered-stream rule error types (`duplicate-rules`, `invalid-rules`), required fields on the media metadata and subtitles requests, a `segment_index` ceiling of 9999 on chunked uploads, and a `broadcast.chat.remove` activity event type. by @brettdavies in [#252](https://github.com/brettdavies/xurl-rs/pull/252)

### Fixed

- Fix `--raw` and `XURL_RAW` being ignored for a usage error under a subcommand, a missing argument, or an unknown flag: the JSON envelope printed pretty-printed and text mode kept its color. It now prints compact and uncolored, as for every other error. by @brettdavies in [#242](https://github.com/brettdavies/xurl-rs/pull/242)
- Fix `XURL_JSON` and `XURL_JSONL` being ignored for a usage error under a subcommand: `XURL_JSON=true xr auth zzz` printed a text error where `XURL_JSON=true xr zzz` printed the JSON envelope.
- Fix a help or version flag beside a mistyped word printing the root help at exit 0 when the invocation also repeats a flag or passes an unknown one (`xr --raw webhooks --help --raw=false`); it now reports that usage error at exit 2, as the invocation does without the flag. by @brettdavies in [#245](https://github.com/brettdavies/xurl-rs/pull/245)
- Fix the Codex skill destination: `xr skill install codex` clones into `~/.agents/skills/xurl-rs`, where Codex reads user skills, instead of the deprecated `~/.codex/skills/xurl-rs`. `xr skill update codex` and `xr skill update --all` move an existing copy by removing the old one and installing at the new path. by @brettdavies in [#250](https://github.com/brettdavies/xurl-rs/pull/250)
- Fix raw-mode DELETE requests dropping their `-d` body, which left `xr -X DELETE /2/connections -d '...'` unable to reach the endpoint. by @brettdavies in [#252](https://github.com/brettdavies/xurl-rs/pull/252)

### Documentation

- Correct the non-interactive `delete`, `auth apps remove`, and `auth clear` examples in `xr examples` and `--help`, which failed with `confirmation-required` as written; they now pass `--force`, which a destructive op under `--no-interactive` needs. by @brettdavies in [#243](https://github.com/brettdavies/xurl-rs/pull/243)
- Correct the `--limit` and `--cursor` help to name the commands that page (`search`, `timeline`, `mentions`, `bookmarks`, `likes`, `following`, `followers`, `muted`, `blocked`, `dms`); other commands, including `broadcasts moderators list`, ignore both flags.

**Full Changelog**: [v4.1.1...v4.2.0](https://github.com/brettdavies/xurl-rs/compare/v4.1.1...v4.2.0)

## [4.1.1] - 2026-09-24

### Changed

- Change the vendored X API spec to the 2026-09-24 upstream content of 2.168, which adds an optional `expires_at` to activity subscriptions; shortcut behavior is unchanged. by @brettdavies in [#219](https://github.com/brettdavies/xurl-rs/pull/219)

**Full Changelog**: [v4.1.0...v4.1.1](https://github.com/brettdavies/xurl-rs/compare/v4.1.0...v4.1.1)

## [4.1.0] - 2026-09-24

### Added

- Add a `--verbose` note for each key X sent in its legacy post vocabulary, naming both spellings and the JSON type of the value: `info: X sent edit_history_tweet_ids; read as edit_history_post_ids (array, length 1)`. It prints in text mode only, never under `--quiet`, `--output json`, or `--output jsonl`. by @brettdavies in [#215](https://github.com/brettdavies/xurl-rs/pull/215)
- Add a runnable `next_step` to the `unknown-command` envelope: action `show-help` with the help of the nearest command (`xr auth status --help` for `xr auth statsu`), or of the family the word was typed under when nothing is close. by @brettdavies in [#225](https://github.com/brettdavies/xurl-rs/pull/225)

### Changed

- Change typed output to report X's post vocabulary throughout: when X answers in a legacy spelling, `xr post --output json` and every other typed command print the name the spec uses, with X's value unchanged. Raw requests (`xr /2/...`) still print exactly what X sent. by @brettdavies in [#214](https://github.com/brettdavies/xurl-rs/pull/214)

  ```text
  // Before: xr post "hi" --output json
  {
    "data": {
      "edit_history_tweet_ids": [
        ""
      ],
      "id": "2101712260468977783",
      "text": "hi"
    }
  }

  // After: xr post "hi" --output json
  {
    "data": {
      "edit_history_post_ids": [
        ""
      ],
      "id": "2101712260468977783",
      "text": "hi"
    }
  }
  ```
- Change every error clap raises to read like the rest of `xr`: it opens with `Error:` and closes with `Try '<command> --help'.` naming the command it belongs to, and it honors `--color` and `NO_COLOR`. The `invalid-args` envelope's `message` no longer carries clap's `error:` prefix; its `reason` and exit code are unchanged. by @brettdavies in [#224](https://github.com/brettdavies/xurl-rs/pull/224)
- Change the unknown-command message to point at the help of the suggested command (`xr auth statsu` ends `Try 'xr auth status --help'.`, `xr whoam` ends `Try 'xr whoami --help'.`), or at the family's help when nothing is close.

### Fixed

- Fix raw requests to seven streaming endpoints the X spec declares (`/2/activity/stream`, `/2/likes/firehose/stream`, `/2/likes/sample10/stream`, `/2/tweets/label/stream`, and the post, like, and user compliance streams) being read as one buffered response unless `-s` was passed. `xr` now streams every endpoint the spec marks as streaming. by @brettdavies in [#220](https://github.com/brettdavies/xurl-rs/pull/220)
- Fix `xr <word> --help` and `xr <word> -h` printing the root help at exit 0 when the word names no command. They now exit 2 with reason `unknown-command` and the nearest command in `suggestion`, exactly as `xr <word>` does. by @brettdavies in [#221](https://github.com/brettdavies/xurl-rs/pull/221)
- Fix `xr help --help` and `xr help -h` reporting "unknown command '--help'": they print the help command's own page at exit 0, as `xr help help` does. A flag spelling where a command was expected (`xr -- webhooks --help`) is now an unexpected argument, never an unknown command with a suggestion to run `help`. by @brettdavies in [#222](https://github.com/brettdavies/xurl-rs/pull/222)
- Fix `--color` and `XURL_COLOR` being ignored when a mistyped command is reported from a parse failure (`xr --color always auth statsu`, `xr --color always webhooks --help`). The flag counts wherever it sits in `argv`, and `NO_COLOR` still wins. by @brettdavies in [#223](https://github.com/brettdavies/xurl-rs/pull/223)
- Fix `xr <word> --version` and `xr -V <word>` printing the version at exit 0 when the word names no command. They now exit 2 with reason `unknown-command`, exactly as `xr <word>` does; `xr --version` is unchanged. by @brettdavies in [#226](https://github.com/brettdavies/xurl-rs/pull/226)

### Documentation

- Document the `--verbose` legacy-vocabulary note in `xr --help` and on the `XURL_VERBOSE` line. by @brettdavies in [#218](https://github.com/brettdavies/xurl-rs/pull/218)
- Describe the renamed post fields, the response envelope, and the redirect-URI source in `xr schema` output by what X sends, without Rust library wording.
- Document how `xr` releases are versioned: a patch carries only fixes, a minor carries anything new that breaks nothing, and a major removes, renames, or retypes something in the command, exit-code, or structured-output contract. Text-mode output is not part of that contract; the plain `xr --version` line is. by @brettdavies in [#228](https://github.com/brettdavies/xurl-rs/pull/228)
- Document that a newer release can add a `reason` or a `next_step.action`, so a consumer of the JSON error envelope treats a value it does not recognize as its default branch; the envelope schema and the README both say so. by @brettdavies in [#229](https://github.com/brettdavies/xurl-rs/pull/229)

**Full Changelog**: [v4.0.0...v4.1.0](https://github.com/brettdavies/xurl-rs/compare/v4.0.0...v4.1.0)

## [4.0.0] - 2026-09-18

### Added

- Add `xr block` and `xr unblock` for blocking and unblocking a user by username, with the same flags, dry-run envelope, and output modes as `xr mute` and `xr unmute`. by @brettdavies in [#165](https://github.com/brettdavies/xurl-rs/pull/165)
- Add `xr blocked` and `xr muted`, which list the users you have blocked or muted with `-n` paging, matching `xr following` and `xr followers`.
- Add the `xdk-rs` crate: the X API client library, published on its own, with `use xdk::` as its import path. by @brettdavies in [#174](https://github.com/brettdavies/xurl-rs/pull/174)
- Add `xr version --verbose` and the structured forms of `xr version`, which report the `xdk-rs` library version beside the CLI version; plain `xr --version` is unchanged. by @brettdavies in [#177](https://github.com/brettdavies/xurl-rs/pull/177)
- Add `docs/migrating/v4.0.0.md`, the migration guide for library users moving from `xurl` 3.x to `xdk-rs` 0.1.
- Add `xr broadcasts moderators list`, `add <username>`, and `remove <username>` for managing who moderates your broadcast chats, with `--dry-run` on the write verbs and typed JSON output. by @brettdavies in [#182](https://github.com/brettdavies/xurl-rs/pull/182)
- Add `moderators` to `xr validate --schema`, validating a broadcast chat moderators response. by @brettdavies in [#186](https://github.com/brettdavies/xurl-rs/pull/186)
- Add the nine command families `xr examples` did not show (`delete`, `unrepost`, `unbookmark`, `following`, `followers`, `validate`, `skill`, `completions`, `version`), with a TOOLING section for the skill installer, shell completions, and version. by @brettdavies in [#190](https://github.com/brettdavies/xurl-rs/pull/190)
- Add `dm-event` to `xr validate --schema` for a single direct-message event; `dm` now validates the confirmation `xr dm` prints. by @brettdavies in [#194](https://github.com/brettdavies/xurl-rs/pull/194)

### Changed

- Change the vendored X API spec to the 2026-09-15 upstream revision of 2.168, in which X adds the block and unblock write endpoints and two activity-subscription event types; the library constants `API_SPEC_SHA256` and `API_SPEC_DATE` follow, and shortcut behavior is unchanged. by @brettdavies in [#127](https://github.com/brettdavies/xurl-rs/pull/127)
- Change `xr auth status` and `xr auth apps list` under `--output json` and the other structured formats to emit `{"status": "ok", "apps": [...]}` instead of a bare array, so one `status` field covers every auth verb. Scripts reading the top-level array need `.apps[]`; an empty store now emits `"apps": []` rather than `[]`. Text output is unchanged. by @brettdavies in [#157](https://github.com/brettdavies/xurl-rs/pull/157)
- Change `xr skill update --all` to refresh only the hosts that already have an installation, reporting the rest with `status: "skipped"` and `reason: "not-installed"` at exit code 0. It previously ran the remove-and-reinstall pipeline against every known host, creating installations for hosts never installed to, which is what `xr skill install --all` is for. by @brettdavies in [#162](https://github.com/brettdavies/xurl-rs/pull/162)
- Change `xr skill update --all` to answer with one aggregated envelope carrying an `installations` array, matching `xr skill install --all`, instead of emitting a separate envelope per host. Scripts reading a stream of per-host objects need `.installations[]`.
- Add `command_preview` to the `xr skill update` envelope, so update and install report the same per-host fields.
- Change a failed destination removal during `xr skill update` to report `reason: "remove-failed"` from the closed reason set rather than appending the operating system error text.
- Mark the public `Commands` enum `#[non_exhaustive]`. Library consumers that match it exhaustively need a wildcard arm; in exchange, adding a command stops being a major break. by @brettdavies in [#165](https://github.com/brettdavies/xurl-rs/pull/165)
- Change the vendored X API spec to the 2026-09-17 upstream content of 2.168, which adds the broadcast chat moderator endpoints under `/2/broadcasts/chat/moderators`; the library constants `API_SPEC_SHA256` and `API_SPEC_DATE` follow, and shortcut behavior is unchanged. by @brettdavies in [#166](https://github.com/brettdavies/xurl-rs/pull/166)
- Change `xr --auth oauth2 <command>` with client credentials but no stored token to fail with `TokenNotFound` and a `Sign in first. Run: xr auth oauth2` hint instead of opening a browser mid-request. by @brettdavies in [#170](https://github.com/brettdavies/xurl-rs/pull/170)
- Change `xurl-rs` to a CLI-only package at 4.0.0; the `xr` binary, its commands, output, exit codes, and User-Agent are unchanged, and the `xurl` library target is gone. Rust programs depend on `xdk-rs` instead. by @brettdavies in [#174](https://github.com/brettdavies/xurl-rs/pull/174)
- Change the repository README into a router between the library and the CLI; the CLI reference now lives at `crates/xurl-cli/README.md`. by @brettdavies in [#176](https://github.com/brettdavies/xurl-rs/pull/176)
- Change `xr version --output json` to print one JSON object instead of the plain text line. by @brettdavies in [#177](https://github.com/brettdavies/xurl-rs/pull/177)
- Change the OAuth2 scopes requested at enrollment to include `broadcast.read` and `broadcast.write`; tokens issued before this release must be re-enrolled before the broadcasts commands work. by @brettdavies in [#182](https://github.com/brettdavies/xurl-rs/pull/182)
- Change the `testing` mock's media append reply to `200` with the `expires_at` body the spec documents, instead of an empty `204`. by @brettdavies in [#195](https://github.com/brettdavies/xurl-rs/pull/195)
- Change the error envelope's `reason` for API refusals: 403 reports `forbidden`, 400 and 422 report `invalid-request`, 5xx reports `server-error`, and any other status reports `api-error`; `network-error` now means only that the request never got an answer. by @brettdavies in [#198](https://github.com/brettdavies/xurl-rs/pull/198)
- Change `CHANGELOG.md` to `crates/xurl-cli/CHANGELOG.md`, beside the crate it describes. by @brettdavies in [#202](https://github.com/brettdavies/xurl-rs/pull/202)
- Change the PR template to carry a `## Changelog (xurl-rs)` and a `## Changelog (xdk-rs)` section. A bare `## Changelog` heading now reaches neither crate.

### Fixed

- Fix the `xr examples` gallery, which labelled `mute` irreversible and listed it without `unmute`, unlike every other pair in the gallery. by @brettdavies in [#165](https://github.com/brettdavies/xurl-rs/pull/165)
- Fix the `-n` help text on `xr following` and `xr followers`, which advertised a 1-1000 range while the runtime clamps every list command to 100.
- Fix credential writes made during a legacy-store migration or a twurlrc import bypassing the store lock, and make every store write durable across a crash, unique per writer, and safe on a symlinked `~/.xurl`. by @brettdavies in [#171](https://github.com/brettdavies/xurl-rs/pull/171)
- Fix `xr media upload --wait` to print the FINALIZE response before reporting a processing failure. by @brettdavies in [#177](https://github.com/brettdavies/xurl-rs/pull/177)
- Fix credential-store writes so a crash or a concurrent `xr` process can no longer leave `~/.xurl` truncated or drop a rotated refresh token: saves are atomic, `0600` from creation, and serialized by an OS file lock on `~/.xurl.lock`. by @brettdavies in [#180](https://github.com/brettdavies/xurl-rs/pull/180)
- Fix `xr schema <command>` reporting `validate`, `skill`, `examples`, and every `auth`, `media`, and `skill` subcommand as an unknown command; each now answers `schema not available`, as `xr schema auth` already did. by @brettdavies in [#184](https://github.com/brettdavies/xurl-rs/pull/184)
- Fix `xr validate --help` omitting seven of the schema names `--schema` accepts (`like`, `follow`, `delete`, `repost`, `bookmark`, `mute`, `block`); the help text and the `unknown-schema` envelope now list the same set. by @brettdavies in [#186](https://github.com/brettdavies/xurl-rs/pull/186)
- Fix `xr dm` failing with `missing field id` after the message was sent. by @brettdavies in [#194](https://github.com/brettdavies/xurl-rs/pull/194)
- Fix a transport failure exiting 3, 4, or 77 when the request URL happened to contain `429`, `404`, or `401`; a connection or TLS failure now exits 5 (`EXIT_NETWORK_ERROR`) with reason `network-error`. by @brettdavies in [#198](https://github.com/brettdavies/xurl-rs/pull/198)
- Verify the correct pipeline when a library tag is pushed. A `xdk-rs-vX.Y.Z` tag now checks `release-lib.yml` and skips the Homebrew and finalize gates, instead of checking the binary's `release.yml` and reporting on a run that never happened. by @brettdavies in [#200](https://github.com/brettdavies/xurl-rs/pull/200)
- Fix the release changelog silently omitting every stacked pull request. The PR list came from `gh pr list --base dev`, which never returns a PR targeting the branch below it in a stack; it is now read from the integration branch's own history. For this release that is the difference between 14 pull requests and 39. by @brettdavies in [#202](https://github.com/brettdavies/xurl-rs/pull/202)

### Documentation

- Document the library's release path, its rehearsal, the tag order, and the breaking-change and MSRV policies. by @brettdavies in [#177](https://github.com/brettdavies/xurl-rs/pull/177)
- Document where a change goes between `xdk-rs` and `xurl-rs` in AGENTS.md, with the rules the split holds and a decision test for new features. by @brettdavies in [#178](https://github.com/brettdavies/xurl-rs/pull/178)
- Add an `Adding a command family` recipe to `AGENTS.md` naming every surface and the test that guards it, and an `Adding a command` pointer in `CONTRIBUTING.md`. by @brettdavies in [#193](https://github.com/brettdavies/xurl-rs/pull/193)
- Correct the `xdk-rs` first-time publish steps so the crate name is claimed with a `0.0.0` placeholder. Publishing the crate at the version its first tag will release leaves that release with nothing to publish. by @brettdavies in [#199](https://github.com/brettdavies/xurl-rs/pull/199)

**Full Changelog**: [v3.2.0...v4.0.0](https://github.com/brettdavies/xurl-rs/compare/v3.2.0...v4.0.0)

## [3.2.0] - 2026-09-14

### Added

- Add a `bearer_source` field (`env` or `store`) to each `auth status` and `auth apps list` entry under structured output, and name the environment variable beside the bearer mark in text mode. by @brettdavies in [#129](https://github.com/brettdavies/xurl-rs/pull/129)
- Add a refusal to overwrite a token store that exists but could not be read or parsed; the error names the path and the file is left byte-identical. An unrelated JSON file at the store path is no longer adopted and rewritten as a store. by @brettdavies in [#137](https://github.com/brettdavies/xurl-rs/pull/137)
- Add a `client-credentials-missing` error at exit 2 when `xr auth oauth2` has no client id to sign in with, carrying a `next_step` that names the app to use or the registration template.
- Add a recovery step to the no-credentials error. Text mode names the command to run; structured output carries a `next_step` object with a closed-set `action` and either a runnable `command` or a `template` to fill in. `--quiet` suppresses the advice and never the error. by @brettdavies in [#140](https://github.com/brettdavies/xurl-rs/pull/140)
- Add an enrollment hint to the 403 that follows a successful sign-in. When the body says the client is not enrolled or is forbidden, the error quotes that detail and points at the Troubleshooting recipe; structured output carries a `next_step` with the `enroll-app` action and a documentation link. by @brettdavies in [#141](https://github.com/brettdavies/xurl-rs/pull/141)
- Add an `unknown-command` error: a mistyped command exits 2 and names the nearest real command, in text and in every structured format, with the word in `command` and the correction in `suggestion`. by @brettdavies in [#143](https://github.com/brettdavies/xurl-rs/pull/143)

### Changed

- Change the vendored X API spec to the 2026-09-04 upstream revision of 2.168, in which the webhook and activity-subscription endpoints also accept OAuth 2.0 user context and OAuth 1.0a; the library constants `API_SPEC_SHA256` and `API_SPEC_DATE` follow, and shortcut behavior is unchanged. by @brettdavies in [#124](https://github.com/brettdavies/xurl-rs/pull/124)
- Change the release runbook to build `release/*` as an overlay of `dev` on `main`, sync `dev` by `scripts/sync-dev-after-release.sh` after publish, and document the crates.io, GitHub Release, and Homebrew rollback commands. by @brettdavies in [#128](https://github.com/brettdavies/xurl-rs/pull/128)
- Change the crates.io package to exclude the release runbooks and `CONCEPTS.md`; the `MIGRATING.md` guide links now point at GitHub. by @brettdavies in [#131](https://github.com/brettdavies/xurl-rs/pull/131)
- Change the six optional-value true-or-false flags (`--verbose`, `--quiet`, `--no-interactive`, `--dry-run`, `--raw`, `--no-browser`) to require `=` for an explicit value, so `xr --quiet whoami` runs `whoami`; the space-separated `--quiet true` form no longer parses as a value, and an attached short value such as `-q0` is rejected as a stacked flag. by @brettdavies in [#133](https://github.com/brettdavies/xurl-rs/pull/133)
- Change a store loaded from a missing or empty file to register no apps, so the first app you add becomes the default and the Quick Start sign-in carries its client id. by @brettdavies in [#137](https://github.com/brettdavies/xurl-rs/pull/137)
- Change `xr auth apps add` to reject new app names outside `[A-Za-z0-9_.-]`, and to promote the new app past a default that carries neither a client id nor any token.
- Change `xr auth status` and `xr auth apps list` with no apps registered to name the registration command in text and emit an empty array under structured output.
- Change the registration message to name the next command: `App "NAME" registered (default). Next: xr auth oauth2`, or `--app NAME` when another app holds the default.
- Change `Envelope::Error` from a struct variant with `reason`, `exit_code`, and `message` fields to `Error(Box<ErrorBody>)`, so one declared body is the whole error surface. The emitted JSON is unchanged; Rust callers matching on the variant's named fields need updating.
- Change the structured success of every message-shaped `xr auth` verb to carry `status: ok` alongside its existing keys, so an agent can branch on one field across the auth surface. Text output and the `auth status` and `auth apps list` arrays are unchanged. by @brettdavies in [#139](https://github.com/brettdavies/xurl-rs/pull/139)
- Change `Envelope`, `ErrorBody`, and `NextAction` to `#[non_exhaustive]`, so future variants and fields are additive for library consumers rather than breaking. by @brettdavies in [#142](https://github.com/brettdavies/xurl-rs/pull/142)
- Change bare `xr` to print the root help at exit 0 instead of failing with `No URL provided` at exit 1. Under `--output json` and the other structured formats it is an `invalid-args` envelope at exit 2. by @brettdavies in [#143](https://github.com/brettdavies/xurl-rs/pull/143)
- Change a mistyped command from a raw request against a nonexistent path to a usage error. `xr whoam` exits 2 rather than 1, and no longer reaches the network. A positional that starts with `http://`, `https://`, or `/`, and any invocation carrying `-X`, `-H`, `-d`, `-F`, `-u`, `--auth`, `-t`, or `-s`, is still a raw request.
- Change the usage-error envelope to render in whichever structured format was named, so `--output yaml`, `ndjson`, `csv`, and `tsv` get their own rendering rather than falling back to clap's text.

### Fixed

- Fix shortcut commands ignoring `XURL_BEARER_TOKEN` during auth auto-detect, so `xr search`, `xr whoami`, and the other bearer-capable shortcuts work on an empty token store when the variable is set. by @brettdavies in [#129](https://github.com/brettdavies/xurl-rs/pull/129)
- Fix `xr -q search "topic"` and every other bare-flag-then-command invocation, which fell into raw mode with "No URL provided". by @brettdavies in [#133](https://github.com/brettdavies/xurl-rs/pull/133)
- Fix the dependency audit by updating `rustls` to 0.23.45, which carries the fix for a flaw that let a peer send handshake messages in the clear and still have the connection accepted. by @brettdavies in [#138](https://github.com/brettdavies/xurl-rs/pull/138)
- Fix release notes carrying raw PR titles for changes that ship nothing user-facing. A pull request that fills in the template and leaves `## Changelog` empty is taken at its word instead of having its title promoted to a `Changed` bullet. by @brettdavies in [#151](https://github.com/brettdavies/xurl-rs/pull/151)

### Documentation

- Document the branch-drift steps (including the gate 0 bookkeeping check) and the Dependabot preflight in RELEASES-PREFLIGHT.md and the last-good identifier and rollback items in RELEASES-POSTFLIGHT.md. by @brettdavies in [#128](https://github.com/brettdavies/xurl-rs/pull/128)
- Add a security policy that routes vulnerability reports through GitHub private vulnerability reporting and states the acknowledgement and fix windows. by @brettdavies in [#130](https://github.com/brettdavies/xurl-rs/pull/130)
- Add issue forms for bug reports, feature requests, and skill-bundle reports, with blank issues disabled and contact links for X platform questions and the original `xurl` tool. by @brettdavies in [#132](https://github.com/brettdavies/xurl-rs/pull/132)
- Document the error contract in `AGENTS.md`, with the recipe for recovering from exit 77, and add the envelope example and exit-code pointer to the README. by @brettdavies in [#140](https://github.com/brettdavies/xurl-rs/pull/140)
- Document the unknown-command path in `KNOWN_DIFFERENCES.md`, where Go `xurl` sends a bare word as the endpoint and `xr` rejects it. by @brettdavies in [#143](https://github.com/brettdavies/xurl-rs/pull/143)
- Add `CONTRIBUTING.md`: dev setup, the branch and PR flow, the error contract, where to file issues, and a recipe for exercising `xr` against X's API Playground without an account. by @brettdavies in [#144](https://github.com/brettdavies/xurl-rs/pull/144)
- Add a "Before you start" section to the README covering the X developer-portal prerequisites, the OAuth2 redirect URI, and the pay-per-use pricing link, above the Quick Start that assumes them. by @brettdavies in [#145](https://github.com/brettdavies/xurl-rs/pull/145)
- Add crates.io, CI, and license badges, a statement that the project is independent of X, the `xr` binary name at the top of Install, and the `xr skill install` and `xr skill update` verbs under Agent-Native Features.
- Change the Contributing section to point at `CONTRIBUTING.md`, and the release-runbook link to a GitHub URL, since `RELEASES*.md` is excluded from the published crate.

**Full Changelog**: [v3.1.0...v3.2.0](https://github.com/brettdavies/xurl-rs/compare/v3.1.0...v3.2.0)

## [3.1.0] - 2026-09-03

### Added

- Add `XURL_TOKEN_STORE=<path>` to point `xr` at a token-store file other than `~/.xurl`; the OAuth2 headless pending state follows it. by @brettdavies in [#119](https://github.com/brettdavies/xurl-rs/pull/119)

### Fixed

- Fix the token store rewriting `~/.xurl` on every load when `CLIENT_ID` is exported. Concurrent `xr` processes could race on the file and lose every stored app and token; backfilled credentials now persist on the next explicit save instead. by @brettdavies in [#117](https://github.com/brettdavies/xurl-rs/pull/117)
- Fix the headless OAuth2 flow writing `.xurl.pending` under the home directory regardless of the token store in use. The pending state now sits beside the store.
- Fix `UserPublicMetrics.post_count` reading 0 on live responses: X sends the field as `tweet_count`, and the typed field now accepts either key. `xr user` output carries the real value under `post_count` instead of a zero beside `tweet_count`. by @brettdavies in [#118](https://github.com/brettdavies/xurl-rs/pull/118)
- Fix `xr --help` omitting `XURL_TOKEN_STORE`, `XURL_BEARER_TOKEN`, `CLIENT_ID`, `CLIENT_SECRET`, `AUTH_URL`, `TOKEN_URL`, `API_BASE_URL`, and `INFO_URL`; the environment block now lists every variable `xr` reads with its precedence. by @brettdavies in [#121](https://github.com/brettdavies/xurl-rs/pull/121)

### Documentation

- Correct the v3.0.0 migration guide on user metrics: shortcut commands print the typed response with `post_count` filled from the `tweet_count` X sends, and raw `xr <URL>` calls still show `tweet_count`. by @brettdavies in [#118](https://github.com/brettdavies/xurl-rs/pull/118)

**Full Changelog**: [v3.0.0...v3.1.0](https://github.com/brettdavies/xurl-rs/compare/v3.0.0...v3.1.0)

## [3.0.0] - 2026-09-02

### Added

- Add `xr usage credits`, a typed shortcut for the new `/2/usage/credits` endpoint; bare `xr usage` keeps reading `/2/usage/tweets`. by @brettdavies in [#108](https://github.com/brettdavies/xurl-rs/pull/108)

### Changed

- Change typed JSON output to the X API 2.168 post vocabulary: `referenced_posts`, `public_metrics.repost_count`, and `includes.posts` on post responses, `public_metrics.post_count` on user responses (breaking). by @brettdavies in [#108](https://github.com/brettdavies/xurl-rs/pull/108)
- Change library response types `Tweet`, `TweetPublicMetrics`, and `ReferencedTweet` to `Post`, `PostPublicMetrics`, and `ReferencedPost` (breaking).
- Change shortcut requests to send `post.fields`; `in_reply_to_user_id` and referenced posts are now requested through expansions.
- Change `xr validate --schema` tokens from `tweet`/`tweets` to `post`/`posts` with no aliases, and add a `credits` token; auto-detect labels follow (breaking).
- Change the library action-result type `RetweetedResult` to `RepostedResult` (breaking).

### Fixed

- Remove `schema/responses/block.schema.json` and `unblock.schema.json`, stale since the block shortcuts left in v2.0.0 and still describing the 2.x `Tweet` types. by @brettdavies in [#112](https://github.com/brettdavies/xurl-rs/pull/112)

### Documentation

- Add `docs/migrating/v3.0.0.md`, the post-vocabulary migration guide for library consumers and `xr` script authors, linked from MIGRATING.md. by @brettdavies in [#111](https://github.com/brettdavies/xurl-rs/pull/111)

**Full Changelog**: [v2.2.0...v3.0.0](https://github.com/brettdavies/xurl-rs/compare/v2.2.0...v3.0.0)

## [2.2.0] - 2026-09-01

### Added

- `run_with_overrides`, `Config::from_overrides`, `EnvOverrides`, and `Auth::new_with_store_path_and_overrides` let a library consumer drive a run with the environment supplied explicitly, instead of inheriting whatever the host process exported. by @brettdavies in [#91](https://github.com/brettdavies/xurl-rs/pull/91)

### Changed

- Add a 7-day cooldown before Dependabot opens version-update pull requests. by @brettdavies in [#82](https://github.com/brettdavies/xurl-rs/pull/82)
- TLS certificate verification now uses the operating system trust store instead of the Mozilla root certificates that were compiled into the binary. Corporate TLS-inspecting proxies work without extra configuration; a machine with a missing or stale system CA store needs it repaired. by @brettdavies in [#89](https://github.com/brettdavies/xurl-rs/pull/89)
- The TLS crypto provider is now aws-lc-rs rather than ring, following reqwest 0.13's only supported provider path.
- `Auth` resolves `XURL_BEARER_TOKEN` when it is constructed rather than when a header is requested. A run resolves its environment once at the entrypoint, so the binary is unaffected; a consumer that mutated the variable between constructing `Auth` and requesting a header would see the value from construction. by @brettdavies in [#91](https://github.com/brettdavies/xurl-rs/pull/91)

### Fixed

- Honor caller-supplied `Authorization`, `Content-Type`, `User-Agent`, and `X-B3-Flags` headers passed via `-H` or `RequestOptions::headers` by suppressing xurl's own auto-append for that request. Eliminates duplicate-header requests that servers either reject or resolve arbitrarily. `--auth none` is no longer required as a workaround for `Authorization`. Affects `send_request`, `send_multipart_request`, and `stream_request`. by @brettdavies in [#73](https://github.com/brettdavies/xurl-rs/pull/73)
- Update `h2` to 0.4.19 in the lockfile, clearing RUSTSEC-2026-0258 (unbounded queueing of empty HTTP/2 DATA frames). The crate enters through a dev-dependency and is not linked into `xr`, so this affects `cargo install --locked` builds and the advisory gate rather than the shipped binary. by @brettdavies in [#88](https://github.com/brettdavies/xurl-rs/pull/88)
- Update `quinn-proto` to 0.11.17, clearing the advisory on the lockfile entry that `reqwest`'s unused http3 feature contributes.
- `OutputConfig` gains `new_with_no_color`, which resolves the color decision from a supplied value rather than the process environment. `new_with_raw` keeps reading `NO_COLOR` and delegates to it. by @brettdavies in [#92](https://github.com/brettdavies/xurl-rs/pull/92)

**Full Changelog**: [v2.1.0...v2.2.0](https://github.com/brettdavies/xurl-rs/compare/v2.1.0...v2.2.0)

## [2.1.0] - 2026-06-05

### Added

- Document the entire `xurl` library public surface (crate-level overview, per-module headers, per-item prose, per-field and per-variant docs) so docs.rs renders complete reference material for downstream Rust consumers. by @brettdavies in [#70](https://github.com/brettdavies/xurl-rs/pull/70)
- Enforce `#![deny(missing_docs)]` on the library surface so new undocumented `pub` items fail `cargo build` at compile time.
- Add `no_run` usage examples on the seven entry-point types (`ApiClient`, `OutputConfig`, `TokenStore`, `XurlError`, `get_me`, `create_post`, `search_posts`) so downstream consumers have copyable on-ramps that compile against the public API.
- Five top-level `pub const`s on the `xurl` library surface: `CRATE_VERSION` (this crate's version), `CRATE_GIT_SHA` (`Option<&str>`, the git HEAD SHA at build time when one is available), `API_SPEC_VERSION` (`info.version` of the vendored X API spec), `API_SPEC_SHA256` (content hash of the vendored spec file), and `API_SPEC_DATE` (UTC date of the last refresh). Downstream library consumers can read linked-crate identity and X API spec identity without invoking `xr --version` as a subprocess. by @brettdavies in [#71](https://github.com/brettdavies/xurl-rs/pull/71)
- `vendor/spec-metadata.json`: a checked-in sidecar carrying the vendored spec's identity (info_version, content_sha256, refreshed_at, source_url). `scripts/refresh-x-openapi.sh` writes this atomically alongside `vendor/x-api-openapi.json` so build-time metadata always describes the actual bytes that ship.
- `scripts/diff-x-openapi-spec.sh`: standalone diff/render tool for X OpenAPI snapshots. Surfaces path, schema, and categorical-value (enum + discriminator-mapping) additions and removals as a markdown body, used by the spec-drift workflow and callable manually.

### Changed

- Bump `hmac` 0.12.1 → 0.13.0 and `sha1` 0.10.6 → 0.11.0 so OAuth1 HMAC-SHA1 signing now runs on the `digest` 0.11 line alongside `sha2`. No user-observable behavior change in `xr`; downstream Rust consumers of the `xurl` library that link these crates directly will see the major version bumps. by @dependabot[bot] in [#61](https://github.com/brettdavies/xurl-rs/pull/61)
- Vendored X API spec refreshed to upstream as of 2026-06-05. `info.version` still 2.165; path count still 139. New schema `PostDeleteActivityResponsePayload`; new `post.create`/`post.delete` values in the `ActivityStreamingResponsePayload.discriminator.mapping` and `ActivitySubscriptionCreateRequest.properties.event_type.enum`; new `includes: Expansions` field on the streaming payload. by @brettdavies in [#71](https://github.com/brettdavies/xurl-rs/pull/71)
- Spec-drift workflow report now distinguishes "silent content bump" (`info.version` matches on both sides) from "real version bump," lists path / schema / categorical-value diffs with `+`/`-` value markers, and renders the refresh-locally footer only when no structural diff surfaced.

### Documentation

- Fix four mis-targeted `///` blocks in `src/api/mod.rs`, `src/auth/mod.rs`, `src/cli/mod.rs`, `src/store/mod.rs` where the doc bound to the next item (a `pub mod foo;` or `use ...;`) rather than the enclosing module. Same pattern fixed across additional files surfaced during implementation. by @brettdavies in [#70](https://github.com/brettdavies/xurl-rs/pull/70)

**Full Changelog**: [v2.0.0...v2.1.0](https://github.com/brettdavies/xurl-rs/compare/v2.0.0...v2.1.0)

## [2.0.0] - 2026-06-05

### Added

- Endpoint-aware auth-method validator (`xurl::api::auth_matrix`) refuses `--auth X` invocations against endpoints whose spec entry does not list `X`, with a typed `XurlError::AuthMethodMismatch` and `auth-method-mismatch` envelope by @brettdavies in [#56](https://github.com/brettdavies/xurl-rs/pull/56)
- Auto-detect intersects the active app's stored credentials with the endpoint's accepted schemes and prefers OAuth2 → OAuth1 → Bearer; three envelope shapes surface the outcome: - **Explicit mismatch**: user passed `--auth X` against an endpoint that doesn't accept `X` - **Empty intersection**: active app has creds but none overlap with the endpoint's supported set - **Wrong app**: active app holds nothing, but other apps in the store do; carries `other_apps_with_creds` so the recovery hint can name them
- AuthMethodMismatch envelope carries `endpoint` (spec template, for agent pattern-matching), `rendered_url` (substituted path, for user-facing messages), `app` (active app name), `requested`, `supported`, `available_in_app`, and `other_apps_with_creds`
- `EXIT_AUTH_MISMATCH = 2` (`EX_USAGE`) for the new error variant; distinct from `EXIT_AUTH_REQUIRED = 77`
- CI drift-check workflow (`.github/workflows/spec-drift.yml`): weekly cron + PR-touches-vendor trigger; opens or updates a `[spec-drift]` issue on divergence
- Build-time shortcut coverage check fails `cargo test` when any shortcut targets a path absent from the spec
- `MIGRATING.md` at the repo root documenting the v1 → v2 migration for library consumers
- `vendor/x-api-openapi.json` + `scripts/refresh-x-openapi.sh` + `vendor/README.md`
- `scripts/generate-changelog.py --dry-run` idempotency check that runs the regen pipeline against `CHANGELOG.md`, compares against the current file, restores the original on exit, and exits 1 with a unified diff if regeneration would drift. by @brettdavies in [#57](https://github.com/brettdavies/xurl-rs/pull/57)

### Changed

- Release flow documentation (`RELEASES.md`, `RELEASES-RATIONALE.md`) names `generate-changelog.py` as the entry point. by @brettdavies in [#55](https://github.com/brettdavies/xurl-rs/pull/55)
- BREAKING: `xurl::api::RequestOptions.endpoint: String` replaced by `target: RequestTarget` (`Template { path, path_params, query } | RawUrl(String)`); shortcut callers and direct library consumers MUST construct the new shape by @brettdavies in [#56](https://github.com/brettdavies/xurl-rs/pull/56)
- BREAKING: `ApiClient::get_auth_header_public` takes `&RequestOptions` instead of the four stringly-typed parameters
- `RawUrl` targets now enforce an `http(s)://` scheme allowlist; non-HTTP(S) values return `XurlError::InvalidUrl`
- `cliff.toml` routes `feat!:`/`fix!:` and `BREAKING CHANGE:` footers into a `[Breaking]` changelog section

### Fixed

- `scripts/generate-changelog.py` now ships alongside the docs that reference it; the previous `generate-changelog.sh` delegated to a missing Python helper and could not be run as documented. by @brettdavies in [#55](https://github.com/brettdavies/xurl-rs/pull/55)
- Auto-detect no longer attempts OAuth2 for endpoints the spec lists as Bearer-only (or vice versa); the intersection refuses incompatible combinations before the HTTP round-trip rather than letting X reject with a generic 403 by @brettdavies in [#56](https://github.com/brettdavies/xurl-rs/pull/56)
- When the active app holds no credentials but other apps in the store hold some, the resolver surfaces the wrong-app envelope (exit 2) instead of the misleading generic `auth-required` (exit 77)
- `scripts/generate-changelog.py` no longer prepends a duplicate `## [X.Y.Z]` section when one already exists for the requested tag. by @brettdavies in [#57](https://github.com/brettdavies/xurl-rs/pull/57)

### Removed

- BREAKING: `block_user` and `unblock_user` shortcuts (target paths absent from the current OpenAPI spec); use raw mode if you need the behavior, see MIGRATING.md by @brettdavies in [#56](https://github.com/brettdavies/xurl-rs/pull/56)

**Full Changelog**: [v1.3.0...v2.0.0](https://github.com/brettdavies/xurl-rs/compare/v1.3.0...v2.0.0)

## [1.3.0] - 2026-06-04

### Added

- Add `RELEASES-PREFLIGHT.md`, a pre-cut go/no-go checklist that gates release-branch creation and covers API-contract surface, live-API smoke tests, distribution probes, and release-mechanics sanity. by @brettdavies in [#27](https://github.com/brettdavies/xurl-rs/pull/27)
- Add `RELEASES-RATIONALE.md`, holding the rationale behind every release rule: branching model, PR conventions, triple-diff verification, CHANGELOG generation, release pipeline, prose scrubbing, branch-protection pitfalls.
- Add `.github/pull_request_template.md` so new PRs in this repo get the canonical Summary, Changelog, Testing, and Files Modified prompts.
- Add `.markdownlint-cli2.yaml` so local markdownlint runs use the same line-wrap and style rules as other brettdavies repos.
- Expose `xurl::cli` as a public module with three layered entrypoints: `run_argv() -> i32` for the binary, `run(args, stdout, stderr) -> i32` for library callers, and `run_with_store_path(args, stdout, stderr, store_path) -> i32` for tests. by @brettdavies in [#29](https://github.com/brettdavies/xurl-rs/pull/29)
- Add `Auth::new_with_store_path(cfg, path)` as the canonical constructor and `Config::default_store_path()` exposing the legacy `~/.xurl` resolution to library consumers.
- Add a compile-time `Send + Sync` assertion on `Auth` so regressions break the build instead of leaking into async callers.
- New CLI surface: `auth apps add --redirect-uri`, `auth apps update --redirect-uri`, `auth apps redirect-uri get [NAME]`, `auth apps redirect-uri set NAME URI`. by @brettdavies in [#30](https://github.com/brettdavies/xurl-rs/pull/30)
- Structured JSON output for `auth status`, `auth apps list`, and `auth apps redirect-uri get` under `--output json`.
- `.anc.toml` declaring the X (Twitter) platform's canonical command verbs (post, reply, like, follow, mute, dm, ...) for anc's `p6-may-standard-names` audit. by @brettdavies in [#33](https://github.com/brettdavies/xurl-rs/pull/33)
- New `--color <auto|always|never>` global flag (`XURL_COLOR`); `auto` honors stderr's TTY-ness, and `NO_COLOR` is absolute per https://no-color.org/. by @brettdavies in [#34](https://github.com/brettdavies/xurl-rs/pull/34)
- Environment-variable backing for every agentic flag: `XURL_VERBOSE`, `XURL_QUIET`, `XURL_NO_INTERACTIVE`, `XURL_TIMEOUT`, `XURL_APP` (in addition to the pre-existing `XURL_OUTPUT`). Boolean flags use `FalseyValueParser` so `XURL_QUIET=0` correctly disables quiet.
- Add `xr examples` subcommand that prints a curated, grouped gallery of common invocations covering auth, posting, social graph, account inspection, DMs, media upload, raw mode, schemas, and multi-app workflows. by @brettdavies in [#35](https://github.com/brettdavies/xurl-rs/pull/35)
- Add `Examples:` block with 3 to 5 curated invocations to every `xr` subcommand's `--help`, including paired text and `--output json` forms for scripting.
- Add `ENVIRONMENT VARIABLES:` section to `xr --help` listing `XURL_OUTPUT`, `XURL_QUIET`, `XURL_NO_INTERACTIVE`, `XURL_TIMEOUT`, `XURL_COLOR`, `XURL_VERBOSE`, `XURL_APP`, and `REDIRECT_URI` with their flag equivalents.
- Add `EXIT CODES:` matrix to `xr --help` documenting codes 0 through 5 (success, general error, invalid args or auth required, rate-limited, not found, network error).
- Add TTY-behavior note to `xr --help` explaining that piping to a non-TTY strips color and suppresses human-only banners without extra flags.
- `xr skill install [<host>] [--all] [--dry-run] [--output json]` to install the `AGENTS.md` bundle into per-host skills directories. Supports `claude_code`, `codex`, `cursor`, `factory`, `kiro`, and `opencode`. by @brettdavies in [#36](https://github.com/brettdavies/xurl-rs/pull/36)
- Wire `--timeout` / `XURL_TIMEOUT` through `ApiClient`, OAuth2 token exchange + refresh, and the `/2/users/me` lookup so the flag bounds every HTTP request. Default stays 30 seconds. by @brettdavies in [#37](https://github.com/brettdavies/xurl-rs/pull/37)
- SIGTERM and SIGINT now cancel the OAuth2 callback listener and any active streaming request. Streaming emits a `{"status":"cancelled","reason":"sigterm"}` envelope under `--output json` / `--jsonl` and returns exit code 0.
- Add global `--json` and `--jsonl` flags on the root command as shorthands for `--output json` and `--output jsonl`; both conflict with `--output` and with each other, resolved by `Cli::effective_output()`. by @brettdavies in [#38](https://github.com/brettdavies/xurl-rs/pull/38)
- Add global `--raw` flag (env `XURL_RAW`, falsey-value parser) that emits compact JSON in `json`/`jsonl` modes and strips ANSI styling in text mode; threaded through `OutputConfig::new_with_raw`.
- Add `xurl::envelope::Envelope`, a `#[derive(JsonSchema)]` enum modeling the three response shapes (`ok`, `dry_run`, `error`) every JSON output path serializes into.
- Add `xr schema envelope` and `xr schema --envelope` subcommands that emit the canonical agent-native envelope JSON Schema (Draft 2020-12) for downstream pinning.
- Add `schema/output.schema.json` at the repo root as the pinned envelope schema snapshot, with a drift guard in `tests/schema_tests.rs` that asserts byte-equality against the runtime emitter.
- Add `OutputConfig::print_success(out, payload)` and `OutputConfig::print_dry_run(out, would_succeed, exit_code, ctx)` as the canonical envelope writers alongside `print_error`; object payloads flatten into the envelope for verb context fields.
- Add an `envelope` row to `xr schema --list` so consumers can discover the envelope schema alongside the 29 typed response commands.
- Global `--force` flag (with confirmation gate) on `xr delete`, `xr auth clear`, and `xr auth apps remove`. Under `--no-interactive`, the flag is mandatory; calling the op without it returns `{"status":"error","reason":"confirmation-required","exit_code":1}` on stderr and never touches the API. Under a TTY without `--force`, dialoguer prompts the operator. by @brettdavies in [#39](https://github.com/brettdavies/xurl-rs/pull/39)
- Global `--dry-run` flag (env-backed via `XURL_DRY_RUN`) on every write op: post, reply, quote, delete, like, unlike, repost, unrepost, bookmark, unbookmark, follow, unfollow, block, unblock, mute, unmute, dm, media upload, app add/update/remove, redirect-uri set, and every `xr auth` subcommand. The envelope shape is `{"status":"dry_run","would_succeed":bool,"reason":"<kebab>","exit_code":int,"command":"<verb>",…}`. Pre-flight validators catch `empty-body`, `body-too-long`, `too-many-attachments`, `empty-username`, and `empty-post-id` so the dry-run path predicts failure before any HTTP call.
- Global `--limit <n>` flag (env-backed via `XURL_LIMIT`), clamped to `1..=100`. Applies to search, timeline, mentions, bookmarks, likes, following, followers, and dms. Per-command `-n/--max-results` keeps precedence when both are set.
- Add `OutputConfig::verbose`, `OutputConfig::warning`, and `OutputConfig::progress` helpers so library consumers can emit diagnostics that respect `--verbose`, `--quiet`, `--format json`, and TTY detection. by @brettdavies in [#40](https://github.com/brettdavies/xurl-rs/pull/40)
- Add `scripts/lint-stdio.sh` to fail the build when any `println!`, `eprintln!`, `print!`, or `eprint!` macro appears outside `src/output.rs`.
- Add a `lint-stdio` job to `.github/workflows/ci.yml` that runs the guard on every push and pull request.
- Back `--json` and `--jsonl` with the `XURL_JSON` and `XURL_JSONL` environment variables, parsed via `FalseyValueParser` so `0`, `false`, and empty values disable the flag. by @brettdavies in [#41](https://github.com/brettdavies/xurl-rs/pull/41)
- Introduce the `xr skill update [<host>] [--all] [--dry-run] [--output json]` subcommand, which removes the existing destination and re-runs the install pipeline; the JSON envelope `action` field is `skill-update` so consumers can tell update apart from first-time install.
- Advertise `XURL_JSON`, `XURL_JSONL`, and `XURL_NO_BROWSER` in the `ENVIRONMENT VARIABLES` block of `xr --help`.
- Add global `--no-pager` flag (env `XURL_NO_PAGER`) accepted as a documented no-op so agents and wrappers that reflexively pass `--no-pager` no longer get rejected. by @brettdavies in [#42](https://github.com/brettdavies/xurl-rs/pull/42)
- `XURL_NO_BROWSER` env var for `xr auth oauth2 --no-browser` so headless and CI runners can set the headless flow by default. by @brettdavies in [#43](https://github.com/brettdavies/xurl-rs/pull/43)
- `xr auth oauth2` auto-engages the headless (remote-step-1) flow when stdout is not a TTY and `--no-browser` / `XURL_NO_BROWSER` is unset, emitting `{"status":"awaiting_callback","url":"..."}` on stdout instead of attempting to open a browser.
- Extend `--output` to accept `csv`, `tsv`, `yaml`, and `ndjson` (a strict alias of `jsonl`) so agents trained on delimited or YAML conventions can pipe `xr` output through existing toolchains. by @brettdavies in [#46](https://github.com/brettdavies/xurl-rs/pull/46)
- Emit a `_warning` flattening column on `csv` and `tsv` rows whose nested object or array cells got JSON-stringified, so consumers detect lossy flattening without parsing every cell.
- Add a global `--cursor <token>` flag (env `XURL_CURSOR`) plus an `--after` synonym (env `XURL_AFTER`) that threads `pagination_token` into search, timeline, mentions, bookmarks, likes, following, followers, dms, and raw-mode list URLs.
- URL-encode the cursor token via `form_urlencoded::byte_serialize` on every list shortcut so exotic upstream tokens round-trip back to the X API unchanged.
- Reject `--page <n>` (env `XURL_PAGE`) with the canonical `{status:"error",reason:"unsupported-pagination",exit_code:1,...}` envelope on stderr and exit non-zero before any API call, turning a silent no-op into a structured signal for agents trained on offset pagination.
- Add the `xr validate` subcommand that reads JSON from stdin (no file arg or `-`) or a file path and validates it against bundled response schemas (`tweet`, `tweets`, `user`, `users`, `dm`, `dms`, `usage`, `envelope`, plus the small fixed-shape result types).
- Auto-detect the validation schema from the document's top-level shape when `--schema` is omitted, or pin a target type explicitly via `--schema <name>` for stricter agent pipelines.
- Emit `{status:"ok",schema:...,valid:true}` on validation success and `{status:"error",reason:"validation-failed",...}` on failure, with exit code 1 on validation failure and exit code 2 on argument errors.
- Document the stdin convention in a new `INPUT FROM STDIN` block on the root `--help` so agents discover the no-file-arg input pattern from the top-level help text.
- Document `toml` and `xml` as unsupported `--output` values in the help text so substring searches over `xr --help` surface the complete token catalog.
- `schema/responses/<command>.schema.json` for every entry in `SCHEMA_ENTRIES` (35 files at v1.3.0): the 14 existing typed shortcut commands plus `auth-status`, `auth-apps-list`, `redirect-uri-get`, `redirect-uri-set`, `skill-install`, and `skill-install-all`. Downstream agents and CI consumers can pin against the on-disk JSON Schemas without invoking `xr`. by @brettdavies in [#48](https://github.com/brettdavies/xurl-rs/pull/48)
- `scripts/generate-response-schemas.sh` regenerates the per-command files by enumerating commands via `xr schema --list` and dumping each via `xr schema <cmd> --output json`.
- Typed `RedirectUriGetResponse` and `RedirectUriSetResponse` for the `auth apps redirect-uri get` / `set` JSON output paths, replacing the prior ad-hoc `serde_json::json!()` shapes so the schema is derivable.
- `XURL_BEARER_TOKEN` env var now feeds `Auth::get_bearer_token_header`, so one-shot agent flows can pipe a bearer through the environment without first running `xr auth app --bearer-token`. Resolution order is env-supplied bearer first, then the resolved app's stored bearer; empty env values fall through. Matches the precedence shape of every other agentic flag (`XURL_VERBOSE`, `XURL_OUTPUT`, `XURL_NO_BROWSER`). by @brettdavies in [#51](https://github.com/brettdavies/xurl-rs/pull/51)
- `xurl::auth::resolve_bearer_token` public free function factored out of `get_bearer_token_header` so library consumers and unit tests can exercise every precedence branch without touching the process environment.
- First successful authentication against a named app now promotes that app to default when the previous default holds no credentials of any kind. Applies to `xr auth oauth2 --app NAME`, `xr auth oauth1 --app NAME`, and `xr auth app --bearer-token … --app NAME`. Subsequent invocations resolve NAME without an explicit `--app`, matching the "first signed-in app becomes default" UX. Users who want a different default later can still run `xr auth default <name>` explicitly; this auto-promote helper only fires when the existing default's `oauth2_tokens`, `oauth1_token`, `bearer_token`, and `unnamed_oauth2_token` are all empty. by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)
- `TokenStore::promote_to_default_if_first_credentialed(candidate)` public helper that encapsulates the promotion contract. Idempotent no-op on already-credentialed defaults, unknown candidates, empty candidate names, and self-referential calls.
- `TokenStore::default_app_is_uninitialized` predicate underpinning the promotion check.

### Changed

- Bumped declared `rust-version` from `1.85` to `1.94` to match the pinned toolchain, closing the MSRV gap between the promise to library consumers and the code that actually ships. by @brettdavies in [#28](https://github.com/brettdavies/xurl-rs/pull/28)
- Reshape `OutputConfig` print methods (`info`, `status`, `print_response`, `print_stream_line`, `print_error`, `print_message`) to accept `&mut dyn Write` at the call site, leaving `OutputConfig` as a pure `Send + Sync + Clone` configuration struct with no I/O state. by @brettdavies in [#29](https://github.com/brettdavies/xurl-rs/pull/29)
- Rename leaf formatters in `src/api/response/format.rs` to `format_response`, `colorize_json`, `write_colorized_value`, and `write_colorized_value_ln`; the public re-export in `src/api/response/mod.rs` updates accordingly.
- `auth status` text output now includes a `redirect_uri:` line per app showing the effective URI and source label. When the env var overrides a stored value, a `stored_redirect_uri:` line surfaces the stored value. by @brettdavies in [#30](https://github.com/brettdavies/xurl-rs/pull/30)
- `auth apps list` text output includes the same inline redirect URI hint per app row.
- The OAuth2 callback listener now binds the host, port, and path from the resolved redirect URI rather than the hardcoded `127.0.0.1:8080/callback`.
- Path matching on the callback listener tightens from `starts_with` to exact-or-querystring, so a custom redirect URI like `/oauth/return` no longer matches unrelated prefixes.
- Test failures inside `src/api/response/types.rs`, `src/config/mod.rs`, and `src/auth/callback.rs` now panic with named subjects (e.g., "Tweet response must deserialize") instead of generic `unwrap()` messages. by @brettdavies in [#33](https://github.com/brettdavies/xurl-rs/pull/33)
- `-v/--verbose` is now a single root-level global flag and applies to every subcommand. The duplicate `-v` definitions on `CommonFlags`, `media upload`, and `media status` are removed. by @brettdavies in [#34](https://github.com/brettdavies/xurl-rs/pull/34)
- BREAKING: change `EXIT_AUTH_REQUIRED` from `2` to `77` (`EX_NOPERM` from sysexits); the previous value collided with `EX_USAGE`, so clap parse errors keep `2` and auth-required failures now exit unambiguously. Downstream scripts branching on the auth exit code must update. by @brettdavies in [#38](https://github.com/brettdavies/xurl-rs/pull/38)
- Reshape error JSON under `--output json|jsonl` from the prior `{"error","kind","code"}` triple to the canonical envelope `{"status":"error","reason":<kebab>,"exit_code":<int>,"message":<str>}` on every output path.
- Promote `exit_code_for_error(&XurlError)` and the private `error_kind(&XurlError)` to `XurlError::exit_code()` and `XurlError::kind()` methods, re-exported via `xurl::error`; the free function is preserved as a thin shim so existing call sites keep compiling.
- Narrow `XurlError::kind()` to a closed kebab-case set (`auth-required`, `rate-limited`, `not-found`, `network-error`, `invalid-args`, `invalid-method`, `validation`, `serialization`, `io`, `token-store`) so agents pattern-match on a stable identifier.
- Switch the runner from `Cli::parse_from` to `Cli::try_parse_from` so `DisplayHelp`, `DisplayVersion`, and `DisplayHelpOnMissingArgumentOrSubcommand` exit `0` to stdout, and other parse errors route through envelope emission on stderr when JSON intent is present.
- Emit the `invalid-args` envelope on clap parse failures when JSON intent is detected in the unparsed argv (`--json`, `--jsonl`, `--output json|jsonl`, `--output=json|jsonl`) or in `XURL_OUTPUT`; otherwise preserve clap's text rendering.
- Bump crate version from `1.2.0` to `1.3.0` in `Cargo.toml` and `Cargo.lock`.
- `OutputConfig` gains `print_dry_run` and `print_confirmation_required` helpers so any handler can emit the canonical envelope in one call. by @brettdavies in [#39](https://github.com/brettdavies/xurl-rs/pull/39)
- `XurlError::EnvelopeAlreadyEmitted { exit_code }` is the new sentinel that lets the runner suppress its trailing `print_error` when a typed envelope was already written; agents see exactly one JSON object on stderr.
- `src/api/shortcuts` is now `pub` so the new validators are importable from the command handlers without re-exports.
- Suppress verbose request/response logs, streaming connection banners, OAuth salvage warnings, `.twurlrc` import warnings, and `REDIRECT_URI` rejection warnings when `--format json` or `--quiet` is set, so machine-readable output stays clean. by @brettdavies in [#40](https://github.com/brettdavies/xurl-rs/pull/40)
- Colorize verbose request and response lines only when `OutputConfig::use_color` is enabled, matching the existing color-detection rules for other output.
- Tighten `ROOT_HELP` wording to reference the new `--no-pager` flag and reaffirm that `xr` writes directly to stdout and stderr and never invokes `$PAGER`. by @brettdavies in [#42](https://github.com/brettdavies/xurl-rs/pull/42)
- `xr auth default` without an app argument now gates the dialoguer picker on `--no-interactive` AND stdin/stderr TTY-ness. Non-TTY sessions exit non-zero with a `{"status":"error","reason":"no-tty","exit_code":1,...}` envelope on stderr instead of stranding the dialoguer state machine on `/dev/null`. by @brettdavies in [#43](https://github.com/brettdavies/xurl-rs/pull/43)
- `xr auth oauth2 --no-browser` (no `--step`) now auto-promotes to step 1 and emits the canonical `{"status":"awaiting_callback","url":"..."}` envelope, instead of rejecting the invocation as "requires --step 1 or --step 2". The explicit `--no-browser --step 1` path keeps its prior envelope shape.
- Drop the `dialoguer` dependency from `xurl-rs`, shrinking the dep tree by removing `console`, `shell-words`, `encode_unicode`, and `unicode-width` transitives. by @brettdavies in [#45](https://github.com/brettdavies/xurl-rs/pull/45)
- Replace the `Select` prompts in `xr auth default` with a stdin-based numbered picker that writes choices to stderr and reads one line from stdin.
- Replace the `Confirm` prompt in `confirm_destructive` with a stdin `[y/N]` reader that accepts `y`/`yes` (case-insensitive) and defaults to `false` on EOF or any other input.
- Route every printer in `src/output.rs` (including `print_response`, `print_error`, `print_error_envelope`, `print_success`, `print_dry_run`, `print_confirmation_required`, `print_message`, `info`, `status`, `warning`, `verbose`, `progress`, `print_stream_line`) through `OutputFormat::is_structured()` so the new formats inherit the machine-readable suppression rules and stderr discipline that `json` and `jsonl` already enforce. by @brettdavies in [#46](https://github.com/brettdavies/xurl-rs/pull/46)
- Extend `RequestOptions` and `CallOptions` with a `pagination_token` field defaulting to an empty string so library consumers can thread cursors without breaking non-paginated callers.
- `AppStatusEntry`, `InstallEnvelope`, `InstallMultiEnvelope`, and `ResolveSource` now derive `schemars::JsonSchema`. `AppStatusEntry` is `pub(crate)` so the `xr schema` module can reference it. by @brettdavies in [#48](https://github.com/brettdavies/xurl-rs/pull/48)
- `xr skill install` and `xr skill update` now clone from `brettdavies/xurl-rs-skill` instead of `brettdavies/xurl-rs`. The skill bundle (`AGENTS.md`, fixtures, templates, evals, scripts) lives in its own repo so the install path no longer pulls the entire `xurl-rs` source tree. Install directories (`~/.claude/skills/xurl-rs/`, `~/.codex/skills/xurl-rs/`, …) stay the same; only the source URL moves. by @brettdavies in [#49](https://github.com/brettdavies/xurl-rs/pull/49)
- `.anc.toml` `domain_verbs` declares the three xr-specific surface verbs (`media`, `examples`, `validate`) alongside the X platform vocabulary, so the `p6-may-standard-names` audit recognizes the full xr command surface once upstream anc-cli honors per-CLI domain vocab. by @brettdavies in [#50](https://github.com/brettdavies/xurl-rs/pull/50)
- `xurl::auth::resolve_bearer_token` now accepts an `app_name: &str` parameter so library consumers can scope the resolution to a specific app rather than the default. Library callers that were using the previous two-argument signature pass `""` for the legacy default-app behavior. by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)

### Fixed

- Bump `rand` to 0.9.4 to clear RUSTSEC-2026-0097 (`ThreadRng` unsoundness when accessed from a custom `log::Logger`). by @brettdavies in [#25](https://github.com/brettdavies/xurl-rs/pull/25)
- Bump `rustls-webpki` to 0.103.13 to clear RUSTSEC-2026-0104 (reachable panic in CRL `IssuingDistributionPoint` parsing, triggerable before signature verification).
- Resolved a private-intra-doc-link in `src/auth/pending.rs` that broke `cargo doc -D warnings` and would render badly on docs.rs. by @brettdavies in [#28](https://github.com/brettdavies/xurl-rs/pull/28)
- Stop wrapping JSON schema bodies in `{"message": "..."}` under `--output json` for `xr schema <command>` and `xr schema --all`, so the output stays valid JSON for downstream consumers. by @brettdavies in [#29](https://github.com/brettdavies/xurl-rs/pull/29)
- Route OAuth2 browser-launch-failure messages through `OutputConfig::print_message` so they respect the configured stderr writer rather than going straight to process stdout.
- Browsers that resolve `localhost` to `[::1]` no longer time out during the OAuth2 flow. The listener dual-binds both loopback addresses when the URI host is `localhost`. by @brettdavies in [#30](https://github.com/brettdavies/xurl-rs/pull/30)
- The browser cannot be opened before the callback listener is actively draining the accept queue, eliminating a race on fast machines.
- Invalid redirect URIs are rejected at write time. `http` is allowed only on loopback hosts; non-loopback `http` and other schemes return a validation error.
- `auth status` and `auth apps list` previously read `~/.xurl` directly through a fresh `TokenStore::new()`, bypassing the runner's configured store path. Both now use `&auth.token_store`, which makes tempdir isolation reliable for library-level tests.
- Stop leaking streaming status messages such as `Connecting to streaming endpoint` and `--- Streaming response started ---` to stderr under JSON mode. by @brettdavies in [#40](https://github.com/brettdavies/xurl-rs/pull/40)
- Clear `XURL_DRY_RUN` before spawning `xr` in U9 subprocess tests so parallel test scheduling cannot leak the var and flake the awaiting_callback assertions. by @brettdavies in [#44](https://github.com/brettdavies/xurl-rs/pull/44)
- `tests/schema_tests.rs::committed_response_schemas_match_runtime` drift guard asserts byte-equality between every `schema/responses/*.schema.json` and the runtime-emitted shape; CI fails on drift with the regen command in the error message. by @brettdavies in [#48](https://github.com/brettdavies/xurl-rs/pull/48)
- `xr <subcommand> --app NAME` now correctly resolves `client_id` and `client_secret` to NAME's stored values across `Auth::with_app_name` and `Auth::with_token_store` switches. The previous "preserve if non-empty" check could not distinguish env-supplied values from values copied off the previous app's store entry during `Auth::new_with_store_path`, so `--app NAME` switches silently re-used the previous app's stored `client_id`. The user-facing symptom was an OAuth2 step-1 URL carrying the wrong `client_id`, yielding a 401 from X. `Auth` now tracks origin via two new `client_id_from_env` and `client_secret_from_env` fields, and `with_token_store` drops the older `self.client_id == old_app.client_id` equality heuristic. by @brettdavies in [#51](https://github.com/brettdavies/xurl-rs/pull/51)
- `xr <subcommand> --app NAME --auth oauth2` could send the request unauthenticated and surface upstream 401s instead of the freshly minted token. `oauth2::refresh_oauth2_token` looked up the cached token via `get_first_oauth2_token()` (no arg), which resolved to the empty-string app name and so `resolve_app` fell back to the default app. A token saved under NAME via `xr auth oauth2 --app NAME` was invisible to the refresh path, the token lookup returned None, the request layer silently skipped the Authorization header (see below), and X returned 401 that masqueraded as a token rejection. Both lookups now use `app_name_lookup` consistently and the named-username branch routes through `get_oauth2_token_for_app`. by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)
- `xr <subcommand> --app NAME --auth oauth1` ignored `--app NAME` and read the default app's OAuth1 credentials. `Auth::get_oauth1_header` now calls `get_oauth1_tokens_for_app(&self.app_name)`. Without this, a token saved under NAME by `xr auth oauth1 --app NAME` was unreachable for subsequent invocations targeting NAME.
- `xr <subcommand> --app NAME --auth app` ignored `--app NAME` and read the default app's bearer. `resolve_bearer_token` now takes an `app_name` parameter and uses `get_bearer_token_for_app`; the internal `Auth::get_bearer_token_header` threads `self.app_name`.
- `xr <subcommand> --app NAME` without an explicit `--auth` flag (auto-detect) probed default-app credentials first via `get_first_oauth2_token`, `get_oauth1_tokens`, and `has_bearer_token` (all no-arg). The auto-detect path now scopes every probe to the active app via `*_for_app(&self.auth.app_name())`.
- `xr auth oauth1 --app NAME …` saved the token to the default app instead of NAME because the CLI handler called `save_oauth1_tokens` (no arg). Now routes through `save_oauth1_tokens_for_app(&candidate)` using the active app name.
- `xr auth app --bearer-token … --app NAME` saved the bearer to the default app instead of NAME. Now routes through `save_bearer_token_for_app(&candidate)`.
- `ApiClient::send_request` (and the multipart and streaming siblings) now propagate auth-resolution errors instead of silently sending the request without an Authorization header. The older `if let Ok(auth) = ...` form let any `XurlError::Auth` from `get_auth_header` disappear into a request that went to X without credentials, which X then rejected with 401. The 401 looked identical to "X rejected our token" even though we never sent one. The new `?` propagation surfaces the real `auth-required` envelope with the token-not-found `message`.

### Documentation

- Document the `docs/solutions` archive and `qmd query` retrieval in `AGENTS.md` so agent consumers can find it from the onboarding doc. by @brettdavies in [#26](https://github.com/brettdavies/xurl-rs/pull/26)
- Harden `RELEASES.md` with triple-diff verification, prose scrubbing, the CHANGELOG generation contract and `cliff.toml` chore-skip footgun, non-draft release behavior, and branch-protection rulesets.
- Rewrite `RELEASES.md` as a pure operational runbook with the branch table, daily-dev flow, PR-body rules, dev to main cherry-pick procedure, tagging table, and prose-scrub procedure; rationale moves out via arrow-links. by @brettdavies in [#27](https://github.com/brettdavies/xurl-rs/pull/27)
- Document new CLI surface (per-app redirect URI, `-u USERNAME` fallback, structured output) in README by @brettdavies in [#32](https://github.com/brettdavies/xurl-rs/pull/32)
- Document X platform "Pay-per-use Production" enrollment workaround in README
- Document the headless OAuth2 flow on `xr auth oauth2 --help` with `--no-browser --step 1` and `--step 2 --auth-url '<paste>'` invocations for servers, containers, and SSH sessions. by @brettdavies in [#35](https://github.com/brettdavies/xurl-rs/pull/35)
- Document env-var and flag precedence on `xr search --help` via the `XURL_OUTPUT=json xr search "rustlang" -n 25` example.
- Document the envelope variants, the closed `reason` vocabulary, and the regeneration command for `schema/output.schema.json` in the `xurl::envelope` module rustdoc. by @brettdavies in [#38](https://github.com/brettdavies/xurl-rs/pull/38)
- Update `xr schema --list` rustdoc and the `xr schema` usage string to advertise the new `envelope` row and `xr schema envelope` invocation.
- Note in root `xr --help` that `xr` writes directly to stdout and stderr and never invokes `$PAGER`, so output is pipe-safe by default and no `--no-pager` flag is required. by @brettdavies in [#41](https://github.com/brettdavies/xurl-rs/pull/41)
- Add 11 output-writer tests, 8 agentic subprocess tests, and 2 wiremock pagination tests covering each new format, the cursor query-parameter threading and URL encoding, the unsupported-pagination envelope, and ok and fail paths for `xr validate`. by @brettdavies in [#46](https://github.com/brettdavies/xurl-rs/pull/46)
- Split the bearer preflight gate in `RELEASES-PREFLIGHT.md` into two subgates so the env-var one-shot path (`XURL_BEARER_TOKEN=… xr <shortcut> --auth app`) and the stored two-step path (`xr auth app --bearer-token` then `--auth app`) are both explicitly verified before tagging. The single combined line previously documented only the env-var inline form, which the bearer resolver did not honor until this PR. by @brettdavies in [#51](https://github.com/brettdavies/xurl-rs/pull/51)
- Add a Multi-app credential routing section to RELEASES-PREFLIGHT.md with seven gates covering OAuth1, OAuth2, and bearer save/read isolation across at least two registered apps, auto-detect with `--app NAME`, the first-signed-in-app auto-default behavior on every sign-in handler, promotion idempotence, and the auth-error envelope vs upstream 401 distinction. Replaces the implicit assumption that one default app sufficed for live smoke. by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)

### Tests

- 11 red team tests in `tests/auth_tests.rs`: 7 cover bearer resolution (env-only, env-overrides-store, env-empty falls through, env-unset falls through, neither-set errors, env-empty + store-empty errors, real-env integration via `std::env::set_var` behind an `unsafe` guard mirroring `test_env_redirect_uri_wins_over_app_stored`); 4 cover app-switch precedence (env-supplied `client_id` survives a switch, store-derived `client_id` re-resolves to new app's stored value, round-trip default to bird-dev to default re-resolves correctly, `client_secret` follows the same contract). by @brettdavies in [#51](https://github.com/brettdavies/xurl-rs/pull/51)
- `tests/auth_tests.rs`: 3 red team tests for OAuth2 refresh-context (named app with named username, named app with empty username uses default_user precedence, salvage-state unnamed slot resolution). by @brettdavies in [#52](https://github.com/brettdavies/xurl-rs/pull/52)
- `tests/auth_tests.rs`: 6 multi-app tests covering OAuth1 + bearer + OAuth2 across two apps and runtime `--app NAME` switching, plus env-var precedence over per-app stores.
- `tests/api_tests.rs`: 2 tests asserting `XurlError::Auth` propagates from `ApiClient::get_me` when no token is in the store for the requested auth path, instead of falling through to a wiremock-absent network error.
- `tests/store_tests.rs`: 7 tests covering the promotion contract: promotes on uninitialized default, no-op when default already has bearer (or any other credential), no-op on unknown candidate, on empty candidate, on candidate==default, treats unnamed salvage tokens as "credentialed", and the underlying `default_app_is_uninitialized` signal.

**Full Changelog**: [v1.2.0...v1.3.0](https://github.com/brettdavies/xurl-rs/compare/v1.2.0...v1.3.0)

## [1.2.0] - 2026-04-16

### Added

- Add `ApiClient::from_env()` one-liner constructor that reads `CLIENT_ID`/`CLIENT_SECRET` from environment variables by @brettdavies in [#21](https://github.com/brettdavies/xurl-rs/pull/21)
- Add `CallOptions` consumer-facing struct for shortcut methods, exposing only `auth_type`, `username`, `no_auth`, `verbose`, `trace`
- Add `no_auth` field to skip authentication entirely on a per-request basis
- Add `XurlError::Validation(String)` variant for non-HTTP validation errors (e.g., errors-only 200 responses)

### Changed

- Change `ApiClient` from borrowed `&mut Auth` to owned `Auth` — no lifetime parameter, storable in structs by @brettdavies in [#21](https://github.com/brettdavies/xurl-rs/pull/21)
- Change 29 shortcut functions from free functions (`api::create_post(&mut client, ...)`) to methods (`client.create_post(...)`)
- Change `XurlError::Api(String)` to `Api { status: u16, body: String }` for structured HTTP error matching
- Change `exit_code_for_error()` to pattern-match on `Api { status, .. }` directly instead of string matching

### Fixed

- Bump `rustls-webpki` to 0.103.12 to clear [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098) and [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099) (name constraint validation). by @brettdavies in [#23](https://github.com/brettdavies/xurl-rs/pull/23)

### Documentation

- Document exit code mapping improvement as known difference from Go version in KNOWN_DIFFERENCES.md by @brettdavies in [#21](https://github.com/brettdavies/xurl-rs/pull/21)

**Full Changelog**: [v1.1.0...v1.2.0](https://github.com/brettdavies/xurl-rs/compare/v1.1.0...v1.2.0)

## [1.1.0] - 2026-04-02

### Added

- Add `xr usage` shortcut command that returns full API usage data (tweet caps, daily project breakdown, per-app breakdown) by @brettdavies in [#13](https://github.com/brettdavies/xurl-rs/pull/13)
- Add `--remote` flag for headless OAuth2 authentication on machines without a browser by @brettdavies in [#14](https://github.com/brettdavies/xurl-rs/pull/14)
- Add `--step` (1 or 2) and `--auth-url` (with `-` for stdin) companion flags
- Add JSON output support for step 1 (`--output json` emits `{"auth_url": "...", "instructions": "..."}`)
- Add typed response structs: `Tweet`, `User`, `DmEvent`, `UsageData`, 7 action confirmations, 3 wrapper/meta types, 3 nested types by @brettdavies in [#17](https://github.com/brettdavies/xurl-rs/pull/17)
- Add `deserialize_response<T>()` helper with guards for empty and errors-only 200 responses
- Add `ApiResponse<T>` generic wrapper with `data`, `includes`, `meta`, `errors`, and forward-compatible `extra` fields
- Add `xr schema <command>` to output JSON Schema for any command's response type by @brettdavies in [#18](https://github.com/brettdavies/xurl-rs/pull/18)
- Add `xr schema --list` to show all 29 commands with their response types
- Add `xr schema --all` to output all schemas as a single JSON document
- Add `schemars` dependency for compile-time JSON Schema generation via `#[derive(JsonSchema)]`

### Changed

- Rename `--remote` to `--no-browser` for the headless OAuth2 authentication flow by @brettdavies in [#15](https://github.com/brettdavies/xurl-rs/pull/15)
- Change all 29 shortcut functions from `Value` returns to typed `ApiResponse<T>` returns (**breaking** for library consumers) by @brettdavies in [#17](https://github.com/brettdavies/xurl-rs/pull/17)

### Fixed

- Rename completion files to standard convention (`xr.zsh`, `xr.elvish`, `xr.powershell`) by @brettdavies in [#11](https://github.com/brettdavies/xurl-rs/pull/11)
- Regenerate bash and fish completions for completions subcommand
- Fix test isolation in `Auth::with_token_store()` where real `~/.xurl` credentials leaked into test assertions by @brettdavies in [#12](https://github.com/brettdavies/xurl-rs/pull/12)

### Documentation

- Add shell completions regeneration step to the release process as a safety net for missed completions during development by @brettdavies in [#13](https://github.com/brettdavies/xurl-rs/pull/13)

**Full Changelog**: [v1.0.5...v1.1.0](https://github.com/brettdavies/xurl-rs/compare/v1.0.5...v1.1.0)

## [1.0.5] - 2026-03-21

### Added

- `xr completions <shell>` subcommand replacing hidden `--generate-completion` flag by @brettdavies in [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- PowerShell and Elvish shell completions by @brettdavies in [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- cargo-deny license and advisory auditing via `deny.toml` by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- Draft-then-finalize release pattern via `finalize-release.yml`
- Commit provenance guard requiring PRs go through dev before main
- Changelog CI enforcement — PRs to main must include CHANGELOG.md updates
- `cargo binstall` support for pre-built binary installs

### Changed

- Version and completions commands now exit before config/auth initialization by @brettdavies in [#6](https://github.com/brettdavies/xurl-rs/pull/6)
- CI/CD migrated to centralized reusable workflows from `brettdavies/.github` by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- `ci.yml`, `release.yml`, `guard-main-docs.yml` replaced with thin callers
- Homebrew dispatch secret migrated from `HOMEBREW_TAP_TOKEN` to `CI_RELEASE_TOKEN`
- Release archives now include completions, licenses, README, and sha256sums

### Fixed

- `protect-dev.json` aligned with bird (add deletion, non_fast_forward rules, admin bypass) by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)
- Update `rustls-webpki` to 0.103.10 to fix [RUSTSEC-2026-0049](https://rustsec.org/advisories/RUSTSEC-2026-0049)
- Move `thiserror`, `anyhow`, `dirs`, `percent-encoding` to platform-independent `[dependencies]` — fixes Windows build by @brettdavies in [#9](https://github.com/brettdavies/xurl-rs/pull/9)

### Documentation

- RELEASING.md rewritten for reusable workflow pipeline and changelog-as-committed-artifact by @brettdavies in [#7](https://github.com/brettdavies/xurl-rs/pull/7)

**Full Changelog**: [v1.0.4...v1.0.5](https://github.com/brettdavies/xurl-rs/compare/v1.0.4...v1.0.5)

## [1.0.4] - 2026-03-16

### Changed

- Switch to Trusted Publishing (OIDC) for crates.io authentication — no static secrets by @brettdavies in
  [#4](https://github.com/brettdavies/xurl-rs/pull/4)
- Pin all GitHub Actions by commit SHA for supply-chain security by @brettdavies in
  [#4](https://github.com/brettdavies/xurl-rs/pull/4)
- Switch reqwest from native-tls to rustls-tls for cross-compilation compatibility by @brettdavies in
  [#3](https://github.com/brettdavies/xurl-rs/pull/3)
- Update macOS CI runner from deprecated `macos-13` to `macos-latest` by @brettdavies in
  [#3](https://github.com/brettdavies/xurl-rs/pull/3)
- Opt into Node.js 24 for GitHub Actions (`FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`) by @brettdavies in
  [#4](https://github.com/brettdavies/xurl-rs/pull/4)

**Full Changelog**: [v1.0.3...v1.0.4](https://github.com/brettdavies/xurl-rs/compare/v1.0.3...v1.0.4)

## [1.0.3] - 2026-03-16

### Added

- Full xurl-rs implementation — Rust port of Go [xurl](https://github.com/xdevplatform/xurl) CLI by @brettdavies in
  [#2](https://github.com/brettdavies/xurl-rs/pull/2)
- 28 shortcut commands: post, reply, quote, delete, read, search, like, repost, bookmark, follow, block, mute, dm,
  timeline, mentions, whoami, and more
- Raw API mode: `xr /2/users/me`, `xr -X POST /2/tweets -d '{...}'`
- OAuth2 PKCE, OAuth1 HMAC-SHA1, and Bearer token authentication
- YAML token store with multi-app management at `~/.xurl`
- Media upload (chunked) with status polling
- Shell completions for bash, zsh, fish, powershell, elvish
- Agent-native features: `--output json/jsonl`, `--quiet`, `--no-interactive`, structured exit codes (0-5)
- `NO_COLOR` and `XURL_OUTPUT` environment variable support
- Release infrastructure: CI, cross-platform builds (5 targets), crates.io Trusted Publishing, Homebrew tap dispatch

### Fixed

- Switch reqwest from native-tls to rustls-tls for cross-compilation by @brettdavies in
  [#3](https://github.com/brettdavies/xurl-rs/pull/3)

### New Contributors

- @brettdavies made their first contribution in [#2](https://github.com/brettdavies/xurl-rs/pull/2)
