//! API shortcut functions — high-level X API v2 operations.
//!
//! Each method maps to one shortcut command, building the appropriate
//! endpoint target and request body into a [`Call`] the caller configures
//! and sends. Paths are spec-shaped templates (e.g.
//! `/2/users/{id}/likes`) so the auth-matrix validator can key on the same
//! string the build-time codegen ingests.

use std::collections::HashMap;

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::auth_matrix::endpoints;
use super::request::{Call, Client, RequestOptions, RequestTarget};
use super::response::types::{
    AccountActivitySubscriptionCount, AccountActivitySubscriptions, ApiResponse, BlockingResult,
    BookmarkFolder, BookmarkFolderPost, BookmarkedResult, ChatModeratorsResult, DeletedResult,
    DmEvent, DmSentResult, FollowingResult, HiddenResult, LikedResult, MediaMetadataResult,
    MediaSubtitlesResult, MutingResult, NewsStory, PersonalizedTrend, Post, PostAnalytics,
    PostCount, ProvisionedResult, RepostedResult, ScheduledCall, ScheduledCallCancelled,
    ScheduledCallRsvp, ScheduledCallUpdated, StreamSourceDeleted, StreamSourceResult,
    StreamSources, SubscribedResult, Trend, UsageCreditsData, UsageData, User, Webhook,
    WebhookReplayJob, WebhookStreamLink, WebhookValidation, XCall,
};

// ── Request body types ───────────────────────────────────────────────

#[derive(Serialize)]
struct WebhookBody {
    url: String,
}

#[derive(Serialize)]
struct WebhookReplayBody {
    webhook_id: String,
    from_date: String,
    to_date: String,
}

#[derive(Serialize)]
struct PostBody {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<PostReply>,
    #[serde(rename = "quote_tweet_id", skip_serializing_if = "Option::is_none")]
    quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media: Option<PostMedia>,
}

#[derive(Serialize)]
struct PostReply {
    #[serde(rename = "in_reply_to_tweet_id")]
    in_reply_to_post_id: String,
}

#[derive(Serialize)]
struct PostMedia {
    media_ids: Vec<String>,
}

#[derive(Serialize)]
struct ChatModeratorBody {
    user_id: String,
}

#[derive(Serialize)]
struct StreamSourceBody {
    name: String,
    region: String,
}

#[derive(Serialize)]
struct NameBody {
    name: String,
}

#[derive(Serialize)]
struct HiddenBody {
    hidden: bool,
}

#[derive(Serialize)]
struct MediaMetadataBody {
    id: String,
    metadata: MediaMetadata,
}

#[derive(Serialize)]
struct MediaMetadata {
    alt_text: AltText,
}

#[derive(Serialize)]
struct AltText {
    text: String,
}

#[derive(Serialize)]
struct AddSubtitlesBody {
    id: String,
    media_category: &'static str,
    subtitles: SubtitleTrack,
}

#[derive(Serialize)]
struct SubtitleTrack {
    id: String,
    language_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
}

#[derive(Serialize)]
struct RemoveSubtitlesBody {
    id: String,
    media_category: &'static str,
    language_code: String,
}

#[derive(Serialize)]
struct RsvpBody<'a> {
    response: RsvpAnswer,
    #[serde(flatten)]
    occurrences: &'a Occurrences,
}

/// Which occurrences of a recurring scheduled call a change, cancellation,
/// or answer applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OccurrenceScope {
    /// Every occurrence: what X assumes when no occurrence is named.
    All,
    /// The named occurrence alone: what X assumes when one is named.
    This,
    /// The named occurrence and every one after it.
    ThisAndFollowing,
}

impl OccurrenceScope {
    /// Every scope, in the order help text lists them.
    pub const ALL: [Self; 3] = [Self::All, Self::This, Self::ThisAndFollowing];

    /// The scope as the API spells it, such as `this_and_following`.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::This => "this",
            Self::ThisAndFollowing => "this_and_following",
        }
    }
}

/// The occurrences of a recurring scheduled call a request names. Left
/// empty, it names every occurrence.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Occurrences {
    /// Which occurrences, counted from `occurrence_start_time`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<OccurrenceScope>,
    /// The original start of the occurrence `scope` counts from, RFC 3339,
    /// as a listed call's `occurrence_start_time` gives it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_start_time: Option<String>,
}

impl Occurrences {
    fn query(&self) -> Vec<(String, String)> {
        let mut query = Vec::new();
        if let Some(scope) = self.scope {
            query.push(("scope".to_string(), scope.wire_name().to_string()));
        }
        if let Some(start) = &self.occurrence_start_time {
            query.push(("occurrence_start_time".to_string(), start.clone()));
        }
        query
    }
}

/// An answer to a scheduled-call invitation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RsvpAnswer {
    /// The invitee will attend.
    Accepted,
    /// The invitee will not attend.
    Declined,
    /// The invitee might attend.
    Tentative,
}

impl RsvpAnswer {
    /// The answer as the API spells it, such as `tentative`.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Declined => "declined",
            Self::Tentative => "tentative",
        }
    }
}

/// A call to put on the authenticated user's X Calls calendar, with them as
/// its organizer. A field left unset takes X's default.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct NewScheduledCall {
    /// When the call starts, RFC 3339 with a zone; not in the past.
    pub start_time: String,
    /// The title invitees see.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// What the call is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// How long the call lasts, in minutes; X assumes 30.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<u32>,
    /// User ids of the people to invite.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub invitee_ids: Vec<String>,
    /// The RFC 5545 RRULE to repeat the call by, such as
    /// `FREQ=WEEKLY;BYDAY=MO`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<String>,
    /// The IANA time zone the recurrence follows, such as
    /// `America/Los_Angeles`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    /// Whether invitees get an email about the call; X assumes they do.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_invitees: Option<bool>,
}

/// What to change on a scheduled call. A field left unset keeps its value;
/// `invitee_ids`, when set, replaces the whole invite list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ScheduledCallChanges {
    /// When the call, or the named occurrence, now starts, RFC 3339.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// The title invitees see.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// What the call is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// How long the call lasts, in minutes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<u32>,
    /// Everyone to invite, replacing the invite list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitee_ids: Option<Vec<String>>,
    /// The RFC 5545 RRULE to repeat the call by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<String>,
    /// The IANA time zone the recurrence follows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    /// Whether invitees get an email about the change; X assumes they do.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_invitees: Option<bool>,
    /// Which occurrences of a recurring call the change applies to.
    #[serde(flatten)]
    pub occurrences: Occurrences,
}

impl ScheduledCallChanges {
    /// Whether the change set leaves the call as it is: no field it would
    /// change is set. Naming occurrences or the email setting changes
    /// nothing on its own.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start_time.is_none()
            && self.title.is_none()
            && self.description.is_none()
            && self.duration_minutes.is_none()
            && self.invitee_ids.is_none()
            && self.recurrence.is_none()
            && self.time_zone.is_none()
    }
}

/// The upload category of a video that carries subtitles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCategory {
    /// A video uploaded with `media_category=amplify_video`.
    AmplifyVideo,
    /// A video uploaded with `media_category=tweet_video`.
    TweetVideo,
}

impl VideoCategory {
    /// Every category, in the order help text lists them.
    pub const ALL: [Self; 2] = [Self::AmplifyVideo, Self::TweetVideo];

    /// The `media_category` the video was uploaded with, such as
    /// `amplify_video`.
    #[must_use]
    pub const fn upload_name(self) -> &'static str {
        match self {
            Self::AmplifyVideo => "amplify_video",
            Self::TweetVideo => "tweet_video",
        }
    }

    /// The category from its upload name, the inverse of
    /// [`Self::upload_name`].
    #[must_use]
    pub fn from_upload_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.upload_name() == name)
    }

    /// The form the subtitles endpoints take, such as `AmplifyVideo`.
    const fn wire_name(self) -> &'static str {
        match self {
            Self::AmplifyVideo => "AmplifyVideo",
            Self::TweetVideo => "TweetVideo",
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Extracts a post ID from a full URL or returns the input as-is.
#[must_use]
pub fn resolve_post_id(input: &str) -> String {
    let input = input.trim();

    if (input.starts_with("http://") || input.starts_with("https://"))
        && let Ok(parsed) = url::Url::parse(input)
    {
        let parts: Vec<&str> = parsed.path().trim_matches('/').split('/').collect();
        for (i, p) in parts.iter().enumerate() {
            if *p == "status" && i + 1 < parts.len() {
                return parts[i + 1].to_string();
            }
        }
    }

    input.to_string()
}

/// Normalizes a username — strips a leading "@" if present.
#[must_use]
pub fn resolve_username(input: &str) -> String {
    input.trim().trim_start_matches('@').to_string()
}

// ── Write-op validators ─────────────────────────────────────────
//
// Each validator returns `Ok(())` when the inputs would be accepted by the
// API and `Err(reason)` with a kebab-case reason otherwise. Callers compose
// these into the canonical dry-run envelope without issuing HTTP.

/// The post length X documents for a standard account.
#[deprecated(
    note = "X sets a post's length limit per account, so `validate_post_body` no longer checks it"
)]
pub const POST_BODY_MAX_CHARS: usize = 280;

