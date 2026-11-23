//! A URL cut at its authority, read the way a WHATWG URL parser reads a
//! special scheme.

/// The schemes the WHATWG URL standard calls special.
const SPECIAL_SCHEMES: [&str; 6] = ["http", "https", "ws", "wss", "ftp", "file"];

/// A URL cut around its authority; `head + authority + tail` is the URL
/// after the parser's preprocessing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlParts {
    /// The scheme, its colon, and any slashes before the authority; empty
    /// when the text names no scheme the split recognizes.
    pub head: String,
    /// The userinfo, host, and port as written.
    pub authority: String,
    /// The path, query, and fragment.
    pub tail: String,
}

/// Cuts `input` around its authority.
///
/// For a special scheme this follows the standard's own steps: strip leading
/// and trailing C0 control or space, remove every ASCII tab and newline, read
/// `\` as `/`, skip every slash after the scheme (there may be none), and end
/// the authority at the first `/`, `\`, `?`, or `#`. A non-special scheme
/// followed by `//` ends its authority at `/`, `?`, or `#` only, so a `\`
/// stays inside it. Text with no scheme the split recognizes is all
/// authority up to its query or fragment, so a caller that drops the
/// userinfo drops every `@`-delimited part it could be.
#[must_use]
pub fn split_special_url(input: &str) -> UrlParts {
    let cleaned: String = input
        .trim_matches(|c: char| c <= ' ')
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();

    if let Some(colon) = scheme_end(&cleaned) {
        let special = SPECIAL_SCHEMES
            .iter()
            .any(|scheme| scheme.eq_ignore_ascii_case(&cleaned[..colon]));
        let after = &cleaned[colon + 1..];
        let slashes = if special {
            after.len() - after.trim_start_matches(['/', '\\']).len()
        } else if after.starts_with("//") {
            2
        } else {
            return unrecognized(&cleaned);
        };
        let terminators: &[char] = if special {
            &['/', '\\', '?', '#']
        } else {
            &['/', '?', '#']
        };
        let (head, rest) = cleaned.split_at(colon + 1 + slashes);
        let end = rest.find(terminators).unwrap_or(rest.len());
        let (authority, tail) = rest.split_at(end);
        return UrlParts {
            head: head.to_string(),
            authority: authority.to_string(),
            tail: tail.to_string(),
        };
    }
    unrecognized(&cleaned)
}

/// The index of the colon that ends a scheme at the start of `text`.
fn scheme_end(text: &str) -> Option<usize> {
    let mut chars = text.char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    for (index, c) in chars {
        match c {
            ':' => return Some(index),
            c if c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.') => {}
            _ => return None,
        }
    }
    None
}

fn unrecognized(cleaned: &str) -> UrlParts {
    let (authority, tail) = cleaned.split_at(cleaned.find(['?', '#']).unwrap_or(cleaned.len()));
    UrlParts {
        head: String::new(),
        authority: authority.to_string(),
        tail: tail.to_string(),
    }
}
