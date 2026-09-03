//! Helpers shared by the source-scanning guard tests.

/// Returns the name of the `fn` a given byte offset falls inside.
pub fn enclosing_test(source: &str, offset: usize) -> String {
    source[..offset]
        .rmatch_indices("fn ")
        .find_map(|(i, _)| {
            let rest = &source[i + 3..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            (!name.is_empty()).then_some(name)
        })
        .unwrap_or_else(|| "<unknown>".to_string())
}