/// X API media attachment cap per post.
pub const POST_MEDIA_MAX: usize = 4;

/// The most ids one lookup takes, the spec's `maxItems` on `ids`.
pub const LOOKUP_IDS_MAX: usize = 100;

/// The longest user search query, from the spec's pattern on `query`.
pub const USER_SEARCH_QUERY_MAX_CHARS: usize = 50;

/// Validates a post / reply / quote body.
///
/// Length is left to X: the limit belongs to the account (a Premium account
/// posts past 280 characters) and X counts a URL as 23, so a character count
/// taken here refuses posts X accepts.
///
/// # Errors
/// Returns the kebab-case reason `empty-body`.
pub fn validate_post_body(text: &str) -> std::result::Result<(), &'static str> {
    if text.is_empty() {
        return Err("empty-body");
    }
    Ok(())
}

/// Validates a media-attachment list for a post.
///
/// # Errors
/// Returns `too-many-attachments` when more than [`POST_MEDIA_MAX`] are passed.
pub fn validate_media_attachments(ids: &[String]) -> std::result::Result<(), &'static str> {
    if ids.len() > POST_MEDIA_MAX {
        return Err("too-many-attachments");
    }
    Ok(())
}

/// Validates the number of ids a batch lookup sends.
///
/// # Errors
/// Returns `too-many-ids` when more than [`LOOKUP_IDS_MAX`] are passed.
pub fn validate_lookup_ids<S: AsRef<str>>(ids: &[S]) -> std::result::Result<(), &'static str> {
    if ids.len() > LOOKUP_IDS_MAX {
        return Err("too-many-ids");
    }
    Ok(())
}

/// Validates a user search query against the spec's pattern: 1 to
/// [`USER_SEARCH_QUERY_MAX_CHARS`] letters, digits, underscores, apostrophes,
/// or spaces.
///
/// # Errors
/// Returns `invalid-user-query` for a query outside that pattern.
pub fn validate_user_search_query(query: &str) -> std::result::Result<(), &'static str> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '\'' | ' ');
    if query.is_empty()
        || query.chars().count() > USER_SEARCH_QUERY_MAX_CHARS
        || !query.chars().all(allowed)
    {
        return Err("invalid-user-query");
    }
    Ok(())
}

/// Validates a news story id against the spec's pattern: 1 to 19 digits.
///
/// # Errors
/// Returns `invalid-news-id` for anything else.
pub fn validate_news_id(id: &str) -> std::result::Result<(), &'static str> {
    if id.is_empty() || id.len() > 19 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid-news-id");
    }
    Ok(())
}

/// Validates a DM body.
///
/// # Errors
/// Returns `empty-body` for an empty string. (DM length is enforced server-side.)
pub fn validate_dm_body(text: &str) -> std::result::Result<(), &'static str> {
    if text.is_empty() {
        return Err("empty-body");
    }
    Ok(())
}

/// Validates a username target (strips a leading `@` first).
///
/// # Errors
/// Returns `empty-username` when the trimmed input is empty.
pub fn validate_target_username(input: &str) -> std::result::Result<(), &'static str> {
    if resolve_username(input).is_empty() {
        return Err("empty-username");
    }
    Ok(())
}

/// Validates a post identifier (URL or ID).
///
/// # Errors
/// Returns `empty-post-id` when the resolver yields the empty string.
pub fn validate_post_id(input: &str) -> std::result::Result<(), &'static str> {
    if resolve_post_id(input).is_empty() {
        return Err("empty-post-id");
    }
    Ok(())
}

/// X API alt text length budget, the spec's `maxLength` on `alt_text.text`.
pub const ALT_TEXT_MAX_CHARS: usize = 1000;

/// Validates a media id: one to nineteen ASCII digits, the spec's pattern.
///
/// # Errors
/// Returns `invalid-media-id` for anything else, the empty string included.
pub fn validate_media_id(input: &str) -> std::result::Result<(), &'static str> {
    let id = input.trim();
    if id.is_empty() || id.len() > 19 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid-media-id");
    }
    Ok(())
}

/// Longest webhook URL X accepts, the spec's `maxLength` on the `url` of a
/// webhook registration.
pub const WEBHOOK_URL_MAX_CHARS: usize = 200;

/// Validates a webhook id: one to nineteen ASCII digits, the spec's pattern.
///
/// # Errors
/// Returns `invalid-webhook-id` for anything else, the empty string included.
pub fn validate_webhook_id(input: &str) -> std::result::Result<(), &'static str> {
    if input.is_empty() || input.len() > 19 || !input.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid-webhook-id");
    }
    Ok(())
}

/// Validates a user id: one to nineteen ASCII digits, the spec's pattern.
///
/// # Errors
/// Returns `invalid-user-id` for anything else, a handle included.
pub fn validate_user_id(input: &str) -> std::result::Result<(), &'static str> {
    if input.is_empty() || input.len() > 19 || !input.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid-user-id");
    }
    Ok(())
}

/// Validates a webhook URL against the spec's length bounds and the two
/// rules X's webhook documentation gives: the URL uses HTTPS, and it names
/// no port.
///
/// # Errors
/// Returns the kebab-case reason: `empty-webhook-url`, `webhook-url-too-long`,
/// `webhook-url-not-https`, `webhook-url-has-port`.
pub fn validate_webhook_url(url: &str) -> std::result::Result<(), &'static str> {
    if url.is_empty() {
        return Err("empty-webhook-url");
    }
    if url.chars().count() > WEBHOOK_URL_MAX_CHARS {
        return Err("webhook-url-too-long");
    }
    let Some(rest) = url.strip_prefix("https://") else {
        return Err("webhook-url-not-https");
    };
    if url::Url::parse(url).is_err() {
        return Err("webhook-url-not-https");
    }
    // The authority as written: `Url` drops a port that is the scheme's
    // default, and X refuses one that is spelled out.
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_and_port = authority.rsplit('@').next().unwrap_or_default();
    let has_port = match host_and_port.rsplit_once(']') {
        Some((_, after_bracket)) => after_bracket.starts_with(':'),
        None => host_and_port.contains(':'),
    };
    if has_port {
        return Err("webhook-url-has-port");
    }
    Ok(())
}

/// Validates one bound of a webhook replay window: twelve ASCII digits,
/// `yyyymmddhhmm` in UTC, the spec's pattern.
///
/// # Errors
/// Returns `invalid-replay-time` for anything else.
pub fn validate_replay_time(input: &str) -> std::result::Result<(), &'static str> {
    if input.len() != 12 || !input.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid-replay-time");
    }
    Ok(())
}

/// Validates a stream source's name: at least one character that is not
/// whitespace, the spec's `minLength`.
///
/// # Errors
/// Returns `empty-source-name` for an empty or blank name.
pub fn validate_source_name(name: &str) -> std::result::Result<(), &'static str> {
    if name.trim().is_empty() {
        return Err("empty-source-name");
    }
    Ok(())
}

/// Validates a stream source's ingest region: not empty, the spec's
/// `minLength`. The spec lists the regions in prose rather than as an enum,
/// so a region X adds later is not refused here.
///
/// # Errors
/// Returns `empty-source-region` for an empty or blank region.
pub fn validate_source_region(region: &str) -> std::result::Result<(), &'static str> {
    if region.trim().is_empty() {
        return Err("empty-source-region");
    }
    Ok(())
}

/// Longest bookmark folder name X accepts, the spec's `maxLength`.
pub const BOOKMARK_FOLDER_NAME_MAX_CHARS: usize = 25;

/// Validates a bookmark folder's name: at least one character that is not
/// whitespace, the spec's `minLength`, and at most
/// [`BOOKMARK_FOLDER_NAME_MAX_CHARS`], its `maxLength`.
///
/// # Errors
/// Returns `empty-folder-name` for an empty or blank name and
/// `folder-name-too-long` for a longer one.
pub fn validate_bookmark_folder_name(name: &str) -> std::result::Result<(), &'static str> {
    if name.trim().is_empty() {
        return Err("empty-folder-name");
    }
    if name.chars().count() > BOOKMARK_FOLDER_NAME_MAX_CHARS {
        return Err("folder-name-too-long");
    }
    Ok(())
}

/// Validates a bookmark folder id against the spec's pattern: 1 to 19
/// digits.
///
/// # Errors
/// Returns `invalid-folder-id` for anything else.
pub fn validate_bookmark_folder_id(id: &str) -> std::result::Result<(), &'static str> {
    if id.is_empty() || id.len() > 19 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid-folder-id");
    }
    Ok(())
}

/// Longest scheduled-call title X accepts, the spec's `maxLength`.
pub const CALL_TITLE_MAX_CHARS: usize = 100;

/// Longest scheduled-call description X accepts, the spec's `maxLength`.
pub const CALL_DESCRIPTION_MAX_CHARS: usize = 1000;

/// Longest recurrence rule X accepts, the spec's `maxLength`.
pub const CALL_RECURRENCE_MAX_CHARS: usize = 500;

