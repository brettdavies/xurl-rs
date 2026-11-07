//! The text of the token endpoint's answer, made fit to print.
//!
//! A proxy's block page or a debug endpoint can repeat the request it refused,
//! and what a token request carries is a credential: the authorization code and
//! its verifier, the refresh token, the client secret, and the `Authorization`
//! header that holds the secret. [`Sent`] masks them wherever the answer's text
//! reaches an error. A page of tens of kilobytes would bury the rest of the
//! output, so the text is cut to a kilobyte and says how much it left out.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

const MASK: &str = "<redacted>";

/// A value shorter than this would mask ordinary words, and no credential an
/// authorization server issues is this short.
const SHORTEST_CREDENTIAL: usize = 4;

/// The longest answer text an error keeps, in bytes.
const LONGEST_ANSWER: usize = 1_024;

/// The credentials a token request carried.
pub(crate) struct Sent(Vec<String>);

impl Sent {
    /// The client's secret and the `Authorization: Basic` value that encodes
    /// it, with `values`: the others the request's form named.
    pub(crate) fn new(client_id: &str, client_secret: &str, values: &[&str]) -> Self {
        let basic = BASE64_STANDARD.encode(format!("{client_id}:{client_secret}"));
        Self(
            values
                .iter()
                .copied()
                .chain([client_secret])
                .map(str::to_string)
                .chain([basic])
                .collect(),
        )
    }

    /// `text` with every credential masked, in the spelling it was sent and as
    /// a form encodes it, then cut to the length an error keeps.
    pub(crate) fn fit_to_print(&self, text: &str) -> String {
        let mut masked = text.to_string();
        for credential in self.0.iter().filter(|c| c.len() >= SHORTEST_CREDENTIAL) {
            let form: String =
                url::form_urlencoded::byte_serialize(credential.as_bytes()).collect();
            masked = masked
                .replace(credential.as_str(), MASK)
                .replace(&form, MASK);
        }
        clip(masked)
    }
}

/// The start of `text`, and the number of bytes left out.
fn clip(text: String) -> String {
    if text.len() <= LONGEST_ANSWER {
        return text;
    }
    let mut end = LONGEST_ANSWER;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{} [{} more bytes]", &text[..end], text.len() - end)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sent() -> Sent {
        Sent::new("the-client", "s3cr&t value", &["the-code", "a verifier"])
    }

    #[test]
    fn masks_each_credential_as_sent_and_as_a_form_encodes_it() {
        let echoed = "code=the-code&verifier=a+verifier&secret=s3cr%26t+value&raw=a verifier";

        let fit = sent().fit_to_print(echoed);

        assert_eq!(
            fit,
            "code=<redacted>&verifier=<redacted>&secret=<redacted>&raw=<redacted>"
        );
    }

    #[test]
    fn masks_the_basic_authorization_value() {
        let basic = BASE64_STANDARD.encode("the-client:s3cr&t value");

        let fit = sent().fit_to_print(&format!("Authorization: Basic {basic}"));

        assert_eq!(fit, "Authorization: Basic <redacted>");
    }

    #[test]
    fn leaves_a_value_too_short_to_be_a_credential_alone() {
        let short = Sent::new("id", "", &["abc"]);

        assert_eq!(short.fit_to_print("abc and abc"), "abc and abc");
    }

    #[test]
    fn keeps_a_short_answer_whole_and_cuts_a_long_one_with_a_count() {
        assert_eq!(
            sent().fit_to_print("<html>Forbidden</html>"),
            "<html>Forbidden</html>"
        );

        let cut = sent().fit_to_print(&"x".repeat(5_000));

        assert!(cut.starts_with(&"x".repeat(LONGEST_ANSWER)));
        assert!(
            cut.ends_with(" [3976 more bytes]"),
            "{}",
            &cut[LONGEST_ANSWER..]
        );
    }

    #[test]
    fn cuts_on_a_character_boundary() {
        let text = format!("{}é{}", "a".repeat(LONGEST_ANSWER - 1), "b".repeat(100));

        let cut = sent().fit_to_print(&text);

        assert!(cut.starts_with(&"a".repeat(LONGEST_ANSWER - 1)));
        assert!(cut.contains("more bytes]"));
    }

    #[test]
    fn masks_a_credential_before_it_cuts_the_text() {
        let text = format!("{}the-code", "x".repeat(LONGEST_ANSWER - 4));

        let cut = sent().fit_to_print(&text);

        assert!(!cut.contains("the-"), "{cut}");
    }
}
