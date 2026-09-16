/// Chunked media upload — INIT -> APPEND -> FINALIZE -> STATUS.
///
/// Mirrors the Go `MediaUploader` with three-phase upload, 4MB chunks,
/// and status polling with backoff.
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Duration;

use super::request::{ApiClient, MultipartOptions, RequestOptions, RequestTarget};
use super::response::types::{ApiResponse, MediaUploadResponse, deserialize_response};
use crate::error::{Error, Result};

/// Base path for the X API media upload endpoint family.
pub const MEDIA_ENDPOINT: &str = "/2/media/upload";

/// Target of the upload's progress events: `INFO` for each phase's status
/// line and `DEBUG` for per-chunk and per-poll progress.
pub const MEDIA_TARGET: &str = "xurl::media";

/// What a completed upload returned, phase by phase.
#[derive(Debug)]
pub struct MediaUploadOutcome {
    /// The INIT response, carrying the media id the later phases used.
    pub init: ApiResponse<MediaUploadResponse>,
    /// The FINALIZE response.
    pub finalize: ApiResponse<MediaUploadResponse>,
    /// The final STATUS response when the caller waited for processing.
    pub processing: Option<ApiResponse<MediaUploadResponse>>,
}

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
    trace: bool,
    wait_for_processing: bool,
    headers: &[String],
    client: &mut ApiClient,
) -> Result<MediaUploadOutcome> {
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| Error::Io(format!("error accessing file: {e}")))?;

    if !metadata.is_file() {
        return Err(Error::Io(format!("{file_path} is not a regular file")));
    }

    let file_size = metadata.len();

    let base_opts = RequestOptions {
        auth_type: auth_type.to_string(),
        username: username.to_string(),
        trace,
        headers: headers.to_vec(),
        ..Default::default()
    };

    // INIT
    tracing::info!(target: MEDIA_TARGET, "Initializing media upload...");

    let init_body = serde_json::json!({
        "total_bytes": file_size,
        "media_type": media_type,
        "media_category": media_category,
    });

    let mut init_opts = base_opts.clone();
    init_opts.method = "POST".to_string();
    init_opts.target = RequestTarget::Template {
        path: "/2/media/upload/initialize".to_string(),
        path_params: HashMap::new(),
        query: Vec::new(),
    };
    init_opts.data = init_body.to_string();

    let init_response: ApiResponse<MediaUploadResponse> =
        deserialize_response(client.send_request(&init_opts)?)?;
    let media_id = init_response.data.id.clone();
    if media_id.is_empty() {
        return Err(Error::Json(
            "failed to parse media ID from init response".to_string(),
        ));
    }

    // APPEND — upload in 4MB chunks
    upload_chunks(file_path, &media_id, &base_opts, file_size, client)?;

    // FINALIZE
    tracing::info!(target: MEDIA_TARGET, "Finalizing media upload...");

    let mut finalize_opts = base_opts.clone();
    finalize_opts.method = "POST".to_string();
    finalize_opts.target = RequestTarget::Template {
        path: "/2/media/upload/{id}/finalize".to_string(),
        path_params: HashMap::from([("id".to_string(), media_id.clone())]),
        query: Vec::new(),
    };
    finalize_opts.data.clear();

    let finalize_response: ApiResponse<MediaUploadResponse> =
        deserialize_response(client.send_request(&finalize_opts)?)?;

    let processing = if wait_for_processing && media_category.contains("video") {
        tracing::info!(target: MEDIA_TARGET, "Waiting for media processing to complete...");
        Some(wait_for_media_processing(&media_id, &base_opts, client)?)
    } else {
        None
    };

    tracing::info!(target: MEDIA_TARGET, "Media uploaded successfully! Media ID: {media_id}");
    Ok(MediaUploadOutcome {
        init: init_response,
        finalize: finalize_response,
        processing,
    })
}

