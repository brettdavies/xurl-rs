//! X API client — request building, response handling, shortcuts, and media.

pub mod auth_matrix;
mod endpoints;
mod media;
mod request;
pub mod response;
pub mod shortcuts;

pub use endpoints::is_streaming_endpoint;
#[allow(unused_imports)]
pub use media::{
    MEDIA_ENDPOINT, execute_media_status, execute_media_upload, extract_media_id,
    extract_segment_index, handle_media_append_request, is_media_append_request,
};
#[allow(unused_imports)]
pub use request::{
    ApiClient, CallOptions, DEFAULT_TIMEOUT_SECS, MultipartOptions, RequestOptions, RequestTarget,
};
#[allow(unused_imports)]
pub use response::types::{
    ApiError, ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent,
    FollowingResult, Includes, LikedResult, MediaProcessingInfo, MediaUploadResponse, MutingResult,
    Post, PostPublicMetrics, ReferencedPost, RepostedResult, ResponseMeta, UsageCreditsData,
    UsageData, User, UserPublicMetrics, deserialize_response,
};
#[allow(unused_imports)]
pub use shortcuts::{resolve_post_id, resolve_username};