/// Longest time zone name X accepts, the spec's `maxLength`.
pub const CALL_TIME_ZONE_MAX_CHARS: usize = 64;

/// Longest a scheduled call may last, in minutes: 31 days, the spec's
/// `maximum`.
pub const CALL_DURATION_MAX_MINUTES: u32 = 44_640;

/// Most people a new scheduled call may invite, the spec's `maxItems`.
pub const NEW_CALL_INVITEES_MAX: usize = 25;

/// Most people an updated invite list may hold, the spec's `maxItems`.
pub const CHANGED_CALL_INVITEES_MAX: usize = 100;

/// Validates a call time against the spec's pattern: RFC 3339 with seconds,
/// up to nine fractional digits, and a `Z` or `±hh:mm` zone.
///
/// # Errors
/// Returns `invalid-call-time` for anything else.
pub fn validate_call_time(input: &str) -> std::result::Result<(), &'static str> {
    if is_call_time(input.as_bytes()) {
        Ok(())
    } else {
        Err("invalid-call-time")
    }
}

/// Validates an optional time window: each bound given is RFC 3339 with
/// seconds and a zone, as [`validate_call_time`] takes, and the start comes
/// before the end when both are given.
///
/// # Errors
/// Returns `invalid-time-window` for a bound outside that form, or a start
/// at or after the end.
pub fn validate_time_window(
    start: Option<&str>,
    end: Option<&str>,
) -> std::result::Result<(), &'static str> {
    let instant = |bound: Option<&str>| {
        bound
            .map(|time| rfc3339_instant(time.as_bytes()).ok_or("invalid-time-window"))
            .transpose()
    };
    if let (Some(start), Some(end)) = (instant(start)?, instant(end)?)
        && start >= end
    {
        return Err("invalid-time-window");
    }
    Ok(())
}

/// The moment a time in [`is_call_time`]'s form names, as seconds since the
/// Unix epoch and nanoseconds, so two times in different zones compare.
fn rfc3339_instant(bytes: &[u8]) -> Option<(i64, u32)> {
    if !is_call_time(bytes) {
        return None;
    }
    let number = |digits: &[u8]| {
        digits
            .iter()
            .fold(0_i64, |n, digit| n * 10 + i64::from(digit - b'0'))
    };
    let days = days_from_civil(
        number(&bytes[..4]),
        number(&bytes[5..7]),
        number(&bytes[8..10]),
    );
    let seconds = days * 86_400
        + number(&bytes[11..13]) * 3_600
        + number(&bytes[14..16]) * 60
        + number(&bytes[17..19]);
    let mut zone = &bytes[19..];
    let mut nanos = 0_u32;
    if let Some(fraction) = zone.strip_prefix(b".") {
        let digits = fraction.iter().take_while(|b| b.is_ascii_digit()).count();
        nanos = fraction[..digits]
            .iter()
            .chain(std::iter::repeat(&b'0'))
            .take(9)
            .fold(0, |n, digit| n * 10 + u32::from(digit - b'0'));
        zone = &fraction[digits..];
    }
    let offset = match zone {
        [sign @ (b'+' | b'-'), ..] => {
            let minutes = number(&zone[1..3]) * 60 + number(&zone[4..6]);
            if *sign == b'+' {
                minutes * 60
            } else {
                -minutes * 60
            }
        }
        _ => 0,
    };
    Some((seconds - offset, nanos))
}

/// Days from 1970-01-01 to a proleptic Gregorian date, after Howard
/// Hinnant's `days_from_civil`
/// (<https://howardhinnant.github.io/date_algorithms.html#days_from_civil>).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn two_digits(bytes: &[u8], at: usize) -> Option<u8> {
    match bytes.get(at..at + 2)? {
        [tens, units] if tens.is_ascii_digit() && units.is_ascii_digit() => {
            Some((tens - b'0') * 10 + (units - b'0'))
        }
        _ => None,
    }
}

fn is_call_time(bytes: &[u8]) -> bool {
    let year = bytes
        .get(..4)
        .is_some_and(|y| y.iter().all(u8::is_ascii_digit));
    let separators = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')]
        .iter()
        .all(|&(at, byte)| bytes.get(at) == Some(&byte));
    let fields = [
        (5, 1, 12),
        (8, 1, 31),
        (11, 0, 23),
        (14, 0, 59),
        (17, 0, 59),
    ]
    .iter()
    .all(|&(at, low, high)| two_digits(bytes, at).is_some_and(|n| (low..=high).contains(&n)));
    if !(year && separators && fields) {
        return false;
    }
    let mut zone = &bytes[19..];
    if let Some(fraction) = zone.strip_prefix(b".") {
        let digits = fraction.iter().take_while(|b| b.is_ascii_digit()).count();
        if !(1..=9).contains(&digits) {
            return false;
        }
        zone = &fraction[digits..];
    }
    match zone {
        b"Z" => true,
        [b'+' | b'-', ..] => {
            zone.len() == 6
                && zone[3] == b':'
                && two_digits(zone, 1).is_some_and(|hours| hours <= 23)
                && two_digits(zone, 4).is_some_and(|minutes| minutes <= 59)
        }
        _ => false,
    }
}

/// Holds the spec's bounds on what a new call and a change both carry.
fn check_call_details(
    title: Option<&str>,
    description: Option<&str>,
    recurrence: Option<&str>,
    time_zone: Option<&str>,
    duration_minutes: Option<u32>,
) -> std::result::Result<(), &'static str> {
    let longer = |text: Option<&str>, max: usize| text.is_some_and(|t| t.chars().count() > max);
    if longer(title, CALL_TITLE_MAX_CHARS) {
        return Err("call-title-too-long");
    }
    if longer(description, CALL_DESCRIPTION_MAX_CHARS) {
        return Err("call-description-too-long");
    }
    if longer(recurrence, CALL_RECURRENCE_MAX_CHARS) {
        return Err("call-recurrence-too-long");
    }
    if longer(time_zone, CALL_TIME_ZONE_MAX_CHARS) {
        return Err("call-time-zone-too-long");
    }
    if duration_minutes.is_some_and(|m| !(1..=CALL_DURATION_MAX_MINUTES).contains(&m)) {
        return Err("invalid-call-duration");
    }
    Ok(())
}

fn check_invitees(invitee_ids: &[String], max: usize) -> std::result::Result<(), &'static str> {
    if invitee_ids.len() > max {
        return Err("too-many-invitees");
    }
    invitee_ids.iter().try_for_each(|id| validate_user_id(id))
}

/// Validates a call to schedule against the spec's bounds.
///
/// # Errors
/// Returns the kebab-case reason: `invalid-call-time`, `call-title-too-long`,
/// `call-description-too-long`, `call-recurrence-too-long`,
/// `call-time-zone-too-long`, `invalid-call-duration`, `too-many-invitees`,
/// `invalid-user-id`.
pub fn validate_new_scheduled_call(
    call: &NewScheduledCall,
) -> std::result::Result<(), &'static str> {
    validate_call_time(&call.start_time)?;
    check_call_details(
        call.title.as_deref(),
        call.description.as_deref(),
        call.recurrence.as_deref(),
        call.time_zone.as_deref(),
        call.duration_minutes,
    )?;
    check_invitees(&call.invitee_ids, NEW_CALL_INVITEES_MAX)
}

/// Validates a change to a scheduled call against the spec's bounds.
///
/// # Errors
/// Returns the kebab-case reason: `no-call-changes` when nothing would
/// change, `empty-call-recurrence`, and any reason
/// [`validate_new_scheduled_call`] gives.
pub fn validate_scheduled_call_changes(
    changes: &ScheduledCallChanges,
) -> std::result::Result<(), &'static str> {
    if changes.is_empty() {
        return Err("no-call-changes");
    }
    if let Some(start) = &changes.start_time {
        validate_call_time(start)?;
    }
    if changes.recurrence.as_deref() == Some("") {
        return Err("empty-call-recurrence");
    }
    check_call_details(
        changes.title.as_deref(),
        changes.description.as_deref(),
        changes.recurrence.as_deref(),
        changes.time_zone.as_deref(),
        changes.duration_minutes,
    )?;
    if let Some(ids) = &changes.invitee_ids {
        check_invitees(ids, CHANGED_CALL_INVITEES_MAX)?;
    }
    validate_occurrences(&changes.occurrences)
}

/// Validates the occurrence a request names.
///
/// # Errors
/// Returns `invalid-call-time` when `occurrence_start_time` is not RFC 3339.
pub fn validate_occurrences(occurrences: &Occurrences) -> std::result::Result<(), &'static str> {
    match &occurrences.occurrence_start_time {
        Some(start) => validate_call_time(start),
        None => Ok(()),
    }
}

/// Validates the alt text for a media id.
///
/// # Errors
/// Returns the kebab-case reason: `empty-alt-text`, `alt-text-too-long`.
pub fn validate_alt_text(text: &str) -> std::result::Result<(), &'static str> {
    if text.trim().is_empty() {
        return Err("empty-alt-text");
    }
    if text.chars().count() > ALT_TEXT_MAX_CHARS {
        return Err("alt-text-too-long");
    }
    Ok(())
}

