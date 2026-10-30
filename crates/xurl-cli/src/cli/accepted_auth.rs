//! The auth schemes each command's `--auth` help lists, read from the auth
//! matrix for the endpoints the command's requests reach.

use clap::Command;
use xdk::api::auth_matrix::{Endpoint, WireScheme, endpoints as ep, supported_auth};

/// A command and the endpoints its requests reach: by default, and when
/// `--of` names another user, for the commands that take it.
pub struct Row {
    /// The command's words, space-separated, as clap names them.
    pub path: &'static str,
    endpoints: &'static [Endpoint],
    with_of: &'static [Endpoint],
}

const fn row(path: &'static str, endpoints: &'static [Endpoint]) -> Row {
    Row {
        path,
        endpoints,
        with_of: &[],
    }
}

const fn with_of(
    path: &'static str,
    endpoints: &'static [Endpoint],
    with_of: &'static [Endpoint],
) -> Row {
    Row {
        path,
        endpoints,
        with_of,
    }
}

/// One row per command that takes `--auth`.
pub const ROWS: &[Row] = &[
    row("analytics", &[ep::GET_POST_ANALYTICS]),
    row("articles draft", &[ep::CREATE_ARTICLE_DRAFT]),
    row("articles publish", &[ep::PUBLISH_ARTICLE]),
    row(
        "block",
        &[ep::GET_ME, ep::LOOKUP_USERS_BY_USERNAME, ep::BLOCK_USER],
    ),
    row("blocked", &[ep::GET_ME, ep::GET_BLOCKED]),
    row("bookmark", &[ep::GET_ME, ep::BOOKMARK]),
    row(
        "bookmark-folders add",
        &[ep::GET_ME, ep::CREATE_BOOKMARK_FOLDER],
    ),
    row(
        "bookmark-folders list",
        &[ep::GET_ME, ep::GET_BOOKMARK_FOLDERS],
    ),
    row(
        "bookmark-folders posts",
        &[ep::GET_ME, ep::GET_BOOKMARK_FOLDER_POSTS],
    ),
    row("bookmarks", &[ep::GET_ME, ep::GET_BOOKMARKS]),
    row("broadcasts chat history", &[ep::GET_BROADCAST_CHAT]),
    row(
        "broadcasts chat mute",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::MUTE_CHAT_USER],
    ),
    row("broadcasts chat remove", &[ep::DELETE_CHAT_MESSAGE]),
    row("broadcasts chat send", &[ep::SEND_BROADCAST_CHAT]),
    row(
        "broadcasts chat unmute",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::UNMUTE_CHAT_USER],
    ),
    row("broadcasts end", &[ep::END_BROADCAST]),
    row("broadcasts get", &[ep::GET_BROADCAST]),
    row("broadcasts list", &[ep::LIST_BROADCASTS]),
    row(
        "broadcasts moderators add",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::ADD_CHAT_MODERATOR],
    ),
    row("broadcasts moderators list", &[ep::GET_CHAT_MODERATORS]),
    row(
        "broadcasts moderators remove",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::REMOVE_CHAT_MODERATOR],
    ),
    row(
        "broadcasts mutes add",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::ADD_CHAT_MUTE],
    ),
    row("broadcasts mutes list", &[ep::GET_CHAT_MUTES]),
    row(
        "broadcasts mutes remove",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::REMOVE_CHAT_MUTE],
    ),
    row("broadcasts remove", &[ep::DELETE_BROADCAST]),
    row(
        "broadcasts scheduled add",
        &[ep::CREATE_SCHEDULED_BROADCAST],
    ),
    row(
        "broadcasts scheduled cancel",
        &[ep::DELETE_SCHEDULED_BROADCAST],
    ),
    row("broadcasts scheduled get", &[ep::GET_SCHEDULED_BROADCAST]),
    row(
        "broadcasts scheduled list",
        &[ep::LIST_SCHEDULED_BROADCASTS],
    ),
    row(
        "broadcasts scheduled update",
        &[ep::UPDATE_SCHEDULED_BROADCAST],
    ),
    row("broadcasts sources add", &[ep::CREATE_STREAM_SOURCE]),
    row("broadcasts sources get", &[ep::GET_STREAM_SOURCE]),
    row("broadcasts sources list", &[ep::GET_STREAM_SOURCES]),
    row("broadcasts sources region", &[ep::GET_REGION]),
    row("broadcasts sources remove", &[ep::DELETE_STREAM_SOURCE]),
    row("broadcasts sources update", &[ep::RENAME_STREAM_SOURCE]),
    row("calls get", &[ep::GET_CALL]),
    row(
        "calls scheduled add",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::SCHEDULE_CALL],
    ),
    row("calls scheduled cancel", &[ep::CANCEL_SCHEDULED_CALL]),
    row("calls scheduled get", &[ep::GET_SCHEDULED_CALL]),
    row("calls scheduled list", &[ep::GET_SCHEDULED_CALLS]),
    row("calls scheduled rsvp", &[ep::RESPOND_TO_SCHEDULED_CALL]),
    row(
        "calls scheduled update",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::UPDATE_SCHEDULED_CALL],
    ),
    row("communities get", &[ep::GET_COMMUNITY]),
    row("communities search", &[ep::SEARCH_COMMUNITIES]),
    row("conversations history", &[ep::GET_CONVERSATION_EVENTS]),
    row("conversations send", &[ep::SEND_TO_CONVERSATION]),
    row(
        "conversations start",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::CREATE_DM_CONVERSATION],
    ),
    row(
        "conversations with",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::GET_DM_EVENTS_WITH],
    ),
    row("delete", &[ep::DELETE_POST]),
    row("dm", &[ep::LOOKUP_USERS_BY_USERNAME, ep::SEND_DM]),
    row("dm-block", &[ep::LOOKUP_USERS_BY_USERNAME, ep::BLOCK_DMS]),
    row("dm-events get", &[ep::GET_DM_EVENT]),
    row("dm-events remove", &[ep::DELETE_DM_EVENT]),
    row(
        "dm-unblock",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::UNBLOCK_DMS],
    ),
    row("dms", &[ep::GET_DM_EVENTS]),
    row(
        "follow",
        &[ep::GET_ME, ep::LOOKUP_USERS_BY_USERNAME, ep::FOLLOW_USER],
    ),
    with_of(
        "followers",
        &[ep::GET_MY_FOLLOWERS],
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::GET_FOLLOWERS],
    ),
    with_of(
        "following",
        &[ep::GET_MY_FOLLOWING],
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::GET_FOLLOWING],
    ),
    row("hide", &[ep::HIDE_REPLY]),
    row("like", &[ep::GET_ME, ep::LIKE_POST]),
    row("likers", &[ep::GET_LIKING_USERS]),
    row("likes", &[ep::GET_ME, ep::GET_LIKED_POSTS]),
    row("lists add", &[ep::CREATE_LIST]),
    row("lists follow", &[ep::GET_ME, ep::FOLLOW_LIST]),
    with_of(
        "lists followed",
        &[ep::GET_ME, ep::GET_FOLLOWED_LISTS],
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::GET_FOLLOWED_LISTS],
    ),
    row("lists followers", &[ep::GET_LIST_FOLLOWERS]),
    row("lists get", &[ep::GET_LIST]),
    with_of(
        "lists list",
        &[ep::GET_ME, ep::GET_OWNED_LISTS],
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::GET_OWNED_LISTS],
    ),
    row(
        "lists members add",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::ADD_LIST_MEMBER],
    ),
    row("lists members list", &[ep::GET_LIST_MEMBERS]),
    row(
        "lists members remove",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::REMOVE_LIST_MEMBER],
    ),
    with_of(
        "lists memberships",
        &[ep::GET_ME, ep::GET_LIST_MEMBERSHIPS],
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::GET_LIST_MEMBERSHIPS],
    ),
    row("lists pin", &[ep::GET_ME, ep::PIN_LIST]),
    row("lists pinned", &[ep::GET_ME, ep::GET_PINNED_LISTS]),
    row("lists posts", &[ep::GET_LIST_POSTS]),
    row("lists remove", &[ep::DELETE_LIST]),
    row("lists unfollow", &[ep::GET_ME, ep::UNFOLLOW_LIST]),
    row("lists unpin", &[ep::GET_ME, ep::UNPIN_LIST]),
    row("lists update", &[ep::UPDATE_LIST]),
    row("media alt-text", &[ep::CREATE_MEDIA_METADATA]),
    row("media analytics", &[ep::GET_MEDIA_ANALYTICS]),
    row("media get", &[ep::GET_MEDIA]),
    row("media status", &[ep::MEDIA_UPLOAD_STATUS]),
    row("media subtitles add", &[ep::CREATE_MEDIA_SUBTITLES]),
    row("media subtitles remove", &[ep::DELETE_MEDIA_SUBTITLES]),
    row(
        "media upload",
        &[
            ep::MEDIA_UPLOAD_INITIALIZE,
            ep::MEDIA_UPLOAD_APPEND,
            ep::MEDIA_UPLOAD_FINALIZE,
            ep::MEDIA_UPLOAD_STATUS,
        ],
    ),
    row("mentions", &[ep::GET_ME, ep::GET_MENTIONS]),
    row(
        "mute",
        &[ep::GET_ME, ep::LOOKUP_USERS_BY_USERNAME, ep::MUTE_USER],
    ),
    row("muted", &[ep::GET_ME, ep::GET_MUTED]),
    row("news get", &[ep::GET_NEWS]),
    row("news search", &[ep::SEARCH_NEWS]),
    row("notes eligible", &[ep::GET_POSTS_ELIGIBLE_FOR_NOTES]),
    row("notes evaluate", &[ep::EVALUATE_NOTE]),
    row("notes remove", &[ep::DELETE_NOTE]),
    row("notes write", &[ep::CREATE_NOTE]),
    row("notes written", &[ep::GET_NOTES_WRITTEN]),
    row("post", &[ep::CREATE_POST]),
    row("posts", &[ep::READ_POSTS]),
    row("quote", &[ep::CREATE_POST]),
    row("quotes", &[ep::GET_QUOTE_POSTS]),
    row("read", &[ep::READ_POST]),
    row("reply", &[ep::CREATE_POST]),
    row("repost", &[ep::GET_ME, ep::REPOST]),
    row("reposters", &[ep::GET_REPOSTED_BY]),
    row("search", &[ep::SEARCH_POSTS]),
    row("search-counts", &[ep::GET_POST_COUNTS]),
    row("search-users", &[ep::SEARCH_USERS]),
    row(
        "spaces by-creator",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::GET_SPACES_BY_CREATOR],
    ),
    row("spaces get", &[ep::GET_SPACES]),
    row("spaces posts", &[ep::GET_SPACE_POSTS]),
    row("spaces search", &[ep::SEARCH_SPACES]),
    row("stream rules add", &[ep::UPDATE_STREAM_RULES]),
    row("stream rules count", &[ep::GET_STREAM_RULE_COUNTS]),
    row("stream rules list", &[ep::GET_STREAM_RULES]),
    row("stream rules remove", &[ep::UPDATE_STREAM_RULES]),
    row("timeline", &[ep::GET_ME, ep::GET_TIMELINE]),
    row("trends personal", &[ep::GET_PERSONALIZED_TRENDS]),
    row("trends place", &[ep::GET_TRENDS]),
    row(
        "unblock",
        &[ep::GET_ME, ep::LOOKUP_USERS_BY_USERNAME, ep::UNBLOCK_USER],
    ),
    row("unbookmark", &[ep::GET_ME, ep::UNBOOKMARK]),
    row(
        "unfollow",
        &[ep::GET_ME, ep::LOOKUP_USERS_BY_USERNAME, ep::UNFOLLOW_USER],
    ),
    row("unhide", &[ep::HIDE_REPLY]),
    row("unlike", &[ep::GET_ME, ep::UNLIKE_POST]),
    row(
        "unmute",
        &[ep::GET_ME, ep::LOOKUP_USERS_BY_USERNAME, ep::UNMUTE_USER],
    ),
    row("unrepost", &[ep::GET_ME, ep::UNREPOST]),
    row("usage", &[ep::GET_USAGE]),
    row("usage credits", &[ep::GET_USAGE_CREDITS]),
    row("user", &[ep::LOOKUP_USER]),
    row(
        "user-posts",
        &[ep::LOOKUP_USERS_BY_USERNAME, ep::GET_USER_POSTS],
    ),
    row("users", &[ep::LOOKUP_USERS, ep::LOOKUP_USERS_BY_USERNAME]),
    row("webhooks add", &[ep::CREATE_WEBHOOK]),
    row("webhooks list", &[ep::GET_WEBHOOKS]),
    row("webhooks remove", &[ep::DELETE_WEBHOOK]),
    row("webhooks replay", &[ep::CREATE_WEBHOOK_REPLAY]),
    row(
        "webhooks stream-links add",
        &[ep::CREATE_WEBHOOK_STREAM_LINK],
    ),
    row(
        "webhooks stream-links list",
        &[ep::GET_WEBHOOK_STREAM_LINKS],
    ),
    row(
        "webhooks stream-links remove",
        &[ep::DELETE_WEBHOOK_STREAM_LINK],
    ),
    row(
        "webhooks subscriptions add",
        &[ep::CREATE_ACCOUNT_ACTIVITY_SUBSCRIPTION],
    ),
    row(
        "webhooks subscriptions check",
        &[ep::CHECK_ACCOUNT_ACTIVITY_SUBSCRIPTION],
    ),
    row(
        "webhooks subscriptions count",
        &[ep::GET_ACCOUNT_ACTIVITY_SUBSCRIPTION_COUNT],
    ),
    row(
        "webhooks subscriptions list",
        &[ep::GET_ACCOUNT_ACTIVITY_SUBSCRIPTIONS],
    ),
    row(
        "webhooks subscriptions remove",
        &[
            ep::LOOKUP_USERS_BY_USERNAME,
            ep::DELETE_ACCOUNT_ACTIVITY_SUBSCRIPTION,
        ],
    ),
    row("webhooks validate", &[ep::VALIDATE_WEBHOOK]),
    row("whoami", &[ep::GET_ME]),
];

