//! The sentence text mode prints for an input a validator refused.
//!
//! A validator in the library returns a kebab-case code, the fact an agent
//! branches on, and structured output carries it as it is. A person reading
//! text output gets the rule the input broke instead, with the limits read
//! from the same constants the validators check.

use xdk::api::shortcuts::{
    ALT_TEXT_MAX_CHARS, BOOKMARK_FOLDER_NAME_MAX_CHARS, BROADCAST_CHAT_MAX_CHARS,
    CALL_DESCRIPTION_MAX_CHARS, CALL_DURATION_MAX_MINUTES, CALL_RECURRENCE_MAX_CHARS,
    CALL_TIME_ZONE_MAX_CHARS, CALL_TITLE_MAX_CHARS, CHANGED_CALL_INVITEES_MAX,
    DM_GROUP_PARTICIPANTS_MAX, DM_GROUP_PARTICIPANTS_MIN, LIST_DESCRIPTION_MAX_CHARS,
    LIST_NAME_MAX_CHARS, LOOKUP_IDS_MAX, NEW_CALL_INVITEES_MAX, POST_MEDIA_MAX,
    USER_SEARCH_QUERY_MAX_CHARS, WEBHOOK_URL_MAX_CHARS,
};

/// The rule `validate_user_search_query` enforces, as text output prints it
/// for `invalid-user-query` and `xr search-users --help` prints it on
/// `<QUERY>`.
pub(crate) fn user_search_rule() -> String {
    format!(
        "A user search is 1 to {USER_SEARCH_QUERY_MAX_CHARS} letters, digits, underscores, \
         apostrophes, or spaces."
    )
}