/// Validates a subtitle language code: two ASCII letters, in either case.
///
/// # Errors
/// Returns `invalid-language-code` for anything else.
pub fn validate_language_code(code: &str) -> std::result::Result<(), &'static str> {
    let code = code.trim();
    if code.len() != 2 || !code.bytes().all(|b| b.is_ascii_alphabetic()) {
        return Err("invalid-language-code");
    }
    Ok(())
}

// ── Request construction ─────────────────────────────────────────────

fn template(
    path: &str,
    path_params: HashMap<String, String>,
    query: Vec<(String, String)>,
) -> RequestTarget {
    RequestTarget::Template {
        path: path.to_string(),
        path_params,
        query,
    }
}

fn request(method: &str, target: RequestTarget, data: String) -> RequestOptions {
    RequestOptions {
        method: method.to_string(),
        target,
        data,
        ..Default::default()
    }
}

/// Every field the spec gives an X call.
const CALL_FIELDS: &str =
    "conversation_id,created_at,id,join_policy,join_url,participant_count,title,viewer_may_join";

/// Every field the spec gives a scheduled call.
const SCHEDULED_CALL_FIELDS: &str = "calendar,call_id,description,end_time,id,join_url,\
                                     occurrence_start_time,recurrence,start_time,status,\
                                     time_zone,title";

/// The user fields a call's host, organizer, and invitees are expanded with.
const CALL_USER_FIELDS: &str = "username,name";

/// Every field and expansion a scheduled call is read with: its invitees,
/// its call, and its organizer.
fn scheduled_call_query() -> Vec<(String, String)> {
    vec![
        (
            "scheduled_call.fields".to_string(),
            SCHEDULED_CALL_FIELDS.to_string(),
        ),
        (
            "expansions".to_string(),
            "call_id,invitees,organizer_id".to_string(),
        ),
        ("call.fields".to_string(), CALL_FIELDS.to_string()),
        ("user.fields".to_string(), CALL_USER_FIELDS.to_string()),
    ]
}

fn id_param(user_id: &str) -> HashMap<String, String> {
    HashMap::from([("id".to_string(), user_id.to_string())])
}

fn id_and(user_id: &str, key: &str, value: String) -> HashMap<String, String> {
    HashMap::from([
        ("id".to_string(), user_id.to_string()),
        (key.to_string(), value),
    ])
}

fn source_and_target(source_user_id: &str, target_user_id: &str) -> HashMap<String, String> {
    HashMap::from([
        ("source_user_id".to_string(), source_user_id.to_string()),
        ("target_user_id".to_string(), target_user_id.to_string()),
    ])
}

/// Every field the spec's `analytics.fields` lists for a post.
const POST_ANALYTICS_FIELDS: &str = "app_install_attempts,app_opens,bookmarks,detail_expands,\
email_tweet,engagements,follows,hashtag_clicks,id,impressions,likes,media_views,permalink_clicks,\
quote_tweets,replies,retweets,shares,timestamp,timestamped_metrics,unfollows,unlikes,url_clicks,\
user_profile_clicks";

/// Every field the spec's `news.fields` lists.
const NEWS_FIELDS: &str =
    "category,cluster_posts_results,contexts,disclaimer,hook,id,keywords,name,summary,updated_at";

/// The fields and expansions a read of whole posts asks for.
fn post_read_fields() -> Vec<(String, String)> {
    vec![
        (
            "post.fields".to_string(),
            "created_at,public_metrics,conversation_id,entities,attachments".to_string(),
        ),
        (
            "expansions".to_string(),
            "author_id,in_reply_to_user_id,referenced_posts".to_string(),
        ),
        (
            "user.fields".to_string(),
            "username,name,verified".to_string(),
        ),
    ]
}

fn target_user_body(target_user_id: &str) -> String {
    format!(r#"{{"target_user_id":"{target_user_id}"}}"#)
}

fn post_id_body(post_id: &str) -> String {
    format!(r#"{{"tweet_id":"{post_id}"}}"#)
}

impl Client {
    fn call<T: DeserializeOwned>(&self, method: &str, target: RequestTarget) -> Call<T> {
        Call::new(self, request(method, target, String::new()))
    }

    fn call_with_body<T: DeserializeOwned>(
        &self,
        method: &str,
        target: RequestTarget,
        data: String,
    ) -> Call<T> {
        Call::new(self, request(method, target, data))
    }

    /// A POST whose body is `body` serialized as JSON.
    fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        target: RequestTarget,
        body: &B,
    ) -> Call<T> {
        self.json_call("POST", target, body)
    }

    /// A `method` call whose body is `body` serialized as JSON; a body that
    /// cannot be serialized surfaces when the call is sent.
    fn json_call<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &str,
        target: RequestTarget,
        body: &B,
    ) -> Call<T> {
        match serde_json::to_string(body) {
            Ok(data) => self.call_with_body(method, target, data),
            Err(e) => Call::failed(self, e.into()),
        }
    }
}

// ── Shortcut methods on Client ───────────────────────────────────────

impl Client {
    /// Creates a new post.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xdk::api::Client;
    /// use xdk::auth::OAuth2Credential;
    ///
    /// # async fn run(credential: OAuth2Credential) -> xdk::Result<()> {
    /// let client = Client::builder().oauth2(credential).build()?;
    /// let resp = client.create_post("hello from xdk", &[]).send().await?;
    /// println!("created post id={}", resp.data.id);
    /// # Ok(()) }
    /// ```
    pub fn create_post(&self, text: &str, media_ids: &[String]) -> Call<ApiResponse<Post>> {
        let mut body = PostBody {
            text: text.to_string(),
            reply: None,
            quote: None,
            media: None,
        };
        if !media_ids.is_empty() {
            body.media = Some(PostMedia {
                media_ids: media_ids.to_vec(),
            });
        }
        self.post_json(
            template(endpoints::CREATE_POST.path, HashMap::new(), Vec::new()),
            &body,
        )
    }

    /// Replies to an existing post.
    pub fn reply_to_post(
        &self,
        post_id: &str,
        text: &str,
        media_ids: &[String],
    ) -> Call<ApiResponse<Post>> {
        let post_id = resolve_post_id(post_id);
        let mut body = PostBody {
            text: text.to_string(),
            reply: Some(PostReply {
                in_reply_to_post_id: post_id,
            }),
            quote: None,
            media: None,
        };
        if !media_ids.is_empty() {
            body.media = Some(PostMedia {
                media_ids: media_ids.to_vec(),
            });
        }
        self.post_json(
            template(endpoints::CREATE_POST.path, HashMap::new(), Vec::new()),
            &body,
        )
    }

    /// Quotes an existing post.
    pub fn quote_post(&self, post_id: &str, text: &str) -> Call<ApiResponse<Post>> {
        let post_id = resolve_post_id(post_id);
        let body = PostBody {
            text: text.to_string(),
            reply: None,
            quote: Some(post_id),
            media: None,
        };
        self.post_json(
            template(endpoints::CREATE_POST.path, HashMap::new(), Vec::new()),
            &body,
        )
    }

