//! Streaming endpoint detection.
//!
//! The streaming set is every path whose operation the vendored spec marks
//! `x-twitter-streaming`, emitted by `build.rs`.

mod generated {
    include!(concat!(env!("OUT_DIR"), "/streaming.rs"));
}

/// Whether `endpoint`, a path or an absolute URL, is one the vendored spec
/// marks as streaming. A query string and a trailing slash are ignored.
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
    generated::STREAMING_PATHS.contains(&normalized)
}
