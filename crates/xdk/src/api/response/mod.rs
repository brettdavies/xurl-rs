//! Typed API response structs and response formatting.
//!
//! - `types` — Typed structs for X API v2 responses (`Post`, `User`, `ApiResponse<T>`, etc.)
//! - `vocabulary` — Reads X's legacy post vocabulary under the spec's current names

pub mod types;
pub(crate) mod vocabulary;

#[allow(unused_imports)] // Re-exported for library consumers
pub use types::{
    AccountActivitySubscription, AccountActivitySubscriptionCount, AccountActivitySubscriptions,
    ApiError, ApiResponse, BlockingResult, BookmarkedResult, DeletedResult, DmEvent, DmSentResult,
    FollowingResult, Includes, LikedResult, MediaProcessingInfo, MediaUploadResponse, MutingResult,
    Post, PostPublicMetrics, ProvisionedResult, ReferencedPost, RepostedResult, ResponseMeta,
    SubscribedResult, UsageCreditsData, UsageData, User, UserPublicMetrics, Webhook,
    WebhookReplayJob, WebhookStreamLink, WebhookValidation, deserialize_response,
};
