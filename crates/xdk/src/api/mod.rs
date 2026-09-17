//! X API client — request building, response handling, shortcuts, and media.

pub mod auth_matrix;
mod endpoints;
mod media;
mod media_upload;
mod request;
pub mod response;
pub mod shortcuts;

pub use endpoints::is_streaming_endpoint;
#[allow(unused_imports)]
pub use media::{MEDIA_TARGET, MediaUploadOutcome, execute_media_status};
pub use media_upload::MediaUpload;
// Media plumbing the binary drives by hand: `xr <URL>` services an append
// request from the raw path, and `xr media upload` runs the phases with its
// own flags; an embedder uploads through `Client::upload_media`.
#[doc(hidden)]
#[allow(unused_imports)]
pub use media::{
    MEDIA_ENDPOINT, execute_media_upload, extract_media_id, extract_segment_index,
    handle_media_append_request, is_media_append_request,
};
pub use request::{
    Call, Client, ClientBuilder, DEFAULT_TIMEOUT_SECS, MultipartOptions, RequestOptions,
    RequestTarget, StreamLines, WIRE_TARGET,
};
#[allow(unused_imports)]
pub use request::{DEFAULT_USER_AGENT, RateLimit};
#[allow(unused_imports)]
pub use response::types::{
    ApiError, ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent,
    FollowingResult, Includes, LikedResult, MediaProcessingInfo, MediaUploadResponse, MutingResult,
    Post, PostPublicMetrics, ReferencedPost, RepostedResult, ResponseMeta, UsageCreditsData,
    UsageData, User, UserPublicMetrics, deserialize_response,
};
#[allow(unused_imports)]
pub use shortcuts::{resolve_post_id, resolve_username};
