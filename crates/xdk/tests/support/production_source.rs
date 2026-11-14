//! The part of a Rust source file that ships, for a guard that reads source as
//! text and has to leave the file's unit tests out. The tests of both crates
//! include this file with `#[path]`, so the rule has one copy.

use regex::Regex;

/// `source` up to the `#[cfg(test)]` line that opens an inline test module,
/// or all of it when none does. A `#[cfg(test)]` over `mod name;`, a `use`, a
/// `const`, or a function leaves the rest of the file in view, since code
/// after it still ships. The cut falls on a line boundary, so line numbers in
/// the result are the file's own.
pub fn up_to_inline_tests(source: &str) -> &str {
    let inline_module = Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{")
        .expect("the inline-module pattern is valid");
    let mut offset = 0;
    let mut lines = source.split_inclusive('\n').peekable();
    while let Some(line) = lines.next() {
        let opens_tests = line.trim_start().starts_with("#[cfg(test)]")
            && lines
                .peek()
                .is_some_and(|next| inline_module.is_match(next));
        if opens_tests {
            return &source[..offset];
        }
        offset += line.len();
    }
    source
}
