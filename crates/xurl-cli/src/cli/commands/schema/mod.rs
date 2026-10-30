//! Schema subcommand: JSON Schema for each command's response type.
//!
//! This module holds the registry, which names every command with a typed
//! response by its words. `words` turns the words a caller gives into a row,
//! `run` prints what each scope asks for, `resolve` derives the views, and
//! `pointers` points each registered command's help and dry run at its schema.

mod pointers;
mod resolve;
mod run;
mod words;

pub use pointers::{SchemaPointers, pointer_for};
pub use run::{Request, run_schema};
pub use words::{Refusal, scope_words};

use schemars::JsonSchema;
use schemars::generate::SchemaSettings;
use serde_json::Value;

use crate::cli::commands::auth::{AppStatusEntry, RedirectUriGetResponse, RedirectUriSetResponse};
use xdk::api::{
    AccountActivitySubscriptionCount, AccountActivitySubscriptions, ApiResponse, ArticleDraft,
    ArticlePublished, BlockingResult, BookmarkFolder, BookmarkFolderPost, BookmarkedResult,
    Broadcast, BroadcastChatMessage, ChatModeratorsResult, ChatMutedResult, ChatSentResult,
    Community, DeletedResult, DmBlockResult, DmEvent, DmSentResult, FollowingResult, HiddenResult,
    LikedResult, List, ListMemberResult, ListUpdatedResult, Media, MediaAnalytics,
    MediaMetadataResult, MediaSubtitlesResult, MutingResult, NewsStory, Note, NoteCreated,
    NoteEvaluation, PersonalizedTrend, PinnedResult, Post, PostAnalytics, PostCount,
    ProvisionedResult, RepostedResult, ScheduledBroadcast, ScheduledCall, ScheduledCallCancelled,
    ScheduledCallRsvp, ScheduledCallUpdated, Space, StreamRule, StreamRuleCounts,
    StreamSourceDeleted, StreamSourceResult, StreamSources, SubscribedResult, Trend,
    UsageCreditsData, UsageData, User, Webhook, WebhookReplayJob, WebhookStreamLink,
    WebhookValidation, XCall,
};

/// Command-to-response-type mapping entry.
struct SchemaEntry {
    /// Commands that share this response type.
    commands: &'static [&'static str],
    /// Human-readable type name.
    type_name: &'static str,
    /// Builds the response type's JSON Schema under `settings`. The type
    /// must be named where the generator runs, so each row carries its own.
    schema: fn(SchemaSettings) -> Value,
}

/// `T`'s JSON Schema as the generator `settings` describes renders it.
fn generate<T: JsonSchema>(settings: SchemaSettings) -> Value {
    settings.into_generator().into_root_schema_for::<T>().into()
}

