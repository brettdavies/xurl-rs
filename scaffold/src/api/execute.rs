use serde_json;
/// ExecuteRequest handles the execution of a regular API request
pub fn execute_request(options: RequestOptions, client: Client) -> anyhow::Result<()> {
    let (response, client_err) = client.send_request(options);
    if client_err.is_some() {
        Ok(handle_request_error(client_err))
    }
    Ok(utils.format_and_print_response(response))
}
/// ExecuteStreamRequest handles the execution of a streaming API request
pub fn execute_stream_request(
    options: RequestOptions,
    client: Client,
) -> anyhow::Result<()> {
    let mut client_err = client.stream_request(options);
    if client_err.is_some() {
        Ok(handle_request_error(client_err))
    }
    Ok(())
}
/// handleRequestError processes API client errors in a consistent way
fn handle_request_error(client_err: anyhow::Error) -> anyhow::Result<()> {
    let mut raw_json: serde_json::Value = Default::default();
    serde_json::from_str(&client_err.error().as_bytes().to_vec());
    utils.format_and_print_response(raw_json);
    Ok(anyhow::anyhow!("request failed"))
}
/// HandleRequest determines the type of request and executes it accordingly
pub fn handle_request(
    options: RequestOptions,
    force_stream: bool,
    media_file: &str,
    client: Client,
) -> anyhow::Result<()> {
    if is_media_append_request(options.endpoint, media_file) {
        let mut response = handle_media_append_request(options, media_file, client)?;
        Ok(utils.format_and_print_response(response))
    }
    let mut should_stream = force_stream || is_streaming_endpoint(options.endpoint);
    if should_stream {
        Ok(execute_stream_request(options, client))
    } else {
        Ok(execute_request(options, client))
    }
}
