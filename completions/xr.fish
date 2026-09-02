# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_xr_global_optspecs
    string join \n X/method= H/header= d/data= auth= u/username= v/verbose= t/trace s/stream F/file= app= output= json jsonl raw= no-pager q/quiet= no-interactive= timeout= color= dry-run= limit= cursor= page= after= h/help V/version
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
complete -c xr -n "__fish_xr_needs_command" -s u -l username -d 'Username for `OAuth2` authentication' -r
complete -c xr -n "__fish_xr_needs_command" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -s F -l file -d 'File to upload (for multipart requests)' -r
complete -c xr -n "__fish_xr_needs_command" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_needs_command" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_needs_command" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_needs_command" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_needs_command" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_needs_command" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_needs_command" -a "search" -d 'Search recent posts'
complete -c xr -n "__fish_xr_needs_command" -a "whoami" -d 'Show the authenticated user\'s profile'
complete -c xr -n "__fish_xr_needs_command" -a "user" -d 'Look up a user by username'
complete -c xr -n "__fish_xr_needs_command" -a "timeline" -d 'Show your home timeline'
complete -c xr -n "__fish_xr_needs_command" -a "mentions" -d 'Show your recent mentions'
complete -c xr -n "__fish_xr_needs_command" -a "like" -d 'Like a post'
complete -c xr -n "__fish_xr_needs_command" -a "unlike" -d 'Unlike a post'
complete -c xr -n "__fish_xr_needs_command" -a "repost" -d 'Repost a post'
complete -c xr -n "__fish_xr_needs_command" -a "unrepost" -d 'Undo a repost'
complete -c xr -n "__fish_xr_needs_command" -a "bookmark" -d 'Bookmark a post'
complete -c xr -n "__fish_xr_needs_command" -a "unbookmark" -d 'Remove a bookmark'
complete -c xr -n "__fish_xr_needs_command" -a "bookmarks" -d 'List your bookmarks'
complete -c xr -n "__fish_xr_needs_command" -a "likes" -d 'List your liked posts'
complete -c xr -n "__fish_xr_needs_command" -a "follow" -d 'Follow a user'
complete -c xr -n "__fish_xr_needs_command" -a "unfollow" -d 'Unfollow a user'
complete -c xr -n "__fish_xr_needs_command" -a "following" -d 'List users you follow'
complete -c xr -n "__fish_xr_needs_command" -a "followers" -d 'List your followers'
complete -c xr -n "__fish_xr_needs_command" -a "mute" -d 'Mute a user'
complete -c xr -n "__fish_xr_needs_command" -a "unmute" -d 'Unmute a user'
complete -c xr -n "__fish_xr_needs_command" -a "usage" -d 'Show API usage (post caps, daily breakdown)'
complete -c xr -n "__fish_xr_needs_command" -a "dm" -d 'Send a direct message'
complete -c xr -n "__fish_xr_needs_command" -a "dms" -d 'List recent direct messages'
complete -c xr -n "__fish_xr_needs_command" -a "auth" -d 'Authentication management'
complete -c xr -n "__fish_xr_needs_command" -a "media" -d 'Media upload operations'
complete -c xr -n "__fish_xr_needs_command" -a "skill" -d 'Install or manage the xurl-rs skill bundle'
complete -c xr -n "__fish_xr_needs_command" -a "schema" -d 'Show JSON Schema for a command\'s response type'
complete -c xr -n "__fish_xr_needs_command" -a "completions" -d 'Generate shell completion script'
complete -c xr -n "__fish_xr_needs_command" -a "version" -d 'Show xurl version information'
complete -c xr -n "__fish_xr_needs_command" -a "examples" -d 'Print a curated gallery of invocation examples grouped by use case'
complete -c xr -n "__fish_xr_needs_command" -a "validate" -d 'Validate a JSON document against a bundled response schema'
complete -c xr -n "__fish_xr_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand post" -l media-id -d 'Media ID(s) to attach (repeatable)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand post" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand post" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand post" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand post" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand reply" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand reply" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand reply" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand reply" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand reply" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand reply" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand reply" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand reply" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand quote" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand quote" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand quote" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand quote" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand quote" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand quote" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand quote" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand quote" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand delete" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand delete" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand delete" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand delete" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand read" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand read" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand read" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand read" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand read" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand read" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand read" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand read" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand read" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand read" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand search" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand search" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand search" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand search" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand search" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand search" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand search" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand search" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand whoami" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand user" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand user" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand user" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand user" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand user" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand user" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand timeline" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand timeline" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand timeline" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand timeline" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand timeline" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand mentions" -s n -l max-results -d 'Number of results (5-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand mentions" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand mentions" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand mentions" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand mentions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand like" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand like" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand like" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand like" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand like" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand like" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand like" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand like" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand like" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand like" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand like" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unlike" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand repost" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand repost" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand repost" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand repost" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand repost" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand repost" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand repost" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand repost" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand likes" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand likes" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand likes" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand likes" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand likes" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand likes" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand likes" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand likes" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand follow" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand follow" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand follow" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand follow" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand follow" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand follow" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand follow" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand follow" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand following" -s n -l max-results -d 'Number of results (1-1000). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l of -d 'Username to list following for (default: you)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand following" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand following" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand following" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand following" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand following" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand following" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand following" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand following" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand followers" -s n -l max-results -d 'Number of results (1-1000). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l of -d 'Username to list followers for (default: you)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand followers" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand followers" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand followers" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand followers" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand followers" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand followers" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand followers" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand mute" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand mute" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand mute" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand mute" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand mute" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand mute" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand mute" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand mute" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand unmute" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and not __fish_seen_subcommand_from credits help" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand usage; and __fish_seen_subcommand_from credits" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand dm" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand dm" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand dm" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dm" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dm" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dm" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand dm" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand dms" -s n -l max-results -d 'Number of results (1-100). Overrides global `--limit` when set' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand dms" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand dms" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand dms" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dms" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dms" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand dms" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand dms" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l consumer-key -d 'Consumer key' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l consumer-secret -d 'Consumer secret' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l access-token -d 'Access token' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l token-secret -d 'Token secret' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l bearer-token -d 'Bearer token' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l oauth2-username -d 'Clear `OAuth2` token for username' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -f -a "upload" -d 'Upload media file'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -f -a "status" -d 'Check media upload status'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l media-type -d 'Media type (e.g., video/mp4)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l category -d 'Media category (e.g., `amplify_video`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l auth -d 'Authentication type' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s u -l username -d 'Username' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s H -l header -d 'Request headers' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l wait -d 'Wait for media processing to complete'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s t -l trace -d 'Trace header'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l auth -d 'Authentication type' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s u -l username -d 'Username' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s H -l header -d 'Request headers' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s w -l wait -d 'Wait for processing'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s t -l trace -d 'Trace header'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "upload" -d 'Upload media file'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "status" -d 'Check media upload status'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and not __fish_seen_subcommand_from install update help" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l all -d 'Install into every known host in one invocation'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l dry-run -d 'Print the resolved git command without spawning'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from install" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand skill; and __fish_seen_subcommand_from update" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand schema" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand schema" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand schema" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand schema" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
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
complete -c xr -n "__fish_xr_using_subcommand completions" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand completions" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand completions" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand completions" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand completions" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand completions" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand version" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand version" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand version" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand version" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand version" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand version" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand version" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand examples" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand examples" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand examples" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand examples" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand examples" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand examples" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand examples" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand examples" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand validate" -l schema -d 'Schema name to validate against (`post`, `posts`, `user`, `users`, `dm`, `dms`, `usage`, `credits`, `envelope`). Omit for auto-detection' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -s v -l verbose -d 'Print verbose information' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l output -d 'Output format. text (default), json, jsonl, ndjson (alias of jsonl), yaml (`.yml`), csv, tsv. Formats not in the value enum (e.g. toml, xml) are not supported — xurl emits a JSON envelope with reason `invalid-args` if requested' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'
ndjson\t'Newline-delimited JSON; alias of `jsonl`. Same wire shape, different name'
yaml\t'YAML document (best-effort serialization of the JSON shape)'
csv\t'Comma-separated values (best-effort flattening of the top-level shape)'
tsv\t'Tab-separated values (best-effort flattening of the top-level shape)'"
complete -c xr -n "__fish_xr_using_subcommand validate" -l raw -d 'Emit unstyled, compact output. Strips ANSI in text mode; compact (no pretty-printing) JSON in json/jsonl modes' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -l no-interactive -d 'Disable interactive prompts; fail with error instead' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l color -d 'Colorize output: auto (TTY-aware), always, or never' -r -f -a "auto\t'Enable color when stderr is a TTY and `NO_COLOR` is unset'
always\t'Always emit ANSI color escapes (still suppressed by `NO_COLOR`)'
never\t'Never emit ANSI color escapes'"
complete -c xr -n "__fish_xr_using_subcommand validate" -l dry-run -d 'Validate inputs and skip the API call (U7)' -r -f -a "true\t''
false\t''"
complete -c xr -n "__fish_xr_using_subcommand validate" -l limit -d 'Global result-set limit, clamped to 1..=100 (U7)' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l cursor -d 'Pagination cursor / `pagination_token` for list endpoints' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l page -d 'Documented alias for `--cursor`' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l after -d 'Documented alias for `--cursor` (`--after <token>`)' -r
complete -c xr -n "__fish_xr_using_subcommand validate" -l json -d 'Shorthand for `--output json` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand validate" -l jsonl -d 'Shorthand for `--output jsonl` (P2 alias)'
complete -c xr -n "__fish_xr_using_subcommand validate" -l no-pager -d 'Documented no-op. `xr` writes directly to stdout and never invokes `$PAGER`; this flag is advertised so agents can pass `--no-pager` unconditionally without xr rejecting it'
complete -c xr -n "__fish_xr_using_subcommand validate" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "post" -d 'Post to X'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "reply" -d 'Reply to a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "quote" -d 'Quote a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "delete" -d 'Delete a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "read" -d 'Read a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "search" -d 'Search recent posts'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "whoami" -d 'Show the authenticated user\'s profile'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "user" -d 'Look up a user by username'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "timeline" -d 'Show your home timeline'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "mentions" -d 'Show your recent mentions'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "like" -d 'Like a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "unlike" -d 'Unlike a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "repost" -d 'Repost a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "unrepost" -d 'Undo a repost'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "bookmark" -d 'Bookmark a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "unbookmark" -d 'Remove a bookmark'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "bookmarks" -d 'List your bookmarks'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "likes" -d 'List your liked posts'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "follow" -d 'Follow a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "unfollow" -d 'Unfollow a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "following" -d 'List users you follow'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "followers" -d 'List your followers'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "mute" -d 'Mute a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "unmute" -d 'Unmute a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "usage" -d 'Show API usage (post caps, daily breakdown)'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "dm" -d 'Send a direct message'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "dms" -d 'List recent direct messages'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "auth" -d 'Authentication management'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "media" -d 'Media upload operations'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "skill" -d 'Install or manage the xurl-rs skill bundle'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "schema" -d 'Show JSON Schema for a command\'s response type'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "completions" -d 'Generate shell completion script'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "version" -d 'Show xurl version information'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "examples" -d 'Print a curated gallery of invocation examples grouped by use case'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "validate" -d 'Validate a JSON document against a bundled response schema'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers mute unmute usage dm dms auth media skill schema completions version examples validate help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from usage" -f -a "credits" -d 'Show credits-based usage for the project'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "oauth2" -d 'Configure `OAuth2` authentication'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "oauth1" -d 'Configure `OAuth1` authentication'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "app" -d 'Configure app-auth (bearer token)'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "status" -d 'Show authentication status'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "clear" -d 'Clear authentication tokens'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "apps" -d 'Manage registered X API apps'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "default" -d 'Set default app and/or user'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "upload" -d 'Upload media file'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "status" -d 'Check media upload status'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "install" -d 'Install the skill bundle into a host\'s canonical skills directory'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from skill" -f -a "update" -d 'Refresh an existing skill-bundle install in place'
