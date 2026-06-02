//! `xr examples` — top-level curated invocation gallery.
//!
//! Prints a grouped, copy-pasteable list of common `xr` invocations so an
//! operator or agent can discover the surface without paging through every
//! subcommand's `--help`. The same examples are advertised piecemeal via each
//! subcommand's `after_help`; this command is the single-screen overview.
//!
//! Tier 1: no config, no auth, no network. Safe to run before any setup.

use std::io::Write;

use crate::error::Result;

/// Curated examples block. Plain text, no ANSI, grouped by use case.
///
/// Sections mirror the operator's mental model: pick an auth path, then
/// post-and-read, then manage social graph, then stream, then inspect
/// schemas, then install the agent skill. Every section contains at least
/// one paired text + `--output json` invocation per the agent-native CLI
/// spec's progressive-help principle.
pub const EXAMPLES_BLOCK: &str = "\
xr examples — common invocations for the X API CLI

AUTHENTICATE:
  Interactive OAuth2 (browser-based, recommended for desktops):
    xr auth oauth2

  Headless OAuth2 (servers, containers, SSH; copy-paste URL):
    xr auth oauth2 --no-browser --step 1
    xr auth oauth2 --no-browser --step 2 --auth-url '<paste>'

  App-only bearer token (read-only endpoints + search):
    xr auth app --bearer-token \"$XURL_BEARER_TOKEN\"

  Show current auth state, machine-readable:
    xr auth status --output json

POST AND READ:
  Post a status update (text mode):
    xr post \"Shipping today.\"

  Post a status update (JSON for parsing):
    xr post \"Shipping today.\" --output json

  Reply to a post by ID:
    xr reply 1234567890 \"Congrats!\"

  Quote-post:
    xr quote 1234567890 \"Worth a read.\" --output json

  Read a single post and pipe to jaq:
    xr read 1234567890 --output json | jaq '.data.text'

  Search recent posts:
    xr search \"rustlang\" -n 25
    XURL_OUTPUT=json xr search \"rustlang\" -n 25

MANAGE SOCIAL GRAPH:
  Look up a user:
    xr user jack --output json

  Follow / unfollow:
    xr follow @jack
    xr unfollow @jack --output json

  Like / unlike, repost / unrepost, bookmark / unbookmark:
    xr like 1234567890
    xr unlike 1234567890 --output json
    xr repost 1234567890
    xr bookmark 1234567890 --output json

  Block / mute (irreversible from CLI side without confirm):
    xr block @spammer --output json
    xr mute @noisy --output json

INSPECT YOUR ACCOUNT:
  Your profile, timeline, mentions:
    xr whoami --output json
    xr timeline -n 50 --output json
    xr mentions -n 50 --output json

  Your bookmarks and liked posts:
    xr bookmarks -n 100 --output jsonl | jaq '.id'
    xr likes -n 100 --output jsonl

  Your API usage (caps + daily breakdown):
    xr usage --output json

DIRECT MESSAGES:
  Send a DM:
    xr dm @recipient \"Hello\"

  List recent DM events:
    xr dms -n 50 --output json

MEDIA UPLOAD:
  Upload an image (returns media_id):
    xr media upload ./photo.png --media-type image/png --category tweet_image

  Upload a video and wait for processing:
    xr media upload ./clip.mp4 --wait --output json

  Check upload status:
    xr media status 1234567890 --output json

RAW MODE (curl-style):
  Generic GET:
    xr /2/users/me --output json

  POST with body and explicit auth:
    xr -X POST /2/tweets -d '{\"text\":\"Hello\"}' --auth oauth2 --output json

  Stream a filtered live endpoint:
    xr /2/tweets/search/stream --auth app --output jsonl

INSPECT SCHEMAS:
  List response schemas:
    xr schema --list --output json

  Get a single schema:
    xr schema post --output json

MULTI-APP:
  Register, list, set default:
    xr auth apps add my-app --client-id ID --client-secret SECRET
    xr auth apps list --output json
    xr auth default my-app

  Per-request app override:
    xr --app my-app /2/users/me --output json

ENVIRONMENT VARIABLE PRECEDENCE:
  Env vars are equivalent to flags; flags override env when both are set.
    XURL_OUTPUT=json xr search \"x\"            # same as --output json
    XURL_QUIET=1 xr post \"...\" --output json   # JSON envelope, no human text
    XURL_VERBOSE=1 xr whoami                   # verbose request/response logs

See 'xr --help' for the full agentic-flag matrix and exit codes.
See 'xr <COMMAND> --help' for the 3-5 curated examples per command.
";

/// Runs `xr examples`. Writes the curated block to `stdout` and returns Ok.
///
/// # Errors
///
/// Returns an I/O error if writing to `stdout` fails.
pub fn run_examples(stdout: &mut dyn Write) -> Result<()> {
    stdout.write_all(EXAMPLES_BLOCK.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_block_groups_by_section() {
        for section in [
            "AUTHENTICATE:",
            "POST AND READ:",
            "MANAGE SOCIAL GRAPH:",
            "INSPECT YOUR ACCOUNT:",
            "DIRECT MESSAGES:",
            "MEDIA UPLOAD:",
            "RAW MODE",
            "INSPECT SCHEMAS:",
            "MULTI-APP:",
            "ENVIRONMENT VARIABLE PRECEDENCE:",
        ] {
            assert!(
                EXAMPLES_BLOCK.contains(section),
                "examples block missing section {section}"
            );
        }
    }

    #[test]
    fn examples_block_pairs_text_and_json() {
        // At least one text invocation followed within 5 lines by --output json.
        let lines: Vec<&str> = EXAMPLES_BLOCK.lines().collect();
        let mut paired = false;
        for (i, line) in lines.iter().enumerate() {
            if line.contains("xr ") && !line.contains("--output") && !line.contains("XURL_OUTPUT") {
                let window_end = (i + 6).min(lines.len());
                if lines[i + 1..window_end]
                    .iter()
                    .any(|l| l.contains("--output json") || l.contains("XURL_OUTPUT=json"))
                {
                    paired = true;
                    break;
                }
            }
        }
        assert!(
            paired,
            "expected at least one text invocation paired with --output json within 5 lines"
        );
    }

    #[test]
    fn run_examples_writes_block() {
        let mut buf = Vec::new();
        run_examples(&mut buf).expect("run_examples must succeed");
        let out = String::from_utf8(buf).expect("examples block is UTF-8");
        assert!(out.contains("AUTHENTICATE:"));
        assert!(out.contains("xr post"));
        assert!(out.contains("--output json"));
        assert!(out.contains("XURL_OUTPUT=json"));
    }
}
