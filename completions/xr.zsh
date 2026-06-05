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
'-u+[Username for \`OAuth2\` authentication]:USERNAME:_default' \
'--username=[Username for \`OAuth2\` authentication]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'-F+[File to upload (for multipart requests)]:FILE:_default' \
'--file=[File to upload (for multipart requests)]:FILE:_default' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'::url -- URL for raw mode (positional, only when no subcommand):_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(search)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(whoami)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(user)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-n+[Number of results (5-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (5-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(repost)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(likes)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-n+[Number of results (1-1000). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-1000). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--of=[Username to list following for (default\: you)]:OF:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-n+[Number of results (1-1000). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-1000). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--of=[Username to list followers for (default\: you)]:OF:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(mute)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(usage)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(dm)
_arguments "${_arguments_options[@]}" : \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(dms)
_arguments "${_arguments_options[@]}" : \
'-n+[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--max-results=[Number of results (1-100). Overrides global \`--limit\` when set]:MAX_RESULTS:_default' \
'--auth=[Authentication type (oauth1, oauth2, app)]:AUTH_TYPE:_default' \
'-u+[\`OAuth2\` username to act as]:USERNAME:_default' \
'--username=[\`OAuth2\` username to act as]:USERNAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
(auth)
_arguments "${_arguments_options[@]}" : \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__auth_commands" \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'--access-token=[Access token]:ACCESS_TOKEN:_default' \
'--token-secret=[Token secret]:TOKEN_SECRET:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__auth__apps_commands" \
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
'--redirect-uri=[\`OAuth2\` redirect URI (https or http on loopback)]:REDIRECT_URI:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'--redirect-uri=[\`OAuth2\` redirect URI (https or http on loopback); empty string clears]:REDIRECT_URI:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__auth__apps__redirect-uri_commands" \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
":: :_xr__auth__apps__redirect-uri__help_commands" \
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
":: :_xr__auth__apps__help_commands" \
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
":: :_xr__auth__apps__help__redirect-uri_commands" \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
":: :_xr__auth__help_commands" \
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
":: :_xr__auth__help__apps_commands" \
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
":: :_xr__auth__help__apps__redirect-uri_commands" \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__media_commands" \
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
'--auth=[Authentication type]:AUTH_TYPE:_default' \
'-u+[Username]:USERNAME:_default' \
'--username=[Username]:USERNAME:_default' \
'*-H+[Request headers]:HEADERS:_default' \
'*--header=[Request headers]:HEADERS:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'--wait[Wait for media processing to complete]' \
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
'-u+[Username]:USERNAME:_default' \
'--username=[Username]:USERNAME:_default' \
'*-H+[Request headers]:HEADERS:_default' \
'*--header=[Request headers]:HEADERS:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'-w[Wait for processing]' \
'--wait[Wait for processing]' \
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
(help)
_arguments "${_arguments_options[@]}" : \
":: :_xr__media__help_commands" \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
'--cursor=[Pagination cursor / \`pagination_token\` for list endpoints]:TOKEN:_default' \
'(--cursor)--page=[Documented alias for \`--cursor\`]:N:_default' \
'(--cursor --page)--after=[Documented alias for \`--cursor\` (\`--after <token>\`)]:TOKEN:_default' \
'(--output --jsonl)--json[Shorthand for \`--output json\` (P2 alias)]' \
'(--output --json)--jsonl[Shorthand for \`--output jsonl\` (P2 alias)]' \
'--no-pager[Documented no-op. \`xr\` writes directly to stdout and never invokes \`\$PAGER\`; this flag is advertised so agents can pass \`--no-pager\` unconditionally without xr rejecting it]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
":: :_xr__skill_commands" \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
":: :_xr__skill__help_commands" \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
'--schema=[Schema name to validate against (\`tweet\`, \`tweets\`, \`user\`, \`users\`, \`dm\`, \`dms\`, \`usage\`, \`envelope\`). Omit for auto-detection]:NAME:_default' \
'-v+[Print verbose information]::VERBOSE:(true false)' \
'--verbose=[Print verbose information]::VERBOSE:(true false)' \
'--app=[Use a specific registered app (overrides default)]:APP:_default' \
'--output=[Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (\`.yml\`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason \`invalid-args\` if requested]:OUTPUT:((text\:"Default\: colored, human-readable"
json\:"Machine-readable JSON, no color"
jsonl\:"JSON Lines (useful for streaming)"
ndjson\:"Newline-delimited JSON; alias of \`jsonl\`. Same wire shape, different name"
yaml\:"YAML document (best-effort serialization of the JSON shape)"
csv\:"Comma-separated values (best-effort flattening of the top-level shape)"
tsv\:"Tab-separated values (best-effort flattening of the top-level shape)"))' \
'--raw=[Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes]::RAW:(true false)' \
'-q+[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--quiet=[Suppress all non-essential output (errors still go to stderr)]::QUIET:(true false)' \
'--no-interactive=[Disable interactive prompts; fail with error instead]::NO_INTERACTIVE:(true false)' \
'--timeout=[Request timeout in seconds]:TIMEOUT:_default' \
'--color=[Colorize output\: auto (TTY-aware), always, or never]:COLOR:((auto\:"Enable color when stderr is a TTY and \`NO_COLOR\` is unset"
always\:"Always emit ANSI color escapes (still suppressed by \`NO_COLOR\`)"
never\:"Never emit ANSI color escapes"))' \
'--dry-run=[Validate inputs and skip the API call (U7)]::DRY_RUN:(true false)' \
'--limit=[Global result-set limit, clamped to 1..=100 (U7)]:LIMIT:_default' \
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
":: :_xr__help_commands" \
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
(search)
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
(block)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(unblock)
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
(usage)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(dm)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(dms)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(auth)
_arguments "${_arguments_options[@]}" : \
":: :_xr__help__auth_commands" \
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
":: :_xr__help__auth__apps_commands" \
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
":: :_xr__help__auth__apps__redirect-uri_commands" \
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
":: :_xr__help__media_commands" \
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
        esac
    ;;
esac
;;
(skill)
_arguments "${_arguments_options[@]}" : \
":: :_xr__help__skill_commands" \
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
'search:Search recent posts' \
'whoami:Show the authenticated user'\''s profile' \
'user:Look up a user by username' \
'timeline:Show your home timeline' \
'mentions:Show your recent mentions' \
'like:Like a post' \
'unlike:Unlike a post' \
'repost:Repost a post' \
'unrepost:Undo a repost' \
'bookmark:Bookmark a post' \
'unbookmark:Remove a bookmark' \
'bookmarks:List your bookmarks' \
'likes:List your liked posts' \
'follow:Follow a user' \
'unfollow:Unfollow a user' \
'following:List users you follow' \
'followers:List your followers' \
'block:Block a user' \
'unblock:Unblock a user' \
'mute:Mute a user' \
'unmute:Unmute a user' \
'usage:Show API usage (tweet caps, daily breakdown)' \
'dm:Send a direct message' \
'dms:List recent direct messages' \
'auth:Authentication management' \
'media:Media upload operations' \
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
(( $+functions[_xr__auth_commands] )) ||
_xr__auth_commands() {
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
(( $+functions[_xr__auth__app_commands] )) ||
_xr__auth__app_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth app commands' commands "$@"
}
(( $+functions[_xr__auth__apps_commands] )) ||
_xr__auth__apps_commands() {
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
(( $+functions[_xr__auth__apps__add_commands] )) ||
_xr__auth__apps__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps add commands' commands "$@"
}
(( $+functions[_xr__auth__apps__help_commands] )) ||
_xr__auth__apps__help_commands() {
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
(( $+functions[_xr__auth__apps__help__add_commands] )) ||
_xr__auth__apps__help__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help add commands' commands "$@"
}
(( $+functions[_xr__auth__apps__help__help_commands] )) ||
_xr__auth__apps__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help help commands' commands "$@"
}
(( $+functions[_xr__auth__apps__help__list_commands] )) ||
_xr__auth__apps__help__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help list commands' commands "$@"
}
(( $+functions[_xr__auth__apps__help__redirect-uri_commands] )) ||
_xr__auth__apps__help__redirect-uri_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
    )
    _describe -t commands 'xr auth apps help redirect-uri commands' commands "$@"
}
(( $+functions[_xr__auth__apps__help__redirect-uri__get_commands] )) ||
_xr__auth__apps__help__redirect-uri__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help redirect-uri get commands' commands "$@"
}
(( $+functions[_xr__auth__apps__help__redirect-uri__set_commands] )) ||
_xr__auth__apps__help__redirect-uri__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help redirect-uri set commands' commands "$@"
}
(( $+functions[_xr__auth__apps__help__remove_commands] )) ||
_xr__auth__apps__help__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help remove commands' commands "$@"
}
(( $+functions[_xr__auth__apps__help__update_commands] )) ||
_xr__auth__apps__help__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps help update commands' commands "$@"
}
(( $+functions[_xr__auth__apps__list_commands] )) ||
_xr__auth__apps__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps list commands' commands "$@"
}
(( $+functions[_xr__auth__apps__redirect-uri_commands] )) ||
_xr__auth__apps__redirect-uri_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr auth apps redirect-uri commands' commands "$@"
}
(( $+functions[_xr__auth__apps__redirect-uri__get_commands] )) ||
_xr__auth__apps__redirect-uri__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri get commands' commands "$@"
}
(( $+functions[_xr__auth__apps__redirect-uri__help_commands] )) ||
_xr__auth__apps__redirect-uri__help_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr auth apps redirect-uri help commands' commands "$@"
}
(( $+functions[_xr__auth__apps__redirect-uri__help__get_commands] )) ||
_xr__auth__apps__redirect-uri__help__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri help get commands' commands "$@"
}
(( $+functions[_xr__auth__apps__redirect-uri__help__help_commands] )) ||
_xr__auth__apps__redirect-uri__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri help help commands' commands "$@"
}
(( $+functions[_xr__auth__apps__redirect-uri__help__set_commands] )) ||
_xr__auth__apps__redirect-uri__help__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri help set commands' commands "$@"
}
(( $+functions[_xr__auth__apps__redirect-uri__set_commands] )) ||
_xr__auth__apps__redirect-uri__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps redirect-uri set commands' commands "$@"
}
(( $+functions[_xr__auth__apps__remove_commands] )) ||
_xr__auth__apps__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps remove commands' commands "$@"
}
(( $+functions[_xr__auth__apps__update_commands] )) ||
_xr__auth__apps__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth apps update commands' commands "$@"
}
(( $+functions[_xr__auth__clear_commands] )) ||
_xr__auth__clear_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth clear commands' commands "$@"
}
(( $+functions[_xr__auth__default_commands] )) ||
_xr__auth__default_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth default commands' commands "$@"
}
(( $+functions[_xr__auth__help_commands] )) ||
_xr__auth__help_commands() {
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
(( $+functions[_xr__auth__help__app_commands] )) ||
_xr__auth__help__app_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help app commands' commands "$@"
}
(( $+functions[_xr__auth__help__apps_commands] )) ||
_xr__auth__help__apps_commands() {
    local commands; commands=(
'add:Register a new X API app' \
'update:Update credentials for an existing app' \
'remove:Remove a registered app' \
'list:List registered apps' \
'redirect-uri:Inspect or set the stored \`OAuth2\` redirect URI for an app' \
    )
    _describe -t commands 'xr auth help apps commands' commands "$@"
}
(( $+functions[_xr__auth__help__apps__add_commands] )) ||
_xr__auth__help__apps__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps add commands' commands "$@"
}
(( $+functions[_xr__auth__help__apps__list_commands] )) ||
_xr__auth__help__apps__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps list commands' commands "$@"
}
(( $+functions[_xr__auth__help__apps__redirect-uri_commands] )) ||
_xr__auth__help__apps__redirect-uri_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
    )
    _describe -t commands 'xr auth help apps redirect-uri commands' commands "$@"
}
(( $+functions[_xr__auth__help__apps__redirect-uri__get_commands] )) ||
_xr__auth__help__apps__redirect-uri__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps redirect-uri get commands' commands "$@"
}
(( $+functions[_xr__auth__help__apps__redirect-uri__set_commands] )) ||
_xr__auth__help__apps__redirect-uri__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps redirect-uri set commands' commands "$@"
}
(( $+functions[_xr__auth__help__apps__remove_commands] )) ||
_xr__auth__help__apps__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps remove commands' commands "$@"
}
(( $+functions[_xr__auth__help__apps__update_commands] )) ||
_xr__auth__help__apps__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help apps update commands' commands "$@"
}
(( $+functions[_xr__auth__help__clear_commands] )) ||
_xr__auth__help__clear_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help clear commands' commands "$@"
}
(( $+functions[_xr__auth__help__default_commands] )) ||
_xr__auth__help__default_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help default commands' commands "$@"
}
(( $+functions[_xr__auth__help__help_commands] )) ||
_xr__auth__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help help commands' commands "$@"
}
(( $+functions[_xr__auth__help__oauth1_commands] )) ||
_xr__auth__help__oauth1_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help oauth1 commands' commands "$@"
}
(( $+functions[_xr__auth__help__oauth2_commands] )) ||
_xr__auth__help__oauth2_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help oauth2 commands' commands "$@"
}
(( $+functions[_xr__auth__help__status_commands] )) ||
_xr__auth__help__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth help status commands' commands "$@"
}
(( $+functions[_xr__auth__oauth1_commands] )) ||
_xr__auth__oauth1_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth oauth1 commands' commands "$@"
}
(( $+functions[_xr__auth__oauth2_commands] )) ||
_xr__auth__oauth2_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth oauth2 commands' commands "$@"
}
(( $+functions[_xr__auth__status_commands] )) ||
_xr__auth__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr auth status commands' commands "$@"
}
(( $+functions[_xr__block_commands] )) ||
_xr__block_commands() {
    local commands; commands=()
    _describe -t commands 'xr block commands' commands "$@"
}
(( $+functions[_xr__bookmark_commands] )) ||
_xr__bookmark_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmark commands' commands "$@"
}
(( $+functions[_xr__bookmarks_commands] )) ||
_xr__bookmarks_commands() {
    local commands; commands=()
    _describe -t commands 'xr bookmarks commands' commands "$@"
}
(( $+functions[_xr__completions_commands] )) ||
_xr__completions_commands() {
    local commands; commands=()
    _describe -t commands 'xr completions commands' commands "$@"
}
(( $+functions[_xr__delete_commands] )) ||
_xr__delete_commands() {
    local commands; commands=()
    _describe -t commands 'xr delete commands' commands "$@"
}
(( $+functions[_xr__dm_commands] )) ||
_xr__dm_commands() {
    local commands; commands=()
    _describe -t commands 'xr dm commands' commands "$@"
}
(( $+functions[_xr__dms_commands] )) ||
_xr__dms_commands() {
    local commands; commands=()
    _describe -t commands 'xr dms commands' commands "$@"
}
(( $+functions[_xr__examples_commands] )) ||
_xr__examples_commands() {
    local commands; commands=()
    _describe -t commands 'xr examples commands' commands "$@"
}
(( $+functions[_xr__follow_commands] )) ||
_xr__follow_commands() {
    local commands; commands=()
    _describe -t commands 'xr follow commands' commands "$@"
}
(( $+functions[_xr__followers_commands] )) ||
_xr__followers_commands() {
    local commands; commands=()
    _describe -t commands 'xr followers commands' commands "$@"
}
(( $+functions[_xr__following_commands] )) ||
_xr__following_commands() {
    local commands; commands=()
    _describe -t commands 'xr following commands' commands "$@"
}
(( $+functions[_xr__help_commands] )) ||
_xr__help_commands() {
    local commands; commands=(
'post:Post to X' \
'reply:Reply to a post' \
'quote:Quote a post' \
'delete:Delete a post' \
'read:Read a post' \
'search:Search recent posts' \
'whoami:Show the authenticated user'\''s profile' \
'user:Look up a user by username' \
'timeline:Show your home timeline' \
'mentions:Show your recent mentions' \
'like:Like a post' \
'unlike:Unlike a post' \
'repost:Repost a post' \
'unrepost:Undo a repost' \
'bookmark:Bookmark a post' \
'unbookmark:Remove a bookmark' \
'bookmarks:List your bookmarks' \
'likes:List your liked posts' \
'follow:Follow a user' \
'unfollow:Unfollow a user' \
'following:List users you follow' \
'followers:List your followers' \
'block:Block a user' \
'unblock:Unblock a user' \
'mute:Mute a user' \
'unmute:Unmute a user' \
'usage:Show API usage (tweet caps, daily breakdown)' \
'dm:Send a direct message' \
'dms:List recent direct messages' \
'auth:Authentication management' \
'media:Media upload operations' \
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
(( $+functions[_xr__help__auth_commands] )) ||
_xr__help__auth_commands() {
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
(( $+functions[_xr__help__auth__app_commands] )) ||
_xr__help__auth__app_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth app commands' commands "$@"
}
(( $+functions[_xr__help__auth__apps_commands] )) ||
_xr__help__auth__apps_commands() {
    local commands; commands=(
'add:Register a new X API app' \
'update:Update credentials for an existing app' \
'remove:Remove a registered app' \
'list:List registered apps' \
'redirect-uri:Inspect or set the stored \`OAuth2\` redirect URI for an app' \
    )
    _describe -t commands 'xr help auth apps commands' commands "$@"
}
(( $+functions[_xr__help__auth__apps__add_commands] )) ||
_xr__help__auth__apps__add_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps add commands' commands "$@"
}
(( $+functions[_xr__help__auth__apps__list_commands] )) ||
_xr__help__auth__apps__list_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps list commands' commands "$@"
}
(( $+functions[_xr__help__auth__apps__redirect-uri_commands] )) ||
_xr__help__auth__apps__redirect-uri_commands() {
    local commands; commands=(
'get:Show the effective redirect URI, its source, and the stored value' \
'set:Set the stored redirect URI for an app (empty string clears)' \
    )
    _describe -t commands 'xr help auth apps redirect-uri commands' commands "$@"
}
(( $+functions[_xr__help__auth__apps__redirect-uri__get_commands] )) ||
_xr__help__auth__apps__redirect-uri__get_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps redirect-uri get commands' commands "$@"
}
(( $+functions[_xr__help__auth__apps__redirect-uri__set_commands] )) ||
_xr__help__auth__apps__redirect-uri__set_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps redirect-uri set commands' commands "$@"
}
(( $+functions[_xr__help__auth__apps__remove_commands] )) ||
_xr__help__auth__apps__remove_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps remove commands' commands "$@"
}
(( $+functions[_xr__help__auth__apps__update_commands] )) ||
_xr__help__auth__apps__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth apps update commands' commands "$@"
}
(( $+functions[_xr__help__auth__clear_commands] )) ||
_xr__help__auth__clear_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth clear commands' commands "$@"
}
(( $+functions[_xr__help__auth__default_commands] )) ||
_xr__help__auth__default_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth default commands' commands "$@"
}
(( $+functions[_xr__help__auth__oauth1_commands] )) ||
_xr__help__auth__oauth1_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth oauth1 commands' commands "$@"
}
(( $+functions[_xr__help__auth__oauth2_commands] )) ||
_xr__help__auth__oauth2_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth oauth2 commands' commands "$@"
}
(( $+functions[_xr__help__auth__status_commands] )) ||
_xr__help__auth__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr help auth status commands' commands "$@"
}
(( $+functions[_xr__help__block_commands] )) ||
_xr__help__block_commands() {
    local commands; commands=()
    _describe -t commands 'xr help block commands' commands "$@"
}
(( $+functions[_xr__help__bookmark_commands] )) ||
_xr__help__bookmark_commands() {
    local commands; commands=()
    _describe -t commands 'xr help bookmark commands' commands "$@"
}
(( $+functions[_xr__help__bookmarks_commands] )) ||
_xr__help__bookmarks_commands() {
    local commands; commands=()
    _describe -t commands 'xr help bookmarks commands' commands "$@"
}
(( $+functions[_xr__help__completions_commands] )) ||
_xr__help__completions_commands() {
    local commands; commands=()
    _describe -t commands 'xr help completions commands' commands "$@"
}
(( $+functions[_xr__help__delete_commands] )) ||
_xr__help__delete_commands() {
    local commands; commands=()
    _describe -t commands 'xr help delete commands' commands "$@"
}
(( $+functions[_xr__help__dm_commands] )) ||
_xr__help__dm_commands() {
    local commands; commands=()
    _describe -t commands 'xr help dm commands' commands "$@"
}
(( $+functions[_xr__help__dms_commands] )) ||
_xr__help__dms_commands() {
    local commands; commands=()
    _describe -t commands 'xr help dms commands' commands "$@"
}
(( $+functions[_xr__help__examples_commands] )) ||
_xr__help__examples_commands() {
    local commands; commands=()
    _describe -t commands 'xr help examples commands' commands "$@"
}
(( $+functions[_xr__help__follow_commands] )) ||
_xr__help__follow_commands() {
    local commands; commands=()
    _describe -t commands 'xr help follow commands' commands "$@"
}
(( $+functions[_xr__help__followers_commands] )) ||
_xr__help__followers_commands() {
    local commands; commands=()
    _describe -t commands 'xr help followers commands' commands "$@"
}
(( $+functions[_xr__help__following_commands] )) ||
_xr__help__following_commands() {
    local commands; commands=()
    _describe -t commands 'xr help following commands' commands "$@"
}
(( $+functions[_xr__help__help_commands] )) ||
_xr__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr help help commands' commands "$@"
}
(( $+functions[_xr__help__like_commands] )) ||
_xr__help__like_commands() {
    local commands; commands=()
    _describe -t commands 'xr help like commands' commands "$@"
}
(( $+functions[_xr__help__likes_commands] )) ||
_xr__help__likes_commands() {
    local commands; commands=()
    _describe -t commands 'xr help likes commands' commands "$@"
}
(( $+functions[_xr__help__media_commands] )) ||
_xr__help__media_commands() {
    local commands; commands=(
'upload:Upload media file' \
'status:Check media upload status' \
    )
    _describe -t commands 'xr help media commands' commands "$@"
}
(( $+functions[_xr__help__media__status_commands] )) ||
_xr__help__media__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr help media status commands' commands "$@"
}
(( $+functions[_xr__help__media__upload_commands] )) ||
_xr__help__media__upload_commands() {
    local commands; commands=()
    _describe -t commands 'xr help media upload commands' commands "$@"
}
(( $+functions[_xr__help__mentions_commands] )) ||
_xr__help__mentions_commands() {
    local commands; commands=()
    _describe -t commands 'xr help mentions commands' commands "$@"
}
(( $+functions[_xr__help__mute_commands] )) ||
_xr__help__mute_commands() {
    local commands; commands=()
    _describe -t commands 'xr help mute commands' commands "$@"
}
(( $+functions[_xr__help__post_commands] )) ||
_xr__help__post_commands() {
    local commands; commands=()
    _describe -t commands 'xr help post commands' commands "$@"
}
(( $+functions[_xr__help__quote_commands] )) ||
_xr__help__quote_commands() {
    local commands; commands=()
    _describe -t commands 'xr help quote commands' commands "$@"
}
(( $+functions[_xr__help__read_commands] )) ||
_xr__help__read_commands() {
    local commands; commands=()
    _describe -t commands 'xr help read commands' commands "$@"
}
(( $+functions[_xr__help__reply_commands] )) ||
_xr__help__reply_commands() {
    local commands; commands=()
    _describe -t commands 'xr help reply commands' commands "$@"
}
(( $+functions[_xr__help__repost_commands] )) ||
_xr__help__repost_commands() {
    local commands; commands=()
    _describe -t commands 'xr help repost commands' commands "$@"
}
(( $+functions[_xr__help__schema_commands] )) ||
_xr__help__schema_commands() {
    local commands; commands=()
    _describe -t commands 'xr help schema commands' commands "$@"
}
(( $+functions[_xr__help__search_commands] )) ||
_xr__help__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr help search commands' commands "$@"
}
(( $+functions[_xr__help__skill_commands] )) ||
_xr__help__skill_commands() {
    local commands; commands=(
'install:Install the skill bundle into a host'\''s canonical skills directory' \
'update:Refresh an existing skill-bundle install in place' \
    )
    _describe -t commands 'xr help skill commands' commands "$@"
}
(( $+functions[_xr__help__skill__install_commands] )) ||
_xr__help__skill__install_commands() {
    local commands; commands=()
    _describe -t commands 'xr help skill install commands' commands "$@"
}
(( $+functions[_xr__help__skill__update_commands] )) ||
_xr__help__skill__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr help skill update commands' commands "$@"
}
(( $+functions[_xr__help__timeline_commands] )) ||
_xr__help__timeline_commands() {
    local commands; commands=()
    _describe -t commands 'xr help timeline commands' commands "$@"
}
(( $+functions[_xr__help__unblock_commands] )) ||
_xr__help__unblock_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unblock commands' commands "$@"
}
(( $+functions[_xr__help__unbookmark_commands] )) ||
_xr__help__unbookmark_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unbookmark commands' commands "$@"
}
(( $+functions[_xr__help__unfollow_commands] )) ||
_xr__help__unfollow_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unfollow commands' commands "$@"
}
(( $+functions[_xr__help__unlike_commands] )) ||
_xr__help__unlike_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unlike commands' commands "$@"
}
(( $+functions[_xr__help__unmute_commands] )) ||
_xr__help__unmute_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unmute commands' commands "$@"
}
(( $+functions[_xr__help__unrepost_commands] )) ||
_xr__help__unrepost_commands() {
    local commands; commands=()
    _describe -t commands 'xr help unrepost commands' commands "$@"
}
(( $+functions[_xr__help__usage_commands] )) ||
_xr__help__usage_commands() {
    local commands; commands=()
    _describe -t commands 'xr help usage commands' commands "$@"
}
(( $+functions[_xr__help__user_commands] )) ||
_xr__help__user_commands() {
    local commands; commands=()
    _describe -t commands 'xr help user commands' commands "$@"
}
(( $+functions[_xr__help__validate_commands] )) ||
_xr__help__validate_commands() {
    local commands; commands=()
    _describe -t commands 'xr help validate commands' commands "$@"
}
(( $+functions[_xr__help__version_commands] )) ||
_xr__help__version_commands() {
    local commands; commands=()
    _describe -t commands 'xr help version commands' commands "$@"
}
(( $+functions[_xr__help__whoami_commands] )) ||
_xr__help__whoami_commands() {
    local commands; commands=()
    _describe -t commands 'xr help whoami commands' commands "$@"
}
(( $+functions[_xr__like_commands] )) ||
_xr__like_commands() {
    local commands; commands=()
    _describe -t commands 'xr like commands' commands "$@"
}
(( $+functions[_xr__likes_commands] )) ||
_xr__likes_commands() {
    local commands; commands=()
    _describe -t commands 'xr likes commands' commands "$@"
}
(( $+functions[_xr__media_commands] )) ||
_xr__media_commands() {
    local commands; commands=(
'upload:Upload media file' \
'status:Check media upload status' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr media commands' commands "$@"
}
(( $+functions[_xr__media__help_commands] )) ||
_xr__media__help_commands() {
    local commands; commands=(
'upload:Upload media file' \
'status:Check media upload status' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr media help commands' commands "$@"
}
(( $+functions[_xr__media__help__help_commands] )) ||
_xr__media__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help help commands' commands "$@"
}
(( $+functions[_xr__media__help__status_commands] )) ||
_xr__media__help__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help status commands' commands "$@"
}
(( $+functions[_xr__media__help__upload_commands] )) ||
_xr__media__help__upload_commands() {
    local commands; commands=()
    _describe -t commands 'xr media help upload commands' commands "$@"
}
(( $+functions[_xr__media__status_commands] )) ||
_xr__media__status_commands() {
    local commands; commands=()
    _describe -t commands 'xr media status commands' commands "$@"
}
(( $+functions[_xr__media__upload_commands] )) ||
_xr__media__upload_commands() {
    local commands; commands=()
    _describe -t commands 'xr media upload commands' commands "$@"
}
(( $+functions[_xr__mentions_commands] )) ||
_xr__mentions_commands() {
    local commands; commands=()
    _describe -t commands 'xr mentions commands' commands "$@"
}
(( $+functions[_xr__mute_commands] )) ||
_xr__mute_commands() {
    local commands; commands=()
    _describe -t commands 'xr mute commands' commands "$@"
}
(( $+functions[_xr__post_commands] )) ||
_xr__post_commands() {
    local commands; commands=()
    _describe -t commands 'xr post commands' commands "$@"
}
(( $+functions[_xr__quote_commands] )) ||
_xr__quote_commands() {
    local commands; commands=()
    _describe -t commands 'xr quote commands' commands "$@"
}
(( $+functions[_xr__read_commands] )) ||
_xr__read_commands() {
    local commands; commands=()
    _describe -t commands 'xr read commands' commands "$@"
}
(( $+functions[_xr__reply_commands] )) ||
_xr__reply_commands() {
    local commands; commands=()
    _describe -t commands 'xr reply commands' commands "$@"
}
(( $+functions[_xr__repost_commands] )) ||
_xr__repost_commands() {
    local commands; commands=()
    _describe -t commands 'xr repost commands' commands "$@"
}
(( $+functions[_xr__schema_commands] )) ||
_xr__schema_commands() {
    local commands; commands=()
    _describe -t commands 'xr schema commands' commands "$@"
}
(( $+functions[_xr__search_commands] )) ||
_xr__search_commands() {
    local commands; commands=()
    _describe -t commands 'xr search commands' commands "$@"
}
(( $+functions[_xr__skill_commands] )) ||
_xr__skill_commands() {
    local commands; commands=(
'install:Install the skill bundle into a host'\''s canonical skills directory' \
'update:Refresh an existing skill-bundle install in place' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr skill commands' commands "$@"
}
(( $+functions[_xr__skill__help_commands] )) ||
_xr__skill__help_commands() {
    local commands; commands=(
'install:Install the skill bundle into a host'\''s canonical skills directory' \
'update:Refresh an existing skill-bundle install in place' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'xr skill help commands' commands "$@"
}
(( $+functions[_xr__skill__help__help_commands] )) ||
_xr__skill__help__help_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill help help commands' commands "$@"
}
(( $+functions[_xr__skill__help__install_commands] )) ||
_xr__skill__help__install_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill help install commands' commands "$@"
}
(( $+functions[_xr__skill__help__update_commands] )) ||
_xr__skill__help__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill help update commands' commands "$@"
}
(( $+functions[_xr__skill__install_commands] )) ||
_xr__skill__install_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill install commands' commands "$@"
}
(( $+functions[_xr__skill__update_commands] )) ||
_xr__skill__update_commands() {
    local commands; commands=()
    _describe -t commands 'xr skill update commands' commands "$@"
}
(( $+functions[_xr__timeline_commands] )) ||
_xr__timeline_commands() {
    local commands; commands=()
    _describe -t commands 'xr timeline commands' commands "$@"
}
(( $+functions[_xr__unblock_commands] )) ||
_xr__unblock_commands() {
    local commands; commands=()
    _describe -t commands 'xr unblock commands' commands "$@"
}
(( $+functions[_xr__unbookmark_commands] )) ||
_xr__unbookmark_commands() {
    local commands; commands=()
    _describe -t commands 'xr unbookmark commands' commands "$@"
}
(( $+functions[_xr__unfollow_commands] )) ||
_xr__unfollow_commands() {
    local commands; commands=()
    _describe -t commands 'xr unfollow commands' commands "$@"
}
(( $+functions[_xr__unlike_commands] )) ||
_xr__unlike_commands() {
    local commands; commands=()
    _describe -t commands 'xr unlike commands' commands "$@"
}
(( $+functions[_xr__unmute_commands] )) ||
_xr__unmute_commands() {
    local commands; commands=()
    _describe -t commands 'xr unmute commands' commands "$@"
}
(( $+functions[_xr__unrepost_commands] )) ||
_xr__unrepost_commands() {
    local commands; commands=()
    _describe -t commands 'xr unrepost commands' commands "$@"
}
(( $+functions[_xr__usage_commands] )) ||
_xr__usage_commands() {
    local commands; commands=()
    _describe -t commands 'xr usage commands' commands "$@"
}
(( $+functions[_xr__user_commands] )) ||
_xr__user_commands() {
    local commands; commands=()
    _describe -t commands 'xr user commands' commands "$@"
}
(( $+functions[_xr__validate_commands] )) ||
_xr__validate_commands() {
    local commands; commands=()
    _describe -t commands 'xr validate commands' commands "$@"
}
(( $+functions[_xr__version_commands] )) ||
_xr__version_commands() {
    local commands; commands=()
    _describe -t commands 'xr version commands' commands "$@"
}
(( $+functions[_xr__whoami_commands] )) ||
_xr__whoami_commands() {
    local commands; commands=()
    _describe -t commands 'xr whoami commands' commands "$@"
}

if [ "$funcstack[1]" = "_xr" ]; then
    _xr "$@"
else
    compdef _xr xr
fi
