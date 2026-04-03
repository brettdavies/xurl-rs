/// X API client — request building, response handling, shortcuts, and media.
mod endpoints;
mod media;
mod request;
pub mod response;
mod shortcuts;

pub use endpoints::is_streaming_endpoint;
#[allow(unused_imports)]
pub use media::{
    MEDIA_ENDPOINT, execute_media_status, execute_media_upload, extract_media_id,
    extract_segment_index, handle_media_append_request, is_media_append_request,
};
#[allow(unused_imports)]
pub use request::{ApiClient, CallOptions, MultipartOptions, RequestOptions};
#[allow(unused_imports)]
pub use response::types::{
    ApiError, ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent,
    FollowingResult, Includes, LikedResult, MediaProcessingInfo, MediaUploadResponse, MutingResult,
    ReferencedTweet, ResponseMeta, RetweetedResult, Tweet, TweetPublicMetrics, UsageData, User,
    UserPublicMetrics, deserialize_response,
};
pub use shortcuts::*;