/// The sentence for an input that broke `rule`, naming `value`, the
/// offending input, where the rule carries one; `None` for a rule this
/// build has no sentence for, which prints as the library words it.
pub(crate) fn sentence(rule: &str, value: Option<&str>) -> Option<String> {
    let given = value.unwrap_or("the value given");
    let text = match rule {
        "empty-body" => "The text is empty; give at least one character.".to_string(),
        "too-many-attachments" => format!("A post takes at most {POST_MEDIA_MAX} media ids."),
        "empty-username" => "The username is empty.".to_string(),
        "empty-post-id" => "The post id is empty; give an id or a post URL.".to_string(),
        "mixed-ids-and-handles" => {
            "Give every account as a handle or every one as an id; X looks up each form \
             separately."
                .to_string()
        }
        "invalid-username" => username_rule(),
        "too-many-ids" => format!("A lookup takes at most {LOOKUP_IDS_MAX} ids."),
        "invalid-time-window" => "A time is RFC 3339 with seconds and a zone, such as \
             2026-11-02T15:00:00Z, and the start comes before the end."
            .to_string(),
        "invalid-user-query" => user_search_rule(),
        "invalid-media-key" => {
            "A media key is two numbers joined by an underscore, such as 3_1146654567674912769."
                .to_string()
        }
        "invalid-media-id" => "A media id is 1 to 19 digits.".to_string(),
        "invalid-space-id" => "A Space id is 1 to 13 letters or digits.".to_string(),
        "invalid-rule-id" => "A filtered-stream rule id is 1 to 19 digits.".to_string(),
        "invalid-webhook-id" => "A webhook id is 1 to 19 digits.".to_string(),
        "invalid-user-id" => format!("{}; a handle is not one.", user_id_rule()),
        "invalid-folder-id" => "A bookmark folder id is 1 to 19 digits.".to_string(),
        "invalid-news-id" => "A news story id is 1 to 19 digits.".to_string(),
        "invalid-community-id" => "A community id is 1 to 19 digits.".to_string(),
        "empty-article-title" => "The article title is empty.".to_string(),
        "empty-article-body" => "The article body holds no text.".to_string(),
        "invalid-article-id" => "An article id is 1 to 19 digits.".to_string(),
        "invalid-note-id" => "A Community Note id is 1 to 19 digits.".to_string(),
        "note-needs-source-url" => {
            "A Community Note's text carries a source URL, http:// or https://.".to_string()
        }
        "invalid-broadcast-id" => "A broadcast id is 1 to 24 letters and digits.".to_string(),
        "invalid-scheduled-broadcast-id" => {
            "A scheduled broadcast id is 1 to 19 digits.".to_string()
        }
        "invalid-chat-message-id" => "A chat message id is 1 to 19 digits.".to_string(),
        "chat-message-too-long" => {
            format!("A chat message is at most {BROADCAST_CHAT_MAX_CHARS} characters.")
        }
        "invalid-list-id" => "A list id is 1 to 19 digits.".to_string(),
        "empty-list-name" => "The list name is empty.".to_string(),
        "list-name-too-long" => {
            format!("A list name is at most {LIST_NAME_MAX_CHARS} characters.")
        }
        "list-description-too-long" => {
            format!("A list description is at most {LIST_DESCRIPTION_MAX_CHARS} characters.")
        }
        "no-list-changes" => {
            "Nothing would change; give --name, --description, --private, or --public.".to_string()
        }
        "invalid-conversation-id" => "A conversation id is two user ids joined by a hyphen, \
             such as 123-456, or a group's 15 to 19 digits."
            .to_string(),
        "invalid-dm-event-id" => "A direct message event id is 1 to 19 digits.".to_string(),
        "invalid-participant-count" => format!(
            "A group conversation starts with {DM_GROUP_PARTICIPANTS_MIN} to \
             {DM_GROUP_PARTICIPANTS_MAX} people besides you."
        ),
        "empty-webhook-url" => "The webhook URL is empty.".to_string(),
        "webhook-url-too-long" => {
            format!("The webhook URL is longer than {WEBHOOK_URL_MAX_CHARS} characters.")
        }
        "webhook-url-not-https" => "The webhook URL is not an https URL.".to_string(),
        "webhook-url-has-port" => {
            "The webhook URL names a port; X takes one without a port.".to_string()
        }
        "invalid-replay-time" => "A replay time is 12 digits, yyyymmddhhmm in UTC.".to_string(),
        "empty-source-name" => "The source name is empty.".to_string(),
        "empty-source-region" => "The source region is empty.".to_string(),
        "empty-folder-name" => "The folder name is empty.".to_string(),
        "folder-name-too-long" => format!(
            "A bookmark folder name is at most {BOOKMARK_FOLDER_NAME_MAX_CHARS} characters."
        ),
        "invalid-call-time" => {
            "A call time is RFC 3339 with seconds and a zone, such as 2026-11-02T15:00:00Z."
                .to_string()
        }
        "invalid-call-duration" => {
            format!("A call lasts 1 to {CALL_DURATION_MAX_MINUTES} minutes.")
        }
        "call-title-too-long" => {
            format!("A call title is at most {CALL_TITLE_MAX_CHARS} characters.")
        }
        "call-description-too-long" => {
            format!("A call description is at most {CALL_DESCRIPTION_MAX_CHARS} characters.")
        }
        "call-recurrence-too-long" => {
            format!("A recurrence rule is at most {CALL_RECURRENCE_MAX_CHARS} characters.")
        }
        "call-time-zone-too-long" => {
            format!("A time zone is at most {CALL_TIME_ZONE_MAX_CHARS} characters.")
        }
        "empty-call-recurrence" => "The recurrence rule is empty.".to_string(),
        "no-call-changes" => "Nothing would change; give at least one field to update.".to_string(),
        "too-many-invitees" => format!(
            "A call takes at most {NEW_CALL_INVITEES_MAX} invitees when it is scheduled, and \
             {CHANGED_CALL_INVITEES_MAX} in one change."
        ),
        "empty-alt-text" => "The alt text is empty.".to_string(),
        "alt-text-too-long" => format!("Alt text is at most {ALT_TEXT_MAX_CHARS} characters."),
        "invalid-language-code" => "A language code is two letters, such as en.".to_string(),
        "forward-url-invalid" => "The --forward URL is not an absolute URL.".to_string(),
        "forward-url-not-http" => "--forward takes an http or https URL.".to_string(),
        "forward-url-has-credentials" => "The --forward URL carries a username or password; \
             pass the credential with -H instead."
            .to_string(),
        "missing-media-id" => {
            "The media upload URL names no media id; put the id INIT returned in its path."
                .to_string()
        }
        "invalid-media-path" => format!("The media path {given} is not valid UTF-8."),
        "unknown-media-type" => format!(
            "The media type of {given} cannot be read from its extension; name it with \
             --media-type."
        ),
        "unknown-scope" => format!(
            "{given} is not an OAuth2 scope X defines. The scopes are: {}.",
            xdk::auth::oauth2::get_oauth2_scopes().join(", ")
        ),
        "invalid-redirect-uri" => format!(
            "The redirect URI {given} is not an https URL, or an http URL on loopback \
             (localhost, 127.0.0.1, or [::1])."
        ),
        "invalid-app-name" => format!(
            "An app name is one or more letters, digits, '_', '.', or '-'; {given} is not one."
        ),
        "checkpoint-in-the-future" => {
            format!("The checkpoint's time is {given} seconds later than the clock's.")
        }
        "header-not-name-value" => {
            format!("-H header {given} is not `Name: Value`: it has no colon.")
        }
        "empty-header-name" => format!("-H header {given} has an empty name."),
        "invalid-header-name" => format!("-H header {given} has a name HTTP does not allow."),
        "reserved-header" => format!(
            "-H header {given} sets Host, Content-Length, or Idempotency-Key, which a forward sets \
             itself."
        ),
        "invalid-header-value" => format!("-H header {given} has a value HTTP does not allow."),
        "no-token-store" => "This client was built from credentials, not a token store, so it \
             has no store to read or write."
            .to_string(),
        "no-credential" => "No credential was given: set a bearer token, an OAuth2 credential, \
             or OAuth1 credentials before building the client."
            .to_string(),
        "invalid-request-path" => format!("{given} is not a request path."),
        "nothing-to-clear" => "Nothing to clear: pass --all, --oauth1, --oauth2-username <NAME>, \
             or --bearer."
            .to_string(),
        "no-app-changes" => "Nothing to update: pass --client-id, --client-secret, \
             --client-secret-file, or --redirect-uri."
            .to_string(),
        "no-default-app" => "There is no default app set; name the app.".to_string(),
        "missing-program" => {
            "-- names no program; put the program and its arguments after --.".to_string()
        }
        "program-not-found" => format!(
            "The program after --, {given}, is not a path to a file, and no directory on PATH \
             holds it."
        ),
        "unsupported-pagination" => "X API does not support offset-style pagination; pass \
             --cursor <token> from the previous response's meta.next_token instead."
            .to_string(),
        "no-signing-secret" => "There is no secret to verify X's signatures by: give the app's \
             OAuth2 client secret or OAuth1 consumer secret."
            .to_string(),
        "no-typed-response" => {
            let beneath = crate::cli::commands::schema::commands_beneath(given);
            if beneath.is_empty() {
                format!("schema not available for '{given}': it has no typed response.")
            } else {
                format!(
                    "schema not available for '{given}': it has no typed response; these \
                     commands beneath it have one: {}.",
                    beneath.join(", ")
                )
            }
        }
        "missing-command" => "usage: xr schema <COMMAND>... | xr schema --envelope | xr schema \
             --list | xr schema --all"
            .to_string(),
        _ => return None,
    };
    Some(text)
}

