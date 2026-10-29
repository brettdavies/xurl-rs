//! Schema subcommand — outputs JSON Schema for command response types.

use std::collections::BTreeMap;
use std::io::Write;

use clap::CommandFactory;
use schemars::schema_for;
use serde_json::Value;

use crate::cli::Cli;
use crate::cli::classify::{ROOT_COMMAND, nearest};
use crate::cli::commands::auth::{AppStatusEntry, RedirectUriGetResponse, RedirectUriSetResponse};
use crate::cli::output::OutputConfig;
use crate::cli::skill_install::{InstallEnvelope, InstallMultiEnvelope};
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
use xdk::error::{Error, Result};

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
    SchemaEntry {
        commands: &["skill install"],
        type_name: "InstallEnvelope",
        schema: || schema_for!(InstallEnvelope).into(),
    },
    SchemaEntry {
        commands: &["skill install --all"],
        type_name: "InstallMultiEnvelope",
        schema: || schema_for!(InstallMultiEnvelope).into(),
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

/// Registry names that describe a flag form of a command rather than a
/// command of their own, each with the command it belongs to. The flag
/// reaches `xr schema` after `--`, where clap reads it as a word.
pub const FLAG_FORMS: &[(&str, &str)] = &[("skill install --all", "skill install")];

/// Names `xr schema` took before it took command words, which no row's
/// words joined with `-` reproduce: the old name, then the row it meant.
const IRREGULAR_OLD_NAMES: &[(&str, &str)] = &[
    ("redirect-uri-get", "auth apps redirect-uri get"),
    ("redirect-uri-set", "auth apps redirect-uri set"),
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
    command
        .split_whitespace()
        .map(|word| word.trim_start_matches('-'))
        .collect::<Vec<_>>()
        .join("-")
}

/// The invocation that prints `command`'s schema. A flag form's flag goes
/// after `--`, where clap reads it as a word rather than as `xr schema`'s
/// own flag.
#[must_use]
pub fn invocation(command: &str) -> String {
    let mut parts = vec!["xr", "schema"];
    let mut separated = false;
    for word in command.split_whitespace() {
        if word.starts_with('-') && !separated {
            parts.push("--");
            separated = true;
        }
        parts.push(word);
    }
    parts.join(" ")
}

/// The long flags `xr schema` declares, which name its scopes. A bare word
/// spelled like one is refused rather than read as a command.
#[must_use]
pub fn scope_words() -> Vec<String> {
    Cli::command()
        .find_subcommand("schema")
        .map(|schema| {
            schema
                .get_arguments()
                .filter_map(|arg| arg.get_long().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Why `xr schema` refused the words it was given.
#[derive(Debug)]
pub enum Refusal {
    /// A word names no command at its depth, as `xr help` reports it.
    UnknownCommand {
        /// The word that named nothing.
        word: String,
        /// The nearest command at that depth, when one scored close enough.
        suggestion: Option<String>,
        /// The command the word was typed under, `xr` at the root.
        under: String,
    },
    /// A name `xr schema` took before it took command words.
    OldName {
        /// The name as given.
        given: String,
        /// The registry row it meant.
        command: &'static str,
    },
    /// A bare word spelled like one of `xr schema`'s scope flags.
    ScopeWord(String),
    /// Anything else, refused as a value: no typed response, no arguments.
    Invalid(Error),
}

/// The registry row the words name.
fn resolve(words: &[String]) -> std::result::Result<&'static str, Refusal> {
    if let [word] = words
        && scope_words().iter().any(|scope| scope == word)
    {
        return Err(Refusal::ScopeWord(word.clone()));
    }
    let joined = words.join(" ");
    if let Some(command) = registered_commands().find(|command| *command == joined) {
        return Ok(command);
    }
    let root = Cli::command();
    let mut current = &root;
    let mut path: Vec<&str> = Vec::new();
    for word in words {
        let found = current
            .get_subcommands()
            .find(|sub| sub.get_name() == word || sub.get_all_aliases().any(|alias| alias == word));
        let Some(sub) = found else {
            if let [only] = words
                && let Some(command) = old_name(only)
            {
                return Err(Refusal::OldName {
                    given: only.clone(),
                    command,
                });
            }
            let candidates: Vec<String> = current
                .get_subcommands()
                .flat_map(|sub| {
                    std::iter::once(sub.get_name().to_string())
                        .chain(sub.get_all_aliases().map(String::from))
                })
                .collect();
            let under = std::iter::once(ROOT_COMMAND)
                .chain(path.iter().copied())
                .collect::<Vec<_>>()
                .join(" ");
            return Err(Refusal::UnknownCommand {
                word: word.clone(),
                suggestion: nearest(word, &candidates),
                under,
            });
        };
        path.push(sub.get_name());
        current = sub;
    }
    let named = path.join(" ");
    if let Some(command) = registered_commands().find(|command| *command == named) {
        return Ok(command);
    }
    Err(Refusal::Invalid(Error::validation_of(
        "no-typed-response",
        named,
    )))
}

/// The row an old hyphen-joined name meant, when `token` is one.
fn old_name(token: &str) -> Option<&'static str> {
    IRREGULAR_OLD_NAMES
        .iter()
        .find(|(old, _)| *old == token)
        .map(|(_, command)| *command)
        .or_else(|| {
            registered_commands()
                .find(|command| command.contains(' ') && file_stem(command) == token)
        })
}

/// Returns the JSON Schema for a registry row.
fn schema_for_command(command: &str) -> Result<Value> {
    SCHEMA_ENTRIES
        .iter()
        .find(|entry| entry.commands.contains(&command))
        .map(|entry| (entry.schema)())
        .ok_or_else(|| Error::Internal(format!("no schema registered for '{command}'")))
}

/// Runs the schema subcommand.
///
/// JSON-emitting paths route through `OutputConfig::print_response` so a
/// schema body is not double-wrapped in `{"message": "..."}` under
/// `--output json`. The `envelope` flag dumps the canonical output envelope
/// schema instead of a response-type schema.
///
/// # Errors
///
/// Returns a [`Refusal`] when the words name no command, name one with no
/// typed response, use an old name or a scope word, or nothing was asked.
pub fn run_schema(
    command: &[String],
    list: bool,
    all: bool,
    envelope: bool,
    out: &OutputConfig,
    stdout: &mut dyn Write,
) -> std::result::Result<(), Refusal> {
    if all {
        return print_all_schemas(out, stdout).map_err(Refusal::Invalid);
    }
    if envelope {
        let schema = crate::cli::envelope::envelope_schema();
        out.print_response(stdout, &schema);
        return Ok(());
    }
    if list {
        print_schema_list(out, stdout);
        return Ok(());
    }
    if command.is_empty() {
        return Err(Refusal::Invalid(Error::validation("missing-command")));
    }
    let row = resolve(command)?;
    let schema = schema_for_command(row).map_err(Refusal::Invalid)?;
    out.print_response(stdout, &schema);
    Ok(())
}

/// Writes every registered command with its response type and its file.
///
/// Text output aligns one row per command; the structured formats get one
/// record per command, which `scripts/generate-response-schemas.sh` reads.
fn print_schema_list(out: &OutputConfig, stdout: &mut dyn Write) {
    let mut entries: Vec<(&str, &str)> = SCHEMA_ENTRIES
        .iter()
        .flat_map(|entry| entry.commands.iter().map(|cmd| (*cmd, entry.type_name)))
        .collect();
    entries.sort_by_key(|(cmd, _)| *cmd);

    if out.format.is_structured() {
        let records: Vec<Value> = entries
            .iter()
            .map(|(cmd, type_name)| {
                serde_json::json!({
                    "command": cmd,
                    "type": type_name,
                    "file": format!("{}.schema.json", file_stem(cmd)),
                })
            })
            .collect();
        out.print_response(stdout, &Value::Array(records));
        return;
    }
    let max_cmd_len = entries.iter().map(|(cmd, _)| cmd.len()).max().unwrap_or(0);
    for (cmd, type_name) in &entries {
        out.print_message(stdout, &format!("{cmd:<max_cmd_len$}  {type_name}"));
    }
}

/// Writes all schemas as a single JSON object keyed by command name to `stdout`.
fn print_all_schemas(out: &OutputConfig, stdout: &mut dyn Write) -> Result<()> {
    let mut all: BTreeMap<String, Value> = BTreeMap::new();
    for command in registered_commands() {
        all.insert(command.to_string(), schema_for_command(command)?);
    }
    let value = serde_json::to_value(&all)?;
    out.print_response(stdout, &value);
    Ok(())
}