/// All command-to-type mappings, ordered by category.
const SCHEMA_ENTRIES: &[SchemaEntry] = &[
    SchemaEntry {
        commands: &["post", "reply", "quote", "read"],
        type_name: "ApiResponse<Post>",
        schema: generate::<ApiResponse<Post>>,
    },
    SchemaEntry {
        commands: &[
            "posts",
            "user-posts",
            "lists posts",
            "spaces posts",
            "notes eligible",
            "search",
            "timeline",
            "mentions",
            "bookmarks",
            "likes",
            "quotes",
        ],
        type_name: "ApiResponse<Vec<Post>>",
        schema: generate::<ApiResponse<Vec<Post>>>,
    },
    SchemaEntry {
        commands: &["whoami", "user"],
        type_name: "ApiResponse<User>",
        schema: generate::<ApiResponse<User>>,
    },
    SchemaEntry {
        commands: &[
            "following",
            "followers",
            "muted",
            "blocked",
            "likers",
            "reposters",
            "users",
            "lists followers",
            "lists members list",
            "search-users",
        ],
        type_name: "ApiResponse<Vec<User>>",
        schema: generate::<ApiResponse<Vec<User>>>,
    },
    SchemaEntry {
        commands: &["like", "unlike"],
        type_name: "ApiResponse<LikedResult>",
        schema: generate::<ApiResponse<LikedResult>>,
    },
    SchemaEntry {
        commands: &[
            "lists list",
            "lists followed",
            "lists memberships",
            "lists pinned",
        ],
        type_name: "ApiResponse<Vec<List>>",
        schema: generate::<ApiResponse<Vec<List>>>,
    },
    SchemaEntry {
        commands: &["lists get", "lists add"],
        type_name: "ApiResponse<List>",
        schema: generate::<ApiResponse<List>>,
    },
    SchemaEntry {
        commands: &["lists update"],
        type_name: "ApiResponse<ListUpdatedResult>",
        schema: generate::<ApiResponse<ListUpdatedResult>>,
    },
    SchemaEntry {
        commands: &["lists members add", "lists members remove"],
        type_name: "ApiResponse<ListMemberResult>",
        schema: generate::<ApiResponse<ListMemberResult>>,
    },
    SchemaEntry {
        commands: &["lists pin", "lists unpin"],
        type_name: "ApiResponse<PinnedResult>",
        schema: generate::<ApiResponse<PinnedResult>>,
    },
    SchemaEntry {
        commands: &["broadcasts list"],
        type_name: "ApiResponse<Vec<Broadcast>>",
        schema: generate::<ApiResponse<Vec<Broadcast>>>,
    },
    SchemaEntry {
        commands: &["broadcasts get"],
        type_name: "ApiResponse<Broadcast>",
        schema: generate::<ApiResponse<Broadcast>>,
    },
    SchemaEntry {
        commands: &["broadcasts scheduled list"],
        type_name: "ApiResponse<Vec<ScheduledBroadcast>>",
        schema: generate::<ApiResponse<Vec<ScheduledBroadcast>>>,
    },
    SchemaEntry {
        commands: &[
            "broadcasts scheduled get",
            "broadcasts scheduled add",
            "broadcasts scheduled update",
        ],
        type_name: "ApiResponse<ScheduledBroadcast>",
        schema: generate::<ApiResponse<ScheduledBroadcast>>,
    },
    SchemaEntry {
        commands: &["broadcasts chat history"],
        type_name: "ApiResponse<Vec<BroadcastChatMessage>>",
        schema: generate::<ApiResponse<Vec<BroadcastChatMessage>>>,
    },
    SchemaEntry {
        commands: &["broadcasts chat send"],
        type_name: "ApiResponse<ChatSentResult>",
        schema: generate::<ApiResponse<ChatSentResult>>,
    },
    SchemaEntry {
        commands: &["broadcasts chat mute", "broadcasts chat unmute"],
        type_name: "ApiResponse<ChatMutedResult>",
        schema: generate::<ApiResponse<ChatMutedResult>>,
    },
    SchemaEntry {
        commands: &["media get"],
        type_name: "ApiResponse<Vec<Media>>",
        schema: generate::<ApiResponse<Vec<Media>>>,
    },
    SchemaEntry {
        commands: &["media analytics"],
        type_name: "ApiResponse<Vec<MediaAnalytics>>",
        schema: generate::<ApiResponse<Vec<MediaAnalytics>>>,
    },
    SchemaEntry {
        commands: &["spaces search", "spaces get", "spaces by-creator"],
        type_name: "ApiResponse<Vec<Space>>",
        schema: generate::<ApiResponse<Vec<Space>>>,
    },
    SchemaEntry {
        commands: &["stream rules list"],
        type_name: "ApiResponse<Vec<StreamRule>>",
        schema: generate::<ApiResponse<Vec<StreamRule>>>,
    },
    SchemaEntry {
        commands: &["stream rules add", "stream rules remove"],
        type_name: "ApiResponse<Option<Vec<StreamRule>>>",
        schema: generate::<ApiResponse<Option<Vec<StreamRule>>>>,
    },
    SchemaEntry {
        commands: &["stream rules count"],
        type_name: "ApiResponse<StreamRuleCounts>",
        schema: generate::<ApiResponse<StreamRuleCounts>>,
    },
    SchemaEntry {
        commands: &["articles draft"],
        type_name: "ApiResponse<ArticleDraft>",
        schema: generate::<ApiResponse<ArticleDraft>>,
    },
    SchemaEntry {
        commands: &["articles publish"],
        type_name: "ApiResponse<ArticlePublished>",
        schema: generate::<ApiResponse<ArticlePublished>>,
    },
    SchemaEntry {
        commands: &["notes write"],
        type_name: "ApiResponse<NoteCreated>",
        schema: generate::<ApiResponse<NoteCreated>>,
    },
    SchemaEntry {
        commands: &["notes evaluate"],
        type_name: "ApiResponse<NoteEvaluation>",
        schema: generate::<ApiResponse<NoteEvaluation>>,
    },
    SchemaEntry {
        commands: &["notes written"],
        type_name: "ApiResponse<Vec<Note>>",
        schema: generate::<ApiResponse<Vec<Note>>>,
    },
    SchemaEntry {
        commands: &["communities search"],
        type_name: "ApiResponse<Vec<Community>>",
        schema: generate::<ApiResponse<Vec<Community>>>,
    },
    SchemaEntry {
        commands: &["communities get"],
        type_name: "ApiResponse<Community>",
        schema: generate::<ApiResponse<Community>>,
    },
    SchemaEntry {
        commands: &["news search"],
        type_name: "ApiResponse<Vec<NewsStory>>",
        schema: generate::<ApiResponse<Vec<NewsStory>>>,
    },
    SchemaEntry {
        commands: &["news get"],
        type_name: "ApiResponse<NewsStory>",
        schema: generate::<ApiResponse<NewsStory>>,
    },
    SchemaEntry {
        commands: &["trends place"],
        type_name: "ApiResponse<Vec<Trend>>",
        schema: generate::<ApiResponse<Vec<Trend>>>,
    },
    SchemaEntry {
        commands: &["trends personal"],
        type_name: "ApiResponse<Vec<PersonalizedTrend>>",
        schema: generate::<ApiResponse<Vec<PersonalizedTrend>>>,
    },
    SchemaEntry {
        commands: &["analytics"],
        type_name: "ApiResponse<Vec<PostAnalytics>>",
        schema: generate::<ApiResponse<Vec<PostAnalytics>>>,
    },
    SchemaEntry {
        commands: &["search-counts"],
        type_name: "ApiResponse<Vec<PostCount>>",
        schema: generate::<ApiResponse<Vec<PostCount>>>,
    },
    SchemaEntry {
        commands: &["hide", "unhide"],
        type_name: "ApiResponse<HiddenResult>",
        schema: generate::<ApiResponse<HiddenResult>>,
    },
    SchemaEntry {
        commands: &["follow", "unfollow", "lists follow", "lists unfollow"],
        type_name: "ApiResponse<FollowingResult>",
        schema: generate::<ApiResponse<FollowingResult>>,
    },
    SchemaEntry {
        commands: &[
            "delete",
            "dm-events remove",
            "lists remove",
            "broadcasts scheduled cancel",
            "broadcasts chat remove",
            "notes remove",
            "media subtitles remove",
            "webhooks remove",
            "webhooks stream-links remove",
        ],
        type_name: "ApiResponse<DeletedResult>",
        schema: generate::<ApiResponse<DeletedResult>>,
    },
    SchemaEntry {
        commands: &["repost", "unrepost"],
        type_name: "ApiResponse<RepostedResult>",
        schema: generate::<ApiResponse<RepostedResult>>,
    },
    SchemaEntry {
        commands: &["bookmark", "unbookmark"],
        type_name: "ApiResponse<BookmarkedResult>",
        schema: generate::<ApiResponse<BookmarkedResult>>,
    },
    SchemaEntry {
        commands: &["mute", "unmute"],
        type_name: "ApiResponse<MutingResult>",
        schema: generate::<ApiResponse<MutingResult>>,
    },
    SchemaEntry {
        commands: &["block", "unblock"],
        type_name: "ApiResponse<BlockingResult>",
        schema: generate::<ApiResponse<BlockingResult>>,
    },
    SchemaEntry {
        commands: &["dm", "conversations start", "conversations send"],
        type_name: "ApiResponse<DmSentResult>",
        schema: generate::<ApiResponse<DmSentResult>>,
    },
    SchemaEntry {
        commands: &["dms", "conversations history", "conversations with"],
        type_name: "ApiResponse<Vec<DmEvent>>",
        schema: generate::<ApiResponse<Vec<DmEvent>>>,
    },
    SchemaEntry {
        commands: &["dm-events get"],
        type_name: "ApiResponse<DmEvent>",
        schema: generate::<ApiResponse<DmEvent>>,
    },
    SchemaEntry {
        commands: &["dm-block", "dm-unblock"],
        type_name: "ApiResponse<DmBlockResult>",
        schema: generate::<ApiResponse<DmBlockResult>>,
    },
    SchemaEntry {
        commands: &["usage"],
        type_name: "ApiResponse<UsageData>",
        schema: generate::<ApiResponse<UsageData>>,
    },
    SchemaEntry {
        commands: &["usage credits"],
        type_name: "ApiResponse<UsageCreditsData>",
        schema: generate::<ApiResponse<UsageCreditsData>>,
    },
    SchemaEntry {
        commands: &["broadcasts moderators list"],
        type_name: "ApiResponse<Vec<User>>",
        schema: generate::<ApiResponse<Vec<User>>>,
    },
    SchemaEntry {
        commands: &["broadcasts moderators add", "broadcasts moderators remove"],
        type_name: "ApiResponse<ChatModeratorsResult>",
        schema: generate::<ApiResponse<ChatModeratorsResult>>,
    },
    SchemaEntry {
        commands: &["bookmark-folders list"],
        type_name: "ApiResponse<Vec<BookmarkFolder>>",
        schema: generate::<ApiResponse<Vec<BookmarkFolder>>>,
    },
    SchemaEntry {
        commands: &["bookmark-folders add"],
        type_name: "ApiResponse<BookmarkFolder>",
        schema: generate::<ApiResponse<BookmarkFolder>>,
    },
    SchemaEntry {
        commands: &["bookmark-folders posts"],
        type_name: "ApiResponse<Vec<BookmarkFolderPost>>",
        schema: generate::<ApiResponse<Vec<BookmarkFolderPost>>>,
    },
    SchemaEntry {
        commands: &["broadcasts sources list"],
        type_name: "ApiResponse<StreamSources>",
        schema: generate::<ApiResponse<StreamSources>>,
    },
    SchemaEntry {
        commands: &[
            "broadcasts sources get",
            "broadcasts sources add",
            "broadcasts sources rename",
        ],
        type_name: "ApiResponse<StreamSourceResult>",
        schema: generate::<ApiResponse<StreamSourceResult>>,
    },
    SchemaEntry {
        commands: &["broadcasts sources remove"],
        type_name: "ApiResponse<StreamSourceDeleted>",
        schema: generate::<ApiResponse<StreamSourceDeleted>>,
    },
    SchemaEntry {
        commands: &["calls get"],
        type_name: "ApiResponse<XCall>",
        schema: generate::<ApiResponse<XCall>>,
    },
    SchemaEntry {
        commands: &["calls scheduled list"],
        type_name: "ApiResponse<Vec<ScheduledCall>>",
        schema: generate::<ApiResponse<Vec<ScheduledCall>>>,
    },
    SchemaEntry {
        commands: &["calls scheduled get", "calls scheduled add"],
        type_name: "ApiResponse<ScheduledCall>",
        schema: generate::<ApiResponse<ScheduledCall>>,
    },
    SchemaEntry {
        commands: &["calls scheduled update"],
        type_name: "ApiResponse<ScheduledCallUpdated>",
        schema: generate::<ApiResponse<ScheduledCallUpdated>>,
    },
    SchemaEntry {
        commands: &["calls scheduled cancel"],
        type_name: "ApiResponse<ScheduledCallCancelled>",
        schema: generate::<ApiResponse<ScheduledCallCancelled>>,
    },
    SchemaEntry {
        commands: &["calls scheduled rsvp"],
        type_name: "ApiResponse<ScheduledCallRsvp>",
        schema: generate::<ApiResponse<ScheduledCallRsvp>>,
    },
    SchemaEntry {
        commands: &["webhooks list"],
        type_name: "ApiResponse<Vec<Webhook>>",
        schema: generate::<ApiResponse<Vec<Webhook>>>,
    },
    SchemaEntry {
        commands: &["webhooks add"],
        type_name: "ApiResponse<Webhook>",
        schema: generate::<ApiResponse<Webhook>>,
    },
    SchemaEntry {
        commands: &["webhooks validate"],
        type_name: "ApiResponse<WebhookValidation>",
        schema: generate::<ApiResponse<WebhookValidation>>,
    },
    SchemaEntry {
        commands: &["webhooks subscriptions count"],
        type_name: "ApiResponse<AccountActivitySubscriptionCount>",
        schema: generate::<ApiResponse<AccountActivitySubscriptionCount>>,
    },
    SchemaEntry {
        commands: &["webhooks subscriptions list"],
        type_name: "ApiResponse<AccountActivitySubscriptions>",
        schema: generate::<ApiResponse<AccountActivitySubscriptions>>,
    },
    SchemaEntry {
        commands: &[
            "webhooks subscriptions add",
            "webhooks subscriptions check",
            "webhooks subscriptions remove",
        ],
        type_name: "ApiResponse<SubscribedResult>",
        schema: generate::<ApiResponse<SubscribedResult>>,
    },
    SchemaEntry {
        commands: &["webhooks stream-links list"],
        type_name: "ApiResponse<Vec<WebhookStreamLink>>",
        schema: generate::<ApiResponse<Vec<WebhookStreamLink>>>,
    },
    SchemaEntry {
        commands: &["webhooks stream-links add"],
        type_name: "ApiResponse<ProvisionedResult>",
        schema: generate::<ApiResponse<ProvisionedResult>>,
    },
    SchemaEntry {
        commands: &["webhooks replay"],
        type_name: "ApiResponse<WebhookReplayJob>",
        schema: generate::<ApiResponse<WebhookReplayJob>>,
    },
    SchemaEntry {
        commands: &["media alt-text"],
        type_name: "ApiResponse<MediaMetadataResult>",
        schema: generate::<ApiResponse<MediaMetadataResult>>,
    },
    SchemaEntry {
        commands: &["media subtitles add"],
        type_name: "ApiResponse<MediaSubtitlesResult>",
        schema: generate::<ApiResponse<MediaSubtitlesResult>>,
    },
    SchemaEntry {
        commands: &["auth status", "auth apps list"],
        type_name: "Vec<AppStatusEntry>",
        schema: generate::<Vec<AppStatusEntry>>,
    },
    SchemaEntry {
        commands: &["auth apps redirect-uri get"],
        type_name: "RedirectUriGetResponse",
        schema: generate::<RedirectUriGetResponse>,
    },
    SchemaEntry {
        commands: &["auth apps redirect-uri set"],
        type_name: "RedirectUriSetResponse",
        schema: generate::<RedirectUriSetResponse>,
    },
];

