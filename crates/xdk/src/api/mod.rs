//! X API client — request building, response handling, shortcuts, and media.

pub mod auth_matrix;
mod endpoints;
mod media;
mod media_upload;
mod request;
pub mod response;
pub mod shortcuts;
mod url_authority;

pub use endpoints::is_streaming_endpoint;
pub use media::{DEFAULT_PROCESSING_WAIT, MEDIA_TARGET, MediaUploadOutcome, execute_media_status};
pub use media_upload::MediaUpload;
// Media plumbing the binary drives by hand: `xr <URL>` services an append
// request from the raw path, `xr media upload` runs the phases with its own
// flags, and `xr --dry-run media upload` and `xr --dry-run media status`
// check what those phases can be checked for unsent; an embedder uploads
// through `Client::upload_media`.
#[doc(hidden)]
pub use media::{
    MEDIA_ENDPOINT, execute_media_upload, extract_media_id, extract_segment_index,
    handle_media_append_request, is_media_append_request, media_status_auth_preflight,
    media_upload_preflight,
};
// The cut of a URL around its authority, which `validate_webhook_url` reads
// the userinfo and port from and `xr webhooks add` and `xr webhooks listen
// --forward` print their URL without the userinfo of; an embedder reads a URL
// with `url::Url`.
pub(crate) use request::api_error;
pub use request::{
    AuthPreflight, Call, Client, ClientBuilder, DEFAULT_TIMEOUT_SECS, MultipartOptions,
    RequestOptions, RequestTarget, StreamLines, WIRE_TARGET,
};
pub use request::{DEFAULT_USER_AGENT, RateLimit};
pub use response::types::{
    AccountActivitySubscription, AccountActivitySubscriptionCount, AccountActivitySubscriptions,
    AnalyticsBucket, AnalyticsMetrics, ApiError, ApiResponse, ArticleDraft, ArticlePublished,
    BlockingResult, BookmarkFolder, BookmarkFolderPost, BookmarkedResult, Broadcast,
    BroadcastChatMessage, BroadcastEnded, BroadcastUpdated, ChatModeratorsResult, ChatMute,
    ChatMutedResult, ChatSentResult, Community, DeletedResult, DmBlockResult, DmEvent,
    DmSentResult, FollowingResult, HiddenResult, Includes, IngestRegion, LikedResult, List,
    ListMemberResult, ListUpdatedResult, Media, MediaAnalytics, MediaAnalyticsBucket,
    MediaMetadataResult, MediaProcessingInfo, MediaSubtitlesResult, MediaUploadResponse,
    MutingResult, NewsClusterPost, NewsStory, Note, NoteCreated, NoteEvaluation, NoteInfo,
    PersonalizedTrend, PinnedResult, Post, PostAnalytics, PostCount, PostDraft, PostDraftCreated,
    PostDraftMedia, PostDraftReply, PostDraftThreadPost, PostPublicMetrics, ProvisionedResult,
    ReferencedPost, RepostedResult, ResponseMeta, ScheduledBroadcast, ScheduledCall,
    ScheduledCallCancelled, ScheduledCallInvitee, ScheduledCallRsvp, ScheduledCallUpdated, Space,
    StreamRule, StreamRuleAppCount, StreamRuleCounts, StreamSource, StreamSourceCreated,
    StreamSourceDeleted, StreamSourceKey, SubscribedResult, Trend, TrendLocation, UsageCreditsData,
    UsageData, User, UserPublicMetrics, Webhook, WebhookReplayJob, WebhookStreamLink,
    WebhookValidation, XCall, deserialize_response,
};
pub use response::vocabulary::VOCABULARY_TARGET;
pub use shortcuts::{VideoCategory, resolve_post_id, resolve_username};
#[doc(hidden)]
pub use url_authority::{UrlParts, split_special_url};
