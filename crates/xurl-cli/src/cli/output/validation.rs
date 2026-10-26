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

/// The sentence for the validator code `code`; `None` for a message that is
/// not one, which prints as it is.
pub(crate) fn sentence(code: &str) -> Option<String> {
    let text = match code {
        "empty-body" => "The text is empty; give at least one character.".to_string(),
        "too-many-attachments" => format!("A post takes at most {POST_MEDIA_MAX} media ids."),
        "empty-username" => "The username is empty.".to_string(),
        "empty-post-id" => "The post id is empty; give an id or a post URL.".to_string(),
        "too-many-ids" => format!("A lookup takes at most {LOOKUP_IDS_MAX} ids."),
        "invalid-time-window" => "A time is RFC 3339 with seconds and a zone, such as \
             2026-11-02T15:00:00Z, and the start comes before the end."
            .to_string(),
        "invalid-user-query" => format!(
            "A user search is 1 to {USER_SEARCH_QUERY_MAX_CHARS} letters, digits, underscores, \
             apostrophes, or spaces."
        ),
        "invalid-media-id" => "A media id is 1 to 19 digits.".to_string(),
        "invalid-webhook-id" => "A webhook id is 1 to 19 digits.".to_string(),
        "invalid-user-id" => "A user id is 1 to 19 digits; a handle is not one.".to_string(),
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
        _ => return None,
    };
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::sentence;

    /// Every code a library validator returns has a sentence. The codes are
    /// read from the library's sources, so a new validator code without one
    /// fails here.
    #[test]
    fn every_validator_code_has_a_sentence() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../xdk/src");
        let code = regex::Regex::new(r#"Err\("([a-z]+(?:-[a-z0-9]+)+)"\)"#).unwrap();
        let mut found = Vec::new();
        let mut dirs = vec![root];
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
        let missing: Vec<&String> = found.iter().filter(|c| sentence(c).is_none()).collect();
        assert!(
            missing.is_empty(),
            "validator codes with no sentence in crates/xurl-cli/src/cli/output/validation.rs: {missing:?}"
        );
    }

    #[test]
    fn a_message_that_is_not_a_code_has_no_sentence() {
        assert_eq!(sentence("--forward takes an http or https URL"), None);
        assert_eq!(sentence("some-future-code"), None);
    }
}