/// Uploads file data in 4 MB chunks via APPEND requests.
fn upload_chunks(
    file_path: &str,
    media_id: &str,
    base_opts: &RequestOptions,
    file_size: u64,
    client: &mut ApiClient,
) -> Result<()> {
    tracing::info!(target: MEDIA_TARGET, "Uploading media in chunks...");

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

        let file_name = Path::new(file_path)
            .file_name()
            .map_or_else(|| "file".to_string(), |n| n.to_string_lossy().to_string());

        let mut form_fields = HashMap::new();
        form_fields.insert("segment_index".to_string(), segment_index.to_string());

        let multipart_opts = MultipartOptions {
            request: RequestOptions {
                method: "POST".to_string(),
                target: RequestTarget::Template {
                    path: "/2/media/upload/{id}/append".to_string(),
                    path_params: HashMap::from([("id".to_string(), media_id.to_string())]),
                    query: Vec::new(),
                },
                headers: base_opts.headers.clone(),
                auth_type: base_opts.auth_type.clone(),
                username: base_opts.username.clone(),
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

        #[allow(clippy::cast_precision_loss)]
        let pct = (bytes_uploaded as f64 / file_size as f64) * 100.0;
        tracing::debug!(
            target: MEDIA_TARGET,
            "Uploaded {bytes_uploaded} of {file_size} bytes ({pct:.2}%)"
        );
    }

    tracing::info!(target: MEDIA_TARGET, "Upload complete!");
    Ok(())
}

/// Checks or waits for media upload status.
///
/// # Errors
///
/// Returns an error if the status request fails or processing times out.
pub fn execute_media_status(
    media_id: &str,
    auth_type: &str,
    username: &str,
    wait: bool,
    trace: bool,
    headers: &[String],
    client: &mut ApiClient,
) -> Result<ApiResponse<MediaUploadResponse>> {
    let base_opts = RequestOptions {
        auth_type: auth_type.to_string(),
        username: username.to_string(),
        trace,
        headers: headers.to_vec(),
        ..Default::default()
    };

    if wait {
        wait_for_media_processing(media_id, &base_opts, client)
    } else {
        check_media_status(media_id, &base_opts, client)
    }
}

/// Checks media upload status.
fn check_media_status(
    media_id: &str,
    base_opts: &RequestOptions,
    client: &mut ApiClient,
) -> Result<ApiResponse<MediaUploadResponse>> {
    let mut opts = base_opts.clone();
    opts.method = "GET".to_string();
    opts.target = RequestTarget::Template {
        path: "/2/media/upload".to_string(),
        path_params: HashMap::new(),
        query: vec![
            ("command".to_string(), "STATUS".to_string()),
            ("media_id".to_string(), media_id.to_string()),
        ],
    };
    opts.data.clear();

    deserialize_response(client.send_request(&opts)?)
}

/// Polls media processing status until completion.
fn wait_for_media_processing(
    media_id: &str,
    base_opts: &RequestOptions,
    client: &mut ApiClient,
) -> Result<ApiResponse<MediaUploadResponse>> {
    loop {
        let response = check_media_status(media_id, base_opts, client)?;

        let state = response
            .data
            .processing_info
            .as_ref()
            .map_or("", |p| p.state.as_str());

        if state == "succeeded" {
            tracing::info!(target: MEDIA_TARGET, "Media processing complete!");
            return Ok(response);
        } else if state == "failed" {
            return Err(Error::validation("media processing failed"));
        }

        let check_after = response
            .data
            .processing_info
            .as_ref()
            .and_then(|p| p.check_after_secs)
            .unwrap_or(1)
            .max(1);

        let pct = response
            .data
            .processing_info
            .as_ref()
            .and_then(|p| p.progress_percent)
            .unwrap_or(0);
        tracing::debug!(
            target: MEDIA_TARGET,
            "Media processing in progress ({pct}%), checking again in {check_after} seconds..."
        );

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
    // Raw mode is the only caller — its target is a `RawUrl` carrying
    // the user-supplied URL with the media_id embedded in the path.
    // Template targets reach this function only via misuse; their `{id}`
    // segment would silently propagate as the media_id, so we reject
    // explicitly with the path template named in the error.
    let url_for_id = match &options.target {
        RequestTarget::RawUrl(u) => u.clone(),
        RequestTarget::Template { path, .. } => {
            return Err(Error::validation(format!(
                "handle_media_append_request requires a RawUrl target; got Template {{ path: {path:?} }} — call this only from the raw-mode path"
            )));
        }
    };
    let media_id = extract_media_id(&url_for_id);
    if media_id.is_empty() {
        return Err(Error::validation(
            "media_id is required for append endpoint",
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
