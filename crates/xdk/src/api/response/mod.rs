//! Typed API response structs and response formatting.
//!
//! - `types` — Typed structs for X API v2 responses (`Post`, `User`, `ApiResponse<T>`, etc.)

pub mod types;

#[allow(unused_imports)] // Re-exported for library consumers
pub use types::{
    ApiError, ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent, DmSentResult,
    FollowingResult, Includes, LikedResult, MediaProcessingInfo, MediaUploadResponse, MutingResult,
    Post, PostPublicMetrics, ReferencedPost, RepostedResult, ResponseMeta, UsageCreditsData,
    UsageData, User, UserPublicMetrics, deserialize_response,
};