    /// Deletes a post.
    pub fn delete_post(&self, post_id: &str) -> Call<ApiResponse<DeletedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::DELETE_POST.method,
            template(
                endpoints::DELETE_POST.path,
                HashMap::from([("id".to_string(), post_id)]),
                Vec::new(),
            ),
        )
    }

    /// Hides a reply to one of the caller's posts, or shows it again when
    /// `hidden` is false.
    pub fn hide_reply(&self, reply_id: &str, hidden: bool) -> Call<ApiResponse<HiddenResult>> {
        let reply_id = resolve_post_id(reply_id);
        self.json_call(
            endpoints::HIDE_REPLY.method,
            template(
                endpoints::HIDE_REPLY.path,
                HashMap::from([("tweet_id".to_string(), reply_id)]),
                Vec::new(),
            ),
            &HiddenBody { hidden },
        )
    }

    /// Finds accounts by keyword. The endpoint reads its cursor from
    /// `next_token`.
    pub fn search_users(&self, query: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.call(
            endpoints::SEARCH_USERS.method,
            template(
                endpoints::SEARCH_USERS.path,
                HashMap::new(),
                vec![
                    ("query".to_string(), query.to_string()),
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "user.fields".to_string(),
                        "created_at,description,public_metrics,verified".to_string(),
                    ),
                ],
            ),
        )
        .paginated_by("next_token")
    }

    /// Counts the posts from the last seven days that match `query`, per
    /// `granularity` bucket (`minute`, `hour`, or `day`; X's default is
    /// `hour`), between `start_time` and `end_time` when given.
    pub fn count_recent_posts(
        &self,
        query: &str,
        granularity: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
    ) -> Call<ApiResponse<Vec<PostCount>>> {
        let mut params = vec![("query".to_string(), query.to_string())];
        let optional = [
            ("granularity", granularity),
            ("start_time", start_time),
            ("end_time", end_time),
        ];
        params.extend(
            optional
                .into_iter()
                .filter_map(|(name, value)| Some((name.to_string(), value?.to_string()))),
        );
        self.call(
            endpoints::GET_POST_COUNTS.method,
            template(endpoints::GET_POST_COUNTS.path, HashMap::new(), params),
        )
        .paginated()
    }

    /// Reads impressions and engagement for up to [`LOOKUP_IDS_MAX`] of the
    /// caller's posts, each an id or a post URL, between `start_time` and
    /// `end_time`, per `granularity` bucket (`hourly`, `daily`, `weekly`, or
    /// `total`, X's default). Every metric the spec names is requested.
    pub fn post_analytics<S: AsRef<str>>(
        &self,
        post_ids: &[S],
        start_time: &str,
        end_time: &str,
        granularity: Option<&str>,
    ) -> Call<ApiResponse<Vec<PostAnalytics>>> {
        let ids = post_ids
            .iter()
            .map(|id| resolve_post_id(id.as_ref()))
            .collect::<Vec<_>>()
            .join(",");
        let mut query = vec![
            ("ids".to_string(), ids),
            ("start_time".to_string(), start_time.to_string()),
            ("end_time".to_string(), end_time.to_string()),
            (
                "analytics.fields".to_string(),
                POST_ANALYTICS_FIELDS.to_string(),
            ),
        ];
        if let Some(granularity) = granularity {
            query.push(("granularity".to_string(), granularity.to_string()));
        }
        self.call(
            endpoints::GET_POST_ANALYTICS.method,
            template(endpoints::GET_POST_ANALYTICS.path, HashMap::new(), query),
        )
    }

    /// Fetches the trends for a place, named by its Yahoo WOEID (`1` is
    /// worldwide), up to `max_trends` of them (1 to 50; X's default is 20).
    pub fn get_trends(&self, woeid: u32, max_trends: Option<u32>) -> Call<ApiResponse<Vec<Trend>>> {
        let mut query = vec![(
            "trend.fields".to_string(),
            "trend_name,tweet_count".to_string(),
        )];
        if let Some(max) = max_trends {
            query.push(("max_trends".to_string(), max.to_string()));
        }
        self.call(
            endpoints::GET_TRENDS.method,
            template(
                endpoints::GET_TRENDS.path,
                HashMap::from([("woeid".to_string(), woeid.to_string())]),
                query,
            ),
        )
    }

    /// Fetches the trends X picks for the authenticated user.
    pub fn get_personalized_trends(&self) -> Call<ApiResponse<Vec<PersonalizedTrend>>> {
        self.call(
            endpoints::GET_PERSONALIZED_TRENDS.method,
            template(
                endpoints::GET_PERSONALIZED_TRENDS.path,
                HashMap::new(),
                vec![(
                    "personalized_trend.fields".to_string(),
                    "category,post_count,trend_name,trending_since".to_string(),
                )],
            ),
        )
    }

    /// Searches X's news stories, up to `max_results` (1 to 100) updated in
    /// the last `max_age_hours` (1 to 720; X's default is 168).
    pub fn search_news(
        &self,
        query: &str,
        max_results: i32,
        max_age_hours: Option<u32>,
    ) -> Call<ApiResponse<Vec<NewsStory>>> {
        let mut params = vec![
            ("query".to_string(), query.to_string()),
            ("max_results".to_string(), max_results.to_string()),
            ("news.fields".to_string(), NEWS_FIELDS.to_string()),
        ];
        if let Some(hours) = max_age_hours {
            params.push(("max_age_hours".to_string(), hours.to_string()));
        }
        self.call(
            endpoints::SEARCH_NEWS.method,
            template(endpoints::SEARCH_NEWS.path, HashMap::new(), params),
        )
    }

    /// Reads one news story by id.
    pub fn get_news(&self, id: &str) -> Call<ApiResponse<NewsStory>> {
        self.call(
            endpoints::GET_NEWS.method,
            template(
                endpoints::GET_NEWS.path,
                id_param(id),
                vec![("news.fields".to_string(), NEWS_FIELDS.to_string())],
            ),
        )
    }

    /// Fetches the users who liked a post.
    pub fn get_liking_users(
        &self,
        post_id: &str,
        max_results: i32,
    ) -> Call<ApiResponse<Vec<User>>> {
        let post_id = resolve_post_id(post_id);
        self.get_user_list(endpoints::GET_LIKING_USERS.path, &post_id, max_results)
    }

    /// Fetches the users who reposted a post.
    pub fn get_reposted_by(&self, post_id: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        let post_id = resolve_post_id(post_id);
        self.get_user_list(endpoints::GET_REPOSTED_BY.path, &post_id, max_results)
    }

    /// Fetches the posts that quote a post.
    ///
    /// X accepts 10 to 100 results for this endpoint; `max_results` is brought
    /// into that range.
    pub fn get_quote_posts(&self, post_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        let max_results = max_results.clamp(10, 100);
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::GET_QUOTE_POSTS.method,
            template(
                endpoints::GET_QUOTE_POSTS.path,
                id_param(&post_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,conversation_id,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Reads a single post with expansions.
    pub fn read_post(&self, post_id: &str) -> Call<ApiResponse<Post>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::READ_POST.method,
            template(
                endpoints::READ_POST.path,
                HashMap::from([("id".to_string(), post_id)]),
                post_read_fields(),
            ),
        )
    }

    /// Reads up to [`LOOKUP_IDS_MAX`] posts, each an id or a post URL, in one
    /// request. X names the ids it could not return in the response's
    /// `errors`, beside the posts it could.
    pub fn read_posts<S: AsRef<str>>(&self, post_ids: &[S]) -> Call<ApiResponse<Vec<Post>>> {
        let ids = post_ids
            .iter()
            .map(|id| resolve_post_id(id.as_ref()))
            .collect::<Vec<_>>()
            .join(",");
        let mut query = vec![("ids".to_string(), ids)];
        query.extend(post_read_fields());
        self.call(
            endpoints::READ_POSTS.method,
            template(endpoints::READ_POSTS.path, HashMap::new(), query),
        )
    }

    /// Searches recent posts.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xdk::api::Client;
    ///
    /// # async fn run() -> xdk::Result<()> {
    /// let client = Client::builder().bearer("app-only-token").build()?;
    /// let resp = client.search_posts("rustlang", 25).send().await?;
    /// for post in &resp.data {
    ///     println!("{}: {}", post.id, post.text);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn search_posts(&self, query: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        let max_results = max_results.clamp(10, 100);
        self.call(
            endpoints::SEARCH_POSTS.method,
            template(
                endpoints::SEARCH_POSTS.path,
                HashMap::new(),
                vec![
                    ("query".to_string(), query.to_string()),
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,conversation_id,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    (
                        "user.fields".to_string(),
                        "username,name,verified".to_string(),
                    ),
                ],
            ),
        )
        .paginated()
    }

    /// Fetches the authenticated user's profile.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use xdk::api::Client;
    /// use xdk::auth::OAuth2Credential;
    ///
    /// # async fn run(credential: OAuth2Credential) -> xdk::Result<()> {
    /// let client = Client::builder().oauth2(credential).build()?;
    /// let resp = client.get_me().send().await?;
    /// println!("@{} ({})", resp.data.username, resp.data.id);
    /// # Ok(()) }
    /// ```
    pub fn get_me(&self) -> Call<ApiResponse<User>> {
        self.call(
            endpoints::GET_ME.method,
            template(
                endpoints::GET_ME.path,
                HashMap::new(),
                vec![(
                    "user.fields".to_string(),
                    // `active_follower_count` is returned only for the
                    // authenticated user, so only this lookup asks for it.
                    "created_at,description,public_metrics,verified,profile_image_url,active_follower_count"
                        .to_string(),
                )],
            ),
        )
    }

    /// Looks up a user by username.
    pub fn lookup_user(&self, username: &str) -> Call<ApiResponse<User>> {
        let username = resolve_username(username);
        self.call(
            endpoints::LOOKUP_USER.method,
            template(
                endpoints::LOOKUP_USER.path,
                HashMap::from([("username".to_string(), username)]),
                vec![(
                    "user.fields".to_string(),
                    "created_at,description,public_metrics,verified,profile_image_url".to_string(),
                )],
            ),
        )
    }

    /// Fetches the home timeline.
    pub fn get_timeline(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        self.call(
            endpoints::GET_TIMELINE.method,
            template(
                endpoints::GET_TIMELINE.path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,conversation_id,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Fetches the posts a user wrote, newest first.
    ///
    /// X accepts 5 to 100 results for this endpoint; `max_results` is brought
    /// into that range.
    pub fn get_user_posts(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        let max_results = max_results.clamp(5, 100);
        self.call(
            endpoints::GET_USER_POSTS.method,
            template(
                endpoints::GET_USER_POSTS.path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,conversation_id,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Fetches recent mentions.
    ///
    /// X accepts 5 to 100 results for this endpoint; `max_results` is brought
    /// into that range.
    pub fn get_mentions(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        let max_results = max_results.clamp(5, 100);
        self.call(
            endpoints::GET_MENTIONS.method,
            template(
                endpoints::GET_MENTIONS.path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,conversation_id,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Likes a post.
    pub fn like_post(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<LikedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call_with_body(
            endpoints::LIKE_POST.method,
            template(endpoints::LIKE_POST.path, id_param(user_id), Vec::new()),
            post_id_body(&post_id),
        )
    }

    /// Unlikes a post.
    pub fn unlike_post(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<LikedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::UNLIKE_POST.method,
            template(
                endpoints::UNLIKE_POST.path,
                id_and(user_id, "tweet_id", post_id),
                Vec::new(),
            ),
        )
    }

    /// Reposts a post.
    pub fn repost(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<RepostedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call_with_body(
            endpoints::REPOST.method,
            template(endpoints::REPOST.path, id_param(user_id), Vec::new()),
            post_id_body(&post_id),
        )
    }

    /// Removes a repost.
    pub fn unrepost(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<RepostedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::UNREPOST.method,
            template(
                endpoints::UNREPOST.path,
                id_and(user_id, "source_tweet_id", post_id),
                Vec::new(),
            ),
        )
    }

    /// Bookmarks a post.
    pub fn bookmark(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<BookmarkedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call_with_body(
            endpoints::BOOKMARK.method,
            template(endpoints::BOOKMARK.path, id_param(user_id), Vec::new()),
            post_id_body(&post_id),
        )
    }

    /// Removes a bookmark.
    pub fn unbookmark(&self, user_id: &str, post_id: &str) -> Call<ApiResponse<BookmarkedResult>> {
        let post_id = resolve_post_id(post_id);
        self.call(
            endpoints::UNBOOKMARK.method,
            template(
                endpoints::UNBOOKMARK.path,
                id_and(user_id, "tweet_id", post_id),
                Vec::new(),
            ),
        )
    }

    /// Fetches bookmarks.
    pub fn get_bookmarks(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        self.call(
            endpoints::GET_BOOKMARKS.method,
            template(
                endpoints::GET_BOOKMARKS.path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Lists the folders the user sorts bookmarks into.
    pub fn get_bookmark_folders(
        &self,
        user_id: &str,
        max_results: i32,
    ) -> Call<ApiResponse<Vec<BookmarkFolder>>> {
        self.call(
            endpoints::GET_BOOKMARK_FOLDERS.method,
            template(
                endpoints::GET_BOOKMARK_FOLDERS.path,
                id_param(user_id),
                vec![("max_results".to_string(), max_results.to_string())],
            ),
        )
        .paginated()
    }

    /// Creates a bookmark folder named `name`, 1 to 25 characters.
    pub fn create_bookmark_folder(
        &self,
        user_id: &str,
        name: &str,
    ) -> Call<ApiResponse<BookmarkFolder>> {
        self.json_call(
            endpoints::CREATE_BOOKMARK_FOLDER.method,
            template(
                endpoints::CREATE_BOOKMARK_FOLDER.path,
                id_param(user_id),
                Vec::new(),
            ),
            &NameBody {
                name: name.to_string(),
            },
        )
    }

    /// Lists the posts in one of the user's bookmark folders. X names each
    /// post by id alone.
    pub fn get_bookmark_folder_posts(
        &self,
        user_id: &str,
        folder_id: &str,
        max_results: i32,
    ) -> Call<ApiResponse<Vec<BookmarkFolderPost>>> {
        self.call(
            endpoints::GET_BOOKMARK_FOLDER_POSTS.method,
            template(
                endpoints::GET_BOOKMARK_FOLDER_POSTS.path,
                id_and(user_id, "folder_id", folder_id.to_string()),
                vec![("max_results".to_string(), max_results.to_string())],
            ),
        )
        .paginated()
    }

    /// Follows a user.
    pub fn follow_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<FollowingResult>> {
        self.call_with_body(
            endpoints::FOLLOW_USER.method,
            template(
                endpoints::FOLLOW_USER.path,
                id_param(source_user_id),
                Vec::new(),
            ),
            target_user_body(target_user_id),
        )
    }

    /// Unfollows a user.
    pub fn unfollow_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<FollowingResult>> {
        self.call(
            endpoints::UNFOLLOW_USER.method,
            template(
                endpoints::UNFOLLOW_USER.path,
                source_and_target(source_user_id, target_user_id),
                Vec::new(),
            ),
        )
    }

    /// Fetches users that a given user follows.
    pub fn get_following(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.get_user_list(endpoints::GET_FOLLOWING.path, user_id, max_results)
    }

    /// Fetches followers of a given user.
    pub fn get_followers(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.get_user_list(endpoints::GET_FOLLOWERS.path, user_id, max_results)
    }

    /// Fetches the authenticated user's followers, with no id to look up.
    pub fn get_my_followers(&self, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.user_list(
            endpoints::GET_MY_FOLLOWERS.path,
            HashMap::new(),
            max_results,
        )
    }

    /// Fetches the users the authenticated user follows, with no id to look up.
    pub fn get_my_following(&self, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.user_list(
            endpoints::GET_MY_FOLLOWING.path,
            HashMap::new(),
            max_results,
        )
    }

    /// Sends a direct message; the response names the conversation and the
    /// event the message created.
    pub fn send_dm(&self, participant_id: &str, text: &str) -> Call<ApiResponse<DmSentResult>> {
        let body = serde_json::json!({"text": text});
        self.post_json(
            template(
                endpoints::SEND_DM.path,
                HashMap::from([("participant_id".to_string(), participant_id.to_string())]),
                Vec::new(),
            ),
            &body,
        )
    }

    /// Fetches recent DM events.
    pub fn get_dm_events(&self, max_results: i32) -> Call<ApiResponse<Vec<DmEvent>>> {
        self.call(
            endpoints::GET_DM_EVENTS.method,
            template(
                endpoints::GET_DM_EVENTS.path,
                HashMap::new(),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "dm_event.fields".to_string(),
                        "created_at,dm_conversation_id,text".to_string(),
                    ),
                    ("expansions".to_string(), "sender_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Fetches posts liked by a user.
    ///
    /// X accepts 5 to 100 results for this endpoint; `max_results` is brought
    /// into that range.
    pub fn get_liked_posts(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<Post>>> {
        let max_results = max_results.clamp(5, 100);
        self.call(
            endpoints::GET_LIKED_POSTS.method,
            template(
                endpoints::GET_LIKED_POSTS.path,
                id_param(user_id),
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "post.fields".to_string(),
                        "created_at,public_metrics,entities".to_string(),
                    ),
                    ("expansions".to_string(), "author_id".to_string()),
                    ("user.fields".to_string(), "username,name".to_string()),
                ],
            ),
        )
        .paginated()
    }

    /// Mutes a user.
    pub fn mute_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<MutingResult>> {
        self.call_with_body(
            endpoints::MUTE_USER.method,
            template(
                endpoints::MUTE_USER.path,
                id_param(source_user_id),
                Vec::new(),
            ),
            target_user_body(target_user_id),
        )
    }

    /// Fetches API usage data (post caps, daily breakdowns).
    pub fn get_usage(&self) -> Call<ApiResponse<UsageData>> {
        self.call(
            endpoints::GET_USAGE.method,
            template(
                endpoints::GET_USAGE.path,
                HashMap::new(),
                vec![(
                    "usage.fields".to_string(),
                    "daily_project_usage,daily_client_app_usage".to_string(),
                )],
            ),
        )
    }

    /// Fetches credits-based usage for the project.
    pub fn get_usage_credits(&self) -> Call<ApiResponse<UsageCreditsData>> {
        self.call(
            endpoints::GET_USAGE_CREDITS.method,
            template(endpoints::GET_USAGE_CREDITS.path, HashMap::new(), vec![]),
        )
    }

    /// Unmutes a user.
    pub fn unmute_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<MutingResult>> {
        self.call(
            endpoints::UNMUTE_USER.method,
            template(
                endpoints::UNMUTE_USER.path,
                source_and_target(source_user_id, target_user_id),
                Vec::new(),
            ),
        )
    }

    /// Lists the users the authenticated user has muted.
    pub fn get_muted(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.get_user_list(endpoints::GET_MUTED.path, user_id, max_results)
    }

    /// Blocks a user.
    pub fn block_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<BlockingResult>> {
        self.call_with_body(
            endpoints::BLOCK_USER.method,
            template(
                endpoints::BLOCK_USER.path,
                id_param(source_user_id),
                Vec::new(),
            ),
            target_user_body(target_user_id),
        )
    }

    /// Unblocks a user.
    pub fn unblock_user(
        &self,
        source_user_id: &str,
        target_user_id: &str,
    ) -> Call<ApiResponse<BlockingResult>> {
        self.call(
            endpoints::UNBLOCK_USER.method,
            template(
                endpoints::UNBLOCK_USER.path,
                source_and_target(source_user_id, target_user_id),
                Vec::new(),
            ),
        )
    }

    /// Lists the users the authenticated user has blocked.
    pub fn get_blocked(&self, user_id: &str, max_results: i32) -> Call<ApiResponse<Vec<User>>> {
        self.get_user_list(endpoints::GET_BLOCKED.path, user_id, max_results)
    }

    // ── Media metadata ───────────────────────────────────────────────

    /// Sets the alt text X shows for `media_id`, an image or video the
    /// authenticated user uploaded.
    pub fn set_media_alt_text(
        &self,
        media_id: &str,
        text: &str,
    ) -> Call<ApiResponse<MediaMetadataResult>> {
        self.json_call(
            endpoints::CREATE_MEDIA_METADATA.method,
            template(
                endpoints::CREATE_MEDIA_METADATA.path,
                HashMap::new(),
                Vec::new(),
            ),
            &MediaMetadataBody {
                id: media_id.trim().to_string(),
                metadata: MediaMetadata {
                    alt_text: AltText {
                        text: text.to_string(),
                    },
                },
            },
        )
    }

    /// Adds the subtitle track uploaded as `subtitles_id` (media category
    /// `subtitles`) to the video `video_id`, under the two-letter
    /// `language_code` and an optional `display_name`.
    pub fn add_media_subtitles(
        &self,
        video_id: &str,
        category: VideoCategory,
        subtitles_id: &str,
        language_code: &str,
        display_name: Option<&str>,
    ) -> Call<ApiResponse<MediaSubtitlesResult>> {
        self.json_call(
            endpoints::CREATE_MEDIA_SUBTITLES.method,
            template(
                endpoints::CREATE_MEDIA_SUBTITLES.path,
                HashMap::new(),
                Vec::new(),
            ),
            &AddSubtitlesBody {
                id: video_id.trim().to_string(),
                media_category: category.wire_name(),
                subtitles: SubtitleTrack {
                    id: subtitles_id.trim().to_string(),
                    language_code: language_code.trim().to_ascii_uppercase(),
                    display_name: display_name.map(str::to_string),
                },
            },
        )
    }

    /// Removes the `language_code` subtitle track from the video
    /// `video_id`.
    pub fn remove_media_subtitles(
        &self,
        video_id: &str,
        category: VideoCategory,
        language_code: &str,
    ) -> Call<ApiResponse<DeletedResult>> {
        self.json_call(
            endpoints::DELETE_MEDIA_SUBTITLES.method,
            template(
                endpoints::DELETE_MEDIA_SUBTITLES.path,
                HashMap::new(),
                Vec::new(),
            ),
            &RemoveSubtitlesBody {
                id: video_id.trim().to_string(),
                media_category: category.wire_name(),
                language_code: language_code.trim().to_ascii_uppercase(),
            },
        )
    }

    // ── Broadcasts ───────────────────────────────────────────────────

    /// Lists the users who moderate the authenticated user's broadcast chats.
    pub fn get_chat_moderators(&self) -> Call<ApiResponse<Vec<User>>> {
        self.call(
            endpoints::GET_CHAT_MODERATORS.method,
            template(
                endpoints::GET_CHAT_MODERATORS.path,
                HashMap::new(),
                vec![(
                    "user.fields".to_string(),
                    "created_at,description,public_metrics,verified".to_string(),
                )],
            ),
        )
    }

    /// Makes `user_id` a moderator of the authenticated user's broadcast
    /// chats; the response carries the whole moderator set.
    pub fn add_chat_moderator(&self, user_id: &str) -> Call<ApiResponse<ChatModeratorsResult>> {
        self.post_json(
            template(
                endpoints::ADD_CHAT_MODERATOR.path,
                HashMap::new(),
                Vec::new(),
            ),
            &ChatModeratorBody {
                user_id: user_id.to_string(),
            },
        )
    }

    /// Removes `user_id` from the moderators of the authenticated user's
    /// broadcast chats; the response carries the remaining set.
    pub fn remove_chat_moderator(&self, user_id: &str) -> Call<ApiResponse<ChatModeratorsResult>> {
        self.call(
            endpoints::REMOVE_CHAT_MODERATOR.method,
            template(
                endpoints::REMOVE_CHAT_MODERATOR.path,
                HashMap::from([("user_id".to_string(), user_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    /// Lists the authenticated user's RTMP and RTMPS stream sources.
    pub fn get_stream_sources(&self) -> Call<ApiResponse<StreamSources>> {
        self.call(
            endpoints::GET_STREAM_SOURCES.method,
            template(
                endpoints::GET_STREAM_SOURCES.path,
                HashMap::new(),
                Vec::new(),
            ),
        )
    }

    /// Creates a stream source named `name` that ingests in `region`, such
    /// as `us-east-1`. An encoder streams to its `rtmps_url` with its `id`
    /// as the stream key.
    pub fn create_stream_source(
        &self,
        name: &str,
        region: &str,
    ) -> Call<ApiResponse<StreamSourceResult>> {
        self.post_json(
            template(
                endpoints::CREATE_STREAM_SOURCE.path,
                HashMap::new(),
                Vec::new(),
            ),
            &StreamSourceBody {
                name: name.to_string(),
                region: region.to_string(),
            },
        )
    }

    /// Reads one of the authenticated user's stream sources, including
    /// whether it is receiving video.
    pub fn get_stream_source(&self, source_id: &str) -> Call<ApiResponse<StreamSourceResult>> {
        self.call(
            endpoints::GET_STREAM_SOURCE.method,
            template(
                endpoints::GET_STREAM_SOURCE.path,
                id_param(source_id),
                Vec::new(),
            ),
        )
    }

    /// Renames one of the authenticated user's stream sources.
    pub fn rename_stream_source(
        &self,
        source_id: &str,
        name: &str,
    ) -> Call<ApiResponse<StreamSourceResult>> {
        self.json_call(
            endpoints::RENAME_STREAM_SOURCE.method,
            template(
                endpoints::RENAME_STREAM_SOURCE.path,
                id_param(source_id),
                Vec::new(),
            ),
            &NameBody {
                name: name.to_string(),
            },
        )
    }

    /// Deletes one of the authenticated user's stream sources; an encoder
    /// sending to it is no longer received.
    pub fn delete_stream_source(&self, source_id: &str) -> Call<ApiResponse<StreamSourceDeleted>> {
        self.call(
            endpoints::DELETE_STREAM_SOURCE.method,
            template(
                endpoints::DELETE_STREAM_SOURCE.path,
                id_param(source_id),
                Vec::new(),
            ),
        )
    }

    // ── Calls ───────────────────────────────────────────────────────

    /// Reads an X call: its title, join policy, host, and participant
    /// count, with the host expanded.
    pub fn get_call(&self, call_id: &str) -> Call<ApiResponse<XCall>> {
        self.call(
            endpoints::GET_CALL.method,
            template(
                endpoints::GET_CALL.path,
                id_param(call_id),
                vec![
                    ("call.fields".to_string(), CALL_FIELDS.to_string()),
                    ("expansions".to_string(), "host_id".to_string()),
                    ("user.fields".to_string(), CALL_USER_FIELDS.to_string()),
                ],
            ),
        )
    }

    /// Lists the calls on the authenticated user's calendar, earliest first.
    /// X assumes a window from now to 24 hours later for a bound left
    /// `None`; both are RFC 3339. X accepts 1 to 100 results;
    /// `max_results` is brought into that range.
    pub fn get_scheduled_calls(
        &self,
        start_time: Option<&str>,
        end_time: Option<&str>,
        max_results: i32,
    ) -> Call<ApiResponse<Vec<ScheduledCall>>> {
        let mut query = Vec::new();
        if let Some(start) = start_time {
            query.push(("start_time".to_string(), start.to_string()));
        }
        if let Some(end) = end_time {
            query.push(("end_time".to_string(), end.to_string()));
        }
        query.push((
            "max_results".to_string(),
            max_results.clamp(1, 100).to_string(),
        ));
        query.extend(scheduled_call_query());
        self.call(
            endpoints::GET_SCHEDULED_CALLS.method,
            template(endpoints::GET_SCHEDULED_CALLS.path, HashMap::new(), query),
        )
    }

    /// Puts a call on the authenticated user's X Calls calendar and invites
    /// people to it. The call exists at once: its `join_url` works from now.
    pub fn schedule_call(&self, call: &NewScheduledCall) -> Call<ApiResponse<ScheduledCall>> {
        self.post_json(
            template(endpoints::SCHEDULE_CALL.path, HashMap::new(), Vec::new()),
            call,
        )
    }

    /// Reads a scheduled call the authenticated user organizes or is invited
    /// to, at its occurrence running now or its next one.
    pub fn get_scheduled_call(&self, scheduled_call_id: &str) -> Call<ApiResponse<ScheduledCall>> {
        self.call(
            endpoints::GET_SCHEDULED_CALL.method,
            template(
                endpoints::GET_SCHEDULED_CALL.path,
                id_param(scheduled_call_id),
                scheduled_call_query(),
            ),
        )
    }

    /// Moves, renames, or re-invites a scheduled call the authenticated user
    /// organizes. Only the fields `changes` sets are sent.
    pub fn update_scheduled_call(
        &self,
        scheduled_call_id: &str,
        changes: &ScheduledCallChanges,
    ) -> Call<ApiResponse<ScheduledCallUpdated>> {
        self.json_call(
            endpoints::UPDATE_SCHEDULED_CALL.method,
            template(
                endpoints::UPDATE_SCHEDULED_CALL.path,
                id_param(scheduled_call_id),
                Vec::new(),
            ),
            changes,
        )
    }

    /// Cancels the named occurrences of a scheduled call the authenticated
    /// user organizes and takes them off invitees' calendars. Cancelling what
    /// is already cancelled succeeds.
    pub fn cancel_scheduled_call(
        &self,
        scheduled_call_id: &str,
        occurrences: &Occurrences,
        notify_invitees: Option<bool>,
    ) -> Call<ApiResponse<ScheduledCallCancelled>> {
        let mut query = occurrences.query();
        if let Some(notify) = notify_invitees {
            query.push(("notify_invitees".to_string(), notify.to_string()));
        }
        self.call(
            endpoints::CANCEL_SCHEDULED_CALL.method,
            template(
                endpoints::CANCEL_SCHEDULED_CALL.path,
                id_param(scheduled_call_id),
                query,
            ),
        )
    }

    /// Answers an invitation to a scheduled call for the authenticated user,
    /// for the named occurrences.
    pub fn respond_to_scheduled_call(
        &self,
        scheduled_call_id: &str,
        answer: RsvpAnswer,
        occurrences: &Occurrences,
    ) -> Call<ApiResponse<ScheduledCallRsvp>> {
        self.post_json(
            template(
                endpoints::RESPOND_TO_SCHEDULED_CALL.path,
                id_param(scheduled_call_id),
                Vec::new(),
            ),
            &RsvpBody {
                response: answer,
                occurrences,
            },
        )
    }

    // ── Webhooks ────────────────────────────────────────────────────

    /// Lists the webhooks registered for the app.
    pub fn get_webhooks(&self) -> Call<ApiResponse<Vec<Webhook>>> {
        self.call(
            endpoints::GET_WEBHOOKS.method,
            template(endpoints::GET_WEBHOOKS.path, HashMap::new(), Vec::new()),
        )
    }

    /// Registers `url` as a webhook. X sends its CRC check to the URL before
    /// it answers, so the request succeeds only while a receiver is reachable
    /// there.
    pub fn create_webhook(&self, url: &str) -> Call<ApiResponse<Webhook>> {
        self.post_json(
            template(endpoints::CREATE_WEBHOOK.path, HashMap::new(), Vec::new()),
            &WebhookBody {
                url: url.to_string(),
            },
        )
    }

    /// Asks X to send its CRC check to a webhook again; the response says
    /// whether the URL answered it.
    pub fn validate_webhook(&self, webhook_id: &str) -> Call<ApiResponse<WebhookValidation>> {
        self.call(
            endpoints::VALIDATE_WEBHOOK.method,
            template(
                endpoints::VALIDATE_WEBHOOK.path,
                HashMap::from([("webhook_id".to_string(), webhook_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    /// Deletes a webhook. Its subscriptions stop delivering.
    pub fn delete_webhook(&self, webhook_id: &str) -> Call<ApiResponse<DeletedResult>> {
        self.call(
            endpoints::DELETE_WEBHOOK.method,
            template(
                endpoints::DELETE_WEBHOOK.path,
                HashMap::from([("webhook_id".to_string(), webhook_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    /// Starts a job that delivers the events between `from_date` and
    /// `to_date` to a webhook again. Both are twelve digits, `yyyymmddhhmm`
    /// in UTC, which is the form the endpoint takes.
    pub fn create_webhook_replay(
        &self,
        webhook_id: &str,
        from_date: &str,
        to_date: &str,
    ) -> Call<ApiResponse<WebhookReplayJob>> {
        self.post_json(
            template(
                endpoints::CREATE_WEBHOOK_REPLAY.path,
                HashMap::new(),
                Vec::new(),
            ),
            &WebhookReplayBody {
                webhook_id: webhook_id.to_string(),
                from_date: from_date.to_string(),
                to_date: to_date.to_string(),
            },
        )
    }

    /// Counts the Account Activity subscriptions the app holds and may hold.
    pub fn get_account_activity_subscription_count(
        &self,
    ) -> Call<ApiResponse<AccountActivitySubscriptionCount>> {
        self.call(
            endpoints::GET_ACCOUNT_ACTIVITY_SUBSCRIPTION_COUNT.method,
            template(
                endpoints::GET_ACCOUNT_ACTIVITY_SUBSCRIPTION_COUNT.path,
                HashMap::new(),
                Vec::new(),
            ),
        )
    }

    /// Lists the accounts whose activity a webhook receives.
    pub fn get_account_activity_subscriptions(
        &self,
        webhook_id: &str,
    ) -> Call<ApiResponse<AccountActivitySubscriptions>> {
        self.call(
            endpoints::GET_ACCOUNT_ACTIVITY_SUBSCRIPTIONS.method,
            template(
                endpoints::GET_ACCOUNT_ACTIVITY_SUBSCRIPTIONS.path,
                HashMap::from([("webhook_id".to_string(), webhook_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    /// Says whether the authenticated account's activity is subscribed to a
    /// webhook.
    pub fn check_account_activity_subscription(
        &self,
        webhook_id: &str,
    ) -> Call<ApiResponse<SubscribedResult>> {
        self.call(
            endpoints::CHECK_ACCOUNT_ACTIVITY_SUBSCRIPTION.method,
            template(
                endpoints::CHECK_ACCOUNT_ACTIVITY_SUBSCRIPTION.path,
                HashMap::from([("webhook_id".to_string(), webhook_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    /// Subscribes the authenticated account's activity to a webhook. X bills
    /// each event it then delivers.
    pub fn create_account_activity_subscription(
        &self,
        webhook_id: &str,
    ) -> Call<ApiResponse<SubscribedResult>> {
        self.post_json(
            template(
                endpoints::CREATE_ACCOUNT_ACTIVITY_SUBSCRIPTION.path,
                HashMap::from([("webhook_id".to_string(), webhook_id.to_string())]),
                Vec::new(),
            ),
            &serde_json::json!({}),
        )
    }

    /// Ends the subscription of `user_id`'s activity to a webhook.
    pub fn delete_account_activity_subscription(
        &self,
        webhook_id: &str,
        user_id: &str,
    ) -> Call<ApiResponse<SubscribedResult>> {
        self.call(
            endpoints::DELETE_ACCOUNT_ACTIVITY_SUBSCRIPTION.method,
            template(
                endpoints::DELETE_ACCOUNT_ACTIVITY_SUBSCRIPTION.path,
                HashMap::from([
                    ("webhook_id".to_string(), webhook_id.to_string()),
                    ("user_id".to_string(), user_id.to_string()),
                ]),
                Vec::new(),
            ),
        )
    }

    /// Lists the webhooks the app's filtered stream delivers to.
    pub fn get_webhook_stream_links(&self) -> Call<ApiResponse<Vec<WebhookStreamLink>>> {
        self.call(
            endpoints::GET_WEBHOOK_STREAM_LINKS.method,
            template(
                endpoints::GET_WEBHOOK_STREAM_LINKS.path,
                HashMap::new(),
                Vec::new(),
            ),
        )
    }

    /// Links the app's filtered stream to a webhook, so each post that
    /// matches the stream's rules is delivered there. X bills each delivery.
    pub fn create_webhook_stream_link(
        &self,
        webhook_id: &str,
    ) -> Call<ApiResponse<ProvisionedResult>> {
        self.call(
            endpoints::CREATE_WEBHOOK_STREAM_LINK.method,
            template(
                endpoints::CREATE_WEBHOOK_STREAM_LINK.path,
                HashMap::from([("webhook_id".to_string(), webhook_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    /// Ends delivery of the app's filtered stream to a webhook.
    pub fn delete_webhook_stream_link(&self, webhook_id: &str) -> Call<ApiResponse<DeletedResult>> {
        self.call(
            endpoints::DELETE_WEBHOOK_STREAM_LINK.method,
            template(
                endpoints::DELETE_WEBHOOK_STREAM_LINK.path,
                HashMap::from([("webhook_id".to_string(), webhook_id.to_string())]),
                Vec::new(),
            ),
        )
    }

    /// Shared GET for the user-list endpoints keyed on a single `{id}`.
    fn get_user_list(
        &self,
        path: &str,
        user_id: &str,
        max_results: i32,
    ) -> Call<ApiResponse<Vec<User>>> {
        self.user_list(path, id_param(user_id), max_results)
    }

    fn user_list(
        &self,
        path: &str,
        path_params: HashMap<String, String>,
        max_results: i32,
    ) -> Call<ApiResponse<Vec<User>>> {
        self.call(
            "GET",
            template(
                path,
                path_params,
                vec![
                    ("max_results".to_string(), max_results.to_string()),
                    (
                        "user.fields".to_string(),
                        "created_at,description,public_metrics,verified".to_string(),
                    ),
                ],
            ),
        )
        .paginated()
    }
}
