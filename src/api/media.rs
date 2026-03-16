/// Chunked media upload — INIT -> APPEND -> FINALIZE -> STATUS.
///
/// Mirrors the Go `MediaUploader` with three-phase upload, 4MB chunks,
/// and status polling with backoff.
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Duration;

use super::request::{ApiClient, MultipartOptions, RequestOptions};
use crate::error::{Result, XurlError};
use crate::output::OutputConfig;

pub const MEDIA_ENDPOINT: &str = "/2/media/upload";

/// Handles the full media upload lifecycle.
///
/// # Errors
///
/// Returns an error if the file cannot be read, any upload phase (INIT, APPEND,
/// FINALIZE) fails, or media processing times out.
#[allow(clippy::too_many_arguments)]
pub fn execute_media_upload(
    file_path: &str,
    media_type: &str,
    media_category: &str,
    auth_type: &str,
    username: &str,
    verbose: bool,
    trace: bool,
    wait_for_processing: bool,
    headers: &[String],
    client: &mut ApiClient,
    out: &OutputConfig,
) -> Result<()> {
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| XurlError::Io(format!("error accessing file: {e}")))?;

    if !metadata.is_file() {
        return Err(XurlError::Io(format!("{file_path} is not a regular file")));
    }

    let file_size = metadata.len();

    let base_opts = RequestOptions {
        auth_type: auth_type.to_string(),
        username: username.to_string(),
        verbose,
        trace,
        headers: headers.to_vec(),
        ..Default::default()
    };

    // INIT
    out.status("Initializing media upload...");

    let init_body = serde_json::json!({
        "total_bytes": file_size,
        "media_type": media_type,
        "media_category": media_category,
    });

    let mut init_opts = base_opts.clone();
    init_opts.method = "POST".to_string();
    init_opts.endpoint = format!("{MEDIA_ENDPOINT}/initialize");
    init_opts.data = init_body.to_string();

    let init_response = client.send_request(&init_opts)?;
    let media_id = init_response["data"]["id"]
        .as_str()
        .ok_or_else(|| XurlError::Json("failed to parse media ID from init response".to_string()))?
        .to_string();

    if verbose {
        out.print_response(&init_response);
    }

    // APPEND — upload in 4MB chunks
    upload_chunks(
        file_path, &media_id, &base_opts, verbose, file_size, client, out,
    )?;

    // FINALIZE
    out.status("Finalizing media upload...");

    let mut finalize_opts = base_opts.clone();
    finalize_opts.method = "POST".to_string();
    finalize_opts.endpoint = format!("{MEDIA_ENDPOINT}/{media_id}/finalize");
    finalize_opts.data.clear();

    let finalize_response = client.send_request(&finalize_opts)?;
    out.print_response(&finalize_response);

    // Wait for processing if requested
    if wait_for_processing && media_category.contains("video") {
        out.status("Waiting for media processing to complete...");

        let processing_response =
            wait_for_media_processing(&media_id, &base_opts, verbose, client, out)?;
        out.print_response(&processing_response);
    }

    out.status(&format!(
        "Media uploaded successfully! Media ID: {media_id}"
    ));
    Ok(())
}

/// Uploads file data in 4 MB chunks via APPEND requests.
fn upload_chunks(
    file_path: &str,
    media_id: &str,
    base_opts: &RequestOptions,
    verbose: bool,
    file_size: u64,
    client: &mut ApiClient,
    out: &OutputConfig,
) -> Result<()> {
    out.status("Uploading media in chunks...");

    let mut file = std::fs::File::open(file_path)?;
    let chunk_size = 4 * 1024 * 1024;
    let mut buffer = vec![0u8; chunk_size];
    let mut segment_index = 0;
    let mut bytes_uploaded: u64 = 0;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let append_url = format!("{MEDIA_ENDPOINT}/{media_id}/append");
        let file_name = Path::new(file_path)
            .file_name()
            .map_or_else(|| "file".to_string(), |n| n.to_string_lossy().to_string());

        let mut form_fields = HashMap::new();
        form_fields.insert("segment_index".to_string(), segment_index.to_string());

        let multipart_opts = MultipartOptions {
            request: RequestOptions {
                method: "POST".to_string(),
                endpoint: append_url,
                headers: base_opts.headers.clone(),
                auth_type: base_opts.auth_type.clone(),
                username: base_opts.username.clone(),
                verbose,
                trace: base_opts.trace,
                ..Default::default()
            },
            form_fields,
            file_field: "media".to_string(),
            file_path: String::new(),
            file_name,
            file_data: buffer[..bytes_read].to_vec(),
        };

        client.send_multipart_request(&multipart_opts)?;

        bytes_uploaded += bytes_read as u64;
        segment_index += 1;

        if verbose {
            #[allow(clippy::cast_precision_loss)]
            let pct = (bytes_uploaded as f64 / file_size as f64) * 100.0;
            out.info(&format!(
                "Uploaded {bytes_uploaded} of {file_size} bytes ({pct:.2}%)"
            ));
        }
    }

    out.status("Upload complete!");
    Ok(())
}

