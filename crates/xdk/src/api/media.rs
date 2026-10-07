/// Chunked media upload — INIT -> APPEND -> FINALIZE -> STATUS.
///
/// Mirrors the Go `MediaUploader` with three-phase upload, 4MB chunks,
/// and status polling with backoff.
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

use super::auth_matrix::endpoints::{
    MEDIA_UPLOAD, MEDIA_UPLOAD_APPEND, MEDIA_UPLOAD_FINALIZE, MEDIA_UPLOAD_INITIALIZE,
    MEDIA_UPLOAD_STATUS,
};
use super::request::{Client, MultipartOptions, RequestOptions, RequestTarget};
use super::response::types::{ApiResponse, MediaUploadResponse, deserialize_response};
use crate::error::{Error, Result};

/// Base path for the X API media upload endpoint family.
pub const MEDIA_ENDPOINT: &str = MEDIA_UPLOAD.path;

/// Target of the upload's progress events: `INFO` for each phase's status
/// line and `DEBUG` for per-chunk and per-poll progress.
pub const MEDIA_TARGET: &str = "xdk::media";

/// How long a wait on media processing runs when the caller names no
/// deadline.
///
/// X sets no cap on processing time, so a wait without a deadline can poll a
/// stuck job, one paid status call a second, for as long as the caller lives.
/// Sixty seconds covers an ordinary video and returns inside an agent's
/// default tool-call budget. X keeps a media id valid for 24 hours, so a
/// longer job is resumed rather than lost: wait on the same id again, as
/// `xr media status <id> --wait=<secs>` does.
pub const DEFAULT_PROCESSING_WAIT: Duration = Duration::from_secs(60);

/// What a completed upload returned, phase by phase.
#[derive(Debug)]
pub struct MediaUploadOutcome {
    /// The INIT response, carrying the media id the later phases used.
    pub init: ApiResponse<MediaUploadResponse>,
    /// The FINALIZE response.
    pub finalize: ApiResponse<MediaUploadResponse>,
    /// The outcome of waiting for processing: `None` when the caller did not
    /// wait, `Some(Ok)` with the final STATUS response, `Some(Err)` when
    /// processing failed or timed out after the upload itself completed. The
    /// media id in [`Self::init`] is valid in every case.
    pub processing: Option<Result<ApiResponse<MediaUploadResponse>>>,
}

impl MediaUploadOutcome {
    /// The media id the upload produced, valid whether or not processing
    /// was awaited.
    #[must_use]
    pub fn media_id(&self) -> &str {
        &self.init.data.id
    }

