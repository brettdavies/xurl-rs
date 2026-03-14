use serde_json;
use url;
/// PostBody is the JSON body for POST /2/tweets
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PostBody {
    #[serde(rename = "text")]
    pub text: String,
    #[serde(rename = "reply")]
    #[serde(default)]
    pub reply: Box<PostReply>,
    #[serde(rename = "quote_tweet_id")]
    #[serde(default)]
    pub quote: Box<String>,
    #[serde(rename = "media")]
    #[serde(default)]
    pub media: Box<PostMedia>,
    #[serde(rename = "poll")]
    #[serde(default)]
    pub poll: Box<PostPoll>,
}
/// PostReply nests inside PostBody for replies
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PostReply {
    #[serde(rename = "in_reply_to_tweet_id")]
    pub in_reply_to_post_id: String,
}
/// PostMedia nests inside PostBody to attach uploaded media
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PostMedia {
    #[serde(rename = "media_ids")]
    pub media_i_ds: Vec<String>,
}
/// PostPoll nests inside PostBody to create a poll
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PostPoll {
    #[serde(rename = "options")]
    pub options: Vec<String>,
    #[serde(rename = "duration_minutes")]
    pub duration_minutes: i64,
}
/// ResolvePostID extracts a post ID from a full URL or returns the input as‑is.
/// Accepts:
///   - https://x.com/user/status/123456
///   - https://x.com/user/status/123456  (legacy domain also works)
///   - 123456
pub fn resolve_post_id(input: &str) -> String {
    input = input.trim();
    if input.starts_with("http://") || input.starts_with("https://") {
        let mut parsed = url.parse(input).unwrap();
        let mut parts = strings.trim(parsed.path, "/").split("/").collect::<Vec<&str>>();
        for (i, p) in parts.iter().enumerate() {
            if p == "status" && i + 1 < parts.len() {
                parts[i + 1]
            }
        }
    }
    input
}
/// ResolveUsername normalises a username – strips a leading "@" if present.
pub fn resolve_username(input: &str) -> String {
    input.trim().strip_prefix("@").unwrap_or(input.trim())
}
/// CreatePost sends a new post and returns the API response.
pub fn create_post(
    client: Client,
    text: &str,
    media_i_ds: &[String],
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    let mut body = PostBody {
        text: text,
        ..Default::default()
    };
    if media_i_ds.len() > 0 {
        body.media = Box::new(PostMedia {
            media_i_ds: media_i_ds,
            ..Default::default()
        });
    }
    let mut data = serde_json::to_string(&body)?;
    opts.method = "POST";
    opts.endpoint = "/2/tweets";
    opts.data = String::from(data);
    Ok(client.send_request(opts))
}
/// ReplyToPost sends a reply to an existing post.
pub fn reply_to_post(
    client: Client,
    post_id: &str,
    text: &str,
    media_i_ds: &[String],
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    let mut body = PostBody {
        text: text,
        reply: Box::new(PostReply {
            in_reply_to_post_id: post_id,
            ..Default::default()
        }),
        ..Default::default()
    };
    if media_i_ds.len() > 0 {
        body.media = Box::new(PostMedia {
            media_i_ds: media_i_ds,
            ..Default::default()
        });
    }
    let mut data = serde_json::to_string(&body)?;
    opts.method = "POST";
    opts.endpoint = "/2/tweets";
    opts.data = String::from(data);
    Ok(client.send_request(opts))
}
/// QuotePost sends a quote post.
pub fn quote_post(
    client: Client,
    post_id: &str,
    text: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    let mut body = PostBody {
        text: text,
        quote: &post_id,
        ..Default::default()
    };
    let mut data = serde_json::to_string(&body)?;
    opts.method = "POST";
    opts.endpoint = "/2/tweets";
    opts.data = String::from(data);
    Ok(client.send_request(opts))
}
/// DeletePost deletes a post by ID.
pub fn delete_post(
    client: Client,
    post_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    opts.method = "DELETE";
    opts.endpoint = format!("/2/tweets/{}", post_id);
    opts.data = "";
    Ok(client.send_request(opts))
}
/// ReadPost fetches a single post with useful expansions.
pub fn read_post(
    client: Client,
    post_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/tweets/{}?tweet.fields=created_at,public_metrics,conversation_id,in_reply_to_user_id,referenced_tweets,entities,attachments&expansions=author_id,referenced_tweets.id&user.fields=username,name,verified",
        post_id
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// SearchPosts searches recent posts.
pub fn search_posts(
    client: Client,
    query: &str,
    max_results: i64,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    let mut q = url.query_escape(query);
    if max_results < 10 {
        max_results = 10;
    } else if max_results > 100 {
        max_results = 100;
    }
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/tweets/search/recent?query={}&max_results={}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=author_id&user.fields=username,name,verified",
        q, max_results
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// GetMe fetches the authenticated user's profile.
pub fn get_me(
    client: Client,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "GET";
    opts.endpoint = "/2/users/me?user.fields=created_at,description,public_metrics,verified,profile_image_url";
    opts.data = "";
    Ok(client.send_request(opts))
}
/// LookupUser fetches a user by username.
pub fn lookup_user(
    client: Client,
    username: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    username = resolve_username(username);
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/users/by/username/{}?user.fields=created_at,description,public_metrics,verified,profile_image_url",
        username
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// GetUserPosts fetches recent posts by a user ID.
pub fn get_user_posts(
    client: Client,
    user_id: &str,
    max_results: i64,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/users/{}/tweets?max_results={}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=referenced_tweets.id",
        user_id, max_results
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// GetTimeline fetches the authenticated user's reverse‑chronological timeline.
/// Route: GET /2/users/{id}/timelines/reverse_chronological
pub fn get_timeline(
    client: Client,
    user_id: &str,
    max_results: i64,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/users/{}/timelines/reverse_chronological?max_results={}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=author_id&user.fields=username,name",
        user_id, max_results
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// GetMentions fetches recent mentions for a user.
pub fn get_mentions(
    client: Client,
    user_id: &str,
    max_results: i64,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/users/{}/mentions?max_results={}&tweet.fields=created_at,public_metrics,conversation_id,entities&expansions=author_id&user.fields=username,name",
        user_id, max_results
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// LikePost likes a post on behalf of the authenticated user.
pub fn like_post(
    client: Client,
    user_id: &str,
    post_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    let mut body = format!(r#"{"tweet_id":"{}"}"#, post_id);
    opts.method = "POST";
    opts.endpoint = format!("/2/users/{}/likes", user_id);
    opts.data = body;
    Ok(client.send_request(opts))
}
/// UnlikePost unlikes a post.
pub fn unlike_post(
    client: Client,
    user_id: &str,
    post_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    opts.method = "DELETE";
    opts.endpoint = format!("/2/users/{}/likes/{}", user_id, post_id);
    opts.data = "";
    Ok(client.send_request(opts))
}
/// Repost reposts a post.
pub fn repost(
    client: Client,
    user_id: &str,
    post_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    let mut body = format!(r#"{"tweet_id":"{}"}"#, post_id);
    opts.method = "POST";
    opts.endpoint = format!("/2/users/{}/retweets", user_id);
    opts.data = body;
    Ok(client.send_request(opts))
}
/// Unrepost removes a repost.
pub fn unrepost(
    client: Client,
    user_id: &str,
    post_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    opts.method = "DELETE";
    opts.endpoint = format!("/2/users/{}/retweets/{}", user_id, post_id);
    opts.data = "";
    Ok(client.send_request(opts))
}
/// Bookmark bookmarks a post.
pub fn bookmark(
    client: Client,
    user_id: &str,
    post_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    let mut body = format!(r#"{"tweet_id":"{}"}"#, post_id);
    opts.method = "POST";
    opts.endpoint = format!("/2/users/{}/bookmarks", user_id);
    opts.data = body;
    Ok(client.send_request(opts))
}
/// Unbookmark removes a bookmark.
pub fn unbookmark(
    client: Client,
    user_id: &str,
    post_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    post_id = resolve_post_id(post_id);
    opts.method = "DELETE";
    opts.endpoint = format!("/2/users/{}/bookmarks/{}", user_id, post_id);
    opts.data = "";
    Ok(client.send_request(opts))
}
/// GetBookmarks fetches the authenticated user's bookmarks.
pub fn get_bookmarks(
    client: Client,
    user_id: &str,
    max_results: i64,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/users/{}/bookmarks?max_results={}&tweet.fields=created_at,public_metrics,entities&expansions=author_id&user.fields=username,name",
        user_id, max_results
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// FollowUser follows a user.
pub fn follow_user(
    client: Client,
    source_user_id: &str,
    target_user_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    let mut body = format!(r#"{"target_user_id":"{}"}"#, target_user_id);
    opts.method = "POST";
    opts.endpoint = format!("/2/users/{}/following", source_user_id);
    opts.data = body;
    Ok(client.send_request(opts))
}
/// UnfollowUser unfollows a user.
pub fn unfollow_user(
    client: Client,
    source_user_id: &str,
    target_user_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "DELETE";
    opts.endpoint = format!("/2/users/{}/following/{}", source_user_id, target_user_id);
    opts.data = "";
    Ok(client.send_request(opts))
}
/// GetFollowing fetches users that a given user follows.
pub fn get_following(
    client: Client,
    user_id: &str,
    max_results: i64,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/users/{}/following?max_results={}&user.fields=created_at,description,public_metrics,verified",
        user_id, max_results
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// GetFollowers fetches followers of a given user.
pub fn get_followers(
    client: Client,
    user_id: &str,
    max_results: i64,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/users/{}/followers?max_results={}&user.fields=created_at,description,public_metrics,verified",
        user_id, max_results
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// SendDM sends a direct message to a user.
pub fn send_dm(
    client: Client,
    participant_id: &str,
    text: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    let mut body = format!(r#"{"text":"{}"}"#, text.replace(r#"""#, r#"\""#));
    opts.method = "POST";
    opts.endpoint = format!("/2/dm_conversations/with/{}/messages", participant_id);
    opts.data = body;
    Ok(client.send_request(opts))
}
/// GetDMEvents fetches recent DM events.
pub fn get_dm_events(
    client: Client,
    max_results: i64,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/dm_events?max_results={}&dm_event.fields=created_at,dm_conversation_id,sender_id,text&expansions=sender_id&user.fields=username,name",
        max_results
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// GetLikedPosts fetches posts liked by a user.
pub fn get_liked_posts(
    client: Client,
    user_id: &str,
    max_results: i64,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "GET";
    opts.endpoint = format!(
        "/2/users/{}/liked_tweets?max_results={}&tweet.fields=created_at,public_metrics,entities&expansions=author_id&user.fields=username,name",
        user_id, max_results
    );
    opts.data = "";
    Ok(client.send_request(opts))
}
/// BlockUser blocks a user.
pub fn block_user(
    client: Client,
    source_user_id: &str,
    target_user_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    let mut body = format!(r#"{"target_user_id":"{}"}"#, target_user_id);
    opts.method = "POST";
    opts.endpoint = format!("/2/users/{}/blocking", source_user_id);
    opts.data = body;
    Ok(client.send_request(opts))
}
/// UnblockUser unblocks a user.
pub fn unblock_user(
    client: Client,
    source_user_id: &str,
    target_user_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "DELETE";
    opts.endpoint = format!("/2/users/{}/blocking/{}", source_user_id, target_user_id);
    opts.data = "";
    Ok(client.send_request(opts))
}
/// MuteUser mutes a user.
pub fn mute_user(
    client: Client,
    source_user_id: &str,
    target_user_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    let mut body = format!(r#"{"target_user_id":"{}"}"#, target_user_id);
    opts.method = "POST";
    opts.endpoint = format!("/2/users/{}/muting", source_user_id);
    opts.data = body;
    Ok(client.send_request(opts))
}
/// UnmuteUser unmutes a user.
pub fn unmute_user(
    client: Client,
    source_user_id: &str,
    target_user_id: &str,
    opts: RequestOptions,
) -> anyhow::Result<serde_json::Value> {
    opts.method = "DELETE";
    opts.endpoint = format!("/2/users/{}/muting/{}", source_user_id, target_user_id);
    opts.data = "";
    Ok(client.send_request(opts))
}
