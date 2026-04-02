# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_xr_global_optspecs
	string join \n X/method= H/header= d/data= auth= u/username= v/verbose t/trace s/stream F/file= app= output= q/quiet no-interactive timeout= h/help V/version
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
complete -c xr -n "__fish_xr_needs_command" -s F -l file -d 'File to upload (for multipart requests)' -r
complete -c xr -n "__fish_xr_needs_command" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_needs_command" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_needs_command" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_needs_command" -s v -l verbose -d 'Print verbose information'
complete -c xr -n "__fish_xr_needs_command" -s t -l trace -d 'Add trace header to request'
complete -c xr -n "__fish_xr_needs_command" -s s -l stream -d 'Force streaming mode'
complete -c xr -n "__fish_xr_needs_command" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_needs_command" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
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
complete -c xr -n "__fish_xr_needs_command" -a "block" -d 'Block a user'
complete -c xr -n "__fish_xr_needs_command" -a "unblock" -d 'Unblock a user'
complete -c xr -n "__fish_xr_needs_command" -a "mute" -d 'Mute a user'
complete -c xr -n "__fish_xr_needs_command" -a "unmute" -d 'Unmute a user'
complete -c xr -n "__fish_xr_needs_command" -a "usage" -d 'Show API usage (tweet caps, daily breakdown)'
complete -c xr -n "__fish_xr_needs_command" -a "dm" -d 'Send a direct message'
complete -c xr -n "__fish_xr_needs_command" -a "dms" -d 'List recent direct messages'
complete -c xr -n "__fish_xr_needs_command" -a "auth" -d 'Authentication management'
complete -c xr -n "__fish_xr_needs_command" -a "media" -d 'Media upload operations'
complete -c xr -n "__fish_xr_needs_command" -a "schema" -d 'Show JSON Schema for a command\'s response type'
complete -c xr -n "__fish_xr_needs_command" -a "completions" -d 'Generate shell completion script'
complete -c xr -n "__fish_xr_needs_command" -a "version" -d 'Show xurl version information'
complete -c xr -n "__fish_xr_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand post" -l media-id -d 'Media ID(s) to attach (repeatable)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand post" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand post" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand post" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand post" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand post" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand post" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand post" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand reply" -l media-id -d 'Media ID(s) to attach (repeatable)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand reply" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand reply" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand reply" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand reply" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand reply" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand reply" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand quote" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand quote" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand quote" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand quote" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand quote" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand quote" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand quote" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand delete" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand delete" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand delete" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand delete" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand delete" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand delete" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand delete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand read" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand read" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand read" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand read" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand read" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand read" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand read" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand read" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand read" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand search" -s n -l max-results -d 'Number of results (min 10, max 100)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand search" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand search" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand search" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand search" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand search" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand search" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand search" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand whoami" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand whoami" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand whoami" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand whoami" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand whoami" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand whoami" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand user" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand user" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand user" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand user" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand user" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand user" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand user" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand user" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand user" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand timeline" -s n -l max-results -d 'Number of results (1-100)' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand timeline" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand timeline" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand timeline" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand timeline" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand timeline" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand timeline" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand mentions" -s n -l max-results -d 'Number of results (5-100)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand mentions" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand mentions" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand mentions" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand mentions" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand mentions" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand mentions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand like" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand like" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand like" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand like" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand like" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand like" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand like" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand like" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand like" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand unlike" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unlike" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand unlike" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unlike" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand unlike" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand unlike" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand repost" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand repost" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand repost" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand repost" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand repost" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand repost" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand repost" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand unrepost" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand bookmark" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand unbookmark" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s n -l max-results -d 'Number of results (1-100)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand bookmarks" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand likes" -s n -l max-results -d 'Number of results (1-100)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand likes" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand likes" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand likes" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand likes" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand likes" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand likes" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand follow" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand follow" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand follow" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand follow" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand follow" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand follow" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand follow" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand unfollow" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand following" -s n -l max-results -d 'Number of results (1-1000)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l of -d 'Username to list following for (default: you)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand following" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand following" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand following" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand following" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand following" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand following" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand following" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand followers" -s n -l max-results -d 'Number of results (1-1000)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l of -d 'Username to list followers for (default: you)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand followers" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand followers" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand followers" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand followers" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand followers" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand followers" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand block" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand block" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand block" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand block" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand block" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand block" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand block" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand block" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand block" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand block" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unblock" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand unblock" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unblock" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand unblock" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unblock" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand unblock" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand unblock" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand mute" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand mute" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand mute" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand mute" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand mute" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand mute" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand mute" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand unmute" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand unmute" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand unmute" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand unmute" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand unmute" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand unmute" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand usage" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand usage" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand usage" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand usage" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand usage" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand usage" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand usage" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand usage" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand usage" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand usage" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand dm" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand dm" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dm" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand dm" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dm" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand dm" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand dm" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand dms" -s n -l max-results -d 'Number of results (1-100)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l auth -d 'Authentication type (oauth1, oauth2, app)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -s u -l username -d '`OAuth2` username to act as' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand dms" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand dms" -s v -l verbose -d 'Print verbose request/response info'
complete -c xr -n "__fish_xr_using_subcommand dms" -s t -l trace -d 'Add X-B3-Flags trace header'
complete -c xr -n "__fish_xr_using_subcommand dms" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand dms" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand dms" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "oauth2" -d 'Configure `OAuth2` authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "oauth1" -d 'Configure `OAuth1` authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "app" -d 'Configure app-auth (bearer token)'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "status" -d 'Show authentication status'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "clear" -d 'Clear authentication tokens'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "apps" -d 'Manage registered X API apps'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "default" -d 'Set default app and/or user'
complete -c xr -n "__fish_xr_using_subcommand auth; and not __fish_seen_subcommand_from oauth2 oauth1 app status clear apps default help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l step -d 'Step number: 1 (generate auth URL) or 2 (complete exchange)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l auth-url -d 'Redirect URL from browser (step 2). Use \'-\' to read from stdin (recommended on shared machines)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l no-browser -d 'Enable manual two-step flow for headless machines (SSH, containers)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth2" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l consumer-key -d 'Consumer key' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l consumer-secret -d 'Consumer secret' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l access-token -d 'Access token' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l token-secret -d 'Token secret' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from oauth1" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l bearer-token -d 'Bearer token' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from app" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l oauth2-username -d 'Clear `OAuth2` token for username' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l all -d 'Clear all authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l oauth1 -d 'Clear `OAuth1` tokens'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l bearer -d 'Clear bearer token'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from clear" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "add" -d 'Register a new X API app'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "update" -d 'Update credentials for an existing app'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "remove" -d 'Remove a registered app'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "list" -d 'List registered apps'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from apps" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from default" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "oauth2" -d 'Configure `OAuth2` authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "oauth1" -d 'Configure `OAuth1` authentication'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "app" -d 'Configure app-auth (bearer token)'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show authentication status'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "clear" -d 'Clear authentication tokens'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "apps" -d 'Manage registered X API apps'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "default" -d 'Set default app and/or user'
complete -c xr -n "__fish_xr_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -f -a "upload" -d 'Upload media file'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -f -a "status" -d 'Check media upload status'
complete -c xr -n "__fish_xr_using_subcommand media; and not __fish_seen_subcommand_from upload status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l media-type -d 'Media type (e.g., video/mp4)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l category -d 'Media category (e.g., `amplify_video`)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l auth -d 'Authentication type' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s u -l username -d 'Username' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s H -l header -d 'Request headers' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l wait -d 'Wait for media processing to complete'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s v -l verbose -d 'Verbose output'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s t -l trace -d 'Trace header'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from upload" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l auth -d 'Authentication type' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s u -l username -d 'Username' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s H -l header -d 'Request headers' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Verbose output'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s w -l wait -d 'Wait for processing'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s t -l trace -d 'Trace header'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "upload" -d 'Upload media file'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "status" -d 'Check media upload status'
complete -c xr -n "__fish_xr_using_subcommand media; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand schema" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand schema" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand schema" -l list -d 'List all commands and their response types'
complete -c xr -n "__fish_xr_using_subcommand schema" -l all -d 'Output all schemas as a single JSON document'
complete -c xr -n "__fish_xr_using_subcommand schema" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand schema" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand schema" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand completions" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand completions" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand completions" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand completions" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand version" -l app -d 'Use a specific registered app (overrides default)' -r
complete -c xr -n "__fish_xr_using_subcommand version" -l output -d 'Output format: text (default), json (machine-readable), jsonl (streaming)' -r -f -a "text\t'Default: colored, human-readable'
json\t'Machine-readable JSON, no color'
jsonl\t'JSON Lines (useful for streaming)'"
complete -c xr -n "__fish_xr_using_subcommand version" -l timeout -d 'Request timeout in seconds' -r
complete -c xr -n "__fish_xr_using_subcommand version" -s q -l quiet -d 'Suppress all non-essential output (errors still go to stderr)'
complete -c xr -n "__fish_xr_using_subcommand version" -l no-interactive -d 'Disable interactive prompts; fail with error instead'
complete -c xr -n "__fish_xr_using_subcommand version" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "post" -d 'Post to X'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "reply" -d 'Reply to a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "quote" -d 'Quote a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "delete" -d 'Delete a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "read" -d 'Read a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "search" -d 'Search recent posts'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "whoami" -d 'Show the authenticated user\'s profile'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "user" -d 'Look up a user by username'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "timeline" -d 'Show your home timeline'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "mentions" -d 'Show your recent mentions'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "like" -d 'Like a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "unlike" -d 'Unlike a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "repost" -d 'Repost a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "unrepost" -d 'Undo a repost'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "bookmark" -d 'Bookmark a post'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "unbookmark" -d 'Remove a bookmark'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "bookmarks" -d 'List your bookmarks'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "likes" -d 'List your liked posts'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "follow" -d 'Follow a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "unfollow" -d 'Unfollow a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "following" -d 'List users you follow'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "followers" -d 'List your followers'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "block" -d 'Block a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "unblock" -d 'Unblock a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "mute" -d 'Mute a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "unmute" -d 'Unmute a user'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "usage" -d 'Show API usage (tweet caps, daily breakdown)'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "dm" -d 'Send a direct message'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "dms" -d 'List recent direct messages'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "auth" -d 'Authentication management'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "media" -d 'Media upload operations'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "schema" -d 'Show JSON Schema for a command\'s response type'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "completions" -d 'Generate shell completion script'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "version" -d 'Show xurl version information'
complete -c xr -n "__fish_xr_using_subcommand help; and not __fish_seen_subcommand_from post reply quote delete read search whoami user timeline mentions like unlike repost unrepost bookmark unbookmark bookmarks likes follow unfollow following followers block unblock mute unmute usage dm dms auth media schema completions version help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "oauth2" -d 'Configure `OAuth2` authentication'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "oauth1" -d 'Configure `OAuth1` authentication'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "app" -d 'Configure app-auth (bearer token)'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "status" -d 'Show authentication status'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "clear" -d 'Clear authentication tokens'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "apps" -d 'Manage registered X API apps'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "default" -d 'Set default app and/or user'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "upload" -d 'Upload media file'
complete -c xr -n "__fish_xr_using_subcommand help; and __fish_seen_subcommand_from media" -f -a "status" -d 'Check media upload status'
