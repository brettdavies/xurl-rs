//! Schema subcommand: JSON Schema for each command's response type.
//!
//! This module holds the registry, which names every command with a typed
//! response by its words. `words` turns the words a caller gives into a row,
//! and `run` prints what `xr schema` is asked for.

mod run;
mod words;

pub use run::run_schema;
pub use words::{Refusal, scope_words};

use schemars::schema_for;
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
    /// Builds the response type's JSON Schema. `schema_for!` needs the type
    /// at its expansion site, so each row carries its own.
    schema: fn() -> Value,
}

/// All command-to-type mappings, ordered by category.
const SCHEMA_ENTRIES: &[SchemaEntry] = &[
    SchemaEntry {
        commands: &["post", "reply", "quote", "read"],
        type_name: "ApiResponse<Post>",
        schema: || schema_for!(ApiResponse<Post>).into(),
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
        schema: || schema_for!(ApiResponse<Vec<Post>>).into(),
    },
    SchemaEntry {
        commands: &["whoami", "user"],
        type_name: "ApiResponse<User>",
        schema: || schema_for!(ApiResponse<User>).into(),
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
        schema: || schema_for!(ApiResponse<Vec<User>>).into(),
    },
    SchemaEntry {
        commands: &["like", "unlike"],
        type_name: "ApiResponse<LikedResult>",
        schema: || schema_for!(ApiResponse<LikedResult>).into(),
    },
    SchemaEntry {
        commands: &[
            "lists list",
            "lists followed",
            "lists memberships",
            "lists pinned",
        ],
        type_name: "ApiResponse<Vec<List>>",
        schema: || schema_for!(ApiResponse<Vec<List>>).into(),
    },
    SchemaEntry {
        commands: &["lists get", "lists add"],
        type_name: "ApiResponse<List>",
        schema: || schema_for!(ApiResponse<List>).into(),
    },
    SchemaEntry {
        commands: &["lists update"],
        type_name: "ApiResponse<ListUpdatedResult>",
        schema: || schema_for!(ApiResponse<ListUpdatedResult>).into(),
    },
    SchemaEntry {
        commands: &["lists members add", "lists members remove"],
        type_name: "ApiResponse<ListMemberResult>",
        schema: || schema_for!(ApiResponse<ListMemberResult>).into(),
    },
    SchemaEntry {
        commands: &["lists pin", "lists unpin"],
        type_name: "ApiResponse<PinnedResult>",
        schema: || schema_for!(ApiResponse<PinnedResult>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts list"],
        type_name: "ApiResponse<Vec<Broadcast>>",
        schema: || schema_for!(ApiResponse<Vec<Broadcast>>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts get"],
        type_name: "ApiResponse<Broadcast>",
        schema: || schema_for!(ApiResponse<Broadcast>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts scheduled list"],
        type_name: "ApiResponse<Vec<ScheduledBroadcast>>",
        schema: || schema_for!(ApiResponse<Vec<ScheduledBroadcast>>).into(),
    },
    SchemaEntry {
        commands: &[
            "broadcasts scheduled get",
            "broadcasts scheduled add",
            "broadcasts scheduled update",
        ],
        type_name: "ApiResponse<ScheduledBroadcast>",
        schema: || schema_for!(ApiResponse<ScheduledBroadcast>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts chat history"],
        type_name: "ApiResponse<Vec<BroadcastChatMessage>>",
        schema: || schema_for!(ApiResponse<Vec<BroadcastChatMessage>>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts chat send"],
        type_name: "ApiResponse<ChatSentResult>",
        schema: || schema_for!(ApiResponse<ChatSentResult>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts chat mute", "broadcasts chat unmute"],
        type_name: "ApiResponse<ChatMutedResult>",
        schema: || schema_for!(ApiResponse<ChatMutedResult>).into(),
    },
    SchemaEntry {
        commands: &["media get"],
        type_name: "ApiResponse<Vec<Media>>",
        schema: || schema_for!(ApiResponse<Vec<Media>>).into(),
    },
    SchemaEntry {
        commands: &["media analytics"],
        type_name: "ApiResponse<Vec<MediaAnalytics>>",
        schema: || schema_for!(ApiResponse<Vec<MediaAnalytics>>).into(),
    },
    SchemaEntry {
        commands: &["spaces search", "spaces get", "spaces by-creator"],
        type_name: "ApiResponse<Vec<Space>>",
        schema: || schema_for!(ApiResponse<Vec<Space>>).into(),
    },
    SchemaEntry {
        commands: &["stream rules list"],
        type_name: "ApiResponse<Vec<StreamRule>>",
        schema: || schema_for!(ApiResponse<Vec<StreamRule>>).into(),
    },
    SchemaEntry {
        commands: &["stream rules add", "stream rules remove"],
        type_name: "ApiResponse<Option<Vec<StreamRule>>>",
        schema: || schema_for!(ApiResponse<Option<Vec<StreamRule>>>).into(),
    },
    SchemaEntry {
        commands: &["stream rules count"],
        type_name: "ApiResponse<StreamRuleCounts>",
        schema: || schema_for!(ApiResponse<StreamRuleCounts>).into(),
    },
    SchemaEntry {
        commands: &["articles draft"],
        type_name: "ApiResponse<ArticleDraft>",
        schema: || schema_for!(ApiResponse<ArticleDraft>).into(),
    },
    SchemaEntry {
        commands: &["articles publish"],
        type_name: "ApiResponse<ArticlePublished>",
        schema: || schema_for!(ApiResponse<ArticlePublished>).into(),
    },
    SchemaEntry {
        commands: &["notes write"],
        type_name: "ApiResponse<NoteCreated>",
        schema: || schema_for!(ApiResponse<NoteCreated>).into(),
    },
    SchemaEntry {
        commands: &["notes evaluate"],
        type_name: "ApiResponse<NoteEvaluation>",
        schema: || schema_for!(ApiResponse<NoteEvaluation>).into(),
    },
    SchemaEntry {
        commands: &["notes written"],
        type_name: "ApiResponse<Vec<Note>>",
        schema: || schema_for!(ApiResponse<Vec<Note>>).into(),
    },
    SchemaEntry {
        commands: &["communities search"],
        type_name: "ApiResponse<Vec<Community>>",
        schema: || schema_for!(ApiResponse<Vec<Community>>).into(),
    },
    SchemaEntry {
        commands: &["communities get"],
        type_name: "ApiResponse<Community>",
        schema: || schema_for!(ApiResponse<Community>).into(),
    },
    SchemaEntry {
        commands: &["news search"],
        type_name: "ApiResponse<Vec<NewsStory>>",
        schema: || schema_for!(ApiResponse<Vec<NewsStory>>).into(),
    },
    SchemaEntry {
        commands: &["news get"],
        type_name: "ApiResponse<NewsStory>",
        schema: || schema_for!(ApiResponse<NewsStory>).into(),
    },
    SchemaEntry {
        commands: &["trends place"],
        type_name: "ApiResponse<Vec<Trend>>",
        schema: || schema_for!(ApiResponse<Vec<Trend>>).into(),
    },
    SchemaEntry {
        commands: &["trends personal"],
        type_name: "ApiResponse<Vec<PersonalizedTrend>>",
        schema: || schema_for!(ApiResponse<Vec<PersonalizedTrend>>).into(),
    },
    SchemaEntry {
        commands: &["analytics"],
        type_name: "ApiResponse<Vec<PostAnalytics>>",
        schema: || schema_for!(ApiResponse<Vec<PostAnalytics>>).into(),
    },
    SchemaEntry {
        commands: &["search-counts"],
        type_name: "ApiResponse<Vec<PostCount>>",
        schema: || schema_for!(ApiResponse<Vec<PostCount>>).into(),
    },
    SchemaEntry {
        commands: &["hide", "unhide"],
        type_name: "ApiResponse<HiddenResult>",
        schema: || schema_for!(ApiResponse<HiddenResult>).into(),
    },
    SchemaEntry {
        commands: &["follow", "unfollow", "lists follow", "lists unfollow"],
        type_name: "ApiResponse<FollowingResult>",
        schema: || schema_for!(ApiResponse<FollowingResult>).into(),
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
        schema: || schema_for!(ApiResponse<DeletedResult>).into(),
    },
    SchemaEntry {
        commands: &["repost", "unrepost"],
        type_name: "ApiResponse<RepostedResult>",
        schema: || schema_for!(ApiResponse<RepostedResult>).into(),
    },
    SchemaEntry {
        commands: &["bookmark", "unbookmark"],
        type_name: "ApiResponse<BookmarkedResult>",
        schema: || schema_for!(ApiResponse<BookmarkedResult>).into(),
    },
    SchemaEntry {
        commands: &["mute", "unmute"],
        type_name: "ApiResponse<MutingResult>",
        schema: || schema_for!(ApiResponse<MutingResult>).into(),
    },
    SchemaEntry {
        commands: &["block", "unblock"],
        type_name: "ApiResponse<BlockingResult>",
        schema: || schema_for!(ApiResponse<BlockingResult>).into(),
    },
    SchemaEntry {
        commands: &["dm", "conversations start", "conversations send"],
        type_name: "ApiResponse<DmSentResult>",
        schema: || schema_for!(ApiResponse<DmSentResult>).into(),
    },
    SchemaEntry {
        commands: &["dms", "conversations history", "conversations with"],
        type_name: "ApiResponse<Vec<DmEvent>>",
        schema: || schema_for!(ApiResponse<Vec<DmEvent>>).into(),
    },
    SchemaEntry {
        commands: &["dm-events get"],
        type_name: "ApiResponse<DmEvent>",
        schema: || schema_for!(ApiResponse<DmEvent>).into(),
    },
    SchemaEntry {
        commands: &["dm-block", "dm-unblock"],
        type_name: "ApiResponse<DmBlockResult>",
        schema: || schema_for!(ApiResponse<DmBlockResult>).into(),
    },
    SchemaEntry {
        commands: &["usage"],
        type_name: "ApiResponse<UsageData>",
        schema: || schema_for!(ApiResponse<UsageData>).into(),
    },
    SchemaEntry {
        commands: &["usage credits"],
        type_name: "ApiResponse<UsageCreditsData>",
        schema: || schema_for!(ApiResponse<UsageCreditsData>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts moderators list"],
        type_name: "ApiResponse<Vec<User>>",
        schema: || schema_for!(ApiResponse<Vec<User>>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts moderators add", "broadcasts moderators remove"],
        type_name: "ApiResponse<ChatModeratorsResult>",
        schema: || schema_for!(ApiResponse<ChatModeratorsResult>).into(),
    },
    SchemaEntry {
        commands: &["bookmark-folders list"],
        type_name: "ApiResponse<Vec<BookmarkFolder>>",
        schema: || schema_for!(ApiResponse<Vec<BookmarkFolder>>).into(),
    },
    SchemaEntry {
        commands: &["bookmark-folders add"],
        type_name: "ApiResponse<BookmarkFolder>",
        schema: || schema_for!(ApiResponse<BookmarkFolder>).into(),
    },
    SchemaEntry {
        commands: &["bookmark-folders posts"],
        type_name: "ApiResponse<Vec<BookmarkFolderPost>>",
        schema: || schema_for!(ApiResponse<Vec<BookmarkFolderPost>>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts sources list"],
        type_name: "ApiResponse<StreamSources>",
        schema: || schema_for!(ApiResponse<StreamSources>).into(),
    },
    SchemaEntry {
        commands: &[
            "broadcasts sources get",
            "broadcasts sources add",
            "broadcasts sources rename",
        ],
        type_name: "ApiResponse<StreamSourceResult>",
        schema: || schema_for!(ApiResponse<StreamSourceResult>).into(),
    },
    SchemaEntry {
        commands: &["broadcasts sources remove"],
        type_name: "ApiResponse<StreamSourceDeleted>",
        schema: || schema_for!(ApiResponse<StreamSourceDeleted>).into(),
    },
    SchemaEntry {
        commands: &["calls get"],
        type_name: "ApiResponse<XCall>",
        schema: || schema_for!(ApiResponse<XCall>).into(),
    },
    SchemaEntry {
        commands: &["calls scheduled list"],
        type_name: "ApiResponse<Vec<ScheduledCall>>",
        schema: || schema_for!(ApiResponse<Vec<ScheduledCall>>).into(),
    },
    SchemaEntry {
        commands: &["calls scheduled get", "calls scheduled add"],
        type_name: "ApiResponse<ScheduledCall>",
        schema: || schema_for!(ApiResponse<ScheduledCall>).into(),
    },
    SchemaEntry {
        commands: &["calls scheduled update"],
        type_name: "ApiResponse<ScheduledCallUpdated>",
        schema: || schema_for!(ApiResponse<ScheduledCallUpdated>).into(),
    },
    SchemaEntry {
        commands: &["calls scheduled cancel"],
        type_name: "ApiResponse<ScheduledCallCancelled>",
        schema: || schema_for!(ApiResponse<ScheduledCallCancelled>).into(),
    },
    SchemaEntry {
        commands: &["calls scheduled rsvp"],
        type_name: "ApiResponse<ScheduledCallRsvp>",
        schema: || schema_for!(ApiResponse<ScheduledCallRsvp>).into(),
    },
    SchemaEntry {
        commands: &["webhooks list"],
        type_name: "ApiResponse<Vec<Webhook>>",
        schema: || schema_for!(ApiResponse<Vec<Webhook>>).into(),
    },
    SchemaEntry {
        commands: &["webhooks add"],
        type_name: "ApiResponse<Webhook>",
        schema: || schema_for!(ApiResponse<Webhook>).into(),
    },
    SchemaEntry {
        commands: &["webhooks validate"],
        type_name: "ApiResponse<WebhookValidation>",
        schema: || schema_for!(ApiResponse<WebhookValidation>).into(),
    },
    SchemaEntry {
        commands: &["webhooks subscriptions count"],
        type_name: "ApiResponse<AccountActivitySubscriptionCount>",
        schema: || schema_for!(ApiResponse<AccountActivitySubscriptionCount>).into(),
    },
    SchemaEntry {
        commands: &["webhooks subscriptions list"],
        type_name: "ApiResponse<AccountActivitySubscriptions>",
        schema: || schema_for!(ApiResponse<AccountActivitySubscriptions>).into(),
    },
    SchemaEntry {
        commands: &[
            "webhooks subscriptions add",
            "webhooks subscriptions check",
            "webhooks subscriptions remove",
        ],
        type_name: "ApiResponse<SubscribedResult>",
        schema: || schema_for!(ApiResponse<SubscribedResult>).into(),
    },
    SchemaEntry {
        commands: &["webhooks stream-links list"],
        type_name: "ApiResponse<Vec<WebhookStreamLink>>",
        schema: || schema_for!(ApiResponse<Vec<WebhookStreamLink>>).into(),
    },
    SchemaEntry {
        commands: &["webhooks stream-links add"],
        type_name: "ApiResponse<ProvisionedResult>",
        schema: || schema_for!(ApiResponse<ProvisionedResult>).into(),
    },
    SchemaEntry {
        commands: &["webhooks replay"],
        type_name: "ApiResponse<WebhookReplayJob>",
        schema: || schema_for!(ApiResponse<WebhookReplayJob>).into(),
    },
    SchemaEntry {
        commands: &["media alt-text"],
        type_name: "ApiResponse<MediaMetadataResult>",
        schema: || schema_for!(ApiResponse<MediaMetadataResult>).into(),
    },
    SchemaEntry {
        commands: &["media subtitles add"],
        type_name: "ApiResponse<MediaSubtitlesResult>",
        schema: || schema_for!(ApiResponse<MediaSubtitlesResult>).into(),
    },
    SchemaEntry {
        commands: &["auth status", "auth apps list"],
        type_name: "Vec<AppStatusEntry>",
        schema: || schema_for!(Vec<AppStatusEntry>).into(),
    },
    SchemaEntry {
        commands: &["auth apps redirect-uri get"],
        type_name: "RedirectUriGetResponse",
        schema: || schema_for!(RedirectUriGetResponse).into(),
    },
    SchemaEntry {
        commands: &["auth apps redirect-uri set"],
        type_name: "RedirectUriSetResponse",
        schema: || schema_for!(RedirectUriSetResponse).into(),
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