    /// The upload's answer as one response: FINALIZE's, carrying the final
    /// processing state when a wait completed.
    ///
    /// With the final STATUS response in [`Self::processing`], its
    /// `processing_info` replaces FINALIZE's, and clears it when the status
    /// has none: X drops the field from media it has finished with, which
    /// leaves FINALIZE's pending state stale. Any other field only the
    /// status carried is added. Without a completed wait this is FINALIZE's
    /// response unchanged.
    #[must_use]
    pub fn response(&self) -> ApiResponse<MediaUploadResponse> {
        let mut response = self.finalize.clone();
        if let Some(Ok(status)) = &self.processing {
            let data = &mut response.data;
            data.processing_info
                .clone_from(&status.data.processing_info);
            if data.media_key.is_none() {
                data.media_key.clone_from(&status.data.media_key);
            }
            if data.expires_after_secs.is_none() {
                data.expires_after_secs = status.data.expires_after_secs;
            }
            for (key, value) in &status.data.extra {
                data.extra
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }
        response
    }
}

/// Handles the full media upload lifecycle.
///
/// `wait_for_processing` is the deadline a video's processing is awaited to;
/// `None` returns after FINALIZE.
///
/// # Errors
///
/// Returns an error if the file cannot be read or any upload phase (INIT,
/// APPEND, FINALIZE) fails. A processing failure or timeout after FINALIZE is
/// reported in [`MediaUploadOutcome::processing`], not here.
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub async fn execute_media_upload(
    file_path: &str,
    media_type: &str,
    media_category: &str,
    auth_type: &str,
    username: &str,
    trace: bool,
    wait_for_processing: Option<Duration>,
    headers: &[String],
    client: &Client,
) -> Result<MediaUploadOutcome> {
    let metadata = std::fs::metadata(file_path)
        .map_err(|e| Error::io(format!("error accessing file: {e}")).with_source(e))?;

    if !metadata.is_file() {
        return Err(Error::io(format!("{file_path} is not a regular file")));
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
    init_opts.method = MEDIA_UPLOAD_INITIALIZE.method.to_string();
    init_opts.target = RequestTarget::Template {
        path: MEDIA_UPLOAD_INITIALIZE.path.to_string(),
        path_params: HashMap::new(),
        query: Vec::new(),
    };
    init_opts.data = init_body.to_string();

    let init_response: ApiResponse<MediaUploadResponse> =
        deserialize_response(client.send_request(&init_opts).await?)?;
    let media_id = init_response.data.id.clone();
    if media_id.is_empty() {
        return Err(Error::json("failed to parse media ID from init response"));
    }

    // APPEND — upload in 4MB chunks
    upload_chunks(file_path, &media_id, &base_opts, file_size, client).await?;

    // FINALIZE
    tracing::info!(target: MEDIA_TARGET, "Finalizing media upload...");

    let mut finalize_opts = base_opts.clone();
    finalize_opts.method = MEDIA_UPLOAD_FINALIZE.method.to_string();
    finalize_opts.target = RequestTarget::Template {
        path: MEDIA_UPLOAD_FINALIZE.path.to_string(),
        path_params: HashMap::from([("id".to_string(), media_id.clone())]),
        query: Vec::new(),
    };
    finalize_opts.data.clear();

    let finalize_response: ApiResponse<MediaUploadResponse> =
        deserialize_response(client.send_request(&finalize_opts).await?)?;

    let processing = match wait_for_processing {
        Some(deadline) if media_category.contains("video") => {
            tracing::info!(target: MEDIA_TARGET, "Waiting for media processing to complete...");
            Some(wait_for_media_processing(&media_id, &base_opts, client, deadline).await)
        }
        _ => None,
    };

    Ok(MediaUploadOutcome {
        init: init_response,
        finalize: finalize_response,
        processing,
    })
}

/// Uploads file data in 4 MB chunks via APPEND requests.
async fn upload_chunks(
    file_path: &str,
    media_id: &str,
    base_opts: &RequestOptions,
    file_size: u64,
    client: &Client,
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
                method: MEDIA_UPLOAD_APPEND.method.to_string(),
                target: RequestTarget::Template {
                    path: MEDIA_UPLOAD_APPEND.path.to_string(),
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

        client.send_multipart_request(&multipart_opts).await?;

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

/// Reads a media upload's status, or waits on its processing.
///
/// `wait` is the deadline to wait to, [`DEFAULT_PROCESSING_WAIT`] being the
/// usual one; `None` reads the status once. A wait ends when the status
/// carries no processing information, when processing succeeds or fails, or
/// when the next check would fall past the deadline.
///
/// # Errors
///
/// Returns an error if a status request fails, [`Error::Validation`] if
/// processing failed, and [`Error::ProcessingTimeout`] if the deadline
/// passed with the job still running.
pub async fn execute_media_status(
    media_id: &str,
    auth_type: &str,
    username: &str,
    wait: Option<Duration>,
    trace: bool,
    headers: &[String],
    client: &Client,
) -> Result<ApiResponse<MediaUploadResponse>> {
    let base_opts = RequestOptions {
        auth_type: auth_type.to_string(),
        username: username.to_string(),
        trace,
        headers: headers.to_vec(),
        ..Default::default()
    };

    match wait {
        Some(deadline) => wait_for_media_processing(media_id, &base_opts, client, deadline).await,
        None => check_media_status(media_id, &base_opts, client).await,
    }
}

/// Checks media upload status.
async fn check_media_status(
    media_id: &str,
    base_opts: &RequestOptions,
    client: &Client,
) -> Result<ApiResponse<MediaUploadResponse>> {
    let mut opts = base_opts.clone();
    opts.method = MEDIA_UPLOAD_STATUS.method.to_string();
    opts.target = RequestTarget::Template {
        path: MEDIA_UPLOAD_STATUS.path.to_string(),
        path_params: HashMap::new(),
        query: vec![
            ("command".to_string(), "STATUS".to_string()),
            ("media_id".to_string(), media_id.to_string()),
        ],
    };
    opts.data.clear();

    deserialize_response(client.send_request(&opts).await?)
}

/// Polls media processing status until it finishes or `deadline` passes.
///
/// A status with no `processing_info` is a finished one: X attaches it only
/// while there is processing to report, so an image, or a video already
/// done, has nothing to wait for. The wait stops before a sleep that would
/// carry it past the deadline, so it never overruns by a whole interval.
async fn wait_for_media_processing(
    media_id: &str,
    base_opts: &RequestOptions,
    client: &Client,
    deadline: Duration,
) -> Result<ApiResponse<MediaUploadResponse>> {
    let started = tokio::time::Instant::now();
    loop {
        let response = check_media_status(media_id, base_opts, client).await?;

        let Some(info) = response.data.processing_info.as_ref() else {
            return Ok(response);
        };
        match info.state.as_str() {
            "succeeded" => {
                tracing::info!(target: MEDIA_TARGET, "Media processing complete!");
                return Ok(response);
            }
            "failed" => return Err(Error::validation("media processing failed")),
            _ => {}
        }

        let check_after = Duration::from_secs(info.check_after_secs.unwrap_or(1).max(1));
        if started.elapsed() + check_after > deadline {
            return Err(Error::ProcessingTimeout {
                media_id: media_id.to_string(),
                waited: deadline,
            });
        }

        let pct = info.progress_percent.unwrap_or(0);
        tracing::debug!(
            target: MEDIA_TARGET,
            "Media processing in progress ({pct}%), checking again in {} seconds...",
            check_after.as_secs()
        );

        tokio::time::sleep(check_after).await;
    }
}

/// Handles a media append request with a file (raw mode).
///
/// # Errors
///
/// Returns an error if the `media_id` is missing, the file cannot be read,
/// or the multipart request fails.
pub async fn handle_media_append_request(
    options: &RequestOptions,
    media_file: &str,
    client: &Client,
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

    client.send_multipart_request(&multipart_opts).await
}

/// Extracts `media_id` from a URL.
#[must_use]
pub fn extract_media_id(url: &str) -> String {
    if url.is_empty() || !url.contains(MEDIA_ENDPOINT) {
        return String::new();
    }

    if url.ends_with(MEDIA_UPLOAD_INITIALIZE.path) {
        return String::new();
    }

    // Extract media ID from path for append/finalize endpoints
    if let Some(rest) = url
        .split(MEDIA_ENDPOINT)
        .nth(1)
        .and_then(|rest| rest.strip_prefix('/'))
    {
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
    url.contains(MEDIA_ENDPOINT) && url.contains("append") && !media_file.is_empty()
}