/// The rule `validate_username` enforces, as text output prints it for
/// `invalid-username`.
fn username_rule() -> String {
    "A username is 1 to 15 letters, digits, or underscores, with or without a leading @."
        .to_string()
}

/// The rule a numeric user id follows; text output prints it, with the note
/// that a handle is not one, for `invalid-user-id`.
fn user_id_rule() -> String {
    "A user id is 1 to 19 digits".to_string()
}

/// The rules for an account given as a username or a user id, as the help
/// of every argument that takes one states them.
pub(crate) fn account_rule() -> String {
    format!(
        "{} {}. An account made only of digits is a user id, so an all-digit username takes its @.",
        username_rule(),
        user_id_rule()
    )
}

#[cfg(test)]
mod tests {
    use super::sentence;

    /// Every rule a validation refusal can name has a sentence. The rules are
    /// read from both crates' sources (each validator's `Err("<rule>")` and
    /// each `validation("<rule>")` or `validation_of("<rule>", ..)`), so a new
    /// rule without a sentence fails here.
    #[test]
    fn every_validator_code_has_a_sentence() {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let code = regex::Regex::new(
            r#"(?:Err|validation|validation_of|refuse)\("([a-z]+(?:-[a-z0-9]+)+)""#,
        )
        .unwrap();
        let mut found = Vec::new();
        let mut dirs = vec![manifest.join("../xdk/src"), manifest.join("src")];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(&dir).expect("read the library sources") {
                let path = entry.expect("an entry").path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    let text = std::fs::read_to_string(&path).expect("read a source");
                    found.extend(code.captures_iter(&text).map(|c| c[1].to_string()));
                }
            }
        }
        found.sort();
        found.dedup();
        assert!(found.len() > 25, "the validator codes: {found:?}");
        let missing: Vec<&String> = found
            .iter()
            .filter(|c| sentence(c, Some("VALUE")).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "validator codes with no sentence in crates/xurl-cli/src/cli/output/validation.rs: {missing:?}"
        );
    }

    /// The sentence for an unknown scope lists the scopes X defines, as
    /// `xr auth oauth2 --help` promises.
    #[test]
    fn an_unknown_scope_sentence_lists_the_valid_scopes() {
        let text = sentence("unknown-scope", Some("tweet.reed")).expect("a sentence");
        assert!(text.contains("tweet.reed"), "{text}");
        for scope in xdk::auth::oauth2::get_oauth2_scopes() {
            assert!(text.contains(scope), "{scope} is listed: {text}");
        }
    }

    #[test]
    fn a_message_that_is_not_a_code_has_no_sentence() {
        assert_eq!(sentence("--forward takes an http or https URL", None), None);
        assert_eq!(sentence("some-future-code", None), None);
    }
}