/// The `--auth` help of raw mode, and of a command with no row: every scheme
/// `xr` takes, in the order it picks a stored credential in.
pub fn every_scheme_help() -> String {
    format!(
        "Authentication type ({})",
        listed(&WireScheme::ALL_BY_PREFERENCE)
    )
}

/// Sets the `--auth` help of every command under `cmd` that has a row; the
/// root `Cli` applies it to each top-level command.
pub fn annotate(cmd: Command) -> Command {
    annotate_under(cmd, "")
}

fn annotate_under(cmd: Command, parent: &str) -> Command {
    let path = if parent.is_empty() {
        cmd.get_name().to_string()
    } else {
        format!("{parent} {}", cmd.get_name())
    };
    let cmd = cmd.mut_subcommands(|sub| annotate_under(sub, &path));
    let Some(row) = ROWS.iter().find(|row| row.path == path) else {
        return cmd;
    };
    // `mut_args` keeps each argument in its place, so the shell completions
    // list `--auth` where the command declares it.
    cmd.mut_args(|arg| {
        if arg.get_id() == "auth_type" {
            arg.help(help_for(row))
        } else {
            arg
        }
    })
}

fn help_for(row: &Row) -> String {
    let accepted = accepted_by(row.endpoints);
    let mut sets = format!("this command accepts: {}", listed(&accepted));
    if !row.with_of.is_empty() {
        let with_of = accepted_by(row.with_of);
        if with_of != accepted {
            sets.push_str(&format!("; with --of: {}", listed(&with_of)));
        }
    }
    format!(
        "Authentication type ({sets}). Unset, xr uses the first of these it holds a credential for"
    )
}

/// The schemes every endpoint in `endpoints` accepts, in preference order.
fn accepted_by(endpoints: &[Endpoint]) -> Vec<WireScheme> {
    WireScheme::ALL_BY_PREFERENCE
        .into_iter()
        .filter(|scheme| {
            endpoints.iter().all(|endpoint| {
                supported_auth(endpoint.method, endpoint.path)
                    .is_none_or(|accepted| accepted.iter().any(|entry| entry.wire() == *scheme))
            })
        })
        .collect()
}

fn listed(schemes: &[WireScheme]) -> String {
    schemes
        .iter()
        .map(|scheme| scheme.as_wire())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_accepts_some_scheme() {
        let empty: Vec<&str> = ROWS
            .iter()
            .filter(|row| {
                accepted_by(row.endpoints).is_empty()
                    || (!row.with_of.is_empty() && accepted_by(row.with_of).is_empty())
            })
            .map(|row| row.path)
            .collect();
        assert!(empty.is_empty(), "rows no scheme can serve: {empty:?}");
    }
}
