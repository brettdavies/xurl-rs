# Known Differences from Go xurl

Each difference below holds against Go `xurl` 1.3.4, the release the `Go parity` CI job runs the differential suite
against.

## Command names that differ, and commands only one tool has (intentional difference)

`xr` reads the token store Go `xurl` 1.3.4 writes, `~/.xurl/auth.yml`, so the apps and credentials stored with Go are
there on the first `xr` run.

Every Go command not named below has an `xr` command of the same name. These differ:

| Go `xurl` 1.3.4                          | `xr`                                                                      | `xr` given the Go form                                                                                        |
| ---------------------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `xurl posts USERNAME`                    | `xr user-posts USERNAME`                                                  | `xr posts` reads 1 to 100 posts by id or URL, so `xr posts USERNAME` asks for a post whose id is the username |
| `xurl auth app-only`, `xurl auth bearer` | `xr auth app` (`xurl auth app` works in both)                             | Exit 2, `unknown-command`                                                                                     |
| `xurl auth oauth2 --headless`            | `xr auth oauth2 --no-browser`, in two steps (`--step 1`, then `--step 2`) | Exit 2, `invalid-args`                                                                                        |
| `xurl auth clear --app-only`             | `xr auth clear --bearer` (`--bearer` works in both)                       | Exit 2, `invalid-args`                                                                                        |
| `xurl completion SHELL`                  | `xr completions SHELL`                                                    | Exit 2, `unknown-command`, suggests `completions`                                                             |
| `xurl webhook start`                     | `xr webhooks listen` (next section)                                       | Exit 2, `unknown-command`, suggests `webhooks`                                                                |

Three Go commands have no `xr` counterpart, and `xr chat`, `xr mcp`, and `xr token` exit 2 with reason
`unknown-command`:

- `xurl chat`, an end-to-end encrypted XChat client, built into Go's release binaries for macOS and Linux x86_64.
- `xurl mcp`, a stdio bridge from an MCP client to X's MCP server.
- `xurl token`, which prints the active app's OAuth2 access token.

Every other `xr` command has no Go counterpart, and `xr --help` lists them all. With Go, their endpoints are reached
through raw requests, and a Go sign-in does not request the `calls.read` and `calls.write` scopes the `xr calls`
endpoints take.

## Webhooks: no built-in tunnel, verified events, and management commands (intentional difference)

Go `xurl` has one webhook command, `xurl webhook start`. It opens an ngrok tunnel itself (prompting for an authtoken or
reading `NGROK_AUTHTOKEN`), answers X's CRC check, logs each POST body, and prints the public URL for the user to
register by other means. `xr` has a `webhooks` family, and its receiver differs in four ways.

- **No tunnel.** `xr webhooks listen` binds a local address and stops there. X delivers only to a public HTTPS URL with
  no port, and the caller provides it with a tunnel or reverse proxy of their own. Linking a tunnel client into the
  binary would add a third-party account and a large dependency for one command.
- **The signing secret X documents.** Go keys the CRC response with the OAuth1 consumer secret and refuses to start
  without OAuth1 credentials. X's webhook documentation says to use the app's OAuth2 client secret when it has one and
  the consumer secret otherwise. `listen` follows the documentation; `--secret oauth1` selects Go's behavior.
- **Events are verified.** Go prints every POST. X signs each event (`X-Twitter-Webhooks-Signature-OAuth2`, or the
  legacy `X-Twitter-Webhooks-Signature`), and `listen` prints an event only when its signature verifies, answering 401
  otherwise. `--allow-unsigned` prints a POST with no signature.
- **Output is JSON Lines on stdout.** Go logs bodies to the terminal with `-q` and `-P` to shape them and `-o` to append
  them to a file. `listen` writes one event per line to stdout, so a shell redirect or `jaq` does the rest, and
  `--max-events` bounds the run.

The management commands have no Go counterpart: `xr webhooks list`, `add`, `validate`, `remove`, and `replay`,
`xr webhooks subscriptions` for Account Activity, and `xr webhooks stream-links` for filtered-stream delivery. With Go,
those endpoints are reached through raw requests.

Go neither forwards events nor recovers from a restart. `xr webhooks listen --forward` POSTs each verified event to a
URL of the caller's, a program after `--` runs once per event, and `--checkpoint` records how far `listen` got, which
`xr webhooks replay --from-checkpoint` turns into the window X can replay.

## Exit code mapping for HTTP errors (intentional improvement)

