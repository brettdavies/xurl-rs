# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_xr_global_optspecs
    string join \n X/method= H/header= d/data= auth= u/username= v/verbose= t/trace s/stream F/file= app= output= json jsonl raw= no-pager q/quiet= no-interactive= timeout= wait-on-rate-limit= rate-limit-max-wait= color= dry-run= limit= cursor= page= after= h/help V/version
end

function __fish_xr_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_xr_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_xr_using_subcommand
    set -l cmd (__fish_xr_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c xr -n "__fish_xr_needs_command" -s X -l method -d 'HTTP method (GET by default)' -r
complete -c xr -n "__fish_xr_needs_command" -s H -l header -d 'Request headers' -r
complete -c xr -n "__fish_xr_needs_command" -s d -l data -d 'Request body data' -r
complete -c xr -n "__fish_xr_needs_command" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_needs_command" -s u -l username -d '`OAuth2` login to send the request as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_needs_command" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -s F -l file -d 'File to upload (for multipart requests)' -r
complete -c xr -n "__fish_xr_needs_command" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_needs_command" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_needs_command" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_needs_command" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_needs_command" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_needs_command" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_needs_command" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_needs_command" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_needs_command" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_needs_command" -s t -l trace -d 'Add trace header to request'
complete -c xr -n "__fish_xr_needs_command" -s s -l stream -d 'Force streaming mode'
complete -c xr -n "__fish_xr_needs_command" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_needs_command" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_needs_command" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_needs_command" -s V -l version -d 'Print version'
complete -c xr -n "__fish_xr_needs_command" -a "post" -d 'Post to X'
complete -c xr -n "__fish_xr_needs_command" -a "reply" -d 'Reply to a post'
complete -c xr -n "__fish_xr_needs_command" -a "quote" -d 'Quote a post'
complete -c xr -n "__fish_xr_needs_command" -a "delete" -d 'Delete a post'
complete -c xr -n "__fish_xr_needs_command" -a "read" -d 'Read a post'
complete -c xr -n "__fish_xr_needs_command" -a "analytics" -d 'Impressions and engagement for your posts over a time window'
complete -c xr -n "__fish_xr_needs_command" -a "search-counts" -d 'Count the recent posts that match a query, per time bucket'
complete -c xr -n "__fish_xr_needs_command" -a "search-users" -d 'Find accounts by keyword'
complete -c xr -n "__fish_xr_needs_command" -a "users" -d 'Look up 1 to 100 accounts by handle or by id in one request'
complete -c xr -n "__fish_xr_needs_command" -a "user-posts" -d 'List the posts a user wrote'
complete -c xr -n "__fish_xr_needs_command" -a "read-posts" -d 'Read up to 100 posts in one request'
complete -c xr -n "__fish_xr_needs_command" -a "search" -d 'Search recent posts'
complete -c xr -n "__fish_xr_needs_command" -a "likers" -d 'List the users who liked a post'
complete -c xr -n "__fish_xr_needs_command" -a "reposters" -d 'List the users who reposted a post'
complete -c xr -n "__fish_xr_needs_command" -a "quotes" -d 'List the posts that quote a post'
complete -c xr -n "__fish_xr_needs_command" -a "whoami" -d 'Show the authenticated user\'s profile'
complete -c xr -n "__fish_xr_needs_command" -a "user" -d 'Look up a user by username'
complete -c xr -n "__fish_xr_needs_command" -a "timeline" -d 'Show your home timeline'
complete -c xr -n "__fish_xr_needs_command" -a "mentions" -d 'Show your recent mentions'
complete -c xr -n "__fish_xr_needs_command" -a "like" -d 'Like a post'
complete -c xr -n "__fish_xr_needs_command" -a "unlike" -d 'Unlike a post'
complete -c xr -n "__fish_xr_needs_command" -a "hide" -d 'Hide a reply to your post'
complete -c xr -n "__fish_xr_needs_command" -a "unhide" -d 'Show a hidden reply to your post again'
complete -c xr -n "__fish_xr_needs_command" -a "repost" -d 'Repost a post'
complete -c xr -n "__fish_xr_needs_command" -a "unrepost" -d 'Undo a repost'
complete -c xr -n "__fish_xr_needs_command" -a "bookmark" -d 'Bookmark a post'
complete -c xr -n "__fish_xr_needs_command" -a "unbookmark" -d 'Remove a bookmark'
complete -c xr -n "__fish_xr_needs_command" -a "bookmarks" -d 'List your bookmarks'
complete -c xr -n "__fish_xr_needs_command" -a "bookmark-folders" -d 'Your bookmark folders, and the posts in each'
complete -c xr -n "__fish_xr_needs_command" -a "likes" -d 'List your liked posts'
complete -c xr -n "__fish_xr_needs_command" -a "follow" -d 'Follow a user'
complete -c xr -n "__fish_xr_needs_command" -a "unfollow" -d 'Unfollow a user'
complete -c xr -n "__fish_xr_needs_command" -a "following" -d 'List users you follow'
complete -c xr -n "__fish_xr_needs_command" -a "followers" -d 'List your followers'
complete -c xr -n "__fish_xr_needs_command" -a "mute" -d 'Mute a user'
complete -c xr -n "__fish_xr_needs_command" -a "unmute" -d 'Unmute a user'
complete -c xr -n "__fish_xr_needs_command" -a "muted" -d 'List users you have muted'
complete -c xr -n "__fish_xr_needs_command" -a "block" -d 'Block a user'
complete -c xr -n "__fish_xr_needs_command" -a "unblock" -d 'Unblock a user'
complete -c xr -n "__fish_xr_needs_command" -a "blocked" -d 'List users you have blocked'
complete -c xr -n "__fish_xr_needs_command" -a "usage" -d 'Show API usage (post caps, daily breakdown)'
complete -c xr -n "__fish_xr_needs_command" -a "dm" -d 'Send a direct message'
complete -c xr -n "__fish_xr_needs_command" -a "conversations" -d 'Start a group conversation, send into one, or read one'
complete -c xr -n "__fish_xr_needs_command" -a "dm-events" -d 'Read or delete one direct message, by event id'
complete -c xr -n "__fish_xr_needs_command" -a "dm-block" -d 'Stop a user sending you direct messages'
complete -c xr -n "__fish_xr_needs_command" -a "dm-unblock" -d 'Let a user send you direct messages again'
complete -c xr -n "__fish_xr_needs_command" -a "dms" -d 'List recent direct messages'
complete -c xr -n "__fish_xr_needs_command" -a "lists" -d 'Your lists, a list\'s posts and members, and following and pinning'
complete -c xr -n "__fish_xr_needs_command" -a "communities" -d 'Search communities, and read one'
complete -c xr -n "__fish_xr_needs_command" -a "news" -d 'Search news stories, and read one'
complete -c xr -n "__fish_xr_needs_command" -a "spaces" -d 'Find Spaces by keyword, id, or creator, and read their posts'
complete -c xr -n "__fish_xr_needs_command" -a "articles" -d 'Write and publish long-form X Articles'
complete -c xr -n "__fish_xr_needs_command" -a "notes" -d 'Write Community Notes through X\'s AI Note Writer API'
complete -c xr -n "__fish_xr_needs_command" -a "trends" -d 'Trends for a place, and your personalized trends'
complete -c xr -n "__fish_xr_needs_command" -a "broadcasts" -d 'Broadcast chat moderators and stream sources'
complete -c xr -n "__fish_xr_needs_command" -a "calls" -d 'X calls, and the calls on your calendar (preview)'
complete -c xr -n "__fish_xr_needs_command" -a "webhooks" -d 'Webhook registration, validation, and replay'
complete -c xr -n "__fish_xr_needs_command" -a "auth" -d 'Authentication management'
complete -c xr -n "__fish_xr_needs_command" -a "media" -d 'Media upload, alt text, and subtitles'
complete -c xr -n "__fish_xr_needs_command" -a "skill" -d 'Install or manage the xurl-rs skill bundle'
complete -c xr -n "__fish_xr_needs_command" -a "schema" -d 'Show JSON Schema for a command\'s response type'
complete -c xr -n "__fish_xr_needs_command" -a "completions" -d 'Generate shell completion script'
complete -c xr -n "__fish_xr_needs_command" -a "version" -d 'Show xurl version information'
complete -c xr -n "__fish_xr_needs_command" -a "examples" -d 'Print a curated gallery of invocation examples grouped by use case'
complete -c xr -n "__fish_xr_needs_command" -a "validate" -d 'Validate a JSON document against a bundled response schema'
complete -c xr -n "__fish_xr_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand post" -l media-id -d 'Media ID(s) to attach (repeatable)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand post" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand post" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand post" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand post" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand post" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand post" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand post" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand reply" -l media-id -d 'Media ID(s) to attach (repeatable)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand reply" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand reply" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand reply" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand reply" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand reply" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand reply" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand quote" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand quote" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand quote" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand quote" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand quote" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand quote" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand quote" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand delete" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand delete" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand delete" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l force -d 'Skip the confirmation prompt; required under `--no-interactive`'
complete -c xr -n "__fish_xr_using_subcommand delete" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand delete" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand delete" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand delete" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand delete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand read" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand read" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand read" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand read" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand read" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand read" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand read" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand read" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand read" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand read" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand analytics" -l start -d 'Start of the window (RFC 3339 with a zone, such as 2026-10-01T00:00:00Z)' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -l end -d 'End of the window (RFC 3339 with a zone); after the start' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -l granularity -d 'Bucket size (X\'s default is total, one bucket for the window)' -r -f -a "hourly\t''
daily\t''
weekly\t''
total\t''"
complete -c xr -n "__fish_xr_using_subcommand analytics" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand analytics" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand analytics" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand analytics" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand analytics" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand analytics" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand analytics" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand analytics" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand analytics" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand analytics" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand analytics" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand analytics" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand analytics" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand analytics" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l granularity -d 'Bucket size (X\'s default is hour)' -r -f -a "minute\t''
hour\t''
day\t''"
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l start -d 'Oldest moment counted (RFC 3339 with a zone, such as 2026-10-25T00:00:00Z); within the last seven days' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l end -d 'Moment the count stops, exclusive (RFC 3339 with a zone)' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-counts" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand search-counts" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand search-counts" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand search-counts" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand search-users" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-users" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand search-users" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-users" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-users" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-users" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-users" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand search-users" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search-users" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand search-users" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand search-users" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand search-users" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand search-users" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand search-users" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand users" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand users" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand users" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand users" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand users" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand users" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand users" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand users" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand users" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand users" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand users" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand users" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand users" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand users" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand users" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand users" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand users" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand users" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand users" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand users" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand users" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand users" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand user-posts" -s n -l max-results -d 'Number of results (5-100; a lower value is raised to 5, X\'s minimum). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user-posts" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand user-posts" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand user-posts" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand user-posts" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand read-posts" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand read-posts" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read-posts" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand read-posts" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand read-posts" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand read-posts" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand search" -s n -l max-results -d 'Number of results (10-100; a lower value is raised to 10, X\'s minimum). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand search" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand search" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand search" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand search" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand search" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand search" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand likers" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likers" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand likers" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likers" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likers" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likers" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likers" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand likers" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likers" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand likers" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand likers" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand likers" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand likers" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand likers" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand reposters" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reposters" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand reposters" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reposters" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reposters" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reposters" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reposters" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand reposters" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reposters" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand reposters" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand reposters" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand reposters" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand reposters" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand reposters" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand quotes" -s n -l max-results -d 'Number of results (10-100; a lower value is raised to 10, X\'s minimum). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quotes" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand quotes" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quotes" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quotes" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quotes" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quotes" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand quotes" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quotes" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand quotes" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand quotes" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand quotes" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand quotes" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand quotes" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l cached -d 'Answer from the account the store holds for the login (id, username, display name) with no request; ask X when it holds none yet. `source` says which answered'
complete -c xr -n "__fish_xr_using_subcommand whoami" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand whoami" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand user" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand user" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand user" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand user" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand user" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand user" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand user" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand user" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand user" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand user" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand timeline" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand timeline" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand timeline" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand timeline" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand timeline" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand mentions" -s n -l max-results -d 'Number of results (5-100; a lower value is raised to 5, X\'s minimum). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand mentions" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand mentions" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand mentions" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand mentions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand like" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand like" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand like" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand like" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand like" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand like" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand like" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand like" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand like" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand like" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unlike" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand hide" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand hide" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand hide" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand hide" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand hide" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand hide" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand hide" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand hide" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand hide" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand hide" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand hide" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand hide" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand hide" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand hide" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand hide" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand hide" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand hide" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand hide" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand hide" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand hide" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand hide" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand hide" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unhide" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unhide" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand unhide" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unhide" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unhide" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unhide" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unhide" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unhide" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unhide" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unhide" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unhide" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unhide" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unhide" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unhide" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand unhide" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unhide" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unhide" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unhide" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unhide" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unhide" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unhide" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unhide" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand repost" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand repost" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand repost" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand repost" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand repost" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand repost" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand repost" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -f -a "list" -d 'List the folders you sort bookmarks into'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -f -a "get" -d 'List the posts in one of your bookmark folders, by id'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -f -a "add" -d 'Create a bookmark folder'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and not __fish_seen_subcommand_from list get add help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from help" -f -a "list" -d 'List the folders you sort bookmarks into'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from help" -f -a "get" -d 'List the posts in one of your bookmark folders, by id'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from help" -f -a "add" -d 'Create a bookmark folder'
complete -c xr -n "__fish_xr_using_subcommand bookmark-folders; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand likes" -s n -l max-results -d 'Number of results (5-100; a lower value is raised to 5, X\'s minimum). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand likes" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand likes" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand likes" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand likes" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand likes" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand likes" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand follow" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand follow" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand follow" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand follow" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand follow" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand follow" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand follow" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand following" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l of -d 'Username to list following for (default: you)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand following" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand following" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand following" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand following" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand following" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand following" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand following" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand followers" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l of -d 'Username to list followers for (default: you)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand followers" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand followers" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand followers" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand followers" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand followers" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand followers" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand mute" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand mute" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand mute" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand mute" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand mute" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand mute" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand mute" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unmute" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand muted" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand muted" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand muted" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand muted" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand muted" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand muted" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand muted" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand muted" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand muted" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand muted" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand muted" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand muted" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand muted" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand muted" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand block" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand block" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand block" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand block" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand block" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand block" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand block" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand block" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand block" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand block" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand block" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand block" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand block" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand block" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand block" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand block" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand block" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand block" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand block" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand block" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand block" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand block" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unblock" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unblock" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unblock" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unblock" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unblock" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unblock" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unblock" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unblock" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unblock" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unblock" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unblock" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unblock" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unblock" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand blocked" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand blocked" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand blocked" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand blocked" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand blocked" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand blocked" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand blocked" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand blocked" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand blocked" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand blocked" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand blocked" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand blocked" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand blocked" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand blocked" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -f -a "credits" -d 'Show credits-based usage for the project'
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from help" -f -a "credits" -d 'Show credits-based usage for the project'
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand dm" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand dm" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand dm" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dm" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand dm" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -f -a "start" -d 'Start a group conversation and send its first message'
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -f -a "send" -d 'Send a message into a conversation, by id'
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -f -a "history" -d 'List the messages in a conversation, by id'
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -f -a "with" -d 'List the messages between you and one person'
complete -c xr -n "__fish_xr_using_subcommand conversations; and not __fish_seen_subcommand_from start send history with help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l text -d 'The first message' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from start" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from send" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from history" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from with" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from help" -f -a "start" -d 'Start a group conversation and send its first message'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from help" -f -a "send" -d 'Send a message into a conversation, by id'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from help" -f -a "history" -d 'List the messages in a conversation, by id'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from help" -f -a "with" -d 'List the messages between you and one person'
complete -c xr -n "__fish_xr_using_subcommand conversations; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -f -a "get" -d 'Read one direct message, by event id'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -f -a "remove" -d 'Delete one of your direct messages, by event id'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and not __fish_seen_subcommand_from get remove help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l force -d 'Skip the confirmation prompt; required under `--no-interactive`'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from help" -f -a "get" -d 'Read one direct message, by event id'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Delete one of your direct messages, by event id'
complete -c xr -n "__fish_xr_using_subcommand dm-events; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-block" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand dm-block" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-block" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-block" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-block" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand dm-block" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand dm-unblock" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand dms" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand dms" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand dms" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dms" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dms" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dms" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand dms" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "list" -d 'List the lists you own'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "followed" -d 'List the lists you follow'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "memberships" -d 'List the lists you are a member of'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "pinned" -d 'List the lists you have pinned'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "get" -d 'Read one list, by id'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "posts" -d 'List the posts from a list\'s members'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "followers" -d 'List who follows a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "members" -d 'The accounts on a list, and adding or taking one off'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "add" -d 'Create a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "update" -d 'Rename a list, or change its description or privacy'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "remove" -d 'Delete a list you own'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "follow" -d 'Follow a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "unfollow" -d 'Stop following a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "pin" -d 'Pin a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "unpin" -d 'Unpin a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and not __fish_seen_subcommand_from list followed memberships pinned get posts followers members add update remove follow unfollow pin unpin help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followed" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from memberships" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pinned" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from posts" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from followers" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -f -a "list" -d 'List the accounts on a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -f -a "add" -d 'Add an account to a list you own'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -f -a "remove" -d 'Take an account off a list you own'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from members" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l description -d 'What the list is for, at most 100 characters' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l private -d 'Only you can see the list'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l name -d 'New name, 1 to 25 characters' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l description -d 'New description, at most 100 characters' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l private -d 'Make the list private'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l public -d 'Make the list public'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from update" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l force -d 'Skip the confirmation prompt; required under `--no-interactive`'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from follow" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unfollow" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from pin" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from unpin" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "list" -d 'List the lists you own'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "followed" -d 'List the lists you follow'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "memberships" -d 'List the lists you are a member of'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "pinned" -d 'List the lists you have pinned'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "get" -d 'Read one list, by id'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "posts" -d 'List the posts from a list\'s members'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "followers" -d 'List who follows a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "members" -d 'The accounts on a list, and adding or taking one off'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "add" -d 'Create a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "update" -d 'Rename a list, or change its description or privacy'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Delete a list you own'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "follow" -d 'Follow a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "unfollow" -d 'Stop following a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "pin" -d 'Pin a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "unpin" -d 'Unpin a list'
complete -c xr -n "__fish_xr_using_subcommand lists; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -f -a "search" -d 'Search communities'
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -f -a "get" -d 'Read one community, by id'
complete -c xr -n "__fish_xr_using_subcommand communities; and not __fish_seen_subcommand_from search get help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -s n -l max-results -d 'Number of results (10-100; a lower value is raised to 10, X\'s minimum). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search communities'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from help" -f -a "get" -d 'Read one community, by id'
complete -c xr -n "__fish_xr_using_subcommand communities; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -f -a "search" -d 'Search news stories'
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -f -a "get" -d 'Read one news story, by id'
complete -c xr -n "__fish_xr_using_subcommand news; and not __fish_seen_subcommand_from search get help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l max-age-hours -d 'Only stories updated within this many hours, 1 to 720 (X\'s default is 168, one week)' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search news stories'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from help" -f -a "get" -d 'Read one news story, by id'
complete -c xr -n "__fish_xr_using_subcommand news; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -f -a "search" -d 'Search Spaces by keyword'
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -f -a "get" -d 'Read 1 to 100 Spaces by id in one request'
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -f -a "by-creator" -d 'The Spaces 1 to 100 accounts created, in one request'
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -f -a "posts" -d 'The posts shared in a Space'
complete -c xr -n "__fish_xr_using_subcommand spaces; and not __fish_seen_subcommand_from search get by-creator posts help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l state -d 'Only Spaces in this state (X\'s default is all)' -r -f -a "live\t''
scheduled\t''
all\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from by-creator" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from posts" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from help" -f -a "search" -d 'Search Spaces by keyword'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from help" -f -a "get" -d 'Read 1 to 100 Spaces by id in one request'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from help" -f -a "by-creator" -d 'The Spaces 1 to 100 accounts created, in one request'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from help" -f -a "posts" -d 'The posts shared in a Space'
complete -c xr -n "__fish_xr_using_subcommand spaces; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -f -a "draft" -d 'Save a draft X Article from a plain-text body'
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -f -a "publish" -d 'Publish a draft X Article'
complete -c xr -n "__fish_xr_using_subcommand articles; and not __fish_seen_subcommand_from draft publish help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l title -d 'The article\'s title' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l body-file -d 'The file holding the body, or `-` to read it from stdin' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l cover -d 'The media id of an uploaded image to show above the article' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from draft" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l force -d 'Skip the confirmation prompt; required under `--no-interactive`'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from publish" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from help" -f -a "draft" -d 'Save a draft X Article from a plain-text body'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from help" -f -a "publish" -d 'Publish a draft X Article'
complete -c xr -n "__fish_xr_using_subcommand articles; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -f -a "write" -d 'Write a Community Note about a post'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -f -a "evaluate" -d 'Score a draft Community Note without writing it'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -f -a "eligible" -d 'List posts you could write a Community Note on'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -f -a "written" -d 'List the Community Notes you wrote'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -f -a "remove" -d 'Delete a Community Note you wrote'
complete -c xr -n "__fish_xr_using_subcommand notes; and not __fish_seen_subcommand_from write evaluate eligible written remove help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l text -d 'The note\'s text; X requires a source URL in it' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l classification -d 'How the note classifies the post' -r -f -a "misinformed_or_potentially_misleading\t''
not_misleading\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l tag -d 'Why the post may be misleading; repeat for several' -r -f -a "disputed_claim_as_fact\t''
factual_error\t''
manipulated_media\t''
misinterpreted_satire\t''
missing_important_context\t''
other\t''
outdated_information\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l trustworthy-sources -d 'The note cites trustworthy sources'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l media-note -d 'The note is about the post\'s media'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l test -d 'Test mode, for an app X is still admitting to the program'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from write" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l text -d 'The note\'s text; X requires a source URL in it' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from evaluate" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l test -d 'Test mode, for an app X is still admitting to the program'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from eligible" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l test -d 'Test mode, for an app X is still admitting to the program'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from written" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l force -d 'Skip the confirmation prompt; required under `--no-interactive`'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from help" -f -a "write" -d 'Write a Community Note about a post'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from help" -f -a "evaluate" -d 'Score a draft Community Note without writing it'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from help" -f -a "eligible" -d 'List posts you could write a Community Note on'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from help" -f -a "written" -d 'List the Community Notes you wrote'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Delete a Community Note you wrote'
complete -c xr -n "__fish_xr_using_subcommand notes; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -f -a "place" -d 'Trends for a place, by its Yahoo WOEID (1 is worldwide)'
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -f -a "personal" -d 'The trends X picks for you'
complete -c xr -n "__fish_xr_using_subcommand trends; and not __fish_seen_subcommand_from place personal help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l max -d 'Most trends to list, 1 to 50 (X\'s default is 20)' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from place" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from personal" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from help" -f -a "place" -d 'Trends for a place, by its Yahoo WOEID (1 is worldwide)'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from help" -f -a "personal" -d 'The trends X picks for you'
complete -c xr -n "__fish_xr_using_subcommand trends; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -f -a "list" -d 'List your broadcasts'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -f -a "get" -d 'Read one broadcast, by id'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -f -a "scheduled" -d 'Schedule broadcasts, change or cancel them, and start them early'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -f -a "chat" -d 'Read and moderate a broadcast\'s chat'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -f -a "moderators" -d 'Manage who moderates your broadcast chats'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -f -a "sources" -d 'Manage the RTMP and RTMPS ingest points you stream to'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and not __fish_seen_subcommand_from list get scheduled chat moderators sources help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -f -a "list" -d 'List your scheduled broadcasts'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -f -a "get" -d 'Read one scheduled broadcast, by id'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -f -a "add" -d 'Schedule a broadcast from one of your stream sources'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -f -a "update" -d 'Move a scheduled broadcast, or change its details'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -f -a "remove" -d 'Cancel a scheduled broadcast'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -f -a "live" -d 'Start a scheduled broadcast now'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from scheduled" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -f -a "history" -d 'List a broadcast\'s chat'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -f -a "send" -d 'Send a message into a broadcast\'s chat'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -f -a "remove" -d 'Remove a message from a broadcast\'s chat'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -f -a "mute" -d 'Mute someone in a broadcast\'s chat'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -f -a "unmute" -d 'Unmute someone in a broadcast\'s chat'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from chat" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -f -a "list" -d 'List your broadcast chat moderators'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -f -a "add" -d 'Add a broadcast chat moderator'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -f -a "remove" -d 'Remove a broadcast chat moderator'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from moderators" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -f -a "list" -d 'List your stream sources'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -f -a "get" -d 'Show one stream source, and whether it is receiving video'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -f -a "add" -d 'Create a stream source'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -f -a "rename" -d 'Rename a stream source'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -f -a "remove" -d 'Delete a stream source; an encoder sending to it is no longer received'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from sources" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from help" -f -a "list" -d 'List your broadcasts'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from help" -f -a "get" -d 'Read one broadcast, by id'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from help" -f -a "scheduled" -d 'Schedule broadcasts, change or cancel them, and start them early'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from help" -f -a "chat" -d 'Read and moderate a broadcast\'s chat'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from help" -f -a "moderators" -d 'Manage who moderates your broadcast chats'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from help" -f -a "sources" -d 'Manage the RTMP and RTMPS ingest points you stream to'
complete -c xr -n "__fish_xr_using_subcommand broadcasts; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -f -a "get" -d 'Show an X call: its title, join policy, host, and who is in it'
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -f -a "scheduled" -d 'The calls on your calendar: schedule, change, cancel, and answer them'
complete -c xr -n "__fish_xr_using_subcommand calls; and not __fish_seen_subcommand_from get scheduled help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -f -a "list" -d 'List the calls on your calendar, earliest first'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -f -a "get" -d 'Show a scheduled call, with its invitees and their answers'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -f -a "add" -d 'Schedule a call on your X Calls calendar and invite people to it'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -f -a "update" -d 'Move, rename, or re-invite a call you organize'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -f -a "cancel" -d 'Cancel a call you organize and take it off invitees\' calendars'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -f -a "rsvp" -d 'Answer an invitation to a scheduled call'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from scheduled" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from help" -f -a "get" -d 'Show an X call: its title, join policy, host, and who is in it'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from help" -f -a "scheduled" -d 'The calls on your calendar: schedule, change, cancel, and answer them'
complete -c xr -n "__fish_xr_using_subcommand calls; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -f -a "list" -d 'List the webhooks registered for the app'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -f -a "add" -d 'Register a webhook URL'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -f -a "validate" -d 'Ask X to send its CRC check to a webhook again'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -f -a "remove" -d 'Delete a webhook; its subscriptions stop delivering'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -f -a "subscriptions" -d 'Account Activity subscriptions: whose activity a webhook receives'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -f -a "stream-links" -d 'Filtered-stream links: deliver the posts your stream rules match to a webhook'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -f -a "listen" -d 'Receive webhook deliveries on a local address'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -f -a "replay" -d 'Deliver a past window of events to a webhook again'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and not __fish_seen_subcommand_from list add validate remove subscriptions stream-links listen replay help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from validate" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l force -d 'Skip the confirmation prompt; required under `--no-interactive`'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from remove" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -f -a "count" -d 'Count the subscriptions the app holds and may hold'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -f -a "list" -d 'List the accounts whose activity a webhook receives'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -f -a "add" -d 'Subscribe your account\'s activity to a webhook'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -f -a "check" -d 'Say whether your account\'s activity is subscribed to a webhook'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -f -a "remove" -d 'End an account\'s subscription to a webhook'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from subscriptions" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -f -a "list" -d 'List the webhooks the app\'s filtered stream delivers to'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -f -a "add" -d 'Deliver the app\'s filtered stream to a webhook'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -f -a "remove" -d 'Stop delivering the app\'s filtered stream to a webhook'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from stream-links" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -s p -l port -d 'Local port to listen on; 0 takes any free port' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l bind -d 'Local address to bind' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l path -d 'Request path to answer on' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l secret -d 'Which of the app\'s secrets X signs with. Unset, the OAuth2 client secret when the app has one, else the OAuth1 consumer secret' -r -f -a "oauth2\t'The app\'s `OAuth2` client secret'
oauth1\t'The app\'s `OAuth1` consumer secret (API Secret Key)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l max-events -d 'Exit 0 once this many events have printed' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l forward -d 'Also POST each verified event to this http or https URL, after X has its answer: the body and signature header as X sent them, an Idempotency-Key, and each -H. Each delivery is reported on stderr' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -s H -l header -d 'A header for each forward, as \'Name: Value\'; repeat for more. Its value is never printed' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l forward-timeout -d 'Seconds one forward attempt may take before it is tried again (four attempts, 1, 2, then 4 seconds apart). Unset, 30' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l checkpoint -d 'Record, at most once a minute, when the oldest event not yet delivered to every target arrived (the newest event when none is waiting), so `xr webhooks replay --from-checkpoint` can request what a stop missed' -r -F
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l allow-unsigned -d 'Also print a POST that carries no signature, for a local test with curl. A POST with a wrong signature is refused either way'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from listen" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l from -d 'Start of the window: twelve digits, `yyyymmddhhmm` in UTC' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l to -d 'End of the window: twelve digits, `yyyymmddhhmm` in UTC' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l from-checkpoint -d 'Request the window from the minute `xr webhooks listen --checkpoint` recorded to the current minute, at most the 24 hours X replays. Replayed events reach the targets again, repeats included' -r -F
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from replay" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from help" -f -a "list" -d 'List the webhooks registered for the app'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from help" -f -a "add" -d 'Register a webhook URL'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from help" -f -a "validate" -d 'Ask X to send its CRC check to a webhook again'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from help" -f -a "remove" -d 'Delete a webhook; its subscriptions stop delivering'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from help" -f -a "subscriptions" -d 'Account Activity subscriptions: whose activity a webhook receives'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from help" -f -a "stream-links" -d 'Filtered-stream links: deliver the posts your stream rules match to a webhook'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from help" -f -a "listen" -d 'Receive webhook deliveries on a local address'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from help" -f -a "replay" -d 'Deliver a past window of events to a webhook again'
complete -c xr -n "__fish_xr_using_subcommand webhooks; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "oauth2" -d 'Configure `OAuth2` authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "oauth1" -d 'Configure `OAuth1` authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "app" -d 'Configure app-auth (bearer token)'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "status" -d 'Show authentication status'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "clear" -d 'Clear authentication tokens'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "apps" -d 'Manage registered X API apps'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "default" -d 'Set default app and/or user'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l no-browser -d 'Enable manual two-step flow for headless machines (SSH, containers)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l step -d 'Step number: 1 (generate auth URL) or 2 (complete exchange)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l auth-url -d 'Redirect URL from browser (step 2). Use \'-\' to read from stdin (recommended on shared machines)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l scopes -d 'Request only these comma-separated scopes, plus offline.access (default: every scope)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l consumer-key -d 'Consumer key' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l consumer-secret -d 'Consumer secret' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l consumer-secret-file -d 'File holding the consumer secret; \'-\' reads it from stdin' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l access-token -d 'Access token' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l access-token-file -d 'File holding the access token; \'-\' reads it from stdin' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l token-secret -d 'Token secret' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l token-secret-file -d 'File holding the token secret; \'-\' reads it from stdin' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l bearer-token -d 'Bearer token' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l bearer-token-file -d 'File holding the bearer token; \'-\' reads it from stdin' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l oauth2-username -d 'Clear `OAuth2` token for username' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l all -d 'Clear all authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l oauth1 -d 'Clear `OAuth1` tokens'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l bearer -d 'Clear bearer token'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l force -d 'Skip the confirmation prompt; required under `--no-interactive`'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "add" -d 'Register a new X API app'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "update" -d 'Update credentials for an existing app'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "remove" -d 'Remove a registered app'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "list" -d 'List registered apps'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "redirect-uri" -d 'Inspect or set the stored `OAuth2` redirect URI for an app'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "oauth2" -d 'Configure `OAuth2` authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "oauth1" -d 'Configure `OAuth1` authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "app" -d 'Configure app-auth (bearer token)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show authentication status'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "clear" -d 'Clear authentication tokens'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "apps" -d 'Manage registered X API apps'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "default" -d 'Set default app and/or user'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -f -a "upload" -d 'Upload media file'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -f -a "status" -d 'Check media upload status'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -f -a "get" -d 'Read 1 to 100 uploads by media key in one request'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -f -a "analytics" -d 'Video metrics for your uploads over a time window'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -f -a "alt-text" -d 'Set the alt text shown for an uploaded image or video'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -f -a "subtitles" -d 'Add or remove the subtitle tracks of an uploaded video'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status get analytics alt-text subtitles help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l media-type -d 'Media type (e.g., video/mp4)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l category -d 'Media category (e.g., `amplify_video`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l wait -d 'Wait for X to finish processing the upload before returning' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l auth -d 'Authentication type' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s H -l header -d 'Request headers' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s t -l trace -d 'Trace header'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l auth -d 'Authentication type' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s w -l wait -d 'Wait for X to finish processing instead of reading the status once' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s H -l header -d 'Request headers' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s t -l trace -d 'Trace header'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l start -d 'Start of the window (RFC 3339 with a zone, such as 2026-10-01T00:00:00Z)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l end -d 'End of the window (RFC 3339 with a zone); after the start' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l granularity -d 'Bucket size (X\'s default is daily)' -r -f -a "hourly\t''
daily\t''
total\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from analytics" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -s u -l username -d '`OAuth2` login to act as, by the name it is stored under: its username, or the name given to `xr auth oauth2`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from alt-text" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -f -a "add" -d 'Add a subtitle track to an uploaded video'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -f -a "remove" -d 'Remove a subtitle track from an uploaded video'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from subtitles" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "upload" -d 'Upload media file'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "status" -d 'Check media upload status'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "get" -d 'Read 1 to 100 uploads by media key in one request'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "analytics" -d 'Video metrics for your uploads over a time window'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "alt-text" -d 'Set the alt text shown for an uploaded image or video'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "subtitles" -d 'Add or remove the subtitle tracks of an uploaded video'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -f -a "install" -d 'Install the skill bundle into a host\'s canonical skills directory'
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -f -a "update" -d 'Refresh an existing skill-bundle install in place'
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l all -d 'Install into every known host in one invocation'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l dry-run -d 'Print the resolved git command without spawning'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l all -d 'Update every known host in one invocation'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l dry-run -d 'Print the resolved plan without removing or cloning'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "install" -d 'Install the skill bundle into a host\'s canonical skills directory'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "update" -d 'Refresh an existing skill-bundle install in place'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand schema" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand schema" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand schema" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l list -d 'List all commands and their response types'
complete -c xr -n "__fish_xr_using_subcommand schema" -l all -d 'Output all schemas as a single JSON document'
complete -c xr -n "__fish_xr_using_subcommand schema" -l envelope -d 'Output the canonical agent-native output envelope schema'
complete -c xr -n "__fish_xr_using_subcommand schema" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand schema" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand schema" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand schema" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand completions" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand completions" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand completions" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand completions" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand completions" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand version" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand version" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand version" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand version" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand version" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand version" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand examples" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand examples" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand examples" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand examples" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand examples" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand examples" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand validate" -l schema -d 'Schema name to validate against (`post`, `posts`, `user`, `users`, `dm`, `dms`, `dm-event`, `dm-block`, `usage`, `credits`, `envelope`, `like`, `hide`, `post-counts`, `analytics`, `media`, `media-analytics`, `spaces`, `article-draft`, `article-published`, `note-created`, `note-evaluation`, `notes`, `broadcast`, `broadcasts`, `scheduled-broadcast`, `scheduled-broadcasts`, `chat-messages`, `chat-sent`, `chat-muted`, `list`, `lists`, `list-updated`, `list-member`, `pinned`, `communities`, `community`, `news`, `news-story`, `trends`, `personalized-trends`, `follow`, `delete`, `repost`, `bookmark`, `bookmark-folders`, `bookmark-folder`, `bookmark-folder-posts`, `mute`, `block`, `moderators`, `alt-text`, `subtitles`, `webhook`, `webhooks`, `webhook-validation`, `webhook-replay`, `subscription-count`, `subscriptions`, `subscribed`, `stream-links`, `stream-link`, `stream-sources`, `stream-source`, `stream-source-deleted`, `call`, `scheduled-calls`, `scheduled-call`, `scheduled-call-updated`, `scheduled-call-cancelled`, `rsvp`). Omit for auto-detection' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -s v -l verbose -d 'Print request and response lines, and a note for each key X sent in its legacy post vocabulary' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias `yml`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is `invalid-args`' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand validate" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; prints `json` output on one line, as `jsonl` and `ndjson` always are' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l wait-on-rate-limit -d 'Wait out a rate limit and retry once, instead of failing with `rate-limited`' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -l rate-limit-max-wait -d 'Longest `--wait-on-rate-limit` waits before its retry, in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand validate" -l dry-run -d 'Check inputs and credentials offline, and send nothing' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -l limit -d 'Global result-set limit, clamped to 1..=100' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand validate" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand validate" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand validate" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "post" -d 'Post to X'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "reply" -d 'Reply to a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "quote" -d 'Quote a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "delete" -d 'Delete a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "read" -d 'Read a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "analytics" -d 'Impressions and engagement for your posts over a time window'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "search-counts" -d 'Count the recent posts that match a query, per time bucket'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "search-users" -d 'Find accounts by keyword'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "users" -d 'Look up 1 to 100 accounts by handle or by id in one request'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "user-posts" -d 'List the posts a user wrote'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "read-posts" -d 'Read up to 100 posts in one request'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "search" -d 'Search recent posts'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "likers" -d 'List the users who liked a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "reposters" -d 'List the users who reposted a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "quotes" -d 'List the posts that quote a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "whoami" -d 'Show the authenticated user\'s profile'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "user" -d 'Look up a user by username'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "timeline" -d 'Show your home timeline'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "mentions" -d 'Show your recent mentions'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "like" -d 'Like a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "unlike" -d 'Unlike a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "hide" -d 'Hide a reply to your post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "unhide" -d 'Show a hidden reply to your post again'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "repost" -d 'Repost a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "unrepost" -d 'Undo a repost'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "bookmark" -d 'Bookmark a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "unbookmark" -d 'Remove a bookmark'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "bookmarks" -d 'List your bookmarks'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "bookmark-folders" -d 'Your bookmark folders, and the posts in each'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "likes" -d 'List your liked posts'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "follow" -d 'Follow a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "unfollow" -d 'Unfollow a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "following" -d 'List users you follow'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "followers" -d 'List your followers'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "mute" -d 'Mute a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "unmute" -d 'Unmute a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "muted" -d 'List users you have muted'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "block" -d 'Block a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "unblock" -d 'Unblock a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "blocked" -d 'List users you have blocked'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "usage" -d 'Show API usage (post caps, daily breakdown)'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "dm" -d 'Send a direct message'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "conversations" -d 'Start a group conversation, send into one, or read one'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "dm-events" -d 'Read or delete one direct message, by event id'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "dm-block" -d 'Stop a user sending you direct messages'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "dm-unblock" -d 'Let a user send you direct messages again'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "dms" -d 'List recent direct messages'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "lists" -d 'Your lists, a list\'s posts and members, and following and pinning'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "communities" -d 'Search communities, and read one'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "news" -d 'Search news stories, and read one'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "spaces" -d 'Find Spaces by keyword, id, or creator, and read their posts'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "articles" -d 'Write and publish long-form X Articles'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "notes" -d 'Write Community Notes through X\'s AI Note Writer API'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "trends" -d 'Trends for a place, and your personalized trends'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "broadcasts" -d 'Broadcast chat moderators and stream sources'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "calls" -d 'X calls, and the calls on your calendar (preview)'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "webhooks" -d 'Webhook registration, validation, and replay'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "auth" -d 'Authentication management'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "media" -d 'Media upload, alt text, and subtitles'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "skill" -d 'Install or manage the xurl-rs skill bundle'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "schema" -d 'Show JSON Schema for a command\'s response type'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "completions" -d 'Generate shell completion script'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "version" -d 'Show xurl version information'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "examples" -d 'Print a curated gallery of invocation examples grouped by use case'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "validate" -d 'Validate a JSON document against a bundled response schema'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read analytics search-counts search-users users user-posts read-posts search likers reposters quotes whoami user timeline mentions like unlike hide unhide repost unrepost bookmark unbookmark bookmarks bookmark-folders likes follow unfollow following followers mute unmute muted block unblock blocked usage dm conversations dm-events dm-block dm-unblock dms lists communities news spaces articles notes trends broadcasts calls webhooks auth media skill schema completions version examples validate help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from bookmark-folders" -f -a "list" -d 'List the folders you sort bookmarks into'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from bookmark-folders" -f -a "get" -d 'List the posts in one of your bookmark folders, by id'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from bookmark-folders" -f -a "add" -d 'Create a bookmark folder'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from usage" -f -a "credits" -d 'Show credits-based usage for the project'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from conversations" -f -a "start" -d 'Start a group conversation and send its first message'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from conversations" -f -a "send" -d 'Send a message into a conversation, by id'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from conversations" -f -a "history" -d 'List the messages in a conversation, by id'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from conversations" -f -a "with" -d 'List the messages between you and one person'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from dm-events" -f -a "get" -d 'Read one direct message, by event id'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from dm-events" -f -a "remove" -d 'Delete one of your direct messages, by event id'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "list" -d 'List the lists you own'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "followed" -d 'List the lists you follow'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "memberships" -d 'List the lists you are a member of'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "pinned" -d 'List the lists you have pinned'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "get" -d 'Read one list, by id'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "posts" -d 'List the posts from a list\'s members'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "followers" -d 'List who follows a list'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "members" -d 'The accounts on a list, and adding or taking one off'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "add" -d 'Create a list'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "update" -d 'Rename a list, or change its description or privacy'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "remove" -d 'Delete a list you own'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "follow" -d 'Follow a list'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "unfollow" -d 'Stop following a list'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "pin" -d 'Pin a list'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from lists" -f -a "unpin" -d 'Unpin a list'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from communities" -f -a "search" -d 'Search communities'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from communities" -f -a "get" -d 'Read one community, by id'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from news" -f -a "search" -d 'Search news stories'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from news" -f -a "get" -d 'Read one news story, by id'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from spaces" -f -a "search" -d 'Search Spaces by keyword'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from spaces" -f -a "get" -d 'Read 1 to 100 Spaces by id in one request'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from spaces" -f -a "by-creator" -d 'The Spaces 1 to 100 accounts created, in one request'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from spaces" -f -a "posts" -d 'The posts shared in a Space'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from articles" -f -a "draft" -d 'Save a draft X Article from a plain-text body'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from articles" -f -a "publish" -d 'Publish a draft X Article'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from notes" -f -a "write" -d 'Write a Community Note about a post'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from notes" -f -a "evaluate" -d 'Score a draft Community Note without writing it'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from notes" -f -a "eligible" -d 'List posts you could write a Community Note on'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from notes" -f -a "written" -d 'List the Community Notes you wrote'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from notes" -f -a "remove" -d 'Delete a Community Note you wrote'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from trends" -f -a "place" -d 'Trends for a place, by its Yahoo WOEID (1 is worldwide)'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from trends" -f -a "personal" -d 'The trends X picks for you'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from broadcasts" -f -a "list" -d 'List your broadcasts'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from broadcasts" -f -a "get" -d 'Read one broadcast, by id'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from broadcasts" -f -a "scheduled" -d 'Schedule broadcasts, change or cancel them, and start them early'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from broadcasts" -f -a "chat" -d 'Read and moderate a broadcast\'s chat'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from broadcasts" -f -a "moderators" -d 'Manage who moderates your broadcast chats'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from broadcasts" -f -a "sources" -d 'Manage the RTMP and RTMPS ingest points you stream to'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from calls" -f -a "get" -d 'Show an X call: its title, join policy, host, and who is in it'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from calls" -f -a "scheduled" -d 'The calls on your calendar: schedule, change, cancel, and answer them'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from webhooks" -f -a "list" -d 'List the webhooks registered for the app'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from webhooks" -f -a "add" -d 'Register a webhook URL'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from webhooks" -f -a "validate" -d 'Ask X to send its CRC check to a webhook again'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from webhooks" -f -a "remove" -d 'Delete a webhook; its subscriptions stop delivering'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from webhooks" -f -a "subscriptions" -d 'Account Activity subscriptions: whose activity a webhook receives'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from webhooks" -f -a "stream-links" -d 'Filtered-stream links: deliver the posts your stream rules match to a webhook'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from webhooks" -f -a "listen" -d 'Receive webhook deliveries on a local address'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from webhooks" -f -a "replay" -d 'Deliver a past window of events to a webhook again'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "oauth2" -d 'Configure `OAuth2` authentication'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "oauth1" -d 'Configure `OAuth1` authentication'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "app" -d 'Configure app-auth (bearer token)'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "status" -d 'Show authentication status'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "clear" -d 'Clear authentication tokens'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "apps" -d 'Manage registered X API apps'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "default" -d 'Set default app and/or user'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "upload" -d 'Upload media file'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "status" -d 'Check media upload status'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "get" -d 'Read 1 to 100 uploads by media key in one request'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "analytics" -d 'Video metrics for your uploads over a time window'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "alt-text" -d 'Set the alt text shown for an uploaded image or video'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "subtitles" -d 'Add or remove the subtitle tracks of an uploaded video'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "install" -d 'Install the skill bundle into a host\'s canonical skills directory'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "update" -d 'Refresh an existing skill-bundle install in place'
