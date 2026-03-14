/// StreamingEndpoints is a map of endpoint prefixes that should be streamed
pub fn streaming_endpoints_init() -> std::collections::HashMap<String, bool> {
    todo!("package-level var init")
}
/// IsStreamingEndpoint checks if an endpoint should be streamed
pub fn is_streaming_endpoint(endpoint: &str) -> bool {
    let mut path = endpoint;
    if endpoint.to_lowercase().starts_with("http") {
        let mut parsed_url = strings.split_n(endpoint, "/", 4);
        if parsed_url.len() >= 4 {
            path = "/" + parsed_url[3];
        }
    }
    let mut query_index = path.find("?");
    if query_index != -1 {
        path = path[..query_index];
    }
    let mut normalized_endpoint = path.strip_suffix("/").unwrap_or(path);
    streaming_endpoints[normalized_endpoint]
}