Go `xurl` exits 1 on every failure: each `os.Exit` in its source passes 1, and `main.go` exits 1 when Cobra returns an
error. A 404, a 401, and a 429 all end at exit 1.

The Rust version maps the HTTP status itself, through `xdk::Error::Api { status, .. }`, to an exit code, so these exit
codes differ:

| Scenario                                      | Go exit code | Rust exit code     | Rust is more correct |
| --------------------------------------------- | ------------ | ------------------ | -------------------- |
| 404 with JSON body (no literal "404" in body) | 1 (general)  | 4 (not found)      | Yes                  |
| 401 with JSON body (no literal "401" in body) | 1 (general)  | 77 (auth required) | Yes                  |
| 429 with JSON body (no literal "429" in body) | 1 (general)  | 3 (rate limited)   | Yes                  |

The structural mapping is intentional.

## API errors go to stderr, never stdout (intentional improvement)

Go `xurl` prints an API error's body on stdout and exits 1 (`printResult` in `cli/shortcuts.go`, whose comment says
error bodies go to stdout "so they can be piped/parsed the same way as a successful response"). A raw request adds
`Error: request failed` on stderr. A flag error prints twice: Cobra writes `Error: unknown flag: --bogus` and the usage
to stderr, and `main.go` prints the error again on stdout.

`xr` keeps stdout for the command's answer and writes every failure to stderr: a line of text by default, or under
`--output json` an envelope carrying `reason`, `exit_code`, `message`, and a `next_step` where one exists. A 429 that
names its reset also carries `retry_after_secs` and `retry_at`.

| Request                      | Go `xurl`                                                 | `xr`                                                                                  |
| ---------------------------- | --------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `read ID`, X answers 404     | Body on stdout, stderr empty, exit 1                      | stdout empty, `not-found` on stderr, exit 4                                           |
| `/2/users/me`, X answers 401 | Body on stdout, `Error: request failed` on stderr, exit 1 | stdout empty, `auth-required` on stderr, exit 77                                      |
| `read ID`, X answers 429     | Body on stdout, exit 1                                    | stdout empty, `rate-limited` with `retry_after_secs` and `retry_at` on stderr, exit 3 |
| `--bogus`                    | `unknown flag: --bogus` on stdout and on stderr, exit 1   | stdout empty, `invalid-args` on stderr, exit 2                                        |

A script that read Go's error body from stdout reads the envelope from stderr instead, and branches on its `reason` or
on the exit code:

```bash
xr --output json read 1585341984679469056 2>err.json || jaq -r '.reason' err.json
```

## A bare word is a command, not an endpoint (intentional improvement)

Go `xurl` accepts any positional and sends it as the endpoint (`cli/root.go`), so a mistyped command name becomes a
request against a path that does not exist and the caller reads an API error instead of a spelling correction. Go reads
only the first positional and drops every word after it, so `xurl lists list` requests `/lists`.

The Rust version classifies the positional after the parse and before anything is loaded or sent. A token that reads as
a command name and matches none of them is a usage error at exit `2` with reason `unknown-command`, carrying the
offending word in `command` and the nearest real name in `suggestion` when one is close enough:

| Invocation                      | Go behavior                          | Rust behavior                                         |
| ------------------------------- | ------------------------------------ | ----------------------------------------------------- |
| `xr whoam`                      | Request to `/whoam`, API error       | Exit 2, `unknown-command`, suggests `whoami`          |
| `xr zzzzzz`                     | Request to `/zzzzzz`, API error      | Exit 2, `unknown-command`, no suggestion              |
| `xr lists list`                 | Request to `/lists`, `list` dropped  | Lists your lists                                      |
| `xr likers 1585341984679469056` | Request to `/likers`, the id dropped | Lists the users who liked the post                    |
| `xr example.com`                | Request to `/example.com`            | Exit 1, `validation` — a URL, and not an absolute one |
| `xr`                            | Usage error                          | Exit 0, root help on stdout                           |

A help or version flag does not change that outcome: `xr whoam --help`, `xr whoam -h`, and `xr whoam --version` fail
exactly as `xr whoam` does. Neither does a word after it: `xr suth status` fails as `xr suth` does, naming `suth` and
suggesting `auth`.

A positional that starts with `http://`, `https://`, or `/` is still a raw request, and so is any invocation carrying a
raw-only flag (`-X`, `-H`, `-d`, `-F`, `-u`, `--auth`, `-t`, `-s`), which no command reads. A word after a raw target is
an unexpected argument (`invalid-args`), never a command. A raw request has no help page of its own, so
`xr /2/users/me --help` prints the root help.

