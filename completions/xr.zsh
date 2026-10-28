#compdef xr

autoload -U is-at-least

_xr() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-X+[HTTP method (GET by default)]:METHOD:_default' \
'--method=[HTTP method (GET by default)]:METHOD:_default' \
'*-H+[Request headers]:HEADERS:_default' \
'*--header=[Request headers]:HEADERS:_default' \
'-d+[Request body data]:DATA:_default' \
'--data=[Request body data]:DATA:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to send the request as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to send the request as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'-F+[File to upload (for multipart requests)]:FILE:_default' \
'--file=[File to upload (for multipart requests)]:FILE:_default' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add trace header to request]' \
'--trace[Add trace header to request]' \
'-s[Force streaming mode]' \
'--stream[Force streaming mode]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
'::url -- URL for raw mode (positional, only when no subcommand):_urls' \
":: :_xr_commands" \
"*::: :->xr" \
&& ret=0
    case $state in
    (xr)
        words=($line[2] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-command-$line[2]:"
        case $line[2] in
            (post)
_arguments "${_arguments_options[@]}" : \
'*--media-id=[Media ID(s) to attach (repeatable)]:MEDIA_IDS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':text -- The text to post:_default' \
&& ret=0
;;
(reply)
_arguments "${_arguments_options[@]}" : \
'*--media-id=[Media ID(s) to attach (repeatable)]:MEDIA_IDS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL to reply to:_default' \
':text -- The reply text:_default' \
&& ret=0
;;
(quote)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL to quote:_default' \
':text -- The quote text:_default' \
&& ret=0
;;
(delete)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL to delete:_default' \
&& ret=0
;;
(read)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL to read:_default' \
&& ret=0
;;
(analytics)
_arguments "${_arguments_options[@]}" : \
'--start=[Start of the window (RFC 3339 with a zone, such as 2026-10-01T00\:00\:00Z)]:TIME:_default' \
'--end=[End of the window (RFC 3339 with a zone); after the start]:TIME:_default' \
'--granularity=[Bucket size (X'\''s default is total, one bucket for the window)]:GRANULARITY:(hourly daily weekly total)' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::post_ids -- Post IDs or URLs, 1 to 100:_default' \
&& ret=0
;;
(search-counts)
_arguments "${_arguments_options[@]}" : \
'--granularity=[Bucket size (X'\''s default is hour)]:GRANULARITY:(minute hour day)' \
'--start=[Oldest moment counted (RFC 3339 with a zone, such as 2026-10-25T00\:00\:00Z); within the last seven days]:TIME:_default' \
'--end=[Moment the count stops, exclusive (RFC 3339 with a zone)]:TIME:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':query -- Search query, as `xr search` takes it:_default' \
&& ret=0
;;
(search-users)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':query -- Search query:_default' \
&& ret=0
;;
(users)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::accounts -- Handles (@jack or jack) or numeric user ids, 1 to 100; not both:_default' \
&& ret=0
;;
(user-posts)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (5-100; a lower value is raised to 5, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (5-100; a lower value is raised to 5, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username, with or without the leading @:_default' \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::post_ids -- Post IDs or URLs, 1 to 100:_default' \
&& ret=0
;;
(search)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (10-100; a lower value is raised to 10, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (10-100; a lower value is raised to 10, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':query -- Search query:_default' \
&& ret=0
;;
(likers)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL:_default' \
&& ret=0
;;
(reposters)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL:_default' \
&& ret=0
;;
(quotes)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (10-100; a lower value is raised to 10, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (10-100; a lower value is raised to 10, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL:_default' \
&& ret=0
;;
(whoami)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--cached[Answer from the account the store holds for the login (id, username, display name) with no request; ask X when it holds none yet. \`source\` says which answered]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(user)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to look up:_default' \
&& ret=0
;;
(timeline)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(mentions)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (5-100; a lower value is raised to 5, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (5-100; a lower value is raised to 5, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(like)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL:_default' \
&& ret=0
;;
(unlike)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL:_default' \
&& ret=0
;;
(hide)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Reply post ID or URL:_default' \
&& ret=0
;;
(unhide)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Reply post ID or URL:_default' \
&& ret=0
;;
(repost)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL:_default' \
&& ret=0
;;
(unrepost)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL:_default' \
&& ret=0
;;
(bookmark)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL:_default' \
&& ret=0
;;
(unbookmark)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post_id -- Post ID or URL:_default' \
&& ret=0
;;
(bookmarks)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(bookmark-folders)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__bookmark-folders_commands" \
"*::: :->bookmark-folders" \
&& ret=0

    case $state in
    (bookmark-folders)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-bookmark-folders-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':folder_id -- Folder ID, as `xr bookmark-folders list` prints it:_default' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':name -- The folder'\''s name, 1 to 25 characters:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__bookmark-folders__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-bookmark-folders-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(likes)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (5-100; a lower value is raised to 5, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (5-100; a lower value is raised to 5, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(follow)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to follow:_default' \
&& ret=0
;;
(unfollow)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to unfollow:_default' \
&& ret=0
;;
(following)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--of=[Username to list following for (default\: you)]:USERNAME:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(followers)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--of=[Username to list followers for (default\: you)]:USERNAME:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(mute)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to mute:_default' \
&& ret=0
;;
(unmute)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to unmute:_default' \
&& ret=0
;;
(muted)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(block)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to block:_default' \
&& ret=0
;;
(unblock)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to unblock:_default' \
&& ret=0
;;
(blocked)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(profile)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__profile_commands" \
"*::: :->profile" \
&& ret=0

    case $state in
    (profile)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-profile-command-$line[1]:"
        case $line[1] in
            (update)
_arguments "${_arguments_options[@]}" : \
'--name=[Display name, 1 to 50 characters]:NAME:_default' \
'--description=[Bio, up to 160 characters]:TEXT:_default' \
'--url=[Website URL, up to 100 characters]:URL:_default' \
'--location=[Location, up to 30 characters]:TEXT:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(image)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__profile__subcmd__image_commands" \
"*::: :->image" \
&& ret=0

    case $state in
    (image)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-profile-image-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':media_id -- Media ID, as `xr media upload` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__profile__subcmd__image__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-profile-image-help-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(banner)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__profile__subcmd__banner_commands" \
"*::: :->banner" \
&& ret=0

    case $state in
    (banner)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-profile-banner-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
'--crop=[The region of the image to use, in pixels\: width x height + left offset + top offset, such as \`1500x500+0+120\`]:WxH+LEFT+TOP:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':media_id -- Media ID, as `xr media upload` prints it:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__profile__subcmd__banner__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-profile-banner-help-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__profile__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-profile-help-command-$line[1]:"
        case $line[1] in
            (update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(image)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__profile__subcmd__help__subcmd__image_commands" \
"*::: :->image" \
&& ret=0

    case $state in
    (image)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-profile-help-image-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(banner)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__profile__subcmd__help__subcmd__banner_commands" \
"*::: :->banner" \
&& ret=0

    case $state in
    (banner)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-profile-help-banner-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(settings)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__settings_commands" \
"*::: :->settings" \
&& ret=0

    case $state in
    (settings)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-settings-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
'--allow-dms-from=[Who may message you]:WHO:((all\:"Anyone"
following\:"Accounts you follow"
verified\:"Verified accounts"))' \
'--allow-media-tagging=[Who may tag you in media]:WHO:((all\:"Anyone"
following\:"Accounts you follow"
none\:"No one"))' \
'--always-allow-dms-from-subscribers=[Whether your subscribers may always message you, whatever \`--allow-dms-from\` says]:BOOL:(true false)' \
'--discoverable-by-email=[Whether people with your email address can find you on X]:BOOL:(true false)' \
'--discoverable-by-phone=[Whether people with your phone number can find you on X]:BOOL:(true false)' \
'--display-sensitive-media=[Whether media that may be sensitive is shown to you]:BOOL:(true false)' \
'--language=[The account'\''s language, a BCP 47 tag such as en or pt-BR]:TAG:_default' \
'--location-tagging-enabled=[Whether a precise location may be attached to your posts]:BOOL:(true false)' \
'--protected=[Whether your posts are protected, visible only to approved followers]:BOOL:(true false)' \
'--quiet-hours-enabled=[Whether quiet hours, with no push or SMS notifications, are on]:BOOL:(true false)' \
'--quiet-hours-start=[The hour quiet hours begin, 0 to 23 in your time zone]:HOUR:_default' \
'--quiet-hours-end=[The hour quiet hours end, 0 to 23 in your time zone]:HOUR:_default' \
'--time-zone=[Your time zone\: an IANA name such as America/Chicago, or a display name such as "Central Time (US & Canada)"]:ZONE:_default' \
'--use-cookie-personalization=[Whether X may use your activity off X to personalize what you see]:BOOL:(true false)' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__settings__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-settings-help-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(usage)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__usage_commands" \
"*::: :->usage" \
&& ret=0

    case $state in
    (usage)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-usage-command-$line[1]:"
        case $line[1] in
            (credits)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__usage__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-usage-help-command-$line[1]:"
        case $line[1] in
            (credits)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(dm)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to DM:_default' \
':text -- Message text:_default' \
&& ret=0
;;
(conversations)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__conversations_commands" \
"*::: :->conversations" \
&& ret=0

    case $state in
    (conversations)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-conversations-command-$line[1]:"
        case $line[1] in
            (start)
_arguments "${_arguments_options[@]}" : \
'--text=[The first message]:TEXT:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::participants -- The people to start it with besides you, 2 to 49\: each a handle, or a numeric user id:_default' \
&& ret=0
;;
(send)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':conversation_id -- Conversation ID, as `xr dms` prints it:_default' \
':text -- Message text:_default' \
&& ret=0
;;
(history)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':conversation_id -- Conversation ID, as `xr dms` prints it:_default' \
&& ret=0
;;
(with)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username of the other person:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__conversations__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-conversations-help-command-$line[1]:"
        case $line[1] in
            (start)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(send)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(history)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(with)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(dm-events)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__dm-events_commands" \
"*::: :->dm-events" \
&& ret=0

    case $state in
    (dm-events)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-dm-events-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':event_id -- Event ID, as `xr dms` prints it:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':event_id -- Event ID, as `xr dms` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__dm-events__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-dm-events-help-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(dm-block)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to block from sending you direct messages:_default' \
&& ret=0
;;
(dm-unblock)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to unblock:_default' \
&& ret=0
;;
(dms)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(lists)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__lists_commands" \
"*::: :->lists" \
&& ret=0

    case $state in
    (lists)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-lists-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--of=[Username whose own lists to read (default\: you)]:USERNAME:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(followed)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--of=[Username whose followed lists to read (default\: you)]:USERNAME:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(memberships)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--of=[Username whose list memberships to read (default\: you)]:USERNAME:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(pinned)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(followers)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(members)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__lists__subcmd__members_commands" \
"*::: :->members" \
&& ret=0

    case $state in
    (members)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-lists-members-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
':target_username -- Username of the account:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
':target_username -- Username of the account:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__lists__subcmd__members__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-lists-members-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--description=[What the list is for, at most 100 characters]:DESCRIPTION:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--private[Only you can see the list]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':name -- The list'\''s name, 1 to 25 characters:_default' \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
'--name=[New name, 1 to 25 characters]:NAME:_default' \
'--description=[New description, at most 100 characters]:DESCRIPTION:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--public)--private[Make the list private]' \
'--public[Make the list public]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(follow)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(unfollow)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(pin)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(unpin)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':list_id -- List ID, as `xr lists list` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__lists__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-lists-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(followed)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(memberships)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(pinned)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(followers)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(members)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__lists__subcmd__help__subcmd__members_commands" \
"*::: :->members" \
&& ret=0

    case $state in
    (members)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-lists-help-members-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(follow)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unfollow)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(pin)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unpin)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(communities)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__communities_commands" \
"*::: :->communities" \
&& ret=0

    case $state in
    (communities)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-communities-command-$line[1]:"
        case $line[1] in
            (search)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (10-100; a lower value is raised to 10, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (10-100; a lower value is raised to 10, X'\''s minimum). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':query -- Search query:_default' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':community_id -- Community ID, as `xr communities search` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__communities__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-communities-help-command-$line[1]:"
        case $line[1] in
            (search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(news)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__news_commands" \
"*::: :->news" \
&& ret=0

    case $state in
    (news)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-news-command-$line[1]:"
        case $line[1] in
            (search)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-age-hours=[Only stories updated within this many hours, 1 to 720 (X'\''s default is 168, one week)]:MAX_AGE_HOURS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':query -- Search query:_default' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':story_id -- Story ID, as `xr news search` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__news__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-news-help-command-$line[1]:"
        case $line[1] in
            (search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(spaces)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__spaces_commands" \
"*::: :->spaces" \
&& ret=0

    case $state in
    (spaces)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-spaces-command-$line[1]:"
        case $line[1] in
            (search)
_arguments "${_arguments_options[@]}" : \
'--state=[Only Spaces in this state (X'\''s default is all)]:STATE:(live scheduled all)' \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':query -- Search query:_default' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::space_ids -- Space ids, such as 1DXxyRYNejbKM, 1 to 100:_default' \
&& ret=0
;;
(by-creator)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::accounts -- Handles (@jack or jack) or numeric user ids, 1 to 100, in any mix:_default' \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':space_id -- Space ID, as `xr spaces search` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__spaces__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-spaces-help-command-$line[1]:"
        case $line[1] in
            (search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(by-creator)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(stream)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__stream_commands" \
"*::: :->stream" \
&& ret=0

    case $state in
    (stream)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-stream-command-$line[1]:"
        case $line[1] in
            (rules)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__stream__subcmd__rules_commands" \
"*::: :->rules" \
&& ret=0

    case $state in
    (rules)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-stream-rules-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-1000). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-1000). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--tag=[A label for the rule; the stream marks each post with the tags of the rules it matched]:TAG:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--validate-only[Have X check the rule and save nothing (the endpoint'\''s \`dry_run\`). This sends a request; \`--dry-run\` sends none]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':value -- The rule'\''s filter expression, such as "from\:XDevelopers has\:media":_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'--validate-only[Have X check the removal and delete nothing (the endpoint'\''s \`dry_run\`); it needs no confirmation. This sends a request; \`--dry-run\` sends none]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::rule_ids -- Rule ids, as `xr stream rules list` prints them:_default' \
&& ret=0
;;
(count)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__stream__subcmd__rules__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-stream-rules-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(count)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__stream__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-stream-help-command-$line[1]:"
        case $line[1] in
            (rules)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__stream__subcmd__help__subcmd__rules_commands" \
"*::: :->rules" \
&& ret=0

    case $state in
    (rules)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-stream-help-rules-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(count)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(articles)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__articles_commands" \
"*::: :->articles" \
&& ret=0

    case $state in
    (articles)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-articles-command-$line[1]:"
        case $line[1] in
            (draft)
_arguments "${_arguments_options[@]}" : \
'--title=[The article'\''s title]:TITLE:_default' \
'--body-file=[The file holding the body, or \`-\` to read it from stdin]:PATH:_default' \
'--cover=[The media id of an uploaded image to show above the article]:MEDIA_ID:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(publish)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':article_id -- Article ID, as `xr articles draft` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__articles__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-articles-help-command-$line[1]:"
        case $line[1] in
            (draft)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(publish)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(notes)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__notes_commands" \
"*::: :->notes" \
&& ret=0

    case $state in
    (notes)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-notes-command-$line[1]:"
        case $line[1] in
            (write)
_arguments "${_arguments_options[@]}" : \
'--text=[The note'\''s text; X requires a source URL in it]:TEXT:_default' \
'--classification=[How the note classifies the post]:CLASSIFICATION:(misinformed_or_potentially_misleading not_misleading)' \
'*--tag=[Why the post may be misleading; repeat for several]:TAG:(disputed_claim_as_fact factual_error manipulated_media misinterpreted_satire missing_important_context other outdated_information)' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--trustworthy-sources[The note cites trustworthy sources]' \
'--media-note[The note is about the post'\''s media]' \
'--test[Test mode, for an app X is still admitting to the program]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post -- The post the note is about, by id or URL:_default' \
&& ret=0
;;
(evaluate)
_arguments "${_arguments_options[@]}" : \
'--text=[The note'\''s text; X requires a source URL in it]:TEXT:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':post -- The post the note is about, by id or URL:_default' \
&& ret=0
;;
(eligible)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--test[Test mode, for an app X is still admitting to the program]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(written)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--test[Test mode, for an app X is still admitting to the program]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':note_id -- Note ID, as `xr notes written` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__notes__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-notes-help-command-$line[1]:"
        case $line[1] in
            (write)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(evaluate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(eligible)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(written)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(trends)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__trends_commands" \
"*::: :->trends" \
&& ret=0

    case $state in
    (trends)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-trends-command-$line[1]:"
        case $line[1] in
            (place)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of trends (1-50; X'\''s default is 20). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of trends (1-50; X'\''s default is 20). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':woeid -- The place'\''s Yahoo Where On Earth ID, a positive integer:_default' \
&& ret=0
;;
(personal)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__trends__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-trends-help-command-$line[1]:"
        case $line[1] in
            (place)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(personal)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(broadcasts)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__broadcasts_commands" \
"*::: :->broadcasts" \
&& ret=0

    case $state in
    (broadcasts)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':broadcast_id -- Broadcast ID, as `xr broadcasts list` prints it:_default' \
&& ret=0
;;
(scheduled)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__broadcasts__subcmd__scheduled_commands" \
"*::: :->scheduled" \
&& ret=0

    case $state in
    (scheduled)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-scheduled-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':scheduled_id -- Scheduled broadcast ID, as `xr broadcasts scheduled list` prints it:_default' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--source=[Stream source ID, as \`xr broadcasts sources list\` prints it]:SOURCE_ID:_default' \
'--start=[When the broadcast starts (RFC 3339 with a zone, such as 2026-11-02T16\:00\:00Z)]:TIME:_default' \
'--end=[When the broadcast ends (RFC 3339); open-ended when unset]:TIME:_default' \
'--title=[The broadcast'\''s title]:TITLE:_default' \
'--description=[What the broadcast is about]:DESCRIPTION:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--replay[Let people watch the broadcast after it ends]' \
'--manual-publish[Wait for \`xr broadcasts scheduled live\` rather than starting on time]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
'--start=[When the broadcast starts (RFC 3339 with a zone)]:TIME:_default' \
'--end=[When the broadcast ends (RFC 3339 with a zone)]:TIME:_default' \
'--source=[Stream source ID to take video from instead]:SOURCE_ID:_default' \
'--title=[The broadcast'\''s title]:TITLE:_default' \
'--description=[What the broadcast is about]:DESCRIPTION:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--replay[Let people watch the broadcast after it ends]' \
'--manual-publish[Wait for \`xr broadcasts scheduled live\` rather than starting on time]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':scheduled_id -- Scheduled broadcast ID, as `xr broadcasts scheduled list` prints it:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':scheduled_id -- Scheduled broadcast ID, as `xr broadcasts scheduled list` prints it:_default' \
&& ret=0
;;
(live)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':scheduled_id -- Scheduled broadcast ID, as `xr broadcasts scheduled list` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-scheduled-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(live)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(chat)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__broadcasts__subcmd__chat_commands" \
"*::: :->chat" \
&& ret=0

    case $state in
    (chat)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-chat-command-$line[1]:"
        case $line[1] in
            (history)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-200). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-200). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':broadcast_id -- Broadcast ID, as `xr broadcasts list` prints it:_default' \
&& ret=0
;;
(send)
_arguments "${_arguments_options[@]}" : \
'--reply-to=[The chat message to reply to, by id]:MESSAGE_ID:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':broadcast_id -- Broadcast ID, as `xr broadcasts list` prints it:_default' \
':text -- Message text, 1 to 140 characters:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':broadcast_id -- Broadcast ID, as `xr broadcasts list` prints it:_default' \
':message_id -- Chat message ID, as `xr broadcasts chat history` prints it:_default' \
&& ret=0
;;
(mute)
_arguments "${_arguments_options[@]}" : \
'--until=[When the mute ends (RFC 3339 with a zone); for good when unset]:TIME:_default' \
'--message=[A chat message of theirs to remove while muting, by id]:MESSAGE_ID:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':broadcast_id -- Broadcast ID, as `xr broadcasts list` prints it:_default' \
':target_username -- Username to mute:_default' \
&& ret=0
;;
(unmute)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':broadcast_id -- Broadcast ID, as `xr broadcasts list` prints it:_default' \
':target_username -- Username to unmute:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__chat__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-chat-help-command-$line[1]:"
        case $line[1] in
            (history)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(send)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(mute)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unmute)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(moderators)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__broadcasts__subcmd__moderators_commands" \
"*::: :->moderators" \
&& ret=0

    case $state in
    (moderators)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-moderators-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to add:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to remove:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-moderators-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(mutes)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__broadcasts__subcmd__mutes_commands" \
"*::: :->mutes" \
&& ret=0

    case $state in
    (mutes)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-mutes-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--until=[When the mute ends (RFC 3339 with a zone); for good when unset]:TIME:_default' \
'--broadcast=[A broadcast you moderate\: mute them in its host'\''s chats]:BROADCAST_ID:_default' \
'--message=[A message of theirs in that broadcast'\''s chat to remove, by id]:MESSAGE_ID:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to mute:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--broadcast=[A broadcast you moderate\: unmute them in its host'\''s chats]:BROADCAST_ID:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':target_username -- Username to unmute:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-mutes-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(sources)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__broadcasts__subcmd__sources_commands" \
"*::: :->sources" \
&& ret=0

    case $state in
    (sources)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-sources-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':source_id -- Source ID, as `xr broadcasts sources list` prints it:_default' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--region=[Ingest region\: us-east-1, us-east-2, us-west-1, us-west-2, ca-central-1, sa-east-1, eu-west-1, eu-west-3, eu-central-1, ap-south-1, ap-southeast-1, ap-southeast-2, ap-northeast-1, or ap-northeast-2]:REGION:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':name -- A label for the source:_default' \
&& ret=0
;;
(rename)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':source_id -- Source ID, as `xr broadcasts sources list` prints it:_default' \
':name -- The new label:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':source_id -- Source ID, as `xr broadcasts sources list` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__sources__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-sources-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(rename)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(scheduled)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled_commands" \
"*::: :->scheduled" \
&& ret=0

    case $state in
    (scheduled)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-help-scheduled-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(live)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(chat)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__help__subcmd__chat_commands" \
"*::: :->chat" \
&& ret=0

    case $state in
    (chat)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-help-chat-command-$line[1]:"
        case $line[1] in
            (history)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(send)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(mute)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unmute)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(moderators)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__help__subcmd__moderators_commands" \
"*::: :->moderators" \
&& ret=0

    case $state in
    (moderators)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-help-moderators-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(mutes)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__help__subcmd__mutes_commands" \
"*::: :->mutes" \
&& ret=0

    case $state in
    (mutes)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-help-mutes-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(sources)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__broadcasts__subcmd__help__subcmd__sources_commands" \
"*::: :->sources" \
&& ret=0

    case $state in
    (sources)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-broadcasts-help-sources-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(rename)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(calls)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__calls_commands" \
"*::: :->calls" \
&& ret=0

    case $state in
    (calls)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-calls-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':call_id -- Call ID, as a scheduled call'\''s `call_id` gives it; it is also the call'\''s join secret:_default' \
&& ret=0
;;
(scheduled)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__calls__subcmd__scheduled_commands" \
"*::: :->scheduled" \
&& ret=0

    case $state in
    (scheduled)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-calls-scheduled-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'--start=[Calls still running at this time or starting after it (RFC 3339); X assumes now]:TIME:_default' \
'--end=[Calls starting before this time (RFC 3339); X assumes 24 hours after \`--start\`]:TIME:_default' \
'-n+[Number of calls (1-100)]:MAX_RESULTS:_default' \
'--max-results=[Number of calls (1-100)]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':scheduled_call_id -- Scheduled call ID, as `xr calls scheduled list` prints it:_default' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--start=[When the call starts (RFC 3339 with a zone, such as 2026-11-02T16\:00\:00Z); not in the past]:TIME:_default' \
'--title=[The title invitees see (100 characters at most)]:TITLE:_default' \
'--description=[What the call is about (1000 characters at most)]:DESCRIPTION:_default' \
'--duration=[How long the call lasts, in minutes (at most 44640, 31 days); X assumes 30]:MINUTES:_default' \
'*--invite=[Numeric user ID of someone to invite; repeat for each person]:USER_ID:_default' \
'--recurrence=[The RFC 5545 RRULE to repeat the call by, such as FREQ=WEEKLY;BYDAY=MO]:RRULE:_default' \
'--time-zone=[The IANA time zone the recurrence follows, such as America/Chicago]:ZONE:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--no-notify[Send invitees no email about it]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
'--start=[When the call, or the named occurrence, now starts (RFC 3339)]:TIME:_default' \
'--title=[The title invitees see (100 characters at most)]:TITLE:_default' \
'--description=[What the call is about (1000 characters at most)]:DESCRIPTION:_default' \
'--duration=[How long the call lasts, in minutes (at most 44640, 31 days); X assumes 30]:MINUTES:_default' \
'*--invite=[Numeric user ID of someone to invite; repeat for each person]:USER_ID:_default' \
'--recurrence=[The RFC 5545 RRULE to repeat the call by, such as FREQ=WEEKLY;BYDAY=MO]:RRULE:_default' \
'--time-zone=[The IANA time zone the recurrence follows, such as America/Chicago]:ZONE:_default' \
'--scope=[Which occurrences, counted from \`--occurrence\`\: X assumes \`all\` without it and \`this\` with it]:SCOPE:((all\:"Every occurrence"
this\:"The named occurrence alone"
this-and-following\:"The named occurrence and every one after it"))' \
'--occurrence=[The occurrence'\''s original start (RFC 3339), as a listed call'\''s \`occurrence_start_time\` gives it]:TIME:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--no-notify[Send invitees no email about it]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':scheduled_call_id -- Scheduled call ID, as `xr calls scheduled list` prints it:_default' \
&& ret=0
;;
(cancel)
_arguments "${_arguments_options[@]}" : \
'--scope=[Which occurrences, counted from \`--occurrence\`\: X assumes \`all\` without it and \`this\` with it]:SCOPE:((all\:"Every occurrence"
this\:"The named occurrence alone"
this-and-following\:"The named occurrence and every one after it"))' \
'--occurrence=[The occurrence'\''s original start (RFC 3339), as a listed call'\''s \`occurrence_start_time\` gives it]:TIME:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--no-notify[Send invitees no email about the cancellation]' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':scheduled_call_id -- Scheduled call ID, as `xr calls scheduled list` prints it:_default' \
&& ret=0
;;
(rsvp)
_arguments "${_arguments_options[@]}" : \
'--scope=[Which occurrences, counted from \`--occurrence\`\: X assumes \`all\` without it and \`this\` with it]:SCOPE:((all\:"Every occurrence"
this\:"The named occurrence alone"
this-and-following\:"The named occurrence and every one after it"))' \
'--occurrence=[The occurrence'\''s original start (RFC 3339), as a listed call'\''s \`occurrence_start_time\` gives it]:TIME:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':scheduled_call_id -- Scheduled call ID, as `xr calls scheduled list` prints it:_default' \
':answer -- Your answer:((accepted\:"You will attend"
declined\:"You will not attend"
tentative\:"You might attend"))' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__calls__subcmd__scheduled__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-calls-scheduled-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(cancel)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(rsvp)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__calls__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-calls-help-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(scheduled)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__calls__subcmd__help__subcmd__scheduled_commands" \
"*::: :->scheduled" \
&& ret=0

    case $state in
    (scheduled)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-calls-help-scheduled-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(cancel)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(rsvp)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(webhooks)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__webhooks_commands" \
"*::: :->webhooks" \
&& ret=0

    case $state in
    (webhooks)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-webhooks-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':url -- The public HTTPS URL X delivers events to (200 characters at most):_default' \
&& ret=0
;;
(validate)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':webhook_id -- Webhook ID, as `xr webhooks list` prints it:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':webhook_id -- Webhook ID, as `xr webhooks list` prints it:_default' \
&& ret=0
;;
(subscriptions)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__webhooks__subcmd__subscriptions_commands" \
"*::: :->subscriptions" \
&& ret=0

    case $state in
    (subscriptions)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-webhooks-subscriptions-command-$line[1]:"
        case $line[1] in
            (count)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':webhook_id -- Webhook ID, as `xr webhooks list` prints it:_default' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':webhook_id -- Webhook ID, as `xr webhooks list` prints it:_default' \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':webhook_id -- Webhook ID, as `xr webhooks list` prints it:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':webhook_id -- Webhook ID, as `xr webhooks list` prints it:_default' \
':user_id -- Numeric user ID of the subscribed account, as `subscriptions list` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-webhooks-subscriptions-help-command-$line[1]:"
        case $line[1] in
            (count)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(stream-links)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__webhooks__subcmd__stream-links_commands" \
"*::: :->stream-links" \
&& ret=0

    case $state in
    (stream-links)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-webhooks-stream-links-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':webhook_id -- Webhook ID, as `xr webhooks list` prints it:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':webhook_id -- Webhook ID, as `xr webhooks list` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-webhooks-stream-links-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(listen)
_arguments "${_arguments_options[@]}" : \
'-p+[Local port to listen on; 0 takes any free port]:PORT:_default' \
'--port=[Local port to listen on; 0 takes any free port]:PORT:_default' \
'--bind=[Local address to bind]:ADDRESS:_default' \
'--path=[Request path to answer on]:PATH:_default' \
'--secret=[Which of the app'\''s secrets X signs with. Unset, the OAuth2 client secret when the app has one, else the OAuth1 consumer secret]:SECRET:((oauth2\:"The app'\''s \`OAuth2\` client secret"
oauth1\:"The app'\''s \`OAuth1\` consumer secret (API Secret Key)"))' \
'--max-events=[Exit 0 once this many events have printed]:N:_default' \
'--forward=[Also POST each verified event to this http or https URL, after X has its answer\: the body and signature header as X sent them, an Idempotency-Key, and each -H. Each delivery is reported on stderr]:URL:_default' \
'*-H+[A header for each forward, as '\''Name\: Value'\''; repeat for more. Its value is never printed]:NAME: VALUE:_default' \
'*--header=[A header for each forward, as '\''Name\: Value'\''; repeat for more. Its value is never printed]:NAME: VALUE:_default' \
'--forward-timeout=[Seconds one forward attempt may take before it is tried again (four attempts, 1, 2, then 4 seconds apart). Unset, 30]:SECS:_default' \
'--checkpoint=[Record, at most once a minute, when the oldest event not yet delivered to every target arrived (the newest event when none is waiting), so \`xr webhooks replay --from-checkpoint\` can request what a stop missed]:FILE:_files' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--allow-unsigned[Also print a POST that carries no signature, for a local test with curl. A POST with a wrong signature is refused either way]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::program -- A program to run once per verified event, after `--`, with the event'\''s body on its stdin and XR_EVENT_SHA256 set to the key a --forward target gets. No shell runs it; its stdout and stderr go to stderr. A run past --forward-timeout is stopped with everything it started:_default' \
&& ret=0
;;
(replay)
_arguments "${_arguments_options[@]}" : \
'--from=[Start of the window\: twelve digits, \`yyyymmddhhmm\` in UTC]:YYYYMMDDHHMM:_default' \
'--to=[End of the window\: twelve digits, \`yyyymmddhhmm\` in UTC]:YYYYMMDDHHMM:_default' \
'(--from --to)--from-checkpoint=[Request the window from the minute \`xr webhooks listen --checkpoint\` recorded to the current minute, at most the 24 hours X replays. Replayed events reach the targets again, repeats included]:FILE:_files' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':webhook_id -- Webhook ID, as `xr webhooks list` prints it:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__webhooks__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-webhooks-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(subscriptions)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions_commands" \
"*::: :->subscriptions" \
&& ret=0

    case $state in
    (subscriptions)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-webhooks-help-subscriptions-command-$line[1]:"
        case $line[1] in
            (count)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(stream-links)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__webhooks__subcmd__help__subcmd__stream-links_commands" \
"*::: :->stream-links" \
&& ret=0

    case $state in
    (stream-links)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-webhooks-help-stream-links-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(listen)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(replay)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(auth)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__auth_commands" \
"*::: :->auth" \
&& ret=0

    case $state in
    (auth)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-auth-command-$line[1]:"
        case $line[1] in
            (oauth2)
_arguments "${_arguments_options[@]}" : \
'--no-browser=[Enable manual two-step flow for headless machines (SSH, containers)]::NO_BROWSER:(true false)' \
'--step=[Step number\: 1 (generate auth URL) or 2 (complete exchange)]:STEP:_default' \
'--auth-url=[Redirect URL from browser (step 2). Use '\''-'\'' to read from stdin (recommended on shared machines)]:AUTH_URL:_default' \
'*--scopes=[Request only these comma-separated scopes, plus offline.access (default\: every scope)]:SCOPES:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::username -- Username to label the saved token (bypasses `/2/users/me` lookup when supplied):_default' \
&& ret=0
;;
(oauth1)
_arguments "${_arguments_options[@]}" : \
'--consumer-key=[Consumer key]:CONSUMER_KEY:_default' \
'--consumer-secret=[Consumer secret]:CONSUMER_SECRET:_default' \
'--consumer-secret-file=[File holding the consumer secret; '\''-'\'' reads it from stdin]:PATH:_default' \
'--access-token=[Access token]:ACCESS_TOKEN:_default' \
'--access-token-file=[File holding the access token; '\''-'\'' reads it from stdin]:PATH:_default' \
'--token-secret=[Token secret]:TOKEN_SECRET:_default' \
'--token-secret-file=[File holding the token secret; '\''-'\'' reads it from stdin]:PATH:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(app)
_arguments "${_arguments_options[@]}" : \
'--bearer-token=[Bearer token]:BEARER_TOKEN:_default' \
'--bearer-token-file=[File holding the bearer token; '\''-'\'' reads it from stdin]:PATH:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(clear)
_arguments "${_arguments_options[@]}" : \
'--oauth2-username=[Clear \`OAuth2\` token for username]:OAUTH2_USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--all[Clear all authentication]' \
'--oauth1[Clear \`OAuth1\` tokens]' \
'--bearer[Clear bearer token]' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(apps)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__auth__subcmd__apps_commands" \
"*::: :->apps" \
&& ret=0

    case $state in
    (apps)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-auth-apps-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
'--client-id=[\`OAuth2\` client ID]:CLIENT_ID:_default' \
'--client-secret=[\`OAuth2\` client secret]:CLIENT_SECRET:_default' \
'--client-secret-file=[File holding the \`OAuth2\` client secret; '\''-'\'' reads it from stdin]:PATH:_default' \
'--redirect-uri=[\`OAuth2\` redirect URI (https or http on loopback)]:REDIRECT_URI:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':name -- App name:_default' \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
'--client-id=[\`OAuth2\` client ID]:CLIENT_ID:_default' \
'--client-secret=[\`OAuth2\` client secret]:CLIENT_SECRET:_default' \
'(--client-secret)--client-secret-file=[File holding the \`OAuth2\` client secret; '\''-'\'' reads it from stdin]:PATH:_default' \
'--redirect-uri=[\`OAuth2\` redirect URI (https or http on loopback); empty string clears]:REDIRECT_URI:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':name -- App name:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--force[Skip the confirmation prompt; required under \`--no-interactive\`]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':name -- App name:_default' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(redirect-uri)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri_commands" \
"*::: :->redirect-uri" \
&& ret=0

    case $state in
    (redirect-uri)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-auth-apps-redirect-uri-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::name -- App name (defaults to the configured default app):_default' \
&& ret=0
;;
(set)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':name -- App name:_default' \
':uri -- Redirect URI (https or http on loopback):_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-auth-apps-redirect-uri-help-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__auth__subcmd__apps__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-auth-apps-help-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(redirect-uri)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect-uri_commands" \
"*::: :->redirect-uri" \
&& ret=0

    case $state in
    (redirect-uri)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-auth-apps-help-redirect-uri-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(default)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::app_name -- App name (optional):_default' \
'::username -- Username (optional):_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__auth__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-auth-help-command-$line[1]:"
        case $line[1] in
            (oauth2)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(oauth1)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(app)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(clear)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(apps)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__auth__subcmd__help__subcmd__apps_commands" \
"*::: :->apps" \
&& ret=0

    case $state in
    (apps)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-auth-help-apps-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(redirect-uri)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect-uri_commands" \
"*::: :->redirect-uri" \
&& ret=0

    case $state in
    (redirect-uri)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-auth-help-apps-redirect-uri-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(default)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(media)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__media_commands" \
"*::: :->media" \
&& ret=0

    case $state in
    (media)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-media-command-$line[1]:"
        case $line[1] in
            (upload)
_arguments "${_arguments_options[@]}" : \
'--media-type=[Media type (e.g., video/mp4)]:MEDIA_TYPE:_default' \
'--category=[Media category (e.g., \`amplify_video\`)]:CATEGORY:_default' \
'--wait=[Wait for X to finish processing the upload before returning]::SECS:_default' \
'--auth=[Authentication type]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'*-H+[Request headers]:HEADERS:_default' \
'*--header=[Request headers]:HEADERS:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Trace header]' \
'--trace[Trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':file -- File path:_default' \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-w+[Wait for X to finish processing instead of reading the status once]::SECS:_default' \
'--wait=[Wait for X to finish processing instead of reading the status once]::SECS:_default' \
'*-H+[Request headers]:HEADERS:_default' \
'*--header=[Request headers]:HEADERS:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Trace header]' \
'--trace[Trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':media_id -- Media ID:_default' \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::media_keys -- Media keys, such as 3_1146654567674912769, 1 to 100:_default' \
&& ret=0
;;
(analytics)
_arguments "${_arguments_options[@]}" : \
'--start=[Start of the window (RFC 3339 with a zone, such as 2026-10-01T00\:00\:00Z)]:TIME:_default' \
'--end=[End of the window (RFC 3339 with a zone); after the start]:TIME:_default' \
'--granularity=[Bucket size (X'\''s default is daily)]:GRANULARITY:(hourly daily total)' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'*::media_keys -- Media keys, such as 7_1146654567674912770, 1 to 100:_default' \
&& ret=0
;;
(alt-text)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':media_id -- Media id from `xr media upload`:_default' \
':text -- Alt text, up to 1000 characters:_default' \
&& ret=0
;;
(subtitles)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__media__subcmd__subtitles_commands" \
"*::: :->subtitles" \
&& ret=0

    case $state in
    (subtitles)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-media-subtitles-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
'--language=[Two-letter language code of the track (e.g. en)]:CODE:_default' \
'--name=[Language name viewers pick the track by (e.g. English)]:NAME:_default' \
'--category=[Category the video was uploaded with]:CATEGORY:(amplify_video tweet_video)' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':video_id -- Media id of the video:_default' \
':subtitles_id -- Media id of the subtitle file, uploaded with `--category subtitles`:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'--language=[Two-letter language code of the track to remove (e.g. en)]:CODE:_default' \
'--category=[Category the video was uploaded with]:CATEGORY:(amplify_video tweet_video)' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'--username=[\`OAuth2\` login to act as, by the name it is stored under\: its username, or the name given to \`xr auth oauth2\`]:USERNAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-t[Add X-B3-Flags trace header]' \
'--trace[Add X-B3-Flags trace header]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':video_id -- Media id of the video:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__media__subcmd__subtitles__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-media-subtitles-help-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__media__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-media-help-command-$line[1]:"
        case $line[1] in
            (upload)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(analytics)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(alt-text)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(subtitles)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__media__subcmd__help__subcmd__subtitles_commands" \
"*::: :->subtitles" \
&& ret=0

    case $state in
    (subtitles)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-media-help-subtitles-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(skill)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__subcmd__skill_commands" \
"*::: :->skill" \
&& ret=0

    case $state in
    (skill)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-skill-command-$line[1]:"
        case $line[1] in
            (install)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'()--all[Install into every known host in one invocation]' \
'--dry-run[Print the resolved git command without spawning]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::host -- Target host (e.g. claude_code, codex, cursor). Required unless `--all`:(claude_code codex cursor factory kiro opencode)' \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'()--all[Update every known host in one invocation]' \
'--dry-run[Print the resolved plan without removing or cloning]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::host -- Target host (e.g. claude_code, codex, cursor). Required unless `--all`:(claude_code codex cursor factory kiro opencode)' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__skill__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-skill-help-command-$line[1]:"
        case $line[1] in
            (install)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(schema)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--list[List all commands and their response types]' \
'--all[Output all schemas as a single JSON document]' \
'--envelope[Output the canonical agent-native output envelope schema]' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::command -- Command name to get the schema for (e.g. "post", "whoami", "envelope"):_default' \
&& ret=0
;;
(completions)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':shell -- Shell to generate completions for:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(version)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(examples)
_arguments "${_arguments_options[@]}" : \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
&& ret=0
;;
(validate)
_arguments "${_arguments_options[@]}" : \
'--schema=[Schema name to validate against (\`post\`, \`posts\`, \`user\`, \`users\`, \`dm\`, \`dms\`, \`dm-event\`, \`dm-block\`, \`usage\`, \`credits\`, \`envelope\`, \`like\`, \`hide\`, \`post-counts\`, \`analytics\`, \`media\`, \`media-analytics\`, \`spaces\`, \`stream-rules\`, \`stream-rules-update\`, \`stream-rule-counts\`, \`article-draft\`, \`article-published\`, \`note-created\`, \`note-evaluation\`, \`notes\`, \`broadcast\`, \`broadcasts\`, \`scheduled-broadcast\`, \`scheduled-broadcasts\`, \`chat-messages\`, \`chat-sent\`, \`chat-muted\`, \`chat-mutes\`, \`list\`, \`lists\`, \`list-updated\`, \`list-member\`, \`pinned\`, \`communities\`, \`community\`, \`news\`, \`news-story\`, \`trends\`, \`personalized-trends\`, \`follow\`, \`delete\`, \`repost\`, \`bookmark\`, \`bookmark-folders\`, \`bookmark-folder\`, \`bookmark-folder-posts\`, \`mute\`, \`block\`, \`moderators\`, \`alt-text\`, \`subtitles\`, \`webhook\`, \`webhooks\`, \`webhook-validation\`, \`webhook-replay\`, \`subscription-count\`, \`subscriptions\`, \`subscribed\`, \`stream-links\`, \`stream-link\`, \`stream-sources\`, \`stream-source\`, \`stream-source-deleted\`, \`updated\`, \`settings\`, \`call\`, \`scheduled-calls\`, \`scheduled-call\`, \`scheduled-call-updated\`, \`scheduled-call-cancelled\`, \`rsvp\`). Omit for auto-detection]:NAME:_default' \
'-v+[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--verbose=[Print request and response lines, and a note for each key X sent in its legacy post vocabulary]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (alias \`yml\`), csv, tsv. Any other value (toml, xml) is refused at exit 2 with a JSON envelope whose reason is \`invalid-args\`]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; prints \`json\` output on one line, as \`jsonl\` and \`ndjson\` always are]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--wait-on-rate-limit=[Wait out a rate limit and retry once, instead of failing with \`rate-limited\`]::WAIT_ON_RATE_LIMIT:(true false)' \
'--rate-limit-max-wait=[Longest \`--wait-on-rate-limit\` waits before its retry, in seconds]:SECS:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Check inputs and credentials offline, and send nothing]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::file -- File path to read JSON from. Pass `-` or omit to read from stdin:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-command-$line[1]:"
        case $line[1] in
            (post)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(reply)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(quote)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(delete)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(read)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(analytics)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(search-counts)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(search-users)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(users)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(user-posts)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(likers)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(reposters)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(quotes)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(whoami)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(user)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(timeline)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(mentions)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(like)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unlike)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(hide)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unhide)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(repost)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unrepost)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(bookmark)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unbookmark)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(bookmarks)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(bookmark-folders)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__bookmark-folders_commands" \
"*::: :->bookmark-folders" \
&& ret=0

    case $state in
    (bookmark-folders)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-bookmark-folders-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(likes)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(follow)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unfollow)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(following)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(followers)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(mute)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unmute)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(muted)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(block)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unblock)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(blocked)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(profile)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__profile_commands" \
"*::: :->profile" \
&& ret=0

    case $state in
    (profile)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-profile-command-$line[1]:"
        case $line[1] in
            (update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(image)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__profile__subcmd__image_commands" \
"*::: :->image" \
&& ret=0

    case $state in
    (image)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-profile-image-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(banner)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__profile__subcmd__banner_commands" \
"*::: :->banner" \
&& ret=0

    case $state in
    (banner)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-profile-banner-command-$line[1]:"
        case $line[1] in
            (set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(settings)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__settings_commands" \
"*::: :->settings" \
&& ret=0

    case $state in
    (settings)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-settings-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(usage)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__usage_commands" \
"*::: :->usage" \
&& ret=0

    case $state in
    (usage)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-usage-command-$line[1]:"
        case $line[1] in
            (credits)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(dm)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(conversations)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__conversations_commands" \
"*::: :->conversations" \
&& ret=0

    case $state in
    (conversations)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-conversations-command-$line[1]:"
        case $line[1] in
            (start)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(send)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(history)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(with)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(dm-events)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__dm-events_commands" \
"*::: :->dm-events" \
&& ret=0

    case $state in
    (dm-events)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-dm-events-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(dm-block)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(dm-unblock)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(dms)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(lists)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__lists_commands" \
"*::: :->lists" \
&& ret=0

    case $state in
    (lists)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-lists-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(followed)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(memberships)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(pinned)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(followers)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(members)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__lists__subcmd__members_commands" \
"*::: :->members" \
&& ret=0

    case $state in
    (members)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-lists-members-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(follow)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unfollow)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(pin)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unpin)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(communities)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__communities_commands" \
"*::: :->communities" \
&& ret=0

    case $state in
    (communities)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-communities-command-$line[1]:"
        case $line[1] in
            (search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(news)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__news_commands" \
"*::: :->news" \
&& ret=0

    case $state in
    (news)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-news-command-$line[1]:"
        case $line[1] in
            (search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(spaces)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__spaces_commands" \
"*::: :->spaces" \
&& ret=0

    case $state in
    (spaces)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-spaces-command-$line[1]:"
        case $line[1] in
            (search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(by-creator)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(posts)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(stream)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__stream_commands" \
"*::: :->stream" \
&& ret=0

    case $state in
    (stream)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-stream-command-$line[1]:"
        case $line[1] in
            (rules)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__stream__subcmd__rules_commands" \
"*::: :->rules" \
&& ret=0

    case $state in
    (rules)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-stream-rules-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(count)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(articles)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__articles_commands" \
"*::: :->articles" \
&& ret=0

    case $state in
    (articles)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-articles-command-$line[1]:"
        case $line[1] in
            (draft)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(publish)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(notes)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__notes_commands" \
"*::: :->notes" \
&& ret=0

    case $state in
    (notes)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-notes-command-$line[1]:"
        case $line[1] in
            (write)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(evaluate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(eligible)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(written)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(trends)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__trends_commands" \
"*::: :->trends" \
&& ret=0

    case $state in
    (trends)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-trends-command-$line[1]:"
        case $line[1] in
            (place)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(personal)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(broadcasts)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__broadcasts_commands" \
"*::: :->broadcasts" \
&& ret=0

    case $state in
    (broadcasts)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-broadcasts-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(scheduled)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled_commands" \
"*::: :->scheduled" \
&& ret=0

    case $state in
    (scheduled)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-broadcasts-scheduled-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(live)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(chat)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__broadcasts__subcmd__chat_commands" \
"*::: :->chat" \
&& ret=0

    case $state in
    (chat)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-broadcasts-chat-command-$line[1]:"
        case $line[1] in
            (history)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(send)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(mute)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unmute)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(moderators)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__broadcasts__subcmd__moderators_commands" \
"*::: :->moderators" \
&& ret=0

    case $state in
    (moderators)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-broadcasts-moderators-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(mutes)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__broadcasts__subcmd__mutes_commands" \
"*::: :->mutes" \
&& ret=0

    case $state in
    (mutes)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-broadcasts-mutes-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(sources)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__broadcasts__subcmd__sources_commands" \
"*::: :->sources" \
&& ret=0

    case $state in
    (sources)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-broadcasts-sources-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(rename)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(calls)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__calls_commands" \
"*::: :->calls" \
&& ret=0

    case $state in
    (calls)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-calls-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(scheduled)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__calls__subcmd__scheduled_commands" \
"*::: :->scheduled" \
&& ret=0

    case $state in
    (scheduled)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-calls-scheduled-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(cancel)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(rsvp)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(webhooks)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__webhooks_commands" \
"*::: :->webhooks" \
&& ret=0

    case $state in
    (webhooks)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-webhooks-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(subscriptions)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions_commands" \
"*::: :->subscriptions" \
&& ret=0

    case $state in
    (subscriptions)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-webhooks-subscriptions-command-$line[1]:"
        case $line[1] in
            (count)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(stream-links)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__webhooks__subcmd__stream-links_commands" \
"*::: :->stream-links" \
&& ret=0

    case $state in
    (stream-links)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-webhooks-stream-links-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(listen)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(replay)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(auth)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__auth_commands" \
"*::: :->auth" \
&& ret=0

    case $state in
    (auth)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-auth-command-$line[1]:"
        case $line[1] in
            (oauth2)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(oauth1)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(app)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(clear)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(apps)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__auth__subcmd__apps_commands" \
"*::: :->apps" \
&& ret=0

    case $state in
    (apps)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-auth-apps-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(redirect-uri)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect-uri_commands" \
"*::: :->redirect-uri" \
&& ret=0

    case $state in
    (redirect-uri)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-auth-apps-redirect-uri-command-$line[1]:"
        case $line[1] in
            (get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(default)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(media)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__media_commands" \
"*::: :->media" \
&& ret=0

    case $state in
    (media)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-media-command-$line[1]:"
        case $line[1] in
            (upload)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(get)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(analytics)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(alt-text)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(subtitles)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__media__subcmd__subtitles_commands" \
"*::: :->subtitles" \
&& ret=0

    case $state in
    (subtitles)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-media-subtitles-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(skill)
_arguments "${_arguments_options[@]}" : \
":: :_xr__subcmd__help__subcmd__skill_commands" \
"*::: :->skill" \
&& ret=0

    case $state in
    (skill)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:xr-help-skill-command-$line[1]:"
        case $line[1] in
            (install)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(update)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(schema)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(completions)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(version)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(examples)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_xr_commands] )) ||
_xr_commands() {
    local commands; commands=(
'post:Post to X' \
'reply:Reply to a post' \
'quote:Quote a post' \
'delete:Delete a post' \
'read:Read a post' \
'analytics:Report impressions and engagement for your posts over a time window' \
'search-counts:Count the recent posts that match a query, per time bucket' \
'search-users:Find accounts by keyword' \
'users:Look up 1 to 100 accounts by handle or by id in one request' \
'user-posts:List the posts a user wrote' \
'posts:Read 1 to 100 posts by id or URL in one request' \
'search:Search recent posts' \
'likers:List the users who liked a post' \
'reposters:List the users who reposted a post' \
'quotes:List the posts that quote a post' \
'whoami:Show the authenticated user'\''s profile' \
'user:Look up a user by username' \
'timeline:Show your home timeline' \
'mentions:Show your recent mentions' \
'like:Like a post' \
'unlike:Unlike a post' \
'hide:Hide a reply to your post' \
'unhide:Show a hidden reply to your post again' \
'repost:Repost a post' \
'unrepost:Undo a repost' \
'bookmark:Bookmark a post' \
'unbookmark:Remove a bookmark' \
'bookmarks:List your bookmarks' \
'bookmark-folders:Your bookmark folders, and the posts in each' \
'likes:List your liked posts' \
'follow:Follow a user' \
'unfollow:Unfollow a user' \
'following:List users you follow' \
'followers:List your followers' \
'mute:Mute a user' \
'unmute:Unmute a user' \
'muted:List users you have muted' \
'block:Block a user' \
'unblock:Unblock a user' \
'blocked:List users you have blocked' \
'profile:Change your profile\: display name, bio, website, location, image, and banner' \
'settings:Read or change your account settings\: privacy, discoverability, messages, and notifications' \
'usage:Show API usage (post caps, daily breakdown)' \
'dm:Send a direct message' \
'conversations:Start a group conversation, send into one, or read one' \
'dm-events:Read or delete one direct message, by event id' \
'dm-block:Stop a user sending you direct messages' \
'dm-unblock:Let a user send you direct messages again' \
'dms:List recent direct messages' \
'lists:Your lists, a list'\''s posts and members, and following and pinning' \
'communities:Search communities, and read one' \
'news:Search news stories, and read one' \
'spaces:Find Spaces by keyword, id, or creator, and read their posts' \
'stream:The filtered stream'\''s rules' \
'articles:Write and publish long-form X Articles' \
'notes:Write Community Notes through X'\''s AI Note Writer API' \
'trends:Trends for a place, and your personalized trends' \
'broadcasts:Your broadcasts, scheduled broadcasts, chat, moderators, and stream sources' \
'calls:X calls, and the calls on your calendar (preview)' \
'webhooks:Webhook registration, subscriptions, stream links, a local listener, and replay' \
'auth:Authentication management' \
'media:Media upload, lookup by key, video metrics, alt text, and subtitles' \
'skill:Install or manage the xurl-rs skill bundle' \
'schema:Show JSON Schema for a command'\''s response type' \
'completions:Generate shell completion script' \
'version:Show xurl version information' \
'examples:Print a curated gallery of invocation examples grouped by use case' \
'validate:Validate a JSON document against a bundled response schema' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr commands' commands "$@"
}
(( $+functions[_xr__subcmd__analytics_commands] )) ||
_xr__subcmd__analytics_commands() {
    local commands; commands=()
    _describe -t commands 'xr analytics commands' commands "$@"
}
(( $+functions[_xr__subcmd__articles_commands] )) ||
_xr__subcmd__articles_commands() {
    local commands; commands=(
'draft:Save a draft X Article from a plain-text body' \
'publish:Publish a draft X Article' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr articles commands' commands "$@"
}
(( $+functions[_xr__subcmd__articles__subcmd__draft_commands] )) ||
_xr__subcmd__articles__subcmd__draft_commands() {
    local commands; commands=()
    _describe -t commands 'xr articles draft commands' commands "$@"
}
(( $+functions[_xr__subcmd__articles__subcmd__help_commands] )) ||
_xr__subcmd__articles__subcmd__help_commands() {
    local commands; commands=(
'draft:Save a draft X Article from a plain-text body' \
'publish:Publish a draft X Article' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr articles help commands' commands "$@"
}
(( $+functions[_xr__subcmd__articles__subcmd__help__subcmd__draft_commands] )) ||
_xr__subcmd__articles__subcmd__help__subcmd__draft_commands() {
    local commands; commands=()
    _describe -t commands 'xr articles help draft commands' commands "$@"
}
(( $+functions[_xr__subcmd__articles__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__articles__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr articles help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__articles__subcmd__help__subcmd__publish_commands] )) ||
_xr__subcmd__articles__subcmd__help__subcmd__publish_commands() {
    local commands; commands=()
    _describe -t commands 'xr articles help publish commands' commands "$@"
}
(( $+functions[_xr__subcmd__articles__subcmd__publish_commands] )) ||
_xr__subcmd__articles__subcmd__publish_commands() {
    local commands; commands=()
    _describe -t commands 'xr articles publish commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth_commands] )) ||
_xr__subcmd__auth_commands() {
    local commands; commands=(
'oauth2:Configure \`OAuth2\` authentication' \
'oauth1:Configure \`OAuth1\` authentication' \
'app:Configure app-auth (bearer token)' \
'status:Show authentication status' \
'clear:Clear authentication tokens' \
'apps:Manage registered X API apps' \
'default:Set default app and/or user' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr auth commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__app_commands] )) ||
_xr__subcmd__auth__subcmd__app_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth app commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps_commands] )) ||
_xr__subcmd__auth__subcmd__apps_commands() {
    local commands; commands=(
'add:Register a new X API app' \
'update:Update credentials for an existing app' \
'remove:Remove a registered app' \
'list:List registered apps' \
'redirect-uri:Inspect or set the stored \`OAuth2\` redirect URI for an app' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr auth apps commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__add_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps add commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__help_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__help_commands() {
    local commands; commands=(
'add:Register a new X API app' \
'update:Update credentials for an existing app' \
'remove:Remove a registered app' \
'list:List registered apps' \
'redirect-uri:Inspect or set the stored \`OAuth2\` redirect URI for an app' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr auth apps help commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect-uri_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect-uri_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
    )
    _describe -t commands 'xr auth apps help redirect-uri commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect-uri__subcmd__get_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect-uri__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help redirect-uri get commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect-uri__subcmd__set_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__redirect-uri__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help redirect-uri set commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__update_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__help__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help update commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__list_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps list commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr auth apps redirect-uri commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__get_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri get commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__help_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__help_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr auth apps redirect-uri help commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__help__subcmd__set_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__help__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri help set commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__set_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri set commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__remove_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__apps__subcmd__update_commands] )) ||
_xr__subcmd__auth__subcmd__apps__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps update commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__clear_commands] )) ||
_xr__subcmd__auth__subcmd__clear_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth clear commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__default_commands] )) ||
_xr__subcmd__auth__subcmd__default_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth default commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help_commands] )) ||
_xr__subcmd__auth__subcmd__help_commands() {
    local commands; commands=(
'oauth2:Configure \`OAuth2\` authentication' \
'oauth1:Configure \`OAuth1\` authentication' \
'app:Configure app-auth (bearer token)' \
'status:Show authentication status' \
'clear:Clear authentication tokens' \
'apps:Manage registered X API apps' \
'default:Set default app and/or user' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr auth help commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__app_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__app_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help app commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__apps_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__apps_commands() {
    local commands; commands=(
'add:Register a new X API app' \
'update:Update credentials for an existing app' \
'remove:Remove a registered app' \
'list:List registered apps' \
'redirect-uri:Inspect or set the stored \`OAuth2\` redirect URI for an app' \
    )
    _describe -t commands 'xr auth help apps commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__add_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps add commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__list_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps list commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect-uri_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect-uri_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
    )
    _describe -t commands 'xr auth help apps redirect-uri commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect-uri__subcmd__get_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect-uri__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps redirect-uri get commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect-uri__subcmd__set_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__redirect-uri__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps redirect-uri set commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__remove_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__update_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__apps__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps update commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__clear_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__clear_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help clear commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__default_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__default_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help default commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__oauth1_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__oauth1_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help oauth1 commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__oauth2_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__oauth2_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help oauth2 commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__help__subcmd__status_commands] )) ||
_xr__subcmd__auth__subcmd__help__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help status commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__oauth1_commands] )) ||
_xr__subcmd__auth__subcmd__oauth1_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth oauth1 commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__oauth2_commands] )) ||
_xr__subcmd__auth__subcmd__oauth2_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth oauth2 commands' commands "$@"
}
(( $+functions[_xr__subcmd__auth__subcmd__status_commands] )) ||
_xr__subcmd__auth__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth status commands' commands "$@"
}
(( $+functions[_xr__subcmd__block_commands] )) ||
_xr__subcmd__block_commands() {
    local commands; commands=()
    _describe -t commands 'xr block commands' commands "$@"
}
(( $+functions[_xr__subcmd__blocked_commands] )) ||
_xr__subcmd__blocked_commands() {
    local commands; commands=()
    _describe -t commands 'xr blocked commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark_commands] )) ||
_xr__subcmd__bookmark_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmark commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark-folders_commands] )) ||
_xr__subcmd__bookmark-folders_commands() {
    local commands; commands=(
'list:List the folders you sort bookmarks into' \
'posts:List the posts in one of your bookmark folders, by id' \
'add:Create a bookmark folder' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr bookmark-folders commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark-folders__subcmd__add_commands] )) ||
_xr__subcmd__bookmark-folders__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmark-folders add commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark-folders__subcmd__help_commands] )) ||
_xr__subcmd__bookmark-folders__subcmd__help_commands() {
    local commands; commands=(
'list:List the folders you sort bookmarks into' \
'posts:List the posts in one of your bookmark folders, by id' \
'add:Create a bookmark folder' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr bookmark-folders help commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark-folders__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__bookmark-folders__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmark-folders help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark-folders__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__bookmark-folders__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmark-folders help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark-folders__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__bookmark-folders__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmark-folders help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark-folders__subcmd__help__subcmd__posts_commands] )) ||
_xr__subcmd__bookmark-folders__subcmd__help__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmark-folders help posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark-folders__subcmd__list_commands] )) ||
_xr__subcmd__bookmark-folders__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmark-folders list commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmark-folders__subcmd__posts_commands] )) ||
_xr__subcmd__bookmark-folders__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmark-folders posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__bookmarks_commands] )) ||
_xr__subcmd__bookmarks_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmarks commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts_commands] )) ||
_xr__subcmd__broadcasts_commands() {
    local commands; commands=(
'list:List your broadcasts' \
'get:Read one broadcast, by id' \
'scheduled:Schedule broadcasts, change or cancel them, and start them early' \
'chat:Read and moderate a broadcast'\''s chat' \
'moderators:Manage who moderates your broadcast chats' \
'mutes:Mute people in every chat of your broadcasts' \
'sources:Manage the RTMP and RTMPS ingest points you stream to' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat_commands() {
    local commands; commands=(
'history:List a broadcast'\''s chat' \
'send:Send a message into a broadcast'\''s chat' \
'remove:Remove a message from a broadcast'\''s chat' \
'mute:Mute someone in a broadcast'\''s chat' \
'unmute:Unmute someone in a broadcast'\''s chat' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts chat commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__help_commands() {
    local commands; commands=(
'history:List a broadcast'\''s chat' \
'send:Send a message into a broadcast'\''s chat' \
'remove:Remove a message from a broadcast'\''s chat' \
'mute:Mute someone in a broadcast'\''s chat' \
'unmute:Unmute someone in a broadcast'\''s chat' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts chat help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__history_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat help history commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__mute_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__mute_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat help mute commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__send_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__send_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat help send commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__unmute_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__help__subcmd__unmute_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat help unmute commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__history_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat history commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__mute_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__mute_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat mute commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__send_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__send_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat send commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__chat__subcmd__unmute_commands] )) ||
_xr__subcmd__broadcasts__subcmd__chat__subcmd__unmute_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts chat unmute commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__get_commands] )) ||
_xr__subcmd__broadcasts__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts get commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help_commands() {
    local commands; commands=(
'list:List your broadcasts' \
'get:Read one broadcast, by id' \
'scheduled:Schedule broadcasts, change or cancel them, and start them early' \
'chat:Read and moderate a broadcast'\''s chat' \
'moderators:Manage who moderates your broadcast chats' \
'mutes:Mute people in every chat of your broadcasts' \
'sources:Manage the RTMP and RTMPS ingest points you stream to' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__chat_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__chat_commands() {
    local commands; commands=(
'history:List a broadcast'\''s chat' \
'send:Send a message into a broadcast'\''s chat' \
'remove:Remove a message from a broadcast'\''s chat' \
'mute:Mute someone in a broadcast'\''s chat' \
'unmute:Unmute someone in a broadcast'\''s chat' \
    )
    _describe -t commands 'xr broadcasts help chat commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__history_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help chat history commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__mute_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__mute_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help chat mute commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help chat remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__send_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__send_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help chat send commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__unmute_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__chat__subcmd__unmute_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help chat unmute commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__moderators_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__moderators_commands() {
    local commands; commands=(
'list:List your broadcast chat moderators' \
'add:Add a broadcast chat moderator' \
'remove:Remove a broadcast chat moderator' \
    )
    _describe -t commands 'xr broadcasts help moderators commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__moderators__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__moderators__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help moderators add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__moderators__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__moderators__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help moderators list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__moderators__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__moderators__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help moderators remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__mutes_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__mutes_commands() {
    local commands; commands=(
'list:List who is muted in your broadcast chats, and until when' \
'add:Mute someone in every chat of your broadcasts' \
'remove:Unmute someone in your broadcast chats' \
    )
    _describe -t commands 'xr broadcasts help mutes commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__mutes__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__mutes__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help mutes add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__mutes__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__mutes__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help mutes list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__mutes__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__mutes__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help mutes remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled_commands() {
    local commands; commands=(
'list:List your scheduled broadcasts' \
'get:Read one scheduled broadcast, by id' \
'add:Schedule a broadcast from one of your stream sources' \
'update:Move a scheduled broadcast, or change its details' \
'remove:Cancel a scheduled broadcast' \
'live:Start a scheduled broadcast now' \
    )
    _describe -t commands 'xr broadcasts help scheduled commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help scheduled add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__get_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help scheduled get commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help scheduled list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__live_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__live_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help scheduled live commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help scheduled remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__update_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__scheduled__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help scheduled update commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__sources_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__sources_commands() {
    local commands; commands=(
'list:List your stream sources' \
'get:Show one stream source, and whether it is receiving video' \
'add:Create a stream source' \
'rename:Rename a stream source' \
'remove:Delete a stream source; an encoder sending to it is no longer received' \
    )
    _describe -t commands 'xr broadcasts help sources commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help sources add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__get_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help sources get commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help sources list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help sources remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__rename_commands] )) ||
_xr__subcmd__broadcasts__subcmd__help__subcmd__sources__subcmd__rename_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts help sources rename commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__moderators_commands] )) ||
_xr__subcmd__broadcasts__subcmd__moderators_commands() {
    local commands; commands=(
'list:List your broadcast chat moderators' \
'add:Add a broadcast chat moderator' \
'remove:Remove a broadcast chat moderator' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts moderators commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__moderators__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__moderators__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts moderators add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help_commands() {
    local commands; commands=(
'list:List your broadcast chat moderators' \
'add:Add a broadcast chat moderator' \
'remove:Remove a broadcast chat moderator' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts moderators help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts moderators help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts moderators help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts moderators help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__moderators__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts moderators help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__moderators__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__moderators__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts moderators list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__moderators__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__moderators__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts moderators remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__mutes_commands] )) ||
_xr__subcmd__broadcasts__subcmd__mutes_commands() {
    local commands; commands=(
'list:List who is muted in your broadcast chats, and until when' \
'add:Mute someone in every chat of your broadcasts' \
'remove:Unmute someone in your broadcast chats' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts mutes commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__mutes__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__mutes__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts mutes add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help_commands() {
    local commands; commands=(
'list:List who is muted in your broadcast chats, and until when' \
'add:Mute someone in every chat of your broadcasts' \
'remove:Unmute someone in your broadcast chats' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts mutes help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts mutes help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts mutes help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts mutes help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__mutes__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts mutes help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__mutes__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__mutes__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts mutes list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__mutes__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__mutes__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts mutes remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled_commands() {
    local commands; commands=(
'list:List your scheduled broadcasts' \
'get:Read one scheduled broadcast, by id' \
'add:Schedule a broadcast from one of your stream sources' \
'update:Move a scheduled broadcast, or change its details' \
'remove:Cancel a scheduled broadcast' \
'live:Start a scheduled broadcast now' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts scheduled commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__get_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled get commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help_commands() {
    local commands; commands=(
'list:List your scheduled broadcasts' \
'get:Read one scheduled broadcast, by id' \
'add:Schedule a broadcast from one of your stream sources' \
'update:Move a scheduled broadcast, or change its details' \
'remove:Cancel a scheduled broadcast' \
'live:Start a scheduled broadcast now' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts scheduled help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__live_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__live_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled help live commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__update_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__help__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled help update commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__live_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__live_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled live commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__update_commands] )) ||
_xr__subcmd__broadcasts__subcmd__scheduled__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts scheduled update commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources_commands() {
    local commands; commands=(
'list:List your stream sources' \
'get:Show one stream source, and whether it is receiving video' \
'add:Create a stream source' \
'rename:Rename a stream source' \
'remove:Delete a stream source; an encoder sending to it is no longer received' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts sources commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__get_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources get commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__help_commands() {
    local commands; commands=(
'list:List your stream sources' \
'get:Show one stream source, and whether it is receiving video' \
'add:Create a stream source' \
'rename:Rename a stream source' \
'remove:Delete a stream source; an encoder sending to it is no longer received' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr broadcasts sources help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__rename_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__help__subcmd__rename_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources help rename commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__list_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources list commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__remove_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__broadcasts__subcmd__sources__subcmd__rename_commands] )) ||
_xr__subcmd__broadcasts__subcmd__sources__subcmd__rename_commands() {
    local commands; commands=()
    _describe -t commands 'xr broadcasts sources rename commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls_commands] )) ||
_xr__subcmd__calls_commands() {
    local commands; commands=(
'get:Show an X call\: its title, join policy, host, and who is in it' \
'scheduled:The calls on your calendar\: schedule, change, cancel, and answer them' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr calls commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__get_commands] )) ||
_xr__subcmd__calls__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls get commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help_commands] )) ||
_xr__subcmd__calls__subcmd__help_commands() {
    local commands; commands=(
'get:Show an X call\: its title, join policy, host, and who is in it' \
'scheduled:The calls on your calendar\: schedule, change, cancel, and answer them' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr calls help commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__calls__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__calls__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help__subcmd__scheduled_commands] )) ||
_xr__subcmd__calls__subcmd__help__subcmd__scheduled_commands() {
    local commands; commands=(
'list:List the calls on your calendar, earliest first' \
'get:Show a scheduled call, with its invitees and their answers' \
'add:Schedule a call on your X Calls calendar and invite people to it' \
'update:Move, rename, or re-invite a call you organize' \
'cancel:Cancel a call you organize and take it off invitees'\'' calendars' \
'rsvp:Answer an invitation to a scheduled call' \
    )
    _describe -t commands 'xr calls help scheduled commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__add_commands] )) ||
_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls help scheduled add commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__cancel_commands] )) ||
_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__cancel_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls help scheduled cancel commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__get_commands] )) ||
_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls help scheduled get commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__list_commands] )) ||
_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls help scheduled list commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__rsvp_commands] )) ||
_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__rsvp_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls help scheduled rsvp commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__update_commands] )) ||
_xr__subcmd__calls__subcmd__help__subcmd__scheduled__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls help scheduled update commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled_commands() {
    local commands; commands=(
'list:List the calls on your calendar, earliest first' \
'get:Show a scheduled call, with its invitees and their answers' \
'add:Schedule a call on your X Calls calendar and invite people to it' \
'update:Move, rename, or re-invite a call you organize' \
'cancel:Cancel a call you organize and take it off invitees'\'' calendars' \
'rsvp:Answer an invitation to a scheduled call' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr calls scheduled commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__add_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled add commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__cancel_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__cancel_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled cancel commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__get_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled get commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__help_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__help_commands() {
    local commands; commands=(
'list:List the calls on your calendar, earliest first' \
'get:Show a scheduled call, with its invitees and their answers' \
'add:Schedule a call on your X Calls calendar and invite people to it' \
'update:Move, rename, or re-invite a call you organize' \
'cancel:Cancel a call you organize and take it off invitees'\'' calendars' \
'rsvp:Answer an invitation to a scheduled call' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr calls scheduled help commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__cancel_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__cancel_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled help cancel commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__rsvp_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__rsvp_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled help rsvp commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__update_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__help__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled help update commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__list_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled list commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__rsvp_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__rsvp_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled rsvp commands' commands "$@"
}
(( $+functions[_xr__subcmd__calls__subcmd__scheduled__subcmd__update_commands] )) ||
_xr__subcmd__calls__subcmd__scheduled__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr calls scheduled update commands' commands "$@"
}
(( $+functions[_xr__subcmd__communities_commands] )) ||
_xr__subcmd__communities_commands() {
    local commands; commands=(
'search:Search communities' \
'get:Read one community, by id' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr communities commands' commands "$@"
}
(( $+functions[_xr__subcmd__communities__subcmd__get_commands] )) ||
_xr__subcmd__communities__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr communities get commands' commands "$@"
}
(( $+functions[_xr__subcmd__communities__subcmd__help_commands] )) ||
_xr__subcmd__communities__subcmd__help_commands() {
    local commands; commands=(
'search:Search communities' \
'get:Read one community, by id' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr communities help commands' commands "$@"
}
(( $+functions[_xr__subcmd__communities__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__communities__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr communities help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__communities__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__communities__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr communities help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__communities__subcmd__help__subcmd__search_commands] )) ||
_xr__subcmd__communities__subcmd__help__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr communities help search commands' commands "$@"
}
(( $+functions[_xr__subcmd__communities__subcmd__search_commands] )) ||
_xr__subcmd__communities__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr communities search commands' commands "$@"
}
(( $+functions[_xr__subcmd__completions_commands] )) ||
_xr__subcmd__completions_commands() {
    local commands; commands=()
    _describe -t commands 'xr completions commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations_commands] )) ||
_xr__subcmd__conversations_commands() {
    local commands; commands=(
'start:Start a group conversation and send its first message' \
'send:Send a message into a conversation, by id' \
'history:List the messages in a conversation, by id' \
'with:List the messages between you and one person' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr conversations commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__help_commands] )) ||
_xr__subcmd__conversations__subcmd__help_commands() {
    local commands; commands=(
'start:Start a group conversation and send its first message' \
'send:Send a message into a conversation, by id' \
'history:List the messages in a conversation, by id' \
'with:List the messages between you and one person' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr conversations help commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__conversations__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr conversations help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__help__subcmd__history_commands] )) ||
_xr__subcmd__conversations__subcmd__help__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'xr conversations help history commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__help__subcmd__send_commands] )) ||
_xr__subcmd__conversations__subcmd__help__subcmd__send_commands() {
    local commands; commands=()
    _describe -t commands 'xr conversations help send commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__help__subcmd__start_commands] )) ||
_xr__subcmd__conversations__subcmd__help__subcmd__start_commands() {
    local commands; commands=()
    _describe -t commands 'xr conversations help start commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__help__subcmd__with_commands] )) ||
_xr__subcmd__conversations__subcmd__help__subcmd__with_commands() {
    local commands; commands=()
    _describe -t commands 'xr conversations help with commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__history_commands] )) ||
_xr__subcmd__conversations__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'xr conversations history commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__send_commands] )) ||
_xr__subcmd__conversations__subcmd__send_commands() {
    local commands; commands=()
    _describe -t commands 'xr conversations send commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__start_commands] )) ||
_xr__subcmd__conversations__subcmd__start_commands() {
    local commands; commands=()
    _describe -t commands 'xr conversations start commands' commands "$@"
}
(( $+functions[_xr__subcmd__conversations__subcmd__with_commands] )) ||
_xr__subcmd__conversations__subcmd__with_commands() {
    local commands; commands=()
    _describe -t commands 'xr conversations with commands' commands "$@"
}
(( $+functions[_xr__subcmd__delete_commands] )) ||
_xr__subcmd__delete_commands() {
    local commands; commands=()
    _describe -t commands 'xr delete commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm_commands] )) ||
_xr__subcmd__dm_commands() {
    local commands; commands=()
    _describe -t commands 'xr dm commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm-block_commands] )) ||
_xr__subcmd__dm-block_commands() {
    local commands; commands=()
    _describe -t commands 'xr dm-block commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm-events_commands] )) ||
_xr__subcmd__dm-events_commands() {
    local commands; commands=(
'get:Read one direct message, by event id' \
'remove:Delete one of your direct messages, by event id' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr dm-events commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm-events__subcmd__get_commands] )) ||
_xr__subcmd__dm-events__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr dm-events get commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm-events__subcmd__help_commands] )) ||
_xr__subcmd__dm-events__subcmd__help_commands() {
    local commands; commands=(
'get:Read one direct message, by event id' \
'remove:Delete one of your direct messages, by event id' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr dm-events help commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm-events__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__dm-events__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr dm-events help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm-events__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__dm-events__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr dm-events help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm-events__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__dm-events__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr dm-events help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm-events__subcmd__remove_commands] )) ||
_xr__subcmd__dm-events__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr dm-events remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__dm-unblock_commands] )) ||
_xr__subcmd__dm-unblock_commands() {
    local commands; commands=()
    _describe -t commands 'xr dm-unblock commands' commands "$@"
}
(( $+functions[_xr__subcmd__dms_commands] )) ||
_xr__subcmd__dms_commands() {
    local commands; commands=()
    _describe -t commands 'xr dms commands' commands "$@"
}
(( $+functions[_xr__subcmd__examples_commands] )) ||
_xr__subcmd__examples_commands() {
    local commands; commands=()
    _describe -t commands 'xr examples commands' commands "$@"
}
(( $+functions[_xr__subcmd__follow_commands] )) ||
_xr__subcmd__follow_commands() {
    local commands; commands=()
    _describe -t commands 'xr follow commands' commands "$@"
}
(( $+functions[_xr__subcmd__followers_commands] )) ||
_xr__subcmd__followers_commands() {
    local commands; commands=()
    _describe -t commands 'xr followers commands' commands "$@"
}
(( $+functions[_xr__subcmd__following_commands] )) ||
_xr__subcmd__following_commands() {
    local commands; commands=()
    _describe -t commands 'xr following commands' commands "$@"
}
(( $+functions[_xr__subcmd__help_commands] )) ||
_xr__subcmd__help_commands() {
    local commands; commands=(
'post:Post to X' \
'reply:Reply to a post' \
'quote:Quote a post' \
'delete:Delete a post' \
'read:Read a post' \
'analytics:Report impressions and engagement for your posts over a time window' \
'search-counts:Count the recent posts that match a query, per time bucket' \
'search-users:Find accounts by keyword' \
'users:Look up 1 to 100 accounts by handle or by id in one request' \
'user-posts:List the posts a user wrote' \
'posts:Read 1 to 100 posts by id or URL in one request' \
'search:Search recent posts' \
'likers:List the users who liked a post' \
'reposters:List the users who reposted a post' \
'quotes:List the posts that quote a post' \
'whoami:Show the authenticated user'\''s profile' \
'user:Look up a user by username' \
'timeline:Show your home timeline' \
'mentions:Show your recent mentions' \
'like:Like a post' \
'unlike:Unlike a post' \
'hide:Hide a reply to your post' \
'unhide:Show a hidden reply to your post again' \
'repost:Repost a post' \
'unrepost:Undo a repost' \
'bookmark:Bookmark a post' \
'unbookmark:Remove a bookmark' \
'bookmarks:List your bookmarks' \
'bookmark-folders:Your bookmark folders, and the posts in each' \
'likes:List your liked posts' \
'follow:Follow a user' \
'unfollow:Unfollow a user' \
'following:List users you follow' \
'followers:List your followers' \
'mute:Mute a user' \
'unmute:Unmute a user' \
'muted:List users you have muted' \
'block:Block a user' \
'unblock:Unblock a user' \
'blocked:List users you have blocked' \
'profile:Change your profile\: display name, bio, website, location, image, and banner' \
'settings:Read or change your account settings\: privacy, discoverability, messages, and notifications' \
'usage:Show API usage (post caps, daily breakdown)' \
'dm:Send a direct message' \
'conversations:Start a group conversation, send into one, or read one' \
'dm-events:Read or delete one direct message, by event id' \
'dm-block:Stop a user sending you direct messages' \
'dm-unblock:Let a user send you direct messages again' \
'dms:List recent direct messages' \
'lists:Your lists, a list'\''s posts and members, and following and pinning' \
'communities:Search communities, and read one' \
'news:Search news stories, and read one' \
'spaces:Find Spaces by keyword, id, or creator, and read their posts' \
'stream:The filtered stream'\''s rules' \
'articles:Write and publish long-form X Articles' \
'notes:Write Community Notes through X'\''s AI Note Writer API' \
'trends:Trends for a place, and your personalized trends' \
'broadcasts:Your broadcasts, scheduled broadcasts, chat, moderators, and stream sources' \
'calls:X calls, and the calls on your calendar (preview)' \
'webhooks:Webhook registration, subscriptions, stream links, a local listener, and replay' \
'auth:Authentication management' \
'media:Media upload, lookup by key, video metrics, alt text, and subtitles' \
'skill:Install or manage the xurl-rs skill bundle' \
'schema:Show JSON Schema for a command'\''s response type' \
'completions:Generate shell completion script' \
'version:Show xurl version information' \
'examples:Print a curated gallery of invocation examples grouped by use case' \
'validate:Validate a JSON document against a bundled response schema' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr help commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__analytics_commands] )) ||
_xr__subcmd__help__subcmd__analytics_commands() {
    local commands; commands=()
    _describe -t commands 'xr help analytics commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__articles_commands] )) ||
_xr__subcmd__help__subcmd__articles_commands() {
    local commands; commands=(
'draft:Save a draft X Article from a plain-text body' \
'publish:Publish a draft X Article' \
    )
    _describe -t commands 'xr help articles commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__articles__subcmd__draft_commands] )) ||
_xr__subcmd__help__subcmd__articles__subcmd__draft_commands() {
    local commands; commands=()
    _describe -t commands 'xr help articles draft commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__articles__subcmd__publish_commands] )) ||
_xr__subcmd__help__subcmd__articles__subcmd__publish_commands() {
    local commands; commands=()
    _describe -t commands 'xr help articles publish commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth_commands] )) ||
_xr__subcmd__help__subcmd__auth_commands() {
    local commands; commands=(
'oauth2:Configure \`OAuth2\` authentication' \
'oauth1:Configure \`OAuth1\` authentication' \
'app:Configure app-auth (bearer token)' \
'status:Show authentication status' \
'clear:Clear authentication tokens' \
'apps:Manage registered X API apps' \
'default:Set default app and/or user' \
    )
    _describe -t commands 'xr help auth commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__app_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__app_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth app commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__apps_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__apps_commands() {
    local commands; commands=(
'add:Register a new X API app' \
'update:Update credentials for an existing app' \
'remove:Remove a registered app' \
'list:List registered apps' \
'redirect-uri:Inspect or set the stored \`OAuth2\` redirect URI for an app' \
    )
    _describe -t commands 'xr help auth apps commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect-uri_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect-uri_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
    )
    _describe -t commands 'xr help auth apps redirect-uri commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps redirect-uri get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__set_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__redirect-uri__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps redirect-uri set commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__update_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__apps__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps update commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__clear_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__clear_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth clear commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__default_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__default_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth default commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__oauth1_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__oauth1_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth oauth1 commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__oauth2_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__oauth2_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth oauth2 commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__auth__subcmd__status_commands] )) ||
_xr__subcmd__help__subcmd__auth__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth status commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__block_commands] )) ||
_xr__subcmd__help__subcmd__block_commands() {
    local commands; commands=()
    _describe -t commands 'xr help block commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__blocked_commands] )) ||
_xr__subcmd__help__subcmd__blocked_commands() {
    local commands; commands=()
    _describe -t commands 'xr help blocked commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__bookmark_commands] )) ||
_xr__subcmd__help__subcmd__bookmark_commands() {
    local commands; commands=()
    _describe -t commands 'xr help bookmark commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__bookmark-folders_commands] )) ||
_xr__subcmd__help__subcmd__bookmark-folders_commands() {
    local commands; commands=(
'list:List the folders you sort bookmarks into' \
'posts:List the posts in one of your bookmark folders, by id' \
'add:Create a bookmark folder' \
    )
    _describe -t commands 'xr help bookmark-folders commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__bookmark-folders__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__bookmark-folders__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help bookmark-folders add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__bookmark-folders__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__bookmark-folders__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help bookmark-folders list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__bookmark-folders__subcmd__posts_commands] )) ||
_xr__subcmd__help__subcmd__bookmark-folders__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr help bookmark-folders posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__bookmarks_commands] )) ||
_xr__subcmd__help__subcmd__bookmarks_commands() {
    local commands; commands=()
    _describe -t commands 'xr help bookmarks commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts_commands() {
    local commands; commands=(
'list:List your broadcasts' \
'get:Read one broadcast, by id' \
'scheduled:Schedule broadcasts, change or cancel them, and start them early' \
'chat:Read and moderate a broadcast'\''s chat' \
'moderators:Manage who moderates your broadcast chats' \
'mutes:Mute people in every chat of your broadcasts' \
'sources:Manage the RTMP and RTMPS ingest points you stream to' \
    )
    _describe -t commands 'xr help broadcasts commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__chat_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__chat_commands() {
    local commands; commands=(
'history:List a broadcast'\''s chat' \
'send:Send a message into a broadcast'\''s chat' \
'remove:Remove a message from a broadcast'\''s chat' \
'mute:Mute someone in a broadcast'\''s chat' \
'unmute:Unmute someone in a broadcast'\''s chat' \
    )
    _describe -t commands 'xr help broadcasts chat commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__history_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts chat history commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__mute_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__mute_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts chat mute commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts chat remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__send_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__send_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts chat send commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__unmute_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__chat__subcmd__unmute_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts chat unmute commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__moderators_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__moderators_commands() {
    local commands; commands=(
'list:List your broadcast chat moderators' \
'add:Add a broadcast chat moderator' \
'remove:Remove a broadcast chat moderator' \
    )
    _describe -t commands 'xr help broadcasts moderators commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__moderators__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__moderators__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts moderators add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__moderators__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__moderators__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts moderators list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__moderators__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__moderators__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts moderators remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__mutes_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__mutes_commands() {
    local commands; commands=(
'list:List who is muted in your broadcast chats, and until when' \
'add:Mute someone in every chat of your broadcasts' \
'remove:Unmute someone in your broadcast chats' \
    )
    _describe -t commands 'xr help broadcasts mutes commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__mutes__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__mutes__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts mutes add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__mutes__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__mutes__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts mutes list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__mutes__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__mutes__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts mutes remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled_commands() {
    local commands; commands=(
'list:List your scheduled broadcasts' \
'get:Read one scheduled broadcast, by id' \
'add:Schedule a broadcast from one of your stream sources' \
'update:Move a scheduled broadcast, or change its details' \
'remove:Cancel a scheduled broadcast' \
'live:Start a scheduled broadcast now' \
    )
    _describe -t commands 'xr help broadcasts scheduled commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts scheduled add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts scheduled get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts scheduled list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__live_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__live_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts scheduled live commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts scheduled remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__update_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__scheduled__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts scheduled update commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__sources_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__sources_commands() {
    local commands; commands=(
'list:List your stream sources' \
'get:Show one stream source, and whether it is receiving video' \
'add:Create a stream source' \
'rename:Rename a stream source' \
'remove:Delete a stream source; an encoder sending to it is no longer received' \
    )
    _describe -t commands 'xr help broadcasts sources commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts sources add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts sources get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts sources list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts sources remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__rename_commands] )) ||
_xr__subcmd__help__subcmd__broadcasts__subcmd__sources__subcmd__rename_commands() {
    local commands; commands=()
    _describe -t commands 'xr help broadcasts sources rename commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__calls_commands] )) ||
_xr__subcmd__help__subcmd__calls_commands() {
    local commands; commands=(
'get:Show an X call\: its title, join policy, host, and who is in it' \
'scheduled:The calls on your calendar\: schedule, change, cancel, and answer them' \
    )
    _describe -t commands 'xr help calls commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__calls__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__calls__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help calls get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__calls__subcmd__scheduled_commands] )) ||
_xr__subcmd__help__subcmd__calls__subcmd__scheduled_commands() {
    local commands; commands=(
'list:List the calls on your calendar, earliest first' \
'get:Show a scheduled call, with its invitees and their answers' \
'add:Schedule a call on your X Calls calendar and invite people to it' \
'update:Move, rename, or re-invite a call you organize' \
'cancel:Cancel a call you organize and take it off invitees'\'' calendars' \
'rsvp:Answer an invitation to a scheduled call' \
    )
    _describe -t commands 'xr help calls scheduled commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help calls scheduled add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__cancel_commands] )) ||
_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__cancel_commands() {
    local commands; commands=()
    _describe -t commands 'xr help calls scheduled cancel commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help calls scheduled get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help calls scheduled list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__rsvp_commands] )) ||
_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__rsvp_commands() {
    local commands; commands=()
    _describe -t commands 'xr help calls scheduled rsvp commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__update_commands] )) ||
_xr__subcmd__help__subcmd__calls__subcmd__scheduled__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr help calls scheduled update commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__communities_commands] )) ||
_xr__subcmd__help__subcmd__communities_commands() {
    local commands; commands=(
'search:Search communities' \
'get:Read one community, by id' \
    )
    _describe -t commands 'xr help communities commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__communities__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__communities__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help communities get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__communities__subcmd__search_commands] )) ||
_xr__subcmd__help__subcmd__communities__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr help communities search commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__completions_commands] )) ||
_xr__subcmd__help__subcmd__completions_commands() {
    local commands; commands=()
    _describe -t commands 'xr help completions commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__conversations_commands] )) ||
_xr__subcmd__help__subcmd__conversations_commands() {
    local commands; commands=(
'start:Start a group conversation and send its first message' \
'send:Send a message into a conversation, by id' \
'history:List the messages in a conversation, by id' \
'with:List the messages between you and one person' \
    )
    _describe -t commands 'xr help conversations commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__conversations__subcmd__history_commands] )) ||
_xr__subcmd__help__subcmd__conversations__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'xr help conversations history commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__conversations__subcmd__send_commands] )) ||
_xr__subcmd__help__subcmd__conversations__subcmd__send_commands() {
    local commands; commands=()
    _describe -t commands 'xr help conversations send commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__conversations__subcmd__start_commands] )) ||
_xr__subcmd__help__subcmd__conversations__subcmd__start_commands() {
    local commands; commands=()
    _describe -t commands 'xr help conversations start commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__conversations__subcmd__with_commands] )) ||
_xr__subcmd__help__subcmd__conversations__subcmd__with_commands() {
    local commands; commands=()
    _describe -t commands 'xr help conversations with commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__delete_commands] )) ||
_xr__subcmd__help__subcmd__delete_commands() {
    local commands; commands=()
    _describe -t commands 'xr help delete commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__dm_commands] )) ||
_xr__subcmd__help__subcmd__dm_commands() {
    local commands; commands=()
    _describe -t commands 'xr help dm commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__dm-block_commands] )) ||
_xr__subcmd__help__subcmd__dm-block_commands() {
    local commands; commands=()
    _describe -t commands 'xr help dm-block commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__dm-events_commands] )) ||
_xr__subcmd__help__subcmd__dm-events_commands() {
    local commands; commands=(
'get:Read one direct message, by event id' \
'remove:Delete one of your direct messages, by event id' \
    )
    _describe -t commands 'xr help dm-events commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__dm-events__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__dm-events__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help dm-events get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__dm-events__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__dm-events__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help dm-events remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__dm-unblock_commands] )) ||
_xr__subcmd__help__subcmd__dm-unblock_commands() {
    local commands; commands=()
    _describe -t commands 'xr help dm-unblock commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__dms_commands] )) ||
_xr__subcmd__help__subcmd__dms_commands() {
    local commands; commands=()
    _describe -t commands 'xr help dms commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__examples_commands] )) ||
_xr__subcmd__help__subcmd__examples_commands() {
    local commands; commands=()
    _describe -t commands 'xr help examples commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__follow_commands] )) ||
_xr__subcmd__help__subcmd__follow_commands() {
    local commands; commands=()
    _describe -t commands 'xr help follow commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__followers_commands] )) ||
_xr__subcmd__help__subcmd__followers_commands() {
    local commands; commands=()
    _describe -t commands 'xr help followers commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__following_commands] )) ||
_xr__subcmd__help__subcmd__following_commands() {
    local commands; commands=()
    _describe -t commands 'xr help following commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__hide_commands] )) ||
_xr__subcmd__help__subcmd__hide_commands() {
    local commands; commands=()
    _describe -t commands 'xr help hide commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__like_commands] )) ||
_xr__subcmd__help__subcmd__like_commands() {
    local commands; commands=()
    _describe -t commands 'xr help like commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__likers_commands] )) ||
_xr__subcmd__help__subcmd__likers_commands() {
    local commands; commands=()
    _describe -t commands 'xr help likers commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__likes_commands] )) ||
_xr__subcmd__help__subcmd__likes_commands() {
    local commands; commands=()
    _describe -t commands 'xr help likes commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists_commands] )) ||
_xr__subcmd__help__subcmd__lists_commands() {
    local commands; commands=(
'list:List the lists you own' \
'followed:List the lists you follow' \
'memberships:List the lists you are a member of' \
'pinned:List the lists you have pinned' \
'get:Read one list, by id' \
'posts:List the posts from a list'\''s members' \
'followers:List who follows a list' \
'members:The accounts on a list, and adding or taking one off' \
'add:Create a list' \
'update:Rename a list, or change its description or privacy' \
'remove:Delete a list you own' \
'follow:Follow a list' \
'unfollow:Stop following a list' \
'pin:Pin a list' \
'unpin:Unpin a list' \
    )
    _describe -t commands 'xr help lists commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__follow_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__follow_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists follow commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__followed_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__followed_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists followed commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__followers_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__followers_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists followers commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__members_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__members_commands() {
    local commands; commands=(
'list:List the accounts on a list' \
'add:Add an account to a list you own' \
'remove:Take an account off a list you own' \
    )
    _describe -t commands 'xr help lists members commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__members__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__members__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists members add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__members__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__members__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists members list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__members__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__members__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists members remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__memberships_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__memberships_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists memberships commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__pin_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__pin_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists pin commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__pinned_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__pinned_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists pinned commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__posts_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__unfollow_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__unfollow_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists unfollow commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__unpin_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__unpin_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists unpin commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__lists__subcmd__update_commands] )) ||
_xr__subcmd__help__subcmd__lists__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr help lists update commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__media_commands] )) ||
_xr__subcmd__help__subcmd__media_commands() {
    local commands; commands=(
'upload:Upload media file' \
'status:Check media upload status' \
'get:Read 1 to 100 uploads by media key in one request' \
'analytics:Report video metrics for your uploads over a time window' \
'alt-text:Set the alt text shown for an uploaded image or video' \
'subtitles:Add or remove the subtitle tracks of an uploaded video' \
    )
    _describe -t commands 'xr help media commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__media__subcmd__alt-text_commands] )) ||
_xr__subcmd__help__subcmd__media__subcmd__alt-text_commands() {
    local commands; commands=()
    _describe -t commands 'xr help media alt-text commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__media__subcmd__analytics_commands] )) ||
_xr__subcmd__help__subcmd__media__subcmd__analytics_commands() {
    local commands; commands=()
    _describe -t commands 'xr help media analytics commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__media__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__media__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help media get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__media__subcmd__status_commands] )) ||
_xr__subcmd__help__subcmd__media__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr help media status commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__media__subcmd__subtitles_commands] )) ||
_xr__subcmd__help__subcmd__media__subcmd__subtitles_commands() {
    local commands; commands=(
'add:Add a subtitle track to an uploaded video' \
'remove:Remove a subtitle track from an uploaded video' \
    )
    _describe -t commands 'xr help media subtitles commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__media__subcmd__subtitles__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__media__subcmd__subtitles__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help media subtitles add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__media__subcmd__subtitles__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__media__subcmd__subtitles__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help media subtitles remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__media__subcmd__upload_commands] )) ||
_xr__subcmd__help__subcmd__media__subcmd__upload_commands() {
    local commands; commands=()
    _describe -t commands 'xr help media upload commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__mentions_commands] )) ||
_xr__subcmd__help__subcmd__mentions_commands() {
    local commands; commands=()
    _describe -t commands 'xr help mentions commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__mute_commands] )) ||
_xr__subcmd__help__subcmd__mute_commands() {
    local commands; commands=()
    _describe -t commands 'xr help mute commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__muted_commands] )) ||
_xr__subcmd__help__subcmd__muted_commands() {
    local commands; commands=()
    _describe -t commands 'xr help muted commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__news_commands] )) ||
_xr__subcmd__help__subcmd__news_commands() {
    local commands; commands=(
'search:Search news stories' \
'get:Read one news story, by id' \
    )
    _describe -t commands 'xr help news commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__news__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__news__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help news get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__news__subcmd__search_commands] )) ||
_xr__subcmd__help__subcmd__news__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr help news search commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__notes_commands] )) ||
_xr__subcmd__help__subcmd__notes_commands() {
    local commands; commands=(
'write:Write a Community Note about a post' \
'evaluate:Score a draft Community Note without writing it' \
'eligible:List posts you could write a Community Note on' \
'written:List the Community Notes you wrote' \
'remove:Delete a Community Note you wrote' \
    )
    _describe -t commands 'xr help notes commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__notes__subcmd__eligible_commands] )) ||
_xr__subcmd__help__subcmd__notes__subcmd__eligible_commands() {
    local commands; commands=()
    _describe -t commands 'xr help notes eligible commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__notes__subcmd__evaluate_commands] )) ||
_xr__subcmd__help__subcmd__notes__subcmd__evaluate_commands() {
    local commands; commands=()
    _describe -t commands 'xr help notes evaluate commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__notes__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__notes__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help notes remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__notes__subcmd__write_commands] )) ||
_xr__subcmd__help__subcmd__notes__subcmd__write_commands() {
    local commands; commands=()
    _describe -t commands 'xr help notes write commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__notes__subcmd__written_commands] )) ||
_xr__subcmd__help__subcmd__notes__subcmd__written_commands() {
    local commands; commands=()
    _describe -t commands 'xr help notes written commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__post_commands] )) ||
_xr__subcmd__help__subcmd__post_commands() {
    local commands; commands=()
    _describe -t commands 'xr help post commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__posts_commands] )) ||
_xr__subcmd__help__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr help posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__profile_commands] )) ||
_xr__subcmd__help__subcmd__profile_commands() {
    local commands; commands=(
'update:Change your display name, bio, website, or location' \
'image:Set your profile image' \
'banner:Set or remove your profile banner' \
    )
    _describe -t commands 'xr help profile commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__profile__subcmd__banner_commands] )) ||
_xr__subcmd__help__subcmd__profile__subcmd__banner_commands() {
    local commands; commands=(
'set:Set your profile banner from an uploaded image' \
'remove:Remove your profile banner' \
    )
    _describe -t commands 'xr help profile banner commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__profile__subcmd__banner__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__profile__subcmd__banner__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help profile banner remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__profile__subcmd__banner__subcmd__set_commands] )) ||
_xr__subcmd__help__subcmd__profile__subcmd__banner__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr help profile banner set commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__profile__subcmd__image_commands] )) ||
_xr__subcmd__help__subcmd__profile__subcmd__image_commands() {
    local commands; commands=(
'set:Set your profile image from an uploaded image' \
    )
    _describe -t commands 'xr help profile image commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__profile__subcmd__image__subcmd__set_commands] )) ||
_xr__subcmd__help__subcmd__profile__subcmd__image__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr help profile image set commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__profile__subcmd__update_commands] )) ||
_xr__subcmd__help__subcmd__profile__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr help profile update commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__quote_commands] )) ||
_xr__subcmd__help__subcmd__quote_commands() {
    local commands; commands=()
    _describe -t commands 'xr help quote commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__quotes_commands] )) ||
_xr__subcmd__help__subcmd__quotes_commands() {
    local commands; commands=()
    _describe -t commands 'xr help quotes commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__read_commands] )) ||
_xr__subcmd__help__subcmd__read_commands() {
    local commands; commands=()
    _describe -t commands 'xr help read commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__reply_commands] )) ||
_xr__subcmd__help__subcmd__reply_commands() {
    local commands; commands=()
    _describe -t commands 'xr help reply commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__repost_commands] )) ||
_xr__subcmd__help__subcmd__repost_commands() {
    local commands; commands=()
    _describe -t commands 'xr help repost commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__reposters_commands] )) ||
_xr__subcmd__help__subcmd__reposters_commands() {
    local commands; commands=()
    _describe -t commands 'xr help reposters commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__schema_commands] )) ||
_xr__subcmd__help__subcmd__schema_commands() {
    local commands; commands=()
    _describe -t commands 'xr help schema commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__search_commands] )) ||
_xr__subcmd__help__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr help search commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__search-counts_commands] )) ||
_xr__subcmd__help__subcmd__search-counts_commands() {
    local commands; commands=()
    _describe -t commands 'xr help search-counts commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__search-users_commands] )) ||
_xr__subcmd__help__subcmd__search-users_commands() {
    local commands; commands=()
    _describe -t commands 'xr help search-users commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__settings_commands] )) ||
_xr__subcmd__help__subcmd__settings_commands() {
    local commands; commands=(
'get:Show your account settings' \
'update:Change your account settings' \
    )
    _describe -t commands 'xr help settings commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__settings__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__settings__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help settings get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__settings__subcmd__update_commands] )) ||
_xr__subcmd__help__subcmd__settings__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr help settings update commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__skill_commands] )) ||
_xr__subcmd__help__subcmd__skill_commands() {
    local commands; commands=(
'install:Install the skill bundle into a host'\''s canonical skills directory' \
'update:Refresh an existing skill-bundle install in place' \
    )
    _describe -t commands 'xr help skill commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__skill__subcmd__install_commands] )) ||
_xr__subcmd__help__subcmd__skill__subcmd__install_commands() {
    local commands; commands=()
    _describe -t commands 'xr help skill install commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__skill__subcmd__update_commands] )) ||
_xr__subcmd__help__subcmd__skill__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr help skill update commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__spaces_commands] )) ||
_xr__subcmd__help__subcmd__spaces_commands() {
    local commands; commands=(
'search:Search Spaces by keyword' \
'get:Read 1 to 100 Spaces by id in one request' \
'by-creator:List the Spaces 1 to 100 accounts created, in one request' \
'posts:List the posts shared in a Space' \
    )
    _describe -t commands 'xr help spaces commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__spaces__subcmd__by-creator_commands] )) ||
_xr__subcmd__help__subcmd__spaces__subcmd__by-creator_commands() {
    local commands; commands=()
    _describe -t commands 'xr help spaces by-creator commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__spaces__subcmd__get_commands] )) ||
_xr__subcmd__help__subcmd__spaces__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help spaces get commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__spaces__subcmd__posts_commands] )) ||
_xr__subcmd__help__subcmd__spaces__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr help spaces posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__spaces__subcmd__search_commands] )) ||
_xr__subcmd__help__subcmd__spaces__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr help spaces search commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__stream_commands] )) ||
_xr__subcmd__help__subcmd__stream_commands() {
    local commands; commands=(
'rules:List, add, remove, and count the rules that pick the stream'\''s posts' \
    )
    _describe -t commands 'xr help stream commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__stream__subcmd__rules_commands] )) ||
_xr__subcmd__help__subcmd__stream__subcmd__rules_commands() {
    local commands; commands=(
'list:List the filtered stream'\''s rules' \
'add:Add a rule to the filtered stream' \
'remove:Remove rules from the filtered stream, by id' \
'count:Count the rules your app and its project hold, against X'\''s caps' \
    )
    _describe -t commands 'xr help stream rules commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__stream__subcmd__rules__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__stream__subcmd__rules__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help stream rules add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__stream__subcmd__rules__subcmd__count_commands] )) ||
_xr__subcmd__help__subcmd__stream__subcmd__rules__subcmd__count_commands() {
    local commands; commands=()
    _describe -t commands 'xr help stream rules count commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__stream__subcmd__rules__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__stream__subcmd__rules__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help stream rules list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__stream__subcmd__rules__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__stream__subcmd__rules__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help stream rules remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__timeline_commands] )) ||
_xr__subcmd__help__subcmd__timeline_commands() {
    local commands; commands=()
    _describe -t commands 'xr help timeline commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__trends_commands] )) ||
_xr__subcmd__help__subcmd__trends_commands() {
    local commands; commands=(
'place:List the trends for a place, by its Yahoo WOEID (1 is worldwide)' \
'personal:List the trends X picks for you' \
    )
    _describe -t commands 'xr help trends commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__trends__subcmd__personal_commands] )) ||
_xr__subcmd__help__subcmd__trends__subcmd__personal_commands() {
    local commands; commands=()
    _describe -t commands 'xr help trends personal commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__trends__subcmd__place_commands] )) ||
_xr__subcmd__help__subcmd__trends__subcmd__place_commands() {
    local commands; commands=()
    _describe -t commands 'xr help trends place commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__unblock_commands] )) ||
_xr__subcmd__help__subcmd__unblock_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unblock commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__unbookmark_commands] )) ||
_xr__subcmd__help__subcmd__unbookmark_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unbookmark commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__unfollow_commands] )) ||
_xr__subcmd__help__subcmd__unfollow_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unfollow commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__unhide_commands] )) ||
_xr__subcmd__help__subcmd__unhide_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unhide commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__unlike_commands] )) ||
_xr__subcmd__help__subcmd__unlike_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unlike commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__unmute_commands] )) ||
_xr__subcmd__help__subcmd__unmute_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unmute commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__unrepost_commands] )) ||
_xr__subcmd__help__subcmd__unrepost_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unrepost commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__usage_commands] )) ||
_xr__subcmd__help__subcmd__usage_commands() {
    local commands; commands=(
'credits:Show credits-based usage for the project' \
    )
    _describe -t commands 'xr help usage commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__usage__subcmd__credits_commands] )) ||
_xr__subcmd__help__subcmd__usage__subcmd__credits_commands() {
    local commands; commands=()
    _describe -t commands 'xr help usage credits commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__user_commands] )) ||
_xr__subcmd__help__subcmd__user_commands() {
    local commands; commands=()
    _describe -t commands 'xr help user commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__user-posts_commands] )) ||
_xr__subcmd__help__subcmd__user-posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr help user-posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__users_commands] )) ||
_xr__subcmd__help__subcmd__users_commands() {
    local commands; commands=()
    _describe -t commands 'xr help users commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__validate_commands] )) ||
_xr__subcmd__help__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'xr help validate commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__version_commands] )) ||
_xr__subcmd__help__subcmd__version_commands() {
    local commands; commands=()
    _describe -t commands 'xr help version commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks_commands] )) ||
_xr__subcmd__help__subcmd__webhooks_commands() {
    local commands; commands=(
'list:List the webhooks registered for the app' \
'add:Register a webhook URL' \
'validate:Ask X to send its CRC check to a webhook again' \
'remove:Delete a webhook; its subscriptions stop delivering' \
'subscriptions:Account Activity subscriptions\: whose activity a webhook receives' \
'stream-links:Filtered-stream links\: deliver the posts your stream rules match to a webhook' \
'listen:Receive webhook deliveries on a local address' \
'replay:Deliver a past window of events to a webhook again' \
    )
    _describe -t commands 'xr help webhooks commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__listen_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__listen_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks listen commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__replay_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__replay_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks replay commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__stream-links_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__stream-links_commands() {
    local commands; commands=(
'list:List the webhooks the app'\''s filtered stream delivers to' \
'add:Deliver the app'\''s filtered stream to a webhook' \
'remove:Stop delivering the app'\''s filtered stream to a webhook' \
    )
    _describe -t commands 'xr help webhooks stream-links commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__stream-links__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__stream-links__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks stream-links add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__stream-links__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__stream-links__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks stream-links list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__stream-links__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__stream-links__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks stream-links remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions_commands() {
    local commands; commands=(
'count:Count the subscriptions the app holds and may hold' \
'list:List the accounts whose activity a webhook receives' \
'add:Subscribe your account'\''s activity to a webhook' \
'check:Say whether your account'\''s activity is subscribed to a webhook' \
'remove:End an account'\''s subscription to a webhook' \
    )
    _describe -t commands 'xr help webhooks subscriptions commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__add_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks subscriptions add commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__check_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__check_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks subscriptions check commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__count_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__count_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks subscriptions count commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__list_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks subscriptions list commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__remove_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__subscriptions__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks subscriptions remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__webhooks__subcmd__validate_commands] )) ||
_xr__subcmd__help__subcmd__webhooks__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'xr help webhooks validate commands' commands "$@"
}
(( $+functions[_xr__subcmd__help__subcmd__whoami_commands] )) ||
_xr__subcmd__help__subcmd__whoami_commands() {
    local commands; commands=()
    _describe -t commands 'xr help whoami commands' commands "$@"
}
(( $+functions[_xr__subcmd__hide_commands] )) ||
_xr__subcmd__hide_commands() {
    local commands; commands=()
    _describe -t commands 'xr hide commands' commands "$@"
}
(( $+functions[_xr__subcmd__like_commands] )) ||
_xr__subcmd__like_commands() {
    local commands; commands=()
    _describe -t commands 'xr like commands' commands "$@"
}
(( $+functions[_xr__subcmd__likers_commands] )) ||
_xr__subcmd__likers_commands() {
    local commands; commands=()
    _describe -t commands 'xr likers commands' commands "$@"
}
(( $+functions[_xr__subcmd__likes_commands] )) ||
_xr__subcmd__likes_commands() {
    local commands; commands=()
    _describe -t commands 'xr likes commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists_commands] )) ||
_xr__subcmd__lists_commands() {
    local commands; commands=(
'list:List the lists you own' \
'followed:List the lists you follow' \
'memberships:List the lists you are a member of' \
'pinned:List the lists you have pinned' \
'get:Read one list, by id' \
'posts:List the posts from a list'\''s members' \
'followers:List who follows a list' \
'members:The accounts on a list, and adding or taking one off' \
'add:Create a list' \
'update:Rename a list, or change its description or privacy' \
'remove:Delete a list you own' \
'follow:Follow a list' \
'unfollow:Stop following a list' \
'pin:Pin a list' \
'unpin:Unpin a list' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr lists commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__add_commands] )) ||
_xr__subcmd__lists__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists add commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__follow_commands] )) ||
_xr__subcmd__lists__subcmd__follow_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists follow commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__followed_commands] )) ||
_xr__subcmd__lists__subcmd__followed_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists followed commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__followers_commands] )) ||
_xr__subcmd__lists__subcmd__followers_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists followers commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__get_commands] )) ||
_xr__subcmd__lists__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists get commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help_commands] )) ||
_xr__subcmd__lists__subcmd__help_commands() {
    local commands; commands=(
'list:List the lists you own' \
'followed:List the lists you follow' \
'memberships:List the lists you are a member of' \
'pinned:List the lists you have pinned' \
'get:Read one list, by id' \
'posts:List the posts from a list'\''s members' \
'followers:List who follows a list' \
'members:The accounts on a list, and adding or taking one off' \
'add:Create a list' \
'update:Rename a list, or change its description or privacy' \
'remove:Delete a list you own' \
'follow:Follow a list' \
'unfollow:Stop following a list' \
'pin:Pin a list' \
'unpin:Unpin a list' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr lists help commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__follow_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__follow_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help follow commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__followed_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__followed_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help followed commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__followers_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__followers_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help followers commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__members_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__members_commands() {
    local commands; commands=(
'list:List the accounts on a list' \
'add:Add an account to a list you own' \
'remove:Take an account off a list you own' \
    )
    _describe -t commands 'xr lists help members commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__members__subcmd__add_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__members__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help members add commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__members__subcmd__list_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__members__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help members list commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__members__subcmd__remove_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__members__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help members remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__memberships_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__memberships_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help memberships commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__pin_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__pin_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help pin commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__pinned_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__pinned_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help pinned commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__posts_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__unfollow_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__unfollow_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help unfollow commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__unpin_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__unpin_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help unpin commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__help__subcmd__update_commands] )) ||
_xr__subcmd__lists__subcmd__help__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists help update commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__list_commands] )) ||
_xr__subcmd__lists__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists list commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__members_commands] )) ||
_xr__subcmd__lists__subcmd__members_commands() {
    local commands; commands=(
'list:List the accounts on a list' \
'add:Add an account to a list you own' \
'remove:Take an account off a list you own' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr lists members commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__members__subcmd__add_commands] )) ||
_xr__subcmd__lists__subcmd__members__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists members add commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__members__subcmd__help_commands] )) ||
_xr__subcmd__lists__subcmd__members__subcmd__help_commands() {
    local commands; commands=(
'list:List the accounts on a list' \
'add:Add an account to a list you own' \
'remove:Take an account off a list you own' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr lists members help commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__members__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__lists__subcmd__members__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists members help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__members__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__lists__subcmd__members__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists members help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__members__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__lists__subcmd__members__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists members help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__members__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__lists__subcmd__members__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists members help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__members__subcmd__list_commands] )) ||
_xr__subcmd__lists__subcmd__members__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists members list commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__members__subcmd__remove_commands] )) ||
_xr__subcmd__lists__subcmd__members__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists members remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__memberships_commands] )) ||
_xr__subcmd__lists__subcmd__memberships_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists memberships commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__pin_commands] )) ||
_xr__subcmd__lists__subcmd__pin_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists pin commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__pinned_commands] )) ||
_xr__subcmd__lists__subcmd__pinned_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists pinned commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__posts_commands] )) ||
_xr__subcmd__lists__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__remove_commands] )) ||
_xr__subcmd__lists__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__unfollow_commands] )) ||
_xr__subcmd__lists__subcmd__unfollow_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists unfollow commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__unpin_commands] )) ||
_xr__subcmd__lists__subcmd__unpin_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists unpin commands' commands "$@"
}
(( $+functions[_xr__subcmd__lists__subcmd__update_commands] )) ||
_xr__subcmd__lists__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr lists update commands' commands "$@"
}
(( $+functions[_xr__subcmd__media_commands] )) ||
_xr__subcmd__media_commands() {
    local commands; commands=(
'upload:Upload media file' \
'status:Check media upload status' \
'get:Read 1 to 100 uploads by media key in one request' \
'analytics:Report video metrics for your uploads over a time window' \
'alt-text:Set the alt text shown for an uploaded image or video' \
'subtitles:Add or remove the subtitle tracks of an uploaded video' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr media commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__alt-text_commands] )) ||
_xr__subcmd__media__subcmd__alt-text_commands() {
    local commands; commands=()
    _describe -t commands 'xr media alt-text commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__analytics_commands] )) ||
_xr__subcmd__media__subcmd__analytics_commands() {
    local commands; commands=()
    _describe -t commands 'xr media analytics commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__get_commands] )) ||
_xr__subcmd__media__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr media get commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help_commands] )) ||
_xr__subcmd__media__subcmd__help_commands() {
    local commands; commands=(
'upload:Upload media file' \
'status:Check media upload status' \
'get:Read 1 to 100 uploads by media key in one request' \
'analytics:Report video metrics for your uploads over a time window' \
'alt-text:Set the alt text shown for an uploaded image or video' \
'subtitles:Add or remove the subtitle tracks of an uploaded video' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr media help commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help__subcmd__alt-text_commands] )) ||
_xr__subcmd__media__subcmd__help__subcmd__alt-text_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help alt-text commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help__subcmd__analytics_commands] )) ||
_xr__subcmd__media__subcmd__help__subcmd__analytics_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help analytics commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__media__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__media__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help__subcmd__status_commands] )) ||
_xr__subcmd__media__subcmd__help__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help status commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help__subcmd__subtitles_commands] )) ||
_xr__subcmd__media__subcmd__help__subcmd__subtitles_commands() {
    local commands; commands=(
'add:Add a subtitle track to an uploaded video' \
'remove:Remove a subtitle track from an uploaded video' \
    )
    _describe -t commands 'xr media help subtitles commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help__subcmd__subtitles__subcmd__add_commands] )) ||
_xr__subcmd__media__subcmd__help__subcmd__subtitles__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help subtitles add commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help__subcmd__subtitles__subcmd__remove_commands] )) ||
_xr__subcmd__media__subcmd__help__subcmd__subtitles__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help subtitles remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__help__subcmd__upload_commands] )) ||
_xr__subcmd__media__subcmd__help__subcmd__upload_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help upload commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__status_commands] )) ||
_xr__subcmd__media__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr media status commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__subtitles_commands] )) ||
_xr__subcmd__media__subcmd__subtitles_commands() {
    local commands; commands=(
'add:Add a subtitle track to an uploaded video' \
'remove:Remove a subtitle track from an uploaded video' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr media subtitles commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__subtitles__subcmd__add_commands] )) ||
_xr__subcmd__media__subcmd__subtitles__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr media subtitles add commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__subtitles__subcmd__help_commands] )) ||
_xr__subcmd__media__subcmd__subtitles__subcmd__help_commands() {
    local commands; commands=(
'add:Add a subtitle track to an uploaded video' \
'remove:Remove a subtitle track from an uploaded video' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr media subtitles help commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__subtitles__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__media__subcmd__subtitles__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr media subtitles help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__subtitles__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__media__subcmd__subtitles__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr media subtitles help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__subtitles__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__media__subcmd__subtitles__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr media subtitles help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__subtitles__subcmd__remove_commands] )) ||
_xr__subcmd__media__subcmd__subtitles__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr media subtitles remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__media__subcmd__upload_commands] )) ||
_xr__subcmd__media__subcmd__upload_commands() {
    local commands; commands=()
    _describe -t commands 'xr media upload commands' commands "$@"
}
(( $+functions[_xr__subcmd__mentions_commands] )) ||
_xr__subcmd__mentions_commands() {
    local commands; commands=()
    _describe -t commands 'xr mentions commands' commands "$@"
}
(( $+functions[_xr__subcmd__mute_commands] )) ||
_xr__subcmd__mute_commands() {
    local commands; commands=()
    _describe -t commands 'xr mute commands' commands "$@"
}
(( $+functions[_xr__subcmd__muted_commands] )) ||
_xr__subcmd__muted_commands() {
    local commands; commands=()
    _describe -t commands 'xr muted commands' commands "$@"
}
(( $+functions[_xr__subcmd__news_commands] )) ||
_xr__subcmd__news_commands() {
    local commands; commands=(
'search:Search news stories' \
'get:Read one news story, by id' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr news commands' commands "$@"
}
(( $+functions[_xr__subcmd__news__subcmd__get_commands] )) ||
_xr__subcmd__news__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr news get commands' commands "$@"
}
(( $+functions[_xr__subcmd__news__subcmd__help_commands] )) ||
_xr__subcmd__news__subcmd__help_commands() {
    local commands; commands=(
'search:Search news stories' \
'get:Read one news story, by id' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr news help commands' commands "$@"
}
(( $+functions[_xr__subcmd__news__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__news__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr news help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__news__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__news__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr news help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__news__subcmd__help__subcmd__search_commands] )) ||
_xr__subcmd__news__subcmd__help__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr news help search commands' commands "$@"
}
(( $+functions[_xr__subcmd__news__subcmd__search_commands] )) ||
_xr__subcmd__news__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr news search commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes_commands] )) ||
_xr__subcmd__notes_commands() {
    local commands; commands=(
'write:Write a Community Note about a post' \
'evaluate:Score a draft Community Note without writing it' \
'eligible:List posts you could write a Community Note on' \
'written:List the Community Notes you wrote' \
'remove:Delete a Community Note you wrote' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr notes commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__eligible_commands] )) ||
_xr__subcmd__notes__subcmd__eligible_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes eligible commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__evaluate_commands] )) ||
_xr__subcmd__notes__subcmd__evaluate_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes evaluate commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__help_commands] )) ||
_xr__subcmd__notes__subcmd__help_commands() {
    local commands; commands=(
'write:Write a Community Note about a post' \
'evaluate:Score a draft Community Note without writing it' \
'eligible:List posts you could write a Community Note on' \
'written:List the Community Notes you wrote' \
'remove:Delete a Community Note you wrote' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr notes help commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__help__subcmd__eligible_commands] )) ||
_xr__subcmd__notes__subcmd__help__subcmd__eligible_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes help eligible commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__help__subcmd__evaluate_commands] )) ||
_xr__subcmd__notes__subcmd__help__subcmd__evaluate_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes help evaluate commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__notes__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__notes__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__help__subcmd__write_commands] )) ||
_xr__subcmd__notes__subcmd__help__subcmd__write_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes help write commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__help__subcmd__written_commands] )) ||
_xr__subcmd__notes__subcmd__help__subcmd__written_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes help written commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__remove_commands] )) ||
_xr__subcmd__notes__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__write_commands] )) ||
_xr__subcmd__notes__subcmd__write_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes write commands' commands "$@"
}
(( $+functions[_xr__subcmd__notes__subcmd__written_commands] )) ||
_xr__subcmd__notes__subcmd__written_commands() {
    local commands; commands=()
    _describe -t commands 'xr notes written commands' commands "$@"
}
(( $+functions[_xr__subcmd__post_commands] )) ||
_xr__subcmd__post_commands() {
    local commands; commands=()
    _describe -t commands 'xr post commands' commands "$@"
}
(( $+functions[_xr__subcmd__posts_commands] )) ||
_xr__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile_commands] )) ||
_xr__subcmd__profile_commands() {
    local commands; commands=(
'update:Change your display name, bio, website, or location' \
'image:Set your profile image' \
'banner:Set or remove your profile banner' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr profile commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__banner_commands] )) ||
_xr__subcmd__profile__subcmd__banner_commands() {
    local commands; commands=(
'set:Set your profile banner from an uploaded image' \
'remove:Remove your profile banner' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr profile banner commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__banner__subcmd__help_commands] )) ||
_xr__subcmd__profile__subcmd__banner__subcmd__help_commands() {
    local commands; commands=(
'set:Set your profile banner from an uploaded image' \
'remove:Remove your profile banner' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr profile banner help commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__banner__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__profile__subcmd__banner__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile banner help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__banner__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__profile__subcmd__banner__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile banner help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__banner__subcmd__help__subcmd__set_commands] )) ||
_xr__subcmd__profile__subcmd__banner__subcmd__help__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile banner help set commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__banner__subcmd__remove_commands] )) ||
_xr__subcmd__profile__subcmd__banner__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile banner remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__banner__subcmd__set_commands] )) ||
_xr__subcmd__profile__subcmd__banner__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile banner set commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__help_commands] )) ||
_xr__subcmd__profile__subcmd__help_commands() {
    local commands; commands=(
'update:Change your display name, bio, website, or location' \
'image:Set your profile image' \
'banner:Set or remove your profile banner' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr profile help commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__help__subcmd__banner_commands] )) ||
_xr__subcmd__profile__subcmd__help__subcmd__banner_commands() {
    local commands; commands=(
'set:Set your profile banner from an uploaded image' \
'remove:Remove your profile banner' \
    )
    _describe -t commands 'xr profile help banner commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__help__subcmd__banner__subcmd__remove_commands] )) ||
_xr__subcmd__profile__subcmd__help__subcmd__banner__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile help banner remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__help__subcmd__banner__subcmd__set_commands] )) ||
_xr__subcmd__profile__subcmd__help__subcmd__banner__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile help banner set commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__profile__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__help__subcmd__image_commands] )) ||
_xr__subcmd__profile__subcmd__help__subcmd__image_commands() {
    local commands; commands=(
'set:Set your profile image from an uploaded image' \
    )
    _describe -t commands 'xr profile help image commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__help__subcmd__image__subcmd__set_commands] )) ||
_xr__subcmd__profile__subcmd__help__subcmd__image__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile help image set commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__help__subcmd__update_commands] )) ||
_xr__subcmd__profile__subcmd__help__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile help update commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__image_commands] )) ||
_xr__subcmd__profile__subcmd__image_commands() {
    local commands; commands=(
'set:Set your profile image from an uploaded image' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr profile image commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__image__subcmd__help_commands] )) ||
_xr__subcmd__profile__subcmd__image__subcmd__help_commands() {
    local commands; commands=(
'set:Set your profile image from an uploaded image' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr profile image help commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__image__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__profile__subcmd__image__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile image help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__image__subcmd__help__subcmd__set_commands] )) ||
_xr__subcmd__profile__subcmd__image__subcmd__help__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile image help set commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__image__subcmd__set_commands] )) ||
_xr__subcmd__profile__subcmd__image__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile image set commands' commands "$@"
}
(( $+functions[_xr__subcmd__profile__subcmd__update_commands] )) ||
_xr__subcmd__profile__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr profile update commands' commands "$@"
}
(( $+functions[_xr__subcmd__quote_commands] )) ||
_xr__subcmd__quote_commands() {
    local commands; commands=()
    _describe -t commands 'xr quote commands' commands "$@"
}
(( $+functions[_xr__subcmd__quotes_commands] )) ||
_xr__subcmd__quotes_commands() {
    local commands; commands=()
    _describe -t commands 'xr quotes commands' commands "$@"
}
(( $+functions[_xr__subcmd__read_commands] )) ||
_xr__subcmd__read_commands() {
    local commands; commands=()
    _describe -t commands 'xr read commands' commands "$@"
}
(( $+functions[_xr__subcmd__reply_commands] )) ||
_xr__subcmd__reply_commands() {
    local commands; commands=()
    _describe -t commands 'xr reply commands' commands "$@"
}
(( $+functions[_xr__subcmd__repost_commands] )) ||
_xr__subcmd__repost_commands() {
    local commands; commands=()
    _describe -t commands 'xr repost commands' commands "$@"
}
(( $+functions[_xr__subcmd__reposters_commands] )) ||
_xr__subcmd__reposters_commands() {
    local commands; commands=()
    _describe -t commands 'xr reposters commands' commands "$@"
}
(( $+functions[_xr__subcmd__schema_commands] )) ||
_xr__subcmd__schema_commands() {
    local commands; commands=()
    _describe -t commands 'xr schema commands' commands "$@"
}
(( $+functions[_xr__subcmd__search_commands] )) ||
_xr__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr search commands' commands "$@"
}
(( $+functions[_xr__subcmd__search-counts_commands] )) ||
_xr__subcmd__search-counts_commands() {
    local commands; commands=()
    _describe -t commands 'xr search-counts commands' commands "$@"
}
(( $+functions[_xr__subcmd__search-users_commands] )) ||
_xr__subcmd__search-users_commands() {
    local commands; commands=()
    _describe -t commands 'xr search-users commands' commands "$@"
}
(( $+functions[_xr__subcmd__settings_commands] )) ||
_xr__subcmd__settings_commands() {
    local commands; commands=(
'get:Show your account settings' \
'update:Change your account settings' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr settings commands' commands "$@"
}
(( $+functions[_xr__subcmd__settings__subcmd__get_commands] )) ||
_xr__subcmd__settings__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr settings get commands' commands "$@"
}
(( $+functions[_xr__subcmd__settings__subcmd__help_commands] )) ||
_xr__subcmd__settings__subcmd__help_commands() {
    local commands; commands=(
'get:Show your account settings' \
'update:Change your account settings' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr settings help commands' commands "$@"
}
(( $+functions[_xr__subcmd__settings__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__settings__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr settings help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__settings__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__settings__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr settings help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__settings__subcmd__help__subcmd__update_commands] )) ||
_xr__subcmd__settings__subcmd__help__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr settings help update commands' commands "$@"
}
(( $+functions[_xr__subcmd__settings__subcmd__update_commands] )) ||
_xr__subcmd__settings__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr settings update commands' commands "$@"
}
(( $+functions[_xr__subcmd__skill_commands] )) ||
_xr__subcmd__skill_commands() {
    local commands; commands=(
'install:Install the skill bundle into a host'\''s canonical skills directory' \
'update:Refresh an existing skill-bundle install in place' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr skill commands' commands "$@"
}
(( $+functions[_xr__subcmd__skill__subcmd__help_commands] )) ||
_xr__subcmd__skill__subcmd__help_commands() {
    local commands; commands=(
'install:Install the skill bundle into a host'\''s canonical skills directory' \
'update:Refresh an existing skill-bundle install in place' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr skill help commands' commands "$@"
}
(( $+functions[_xr__subcmd__skill__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__skill__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__skill__subcmd__help__subcmd__install_commands] )) ||
_xr__subcmd__skill__subcmd__help__subcmd__install_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill help install commands' commands "$@"
}
(( $+functions[_xr__subcmd__skill__subcmd__help__subcmd__update_commands] )) ||
_xr__subcmd__skill__subcmd__help__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill help update commands' commands "$@"
}
(( $+functions[_xr__subcmd__skill__subcmd__install_commands] )) ||
_xr__subcmd__skill__subcmd__install_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill install commands' commands "$@"
}
(( $+functions[_xr__subcmd__skill__subcmd__update_commands] )) ||
_xr__subcmd__skill__subcmd__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill update commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces_commands] )) ||
_xr__subcmd__spaces_commands() {
    local commands; commands=(
'search:Search Spaces by keyword' \
'get:Read 1 to 100 Spaces by id in one request' \
'by-creator:List the Spaces 1 to 100 accounts created, in one request' \
'posts:List the posts shared in a Space' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr spaces commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__by-creator_commands] )) ||
_xr__subcmd__spaces__subcmd__by-creator_commands() {
    local commands; commands=()
    _describe -t commands 'xr spaces by-creator commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__get_commands] )) ||
_xr__subcmd__spaces__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr spaces get commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__help_commands] )) ||
_xr__subcmd__spaces__subcmd__help_commands() {
    local commands; commands=(
'search:Search Spaces by keyword' \
'get:Read 1 to 100 Spaces by id in one request' \
'by-creator:List the Spaces 1 to 100 accounts created, in one request' \
'posts:List the posts shared in a Space' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr spaces help commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__help__subcmd__by-creator_commands] )) ||
_xr__subcmd__spaces__subcmd__help__subcmd__by-creator_commands() {
    local commands; commands=()
    _describe -t commands 'xr spaces help by-creator commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__help__subcmd__get_commands] )) ||
_xr__subcmd__spaces__subcmd__help__subcmd__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr spaces help get commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__spaces__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr spaces help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__help__subcmd__posts_commands] )) ||
_xr__subcmd__spaces__subcmd__help__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr spaces help posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__help__subcmd__search_commands] )) ||
_xr__subcmd__spaces__subcmd__help__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr spaces help search commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__posts_commands] )) ||
_xr__subcmd__spaces__subcmd__posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr spaces posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__spaces__subcmd__search_commands] )) ||
_xr__subcmd__spaces__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr spaces search commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream_commands] )) ||
_xr__subcmd__stream_commands() {
    local commands; commands=(
'rules:List, add, remove, and count the rules that pick the stream'\''s posts' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr stream commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__help_commands] )) ||
_xr__subcmd__stream__subcmd__help_commands() {
    local commands; commands=(
'rules:List, add, remove, and count the rules that pick the stream'\''s posts' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr stream help commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__stream__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__help__subcmd__rules_commands] )) ||
_xr__subcmd__stream__subcmd__help__subcmd__rules_commands() {
    local commands; commands=(
'list:List the filtered stream'\''s rules' \
'add:Add a rule to the filtered stream' \
'remove:Remove rules from the filtered stream, by id' \
'count:Count the rules your app and its project hold, against X'\''s caps' \
    )
    _describe -t commands 'xr stream help rules commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__help__subcmd__rules__subcmd__add_commands] )) ||
_xr__subcmd__stream__subcmd__help__subcmd__rules__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream help rules add commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__help__subcmd__rules__subcmd__count_commands] )) ||
_xr__subcmd__stream__subcmd__help__subcmd__rules__subcmd__count_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream help rules count commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__help__subcmd__rules__subcmd__list_commands] )) ||
_xr__subcmd__stream__subcmd__help__subcmd__rules__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream help rules list commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__help__subcmd__rules__subcmd__remove_commands] )) ||
_xr__subcmd__stream__subcmd__help__subcmd__rules__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream help rules remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules_commands] )) ||
_xr__subcmd__stream__subcmd__rules_commands() {
    local commands; commands=(
'list:List the filtered stream'\''s rules' \
'add:Add a rule to the filtered stream' \
'remove:Remove rules from the filtered stream, by id' \
'count:Count the rules your app and its project hold, against X'\''s caps' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr stream rules commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__add_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream rules add commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__count_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__count_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream rules count commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__help_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__help_commands() {
    local commands; commands=(
'list:List the filtered stream'\''s rules' \
'add:Add a rule to the filtered stream' \
'remove:Remove rules from the filtered stream, by id' \
'count:Count the rules your app and its project hold, against X'\''s caps' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr stream rules help commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream rules help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__count_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__count_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream rules help count commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream rules help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream rules help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream rules help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__list_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream rules list commands' commands "$@"
}
(( $+functions[_xr__subcmd__stream__subcmd__rules__subcmd__remove_commands] )) ||
_xr__subcmd__stream__subcmd__rules__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr stream rules remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__timeline_commands] )) ||
_xr__subcmd__timeline_commands() {
    local commands; commands=()
    _describe -t commands 'xr timeline commands' commands "$@"
}
(( $+functions[_xr__subcmd__trends_commands] )) ||
_xr__subcmd__trends_commands() {
    local commands; commands=(
'place:List the trends for a place, by its Yahoo WOEID (1 is worldwide)' \
'personal:List the trends X picks for you' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr trends commands' commands "$@"
}
(( $+functions[_xr__subcmd__trends__subcmd__help_commands] )) ||
_xr__subcmd__trends__subcmd__help_commands() {
    local commands; commands=(
'place:List the trends for a place, by its Yahoo WOEID (1 is worldwide)' \
'personal:List the trends X picks for you' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr trends help commands' commands "$@"
}
(( $+functions[_xr__subcmd__trends__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__trends__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr trends help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__trends__subcmd__help__subcmd__personal_commands] )) ||
_xr__subcmd__trends__subcmd__help__subcmd__personal_commands() {
    local commands; commands=()
    _describe -t commands 'xr trends help personal commands' commands "$@"
}
(( $+functions[_xr__subcmd__trends__subcmd__help__subcmd__place_commands] )) ||
_xr__subcmd__trends__subcmd__help__subcmd__place_commands() {
    local commands; commands=()
    _describe -t commands 'xr trends help place commands' commands "$@"
}
(( $+functions[_xr__subcmd__trends__subcmd__personal_commands] )) ||
_xr__subcmd__trends__subcmd__personal_commands() {
    local commands; commands=()
    _describe -t commands 'xr trends personal commands' commands "$@"
}
(( $+functions[_xr__subcmd__trends__subcmd__place_commands] )) ||
_xr__subcmd__trends__subcmd__place_commands() {
    local commands; commands=()
    _describe -t commands 'xr trends place commands' commands "$@"
}
(( $+functions[_xr__subcmd__unblock_commands] )) ||
_xr__subcmd__unblock_commands() {
    local commands; commands=()
    _describe -t commands 'xr unblock commands' commands "$@"
}
(( $+functions[_xr__subcmd__unbookmark_commands] )) ||
_xr__subcmd__unbookmark_commands() {
    local commands; commands=()
    _describe -t commands 'xr unbookmark commands' commands "$@"
}
(( $+functions[_xr__subcmd__unfollow_commands] )) ||
_xr__subcmd__unfollow_commands() {
    local commands; commands=()
    _describe -t commands 'xr unfollow commands' commands "$@"
}
(( $+functions[_xr__subcmd__unhide_commands] )) ||
_xr__subcmd__unhide_commands() {
    local commands; commands=()
    _describe -t commands 'xr unhide commands' commands "$@"
}
(( $+functions[_xr__subcmd__unlike_commands] )) ||
_xr__subcmd__unlike_commands() {
    local commands; commands=()
    _describe -t commands 'xr unlike commands' commands "$@"
}
(( $+functions[_xr__subcmd__unmute_commands] )) ||
_xr__subcmd__unmute_commands() {
    local commands; commands=()
    _describe -t commands 'xr unmute commands' commands "$@"
}
(( $+functions[_xr__subcmd__unrepost_commands] )) ||
_xr__subcmd__unrepost_commands() {
    local commands; commands=()
    _describe -t commands 'xr unrepost commands' commands "$@"
}
(( $+functions[_xr__subcmd__usage_commands] )) ||
_xr__subcmd__usage_commands() {
    local commands; commands=(
'credits:Show credits-based usage for the project' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr usage commands' commands "$@"
}
(( $+functions[_xr__subcmd__usage__subcmd__credits_commands] )) ||
_xr__subcmd__usage__subcmd__credits_commands() {
    local commands; commands=()
    _describe -t commands 'xr usage credits commands' commands "$@"
}
(( $+functions[_xr__subcmd__usage__subcmd__help_commands] )) ||
_xr__subcmd__usage__subcmd__help_commands() {
    local commands; commands=(
'credits:Show credits-based usage for the project' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr usage help commands' commands "$@"
}
(( $+functions[_xr__subcmd__usage__subcmd__help__subcmd__credits_commands] )) ||
_xr__subcmd__usage__subcmd__help__subcmd__credits_commands() {
    local commands; commands=()
    _describe -t commands 'xr usage help credits commands' commands "$@"
}
(( $+functions[_xr__subcmd__usage__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__usage__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr usage help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__user_commands] )) ||
_xr__subcmd__user_commands() {
    local commands; commands=()
    _describe -t commands 'xr user commands' commands "$@"
}
(( $+functions[_xr__subcmd__user-posts_commands] )) ||
_xr__subcmd__user-posts_commands() {
    local commands; commands=()
    _describe -t commands 'xr user-posts commands' commands "$@"
}
(( $+functions[_xr__subcmd__users_commands] )) ||
_xr__subcmd__users_commands() {
    local commands; commands=()
    _describe -t commands 'xr users commands' commands "$@"
}
(( $+functions[_xr__subcmd__validate_commands] )) ||
_xr__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'xr validate commands' commands "$@"
}
(( $+functions[_xr__subcmd__version_commands] )) ||
_xr__subcmd__version_commands() {
    local commands; commands=()
    _describe -t commands 'xr version commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks_commands] )) ||
_xr__subcmd__webhooks_commands() {
    local commands; commands=(
'list:List the webhooks registered for the app' \
'add:Register a webhook URL' \
'validate:Ask X to send its CRC check to a webhook again' \
'remove:Delete a webhook; its subscriptions stop delivering' \
'subscriptions:Account Activity subscriptions\: whose activity a webhook receives' \
'stream-links:Filtered-stream links\: deliver the posts your stream rules match to a webhook' \
'listen:Receive webhook deliveries on a local address' \
'replay:Deliver a past window of events to a webhook again' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr webhooks commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__add_commands] )) ||
_xr__subcmd__webhooks__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks add commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help_commands] )) ||
_xr__subcmd__webhooks__subcmd__help_commands() {
    local commands; commands=(
'list:List the webhooks registered for the app' \
'add:Register a webhook URL' \
'validate:Ask X to send its CRC check to a webhook again' \
'remove:Delete a webhook; its subscriptions stop delivering' \
'subscriptions:Account Activity subscriptions\: whose activity a webhook receives' \
'stream-links:Filtered-stream links\: deliver the posts your stream rules match to a webhook' \
'listen:Receive webhook deliveries on a local address' \
'replay:Deliver a past window of events to a webhook again' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr webhooks help commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__listen_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__listen_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help listen commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__replay_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__replay_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help replay commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__stream-links_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__stream-links_commands() {
    local commands; commands=(
'list:List the webhooks the app'\''s filtered stream delivers to' \
'add:Deliver the app'\''s filtered stream to a webhook' \
'remove:Stop delivering the app'\''s filtered stream to a webhook' \
    )
    _describe -t commands 'xr webhooks help stream-links commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__stream-links__subcmd__add_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__stream-links__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help stream-links add commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__stream-links__subcmd__list_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__stream-links__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help stream-links list commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__stream-links__subcmd__remove_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__stream-links__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help stream-links remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions_commands() {
    local commands; commands=(
'count:Count the subscriptions the app holds and may hold' \
'list:List the accounts whose activity a webhook receives' \
'add:Subscribe your account'\''s activity to a webhook' \
'check:Say whether your account'\''s activity is subscribed to a webhook' \
'remove:End an account'\''s subscription to a webhook' \
    )
    _describe -t commands 'xr webhooks help subscriptions commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__add_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help subscriptions add commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__check_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__check_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help subscriptions check commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__count_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__count_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help subscriptions count commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__list_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help subscriptions list commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__remove_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__subscriptions__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help subscriptions remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__help__subcmd__validate_commands] )) ||
_xr__subcmd__webhooks__subcmd__help__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks help validate commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__list_commands] )) ||
_xr__subcmd__webhooks__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks list commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__listen_commands] )) ||
_xr__subcmd__webhooks__subcmd__listen_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks listen commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__remove_commands] )) ||
_xr__subcmd__webhooks__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__replay_commands] )) ||
_xr__subcmd__webhooks__subcmd__replay_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks replay commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__stream-links_commands] )) ||
_xr__subcmd__webhooks__subcmd__stream-links_commands() {
    local commands; commands=(
'list:List the webhooks the app'\''s filtered stream delivers to' \
'add:Deliver the app'\''s filtered stream to a webhook' \
'remove:Stop delivering the app'\''s filtered stream to a webhook' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr webhooks stream-links commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__stream-links__subcmd__add_commands] )) ||
_xr__subcmd__webhooks__subcmd__stream-links__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks stream-links add commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help_commands] )) ||
_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help_commands() {
    local commands; commands=(
'list:List the webhooks the app'\''s filtered stream delivers to' \
'add:Deliver the app'\''s filtered stream to a webhook' \
'remove:Stop delivering the app'\''s filtered stream to a webhook' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr webhooks stream-links help commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks stream-links help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks stream-links help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks stream-links help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__webhooks__subcmd__stream-links__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks stream-links help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__stream-links__subcmd__list_commands] )) ||
_xr__subcmd__webhooks__subcmd__stream-links__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks stream-links list commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__stream-links__subcmd__remove_commands] )) ||
_xr__subcmd__webhooks__subcmd__stream-links__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks stream-links remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions_commands() {
    local commands; commands=(
'count:Count the subscriptions the app holds and may hold' \
'list:List the accounts whose activity a webhook receives' \
'add:Subscribe your account'\''s activity to a webhook' \
'check:Say whether your account'\''s activity is subscribed to a webhook' \
'remove:End an account'\''s subscription to a webhook' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr webhooks subscriptions commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__add_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions add commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__check_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__check_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions check commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__count_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__count_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions count commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help_commands() {
    local commands; commands=(
'count:Count the subscriptions the app holds and may hold' \
'list:List the accounts whose activity a webhook receives' \
'add:Subscribe your account'\''s activity to a webhook' \
'check:Say whether your account'\''s activity is subscribed to a webhook' \
'remove:End an account'\''s subscription to a webhook' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr webhooks subscriptions help commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__add_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions help add commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__check_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__check_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions help check commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__count_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__count_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions help count commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__help_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions help help commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__list_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions help list commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__remove_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions help remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__list_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions list commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__remove_commands] )) ||
_xr__subcmd__webhooks__subcmd__subscriptions__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks subscriptions remove commands' commands "$@"
}
(( $+functions[_xr__subcmd__webhooks__subcmd__validate_commands] )) ||
_xr__subcmd__webhooks__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'xr webhooks validate commands' commands "$@"
}
(( $+functions[_xr__subcmd__whoami_commands] )) ||
_xr__subcmd__whoami_commands() {
    local commands; commands=()
    _describe -t commands 'xr whoami commands' commands "$@"
}

if [ "$funcstack[1]" = "_xr" ]; then
    _xr "$@"
else
    compdef _xr xr
fi
