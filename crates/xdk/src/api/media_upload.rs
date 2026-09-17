//! One media upload that has not been sent: the file, the media type and
//! category (inferred from the file when not set), and the per-call options,
//! run through INIT, APPEND, FINALIZE, and STATUS by [`MediaUpload::send`].

use std::path::{Path, PathBuf};

use crate::api::auth_matrix::WireScheme;
use crate::error::{Error, Result};

use super::media::{MediaUploadOutcome, execute_media_upload};
use super::request::Client;

impl Client {
    /// Starts an upload of the file at `path`; [`MediaUpload::send`] runs
    /// every phase and returns each phase's response.
    ///
    /// ```rust,no_run
    /// # async fn run(client: xdk::api::Client) -> xdk::Result<()> {
    /// let upload = client.upload_media("photo.png").send().await?;
    /// let post = client
    ///     .create_post("hello", &[upload.media_id().to_string()])
    ///     .send()
    ///     .await?;
    /// # let _ = post; Ok(()) }
    /// ```
    pub fn upload_media(&self, path: impl Into<PathBuf>) -> MediaUpload {
        MediaUpload {
            client: self.clone(),
            path: path.into(),
            media_type: None,
            category: None,
            wait_for_processing: true,
            auth_type: String::new(),
            username: String::new(),
            trace: false,
            headers: Vec::new(),
        }
    }
}

/// A media upload that has not been sent.
///
/// The media type comes from the file's extension (`png`, `jpg`, `jpeg`,
/// `gif`, `webp`, `mp4`, `m4v`, `mov`) unless [`MediaUpload::media_type`]
/// sets it, and the category from the media type (`tweet_gif`,
/// `tweet_video`, or `tweet_image`) unless [`MediaUpload::category`] sets
/// it. A video is awaited through STATUS until X finishes processing it, so
/// the returned media id is ready to attach to a post; see
/// [`MediaUpload::wait_for_processing`].
#[must_use = "a MediaUpload does nothing until it is sent"]
pub struct MediaUpload {
    client: Client,
    path: PathBuf,
    media_type: Option<String>,
    category: Option<String>,
    wait_for_processing: bool,
    auth_type: String,
    username: String,
    trace: bool,
    headers: Vec<String>,
}

crate::assert_send_sync!(MediaUpload);

impl std::fmt::Debug for MediaUpload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaUpload")
            .field("path", &self.path)
            .field("media_type", &self.media_type)
            .field("category", &self.category)
            .field("wait_for_processing", &self.wait_for_processing)
            .finish_non_exhaustive()
    }
}

impl MediaUpload {
    /// The MIME type X receives at INIT, in place of the one inferred from
    /// the file's extension.
    pub fn media_type(mut self, media_type: impl Into<String>) -> Self {
        self.media_type = Some(media_type.into());
        self
    }

    /// The media category X receives at INIT (`tweet_image`, `tweet_gif`,
    /// `tweet_video`, `dm_image`, `dm_gif`, `dm_video`, `subtitles`, or
    /// `amplify_video`), in place of the one inferred from the media type.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Whether to poll STATUS until X finishes processing a video before
    /// returning; on by default. Images and GIFs are never awaited.
    pub fn wait_for_processing(mut self, wait: bool) -> Self {
        self.wait_for_processing = wait;
        self
    }

    /// Sends every phase under `scheme` instead of the auto-detected one.
    pub fn auth(mut self, scheme: WireScheme) -> Self {
        self.auth_type = scheme.as_wire().to_string();
        self
    }

    /// Selects which stored `OAuth2` user a store-backed client sends as. A
    /// client built from credentials holds one user and ignores it.
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    /// Sends the `X-B3-Flags: 1` header on every phase.
    pub fn trace(mut self, on: bool) -> Self {
        self.trace = on;
        self
    }

    /// Adds a request header to every phase. One the client would set itself
    /// (`Authorization`, `User-Agent`, `Content-Type`, `X-B3-Flags`) is
    /// replaced by yours.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .push(format!("{}: {}", name.into(), value.into()));
        self
    }

    /// Runs INIT, APPEND, FINALIZE, and, for a video, STATUS.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`] when no media type was set and none can be
    /// inferred from the file's extension, or when the path is not valid
    /// UTF-8; otherwise everything the upload phases return: [`Error::Io`]
    /// when the file cannot be read, an [`Error::Api`] for a refused phase,
    /// and [`Error::Validation`] when processing fails or times out after the
    /// upload itself completed.
    pub async fn send(self) -> Result<MediaUploadOutcome> {
        let media_type = match self.media_type {
            Some(media_type) => media_type,
            None => media_type_for(&self.path)?.to_string(),
        };
        let category = self
            .category
            .unwrap_or_else(|| category_for(&media_type).to_string());
        let path = self.path.to_str().ok_or_else(|| {
            Error::validation(format!("{} is not a valid UTF-8 path", self.path.display()))
        })?;
        execute_media_upload(
            path,
            &media_type,
            &category,
            &self.auth_type,
            &self.username,
            self.trace,
            self.wait_for_processing,
            &self.headers,
            &self.client,
        )
        .await
    }
}

/// The MIME type a file's extension implies.
fn media_type_for(path: &Path) -> Result<&'static str> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("png") => Ok("image/png"),
        Some("jpg" | "jpeg") => Ok("image/jpeg"),
        Some("gif") => Ok("image/gif"),
        Some("webp") => Ok("image/webp"),
        Some("mp4" | "m4v") => Ok("video/mp4"),
        Some("mov") => Ok("video/quicktime"),
        _ => Err(Error::validation(format!(
            "cannot infer the media type of {}; set it with MediaUpload::media_type",
            path.display()
        ))),
    }
}

/// The upload category a MIME type implies for a post attachment.
fn category_for(media_type: &str) -> &'static str {
    if media_type == "image/gif" {
        "tweet_gif"
    } else if media_type.starts_with("video/") {
        "tweet_video"
    } else {
        "tweet_image"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_type_follows_the_extension_case_insensitively() {
        assert_eq!(media_type_for(Path::new("a.PNG")).unwrap(), "image/png");
        assert_eq!(media_type_for(Path::new("a.jpeg")).unwrap(), "image/jpeg");
        assert_eq!(media_type_for(Path::new("a.jpg")).unwrap(), "image/jpeg");
        assert_eq!(media_type_for(Path::new("a.gif")).unwrap(), "image/gif");
        assert_eq!(media_type_for(Path::new("a.webp")).unwrap(), "image/webp");
        assert_eq!(media_type_for(Path::new("a.mp4")).unwrap(), "video/mp4");
        assert_eq!(media_type_for(Path::new("a.m4v")).unwrap(), "video/mp4");
        assert_eq!(
            media_type_for(Path::new("a.MOV")).unwrap(),
            "video/quicktime"
        );
    }

    #[test]
    fn an_unknown_or_missing_extension_is_a_validation_error_naming_the_setter() {
        for name in ["notes.txt", "archive.tar.gz", "noext", ".hidden"] {
            let err = media_type_for(Path::new(name)).unwrap_err();
            assert!(err.is_validation(), "{name}: {err:?}");
            assert!(
                err.to_string().contains("MediaUpload::media_type"),
                "{name}: {err}"
            );
        }
    }

    #[test]
    fn category_follows_the_media_type() {
        assert_eq!(category_for("image/gif"), "tweet_gif");
        assert_eq!(category_for("video/mp4"), "tweet_video");
        assert_eq!(category_for("video/quicktime"), "tweet_video");
        assert_eq!(category_for("image/png"), "tweet_image");
        assert_eq!(category_for("image/webp"), "tweet_image");
    }
}
