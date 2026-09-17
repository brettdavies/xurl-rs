/// Streaming endpoint detection.
///
/// Mirrors the Go `StreamingEndpoints` map for auto-detecting endpoints
/// that should use long-lived streaming connections.
use std::sync::LazyLock;

/// Set of endpoint prefixes that should be streamed.
static STREAMING_ENDPOINTS: LazyLock<std::collections::HashSet<&'static str>> =
    LazyLock::new(|| {
        [
            "/2/tweets/search/stream",
            "/2/tweets/sample/stream",
            "/2/tweets/sample10/stream",
            "/2/tweets/firehose/stream",
            "/2/tweets/firehose/stream/lang/en",
            "/2/tweets/firehose/stream/lang/ja",
            "/2/tweets/firehose/stream/lang/ko",
            "/2/tweets/firehose/stream/lang/pt",
        ]
        .into_iter()
        .collect()
    });

/// Checks if an endpoint should be streamed.
pub fn is_streaming_endpoint(endpoint: &str) -> bool {
    let path = if endpoint.to_lowercase().starts_with("http") {
        let parts: Vec<&str> = endpoint.splitn(4, '/').collect();
        if parts.len() >= 4 {
            format!("/{}", parts[3])
        } else {
            endpoint.to_string()
        }
    } else {
        endpoint.to_string()
    };

    // Remove query parameters if present
    let path = match path.find('?') {
        Some(idx) => &path[..idx],
        None => &path,
    };

    let normalized = path.trim_end_matches('/');
    STREAMING_ENDPOINTS.contains(normalized)
}
