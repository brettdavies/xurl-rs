//! Typed API response structs and response formatting.
//!
//! - `types` — Typed structs for X API v2 responses (`Post`, `User`, `ApiResponse<T>`, etc.)
//! - `vocabulary` — Reads X's legacy post vocabulary under the spec's current names

pub mod types;
mod vocabulary;

#[allow(unused_imports)] // Re-exported for library consumers
pub use types::{
    ApiError, ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent, DmSentResult,
    FollowingResult, Includes, LikedResult, MediaProcessingInfo, MediaUploadResponse, MutingResult,
    Post, PostPublicMetrics, ReferencedPost, RepostedResult, ResponseMeta, UsageCreditsData,
    UsageData, User, UserPublicMetrics, deserialize_response,
};