## A raw-only flag before a command word is refused (intentional difference)

Go `xurl` defines `-X`, `-H`, `-d`, `--auth`, `-u`, `-t`, `-s`, and `-F` on its root command (`cli/root.go`), and every
shortcut command defines its own `--auth`, `-u`, and `-t` (`addCommonFlags` in `cli/shortcuts.go`). Cobra parses the
flags given before a command name against that command's own, so Go applies `--auth`, `-u`, and `-t` there as if they
followed the command word, and fails on any other raw-only flag as one the command does not define.

In the Rust version clap reads root flags only before the command word, and each command reads its own copy, so the
root's copy of a flag given there would go unread: `xr -u work whoami` would answer as the default login. Such an
invocation is a usage error at exit `2` with reason `invalid-args`. When each misplaced flag is `-u`, `--auth`, or `-t`
and the command takes it, `next_step` is `fix-input` carrying the invocation with the flags after the command word;
otherwise it is `show-help` with the command's help:

| Invocation             | Go behavior                                 | Rust behavior                                       |
| ---------------------- | ------------------------------------------- | --------------------------------------------------- |
| `xr -u work whoami`    | Runs `whoami -u work`                       | Exit 2, `invalid-args`, step `xr whoami -u work`    |
| `xr --auth app whoami` | Runs `whoami --auth app`                    | Exit 2, `invalid-args`, step `xr whoami --auth app` |
| `xr -X POST whoami`    | Exit 1, `unknown shorthand flag: 'X' in -X` | Exit 2, `invalid-args`, step `xr whoami --help`     |

A raw request keeps every one of these flags: `xr -u work /2/users/me` is unchanged.

## Every stream the spec declares is streamed (intentional improvement)

Go `xurl` streams a raw request without `-s` only when its path is on a fixed list of eight: the filtered search stream,
the sample and sample10 streams, and the firehose stream with its four language variants. Any other path is read as one
buffered response, so nothing from a long-lived stream missing from the list prints as it arrives.

The Rust version streams every path whose operation the vendored X OpenAPI spec marks `x-twitter-streaming`, and the
build fails if a spec refresh leaves none marked. That set covers Go's eight plus seven more:

| Path                          | Go without `-s` | Rust without `-s` |
| ----------------------------- | --------------- | ----------------- |
| `/2/activity/stream`          | Buffered        | Streamed          |
| `/2/likes/firehose/stream`    | Buffered        | Streamed          |
| `/2/likes/sample10/stream`    | Buffered        | Streamed          |
| `/2/tweets/label/stream`      | Buffered        | Streamed          |
| `/2/tweets/compliance/stream` | Buffered        | Streamed          |
| `/2/likes/compliance/stream`  | Buffered        | Streamed          |
| `/2/users/compliance/stream`  | Buffered        | Streamed          |

`-s` still forces streaming on any path in both.

## OAuth1 signatures use RFC 5849 percent-encoding (intentional improvement)

Go `xurl` percent-encodes OAuth1 parameters with `url.QueryEscape` (`auth/auth.go`), a query-string encoder: a space
becomes `+`, `~` becomes `%7E`, and `*` stays bare. RFC 5849 section 3.6 leaves only the RFC 3986 unreserved characters
bare (letters, digits, `-`, `.`, `_`, `~`) and writes every other byte as `%XX`. X documents the RFC's encoding for the
signature it recomputes, so an OAuth1 request whose query or body value carries one of those characters signs a string
that differs from the one X builds.