/// Checks or waits for media upload status.
///
/// # Errors
///
/// Returns an error if the status request fails or processing times out.
#[allow(clippy::too_many_arguments)]
pub fn execute_media_status(
    media_id: &str,
    auth_type: &str,
    username: &str,
    verbose: bool,
    wait: bool,
    trace: bool,
    headers: &[String],
    client: &mut ApiClient,
    out: &OutputConfig,
) -> Result<()> {
    let base_opts = RequestOptions {
        auth_type: auth_type.to_string(),
        username: username.to_string(),
        verbose,
        trace,
        headers: headers.to_vec(),
        ..Default::default()
    };

    if wait {
        let response = wait_for_media_processing(media_id, &base_opts, verbose, client, out)?;
        out.print_response(&response);
    } else {
        let response = check_media_status(media_id, &base_opts, client)?;
        out.print_response(&response);
    }

    Ok(())
}

/// Checks media upload status.
fn check_media_status(
    media_id: &str,
    base_opts: &RequestOptions,
    client: &mut ApiClient,
) -> Result<serde_json::Value> {
    let mut opts = base_opts.clone();
    opts.method = "GET".to_string();
    opts.endpoint = format!("{MEDIA_ENDPOINT}?command=STATUS&media_id={media_id}");
    opts.data.clear();

    client.send_request(&opts)
}

/// Polls media processing status until completion.
fn wait_for_media_processing(
    media_id: &str,
    base_opts: &RequestOptions,
    verbose: bool,
    client: &mut ApiClient,
    out: &OutputConfig,
) -> Result<serde_json::Value> {
    loop {
        let response = check_media_status(media_id, base_opts, client)?;

        let state = response["data"]["processing_info"]["state"]
            .as_str()
            .unwrap_or("");

        if state == "succeeded" {
            out.status("Media processing complete!");
            return Ok(response);
        } else if state == "failed" {
            return Err(XurlError::Api("media processing failed".to_string()));
        }

        let check_after = response["data"]["processing_info"]["check_after_secs"]
            .as_u64()
            .unwrap_or(1)
            .max(1);

        if verbose {
            let pct = response["data"]["processing_info"]["progress_percent"]
                .as_u64()
                .unwrap_or(0);
            out.info(&format!(
                "Media processing in progress ({pct}%), checking again in {check_after} seconds..."
            ));
        }

        thread::sleep(Duration::from_secs(check_after));
    }
}

/// Handles a media append request with a file (raw mode).
///
/// # Errors
///
/// Returns an error if the `media_id` is missing, the file cannot be read,
/// or the multipart request fails.
pub fn handle_media_append_request(
    options: &RequestOptions,
    media_file: &str,
    client: &mut ApiClient,
) -> Result<serde_json::Value> {
    let media_id = extract_media_id(&options.endpoint);
    if media_id.is_empty() {
        return Err(XurlError::Api(
            "media_id is required for append endpoint".to_string(),
        ));
    }

    let segment_index = if options.data.is_empty() {
        "0".to_string()
    } else {
        extract_segment_index(&options.data)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "0".to_string())
    };

    let file_name = Path::new(media_file)
        .file_name()
        .map_or_else(|| "file".to_string(), |n| n.to_string_lossy().to_string());

    let mut form_fields = HashMap::new();
    form_fields.insert("segment_index".to_string(), segment_index);

    let multipart_opts = MultipartOptions {
        request: options.clone(),
        form_fields,
        file_field: "media".to_string(),
        file_path: media_file.to_string(),
        file_name,
        file_data: Vec::new(),
    };

    client.send_multipart_request(&multipart_opts)
}

/// Extracts `media_id` from a URL.
#[must_use]
pub fn extract_media_id(url: &str) -> String {
    if url.is_empty() || !url.contains("/2/media/upload") {
        return String::new();
    }

    if url.ends_with("/2/media/upload/initialize") {
        return String::new();
    }

    // Extract media ID from path for append/finalize endpoints
    if let Some(rest) = url.split("/2/media/upload/").nth(1) {
        for suffix in &["/append", "/finalize"] {
            if let Some(idx) = rest.find(suffix) {
                return rest[..idx].to_string();
            }
        }
    }

    // Try query parameters
    if let Some(query) = url.split('?').nth(1) {
        for param in query.split('&') {
            if let Some(value) = param.strip_prefix("media_id=") {
                return value.to_string();
            }
        }
    }

    String::new()
}

/// Extracts `segment_index` from a JSON data string.
#[must_use]
pub fn extract_segment_index(data: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    json.get("segment_index").and_then(|v| {
        v.as_str()
            .map(std::string::ToString::to_string)
            .or_else(|| Some(v.to_string()))
    })
}

/// Checks if the request is a media append request.
#[must_use]
pub fn is_media_append_request(url: &str, media_file: &str) -> bool {
    url.contains("/2/media/upload") && url.contains("append") && !media_file.is_empty()
}