/// Commands clap parses that emit no typed response, so `xr schema` has no
/// document for them. Every command is in exactly one of this set and
/// `SCHEMA_ENTRIES`.
pub const SCHEMA_LESS_COMMANDS: &[&str] = &[
    "articles",
    "auth",
    "auth app",
    "auth apps",
    "auth apps add",
    "auth apps redirect-uri",
    "auth apps remove",
    "auth apps update",
    "auth clear",
    "auth default",
    "auth oauth1",
    "auth oauth2",
    "bookmark-folders",
    "broadcasts",
    "broadcasts chat",
    "broadcasts moderators",
    "broadcasts scheduled",
    "broadcasts sources",
    "calls",
    "calls scheduled",
    "communities",
    "conversations",
    "dm-events",
    "completions",
    "examples",
    "lists",
    "lists members",
    "media",
    "media status",
    "media subtitles",
    "media upload",
    "news",
    "notes",
    "schema",
    "skill",
    "skill install",
    "skill update",
    "spaces",
    "stream",
    "stream rules",
    "trends",
    "validate",
    "version",
    "webhooks",
    "webhooks listen",
    "webhooks stream-links",
    "webhooks subscriptions",
];

/// Every response type the registry names, one per row, in table order.
pub fn registered_types() -> impl Iterator<Item = &'static str> {
    SCHEMA_ENTRIES.iter().map(|entry| entry.type_name)
}

/// Every command name with a registry row, in table order.
pub fn registered_commands() -> impl Iterator<Item = &'static str> {
    SCHEMA_ENTRIES
        .iter()
        .flat_map(|entry| entry.commands.iter().copied())
}

/// The registered commands beneath the command at `path`, sorted, which the
/// refusal of a family with no schema of its own names.
#[must_use]
pub fn commands_beneath(path: &str) -> Vec<&'static str> {
    let prefix = format!("{path} ");
    let mut beneath: Vec<&str> = registered_commands()
        .filter(|command| command.starts_with(&prefix))
        .collect();
    beneath.sort_unstable();
    beneath
}

/// The file under `schema/responses/` that holds `command`'s schema, without
/// its `.schema.json` suffix: the words joined with `-`.
#[must_use]
pub fn file_stem(command: &str) -> String {
    command.replace(' ', "-")
}

/// The invocation that prints `command`'s schema.
#[must_use]
pub fn invocation(command: &str) -> String {
    format!("xr schema {command}")
}
