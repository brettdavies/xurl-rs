/// Typed API response structs and response formatting.
///
/// - `types` — Typed structs for X API v2 responses (`Tweet`, `User`, `ApiResponse<T>`, etc.)
/// - `format` — JSON pretty-printing with syntax highlighting
mod format;
pub mod types;

pub use format::format_and_print_response;
#[allow(unused_imports)] // Re-exported for library consumers
pub use types::{
    ApiError, ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent,
    FollowingResult, Includes, LikedResult, MediaProcessingInfo, MediaUploadResponse, MutingResult,
    ReferencedTweet, ResponseMeta, RetweetedResult, Tweet, TweetPublicMetrics, UsageData, User,
    UserPublicMetrics, deserialize_response,
};