The Rust version encodes the signature base string, the signing key, and the `Authorization` header parameters as the
RFC says. It reproduces the `oauth_signature` of X's published example,
[Creating a signature](https://docs.x.com/resources/fundamentals/authentication/oauth-1-0a/creating-a-signature), whose
`status` value contains spaces, a `+`, a comma, and a `!`:

| Value | Go `encode` | Rust `encode` |
| ----- | ----------- | ------------- |
| `a b` | `a+b`       | `a%20b`       |
| `~`   | `%7E`       | `~`           |
| `*`   | `*`         | `%2A`         |

A request with none of those characters in a signed value produces the same signature in both.

## A wait on media processing has a deadline (intentional improvement)

Go `xurl` waits on media processing in a loop with no deadline (`api/media.go`, `WaitForProcessing`): it polls until the
status reads `succeeded` or `failed`. A status that carries no `processing_info`, which is what an image reports, reads
as neither, so that wait never ends, and a job X never finishes is polled for as long as the process lives. Its `--wait`
is a boolean flag.

The Rust version always ends the wait:

| Situation                           | Go behavior                     | Rust behavior                                            |
| ----------------------------------- | ------------------------------- | -------------------------------------------------------- |
| Status carries no `processing_info` | Polls once a second without end | Returns the status after one call                        |
| Job still running                   | Polls without a deadline        | Exit 1, reason `processing-timeout`, at the deadline     |
| `--wait`, `--wait=true`             | Waits                           | Waits up to 60 seconds                                   |
| `--wait=<SECS>`                     | Accepts only `0` or `1`         | Waits up to that many seconds; `0` does not wait         |
| `--wait=false`                      | Does not wait                   | Does not wait                                            |
| `--wait 60` (value after a space)   | Not the flag's value            | Exit 2, with a tip pointing at `--wait=60`               |

Which uploads are waited for differs too. Go `xurl` decides by category alone: one whose name contains `video` or
`gif` (`mediaNeedsProcessing`). The Rust version waits for a video category, and for any other upload whose FINALIZE
answer carries a `processing_info` that is not yet final, which is the signal X documents for media that needs
processing. X answers FINALIZE for an animated GIF with `processing_info.state` `pending`, so both tools wait for one. An
image X reports as ready costs no status call here, whatever its category.

`media upload` still waits by default and `media status` still reads the status once unless `--wait` is given. The
timeout leaves the upload intact: its envelope carries `media_id` and a `resume-wait` step whose `command`, `xr media
status <media_id> --wait=<secs>`, waits again for twice as long.

## A waited `media upload` answers one document (intentional improvement)

Go `xurl` prints the FINALIZE response and, when it waited, the final status response after it (`api/media.go`,
`ExecuteMediaUpload`): two JSON documents on stdout, so a caller that parses the output once fails on the second and
`.data.id` reads twice.

The Rust version prints one document. It is FINALIZE's, with `processing_info` replaced by the final status's, or
removed when that status carries none, and with any other field only the status carried added:

```json
{
  "data": {
    "id": "1880028106020515840",
    "media_key": "7_1880028106020515840",
    "expires_after_secs": 86400,
    "processing_info": { "state": "succeeded", "progress_percent": 100 }
  }
}
```

An upload that did not wait prints FINALIZE's document unchanged, and so does one whose wait failed or timed out, with
the error on stderr. Under `--verbose`, INIT's response is printed ahead of it in text output only.

## The caller's id is asked for once per login (intentional improvement)

A command that acts as the caller (`like`, `follow`, `timeline`, `bookmarks`, and the rest) needs the caller's id in its
path. Go `xurl` asks `/2/users/me` for it on every run (`cli/shortcuts.go`, `resolveMyUserID`), or
`/2/users/by/username/<name>` under `--username`. Each is a billed user read.

The Rust version stores the id beside the login the first time `/2/users/me` answers under it: at sign-in, at the
refresh of a login that has none, on `xr whoami`, or on the first command that needs it. Later commands read it from the
store and send one request where Go sends two. The fields are `user_id`, `account_username`, and `display_name` on the
OAuth2 token and on the OAuth1 access pair in `~/.xurl/auth.yml`. Go `xurl` does not read them, and drops them when it
rewrites a login, after which `xr` asks once more. `xr whoami --cached` prints the stored account with no request;
Go's `whoami` always asks.

The id is written only from an answer given under that same credential, and a credential that replaces another starts
without one, so it cannot name a different account. The username and display name of each answer are written beside
every login the store holds for that id, in every app, so a rename seen through one login shows through all of them.
Another user's handle is looked up on every run in both tools.

## `--dry-run` checks any command offline (intentional improvement)

Go `xurl` has no `--dry-run`: `xurl --dry-run /2/users/me` and `xurl delete --dry-run ID` exit 1 with
`unknown flag: --dry-run`, and nothing is sent.

`xr` takes `--dry-run` on every command, raw requests included. It checks the inputs and the stored credential offline
and sends and stores nothing. A dry run that reaches its checks answers on stdout at exit 0 with `status: dry_run`,
`would_succeed`, the credential the request would go out under, and, when the real run would fail, the `reason` and
`exit_code` it would end on.

`XURL_DRY_RUN=1` makes every `xr` call in a process a dry run, so a migrated script can be checked whole before it
spends anything:

```bash
XURL_DRY_RUN=1 ./migrated-script.sh
```

## `delete` and the store commands ask first (intentional improvement)

Go's `xurl delete ID` sends the `DELETE` at once, and `xurl auth clear --all` and `xurl auth apps remove NAME` change
the store at once. Go asks nothing and has no `--force`.

`xr` asks before each of these. On a terminal it prompts on stderr and proceeds on `y`. Under `--no-interactive` or
`--quiet`, or when stdin or stderr is not a terminal, it refuses at exit 1 with reason `confirmation-required` and a
`next_step` of `confirm`, whose `template` is the invocation with `--force`. `--force` proceeds without asking. A dry
run reports `confirmation_required: true` for every `xr` command that asks, including the ones with no Go counterpart.

A script that deleted with Go adds `--force`:

```bash
xr delete 1585341984679469056 --force
```

## A credential the endpoint does not accept is refused before sending (intentional improvement)

Go `xurl` picks a credential by kind, whatever the endpoint accepts: the first OAuth2 token, then OAuth1, then the
bearer token (`getAuthHeader` in `api/client.go`). With only a bearer token stored, `xurl like ID` sends
`GET /2/users/me` with it and reports X's refusal.

`xr` reads the schemes each endpoint accepts from the vendored X OpenAPI spec and checks them against the stored
credentials before sending. With only a bearer token stored, `xr like ID` exits 2 with reason `auth-method-mismatch` and
sends nothing, and its `next_step` says how to add a credential the endpoint accepts.

## `--cursor` asks for the next page (intentional improvement)

Go `xurl`'s list commands take `-n`/`--max-results` for the size of one page and nothing for the next;
`xurl search QUERY --cursor TOKEN` exits 1 with `unknown flag: --cursor`. The next page is a raw request with
`pagination_token`.

`xr`'s list commands take the same `-n`. On a list command whose endpoint pages, `--cursor TOKEN`, or `--after TOKEN`,
sends the token as X's `pagination_token`; an endpoint X does not page, such as Spaces search, takes none. `--page`
exits 1 with reason `unsupported-pagination`, since X pages by token rather than by number.

A script that paged with a raw request passes the token to the command instead:

```bash
xr search "from:XDevelopers" --cursor TOKEN
```

## Shortcut output uses the spec's post vocabulary (intentional difference)

Go `xurl` requests fields under X's legacy names (`tweet.fields`, `expansions=referenced_tweets.id`) and prints X's body
as sent (`api/shortcuts.go`, `printResult` in `cli/shortcuts.go`). Where X answers in its legacy post vocabulary, Go's
output carries those keys.

`xr` requests fields under the names the vendored spec gives them (`post.fields`, `referenced_posts`), and a shortcut
prints each legacy key under the spec's name with its value unchanged; the CLI README's "Machine-Readable Output"
section lists the renamed keys. A raw `xr` request prints X's keys as sent.

| Key in X's answer        | `xurl read`              | `xr read`               | `xr /2/tweets/ID`        |
| ------------------------ | ------------------------ | ----------------------- | ------------------------ |
| `retweet_count`          | `retweet_count`          | `repost_count`          | `retweet_count`          |
| `referenced_tweets`      | `referenced_tweets`      | `referenced_posts`      | `referenced_tweets`      |
| `edit_history_tweet_ids` | `edit_history_tweet_ids` | `edit_history_post_ids` | `edit_history_tweet_ids` |

A filter written against Go's output reads the spec's names from an `xr` shortcut: `.public_metrics.retweet_count`
becomes `.public_metrics.repost_count`.

## Every secret can come from a file or stdin (intentional improvement)

Go `xurl` takes the bearer token as an argument, through `--bearer-token`, or from stdin when the argument is `-`
(`auth app-only` in `cli/auth.go`). The client secret of `auth apps add` and `auth apps update`, and the OAuth1 consumer
secret, access token, and token secret of `auth oauth1`, are flag values only, apart from the `CLIENT_SECRET` Go also
reads from the environment. A flag value shows in the shell history and in any process listing.

`xr` gives each of these a flag that reads the secret from a path, or from stdin as `-`: `--client-secret-file`,
`--consumer-secret-file`, `--access-token-file`, `--token-secret-file`, and `--bearer-token-file`. The flag-value forms
work too.

```bash
<secret-command> | xr auth apps add my-app --client-id <client-id> --client-secret-file -
```
