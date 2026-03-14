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
use super::response::format_and_print_response;
use crate::error::{Result, XurlError};

const MEDIA_ENDPOINT: &str = "/2/media/upload";

/// Handles the full media upload lifecycle.
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
    if verbose {
        eprintln!("\x1b[32mInitializing media upload...\x1b[0m");
    }

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
        format_and_print_response(&init_response);
    }

    // APPEND — upload in 4MB chunks
    if verbose {
        eprintln!("\x1b[32mUploading media in chunks...\x1b[0m");
    }

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
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());

        let mut form_fields = HashMap::new();
        form_fields.insert("segment_index".to_string(), segment_index.to_string());

        let multipart_opts = MultipartOptions {
            request: RequestOptions {
                method: "POST".to_string(),
                endpoint: append_url,
                headers: headers.to_vec(),
                auth_type: auth_type.to_string(),
                username: username.to_string(),
                verbose,
                trace,
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
            let pct = (bytes_uploaded as f64 / file_size as f64) * 100.0;
            eprintln!(
                "\x1b[33mUploaded {bytes_uploaded} of {file_size} bytes ({pct:.2}%)\x1b[0m"
            );
        }
    }

    if verbose {
        eprintln!("\x1b[32mUpload complete!\x1b[0m");
    }

    // FINALIZE
    if verbose {
        eprintln!("\x1b[32mFinalizing media upload...\x1b[0m");
    }

    let mut finalize_opts = base_opts.clone();
    finalize_opts.method = "POST".to_string();
    finalize_opts.endpoint = format!("{MEDIA_ENDPOINT}/{media_id}/finalize");
    finalize_opts.data.clear();

    let finalize_response = client.send_request(&finalize_opts)?;
    format_and_print_response(&finalize_response);

    // Wait for processing if requested
    if wait_for_processing && media_category.contains("video") {
        if verbose {
            eprintln!("\x1b[32mWaiting for media processing to complete...\x1b[0m");
        }

        let processing_response =
            wait_for_media_processing(&media_id, &base_opts, verbose, client)?;
        format_and_print_response(&processing_response);
    }

    println!("\x1b[32mMedia uploaded successfully! Media ID: {media_id}\x1b[0m");
    Ok(())
}

/// Checks or waits for media upload status.
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
        let response = wait_for_media_processing(media_id, &base_opts, verbose, client)?;
        let pretty = serde_json::to_string_pretty(&response)?;
        println!("{pretty}");
    } else {
        let response = check_media_status(media_id, &base_opts, client)?;
        let pretty = serde_json::to_string_pretty(&response)?;
        println!("{pretty}");
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
) -> Result<serde_json::Value> {
    loop {
        let response = check_media_status(media_id, base_opts, client)?;

        let state = response["data"]["processing_info"]["state"]
            .as_str()
            .unwrap_or("");

        if state == "succeeded" {
            if verbose {
                eprintln!("\x1b[32mMedia processing complete!\x1b[0m");
            }
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
            eprintln!(
                "\x1b[33mMedia processing in progress ({pct}%), checking again in {check_after} seconds...\x1b[0m"
            );
        }

        thread::sleep(Duration::from_secs(check_after));
    }
}

/// Handles a media append request with a file (raw mode).
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
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());

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

/// Extracts media_id from a URL.
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

/// Extracts segment_index from a JSON data string.
pub fn extract_segment_index(data: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    json.get("segment_index")
        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| Some(v.to_string())))
}

/// Checks if the request is a media append request.
pub fn is_media_append_request(url: &str, media_file: &str) -> bool {
    url.contains("/2/media/upload") && url.contains("append") && !media_file.is_empty()
}
