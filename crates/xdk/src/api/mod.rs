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
pub use media::{DEFAULT_PROCESSING_WAIT, MEDIA_TARGET, MediaUploadOutcome, execute_media_status};
pub use media_upload::MediaUpload;
// Media plumbing the binary drives by hand: `xr <URL>` services an append
// request from the raw path, `xr media upload` runs the phases with its own
// flags, and `xr --dry-run media upload` and `xr --dry-run media status`
// check what those phases can be checked for unsent; an embedder uploads
// through `Client::upload_media`.
#[doc(hidden)]
#[allow(unused_imports)]
pub use media::{
    MEDIA_ENDPOINT, execute_media_upload, extract_media_id, extract_segment_index,
    handle_media_append_request, is_media_append_request, media_status_auth_preflight,
    media_upload_preflight,
};
pub use request::{
    AuthPreflight, Call, Client, ClientBuilder, DEFAULT_TIMEOUT_SECS, MultipartOptions,
    RequestOptions, RequestTarget, StreamLines, WIRE_TARGET,
};
#[allow(unused_imports)]
pub use request::{DEFAULT_USER_AGENT, RateLimit};
#[allow(unused_imports)]
pub use response::types::{
    AccountActivitySubscription, AccountActivitySubscriptionCount, AccountActivitySubscriptions,
    AnalyticsBucket, AnalyticsMetrics, ApiError, ApiResponse, ArticleDraft, ArticlePublished,
    BlockingResult, BookmarkFolder, BookmarkFolderPost, BookmarkedResult, Broadcast,
    BroadcastChatMessage, ChatModeratorsResult, ChatMutedResult, ChatSentResult, Community,
    DeletedResult, DmBlockResult, DmEvent, DmSentResult, FollowingResult, HiddenResult, Includes,
    LikedResult, List, ListMemberResult, ListUpdatedResult, Media, MediaAnalytics,
    MediaAnalyticsBucket, MediaMetadataResult, MediaProcessingInfo, MediaSubtitlesResult,
    MediaUploadResponse, MutingResult, NewsClusterPost, NewsStory, Note, NoteCreated,
    NoteEvaluation, NoteInfo, PersonalizedTrend, PinnedResult, Post, PostAnalytics, PostCount,
    PostPublicMetrics, ProvisionedResult, ReferencedPost, RepostedResult, ResponseMeta,
    ScheduledBroadcast, ScheduledCall, ScheduledCallCancelled, ScheduledCallInvitee,
    ScheduledCallRsvp, ScheduledCallUpdated, Space, StreamRule, StreamRuleAppCount,
    StreamRuleCounts, StreamSource, StreamSourceDeleted, SubscribedResult, Trend, UsageCreditsData,
    UsageData, User, UserPublicMetrics, Webhook, WebhookReplayJob, WebhookStreamLink,
    WebhookValidation, XCall, deserialize_response,
};
pub use response::vocabulary::VOCABULARY_TARGET;
#[allow(unused_imports)]
pub use shortcuts::{VideoCategory, resolve_post_id, resolve_username};
