// WARNING: Generated code could not be parsed by syn for formatting.
// Run `cargo fmt` manually after fixing any syntax issues.

use serde_json;

/// baseOpts builds a RequestOptions from the common persistent flags.
fn base_opts(cmd: Box<Command /* todo: cobra.Command */>) -> RequestOptions /* todo: api.RequestOptions */ {
    let (auth_type, _) = cmd.flags().get_string("auth");
    let (username, _) = cmd.flags().get_string("username");
    let (verbose, _) = cmd.flags().get_bool("verbose");
    let (trace, _) = cmd.flags().get_bool("trace");
    RequestOptions /* todo: api.RequestOptions */ { auth_type: auth_type, username: username, verbose: verbose, trace: trace, ..Default::default() }
}

/// newClient creates an ApiClient from the auth object.
fn new_client(a: Box<Auth /* todo: auth.Auth */>) -> Box<ApiClient /* todo: api.ApiClient */> {
    let mut cfg = config.new_config();
    api.new_api_client(cfg, a)
}

/// printResult pretty‑prints a JSON response or exits on error.
fn print_result(resp: serde_json::Value, err: anyhow::Error) {
    // if err != nil { ... } — handled by ? above
    utils.format_and_print_response(resp);
}

/// resolveMyUserID calls /2/users/me and returns the authenticated user's ID.
fn resolve_my_user_id(client: Client /* todo: api.Client */, opts: RequestOptions /* todo: api.RequestOptions */) -> anyhow::Result<String> {
    let mut resp = api.get_me(client, opts)?;
    // if err != nil { ... } — handled by ? above
    let mut me: struct_Data_struct_ID_string__json___id______json___data____ /* todo: struct{Data struct{ID string "json:\"id\""} "json:\"data\""} */ = Default::default();
    let mut err = serde_json::from_str(&resp);
    // if err != nil { ... } — handled by ? above
    if me.data.id == "" {
        return Err((anyhow::anyhow!("user ID was empty – check your auth tokens")).into());
    }
    Ok(me.data.id)
}

/// resolveUserID looks up a username and returns its user ID.
fn resolve_user_id(client: Client /* todo: api.Client */, username: &str, opts: RequestOptions /* todo: api.RequestOptions */) -> anyhow::Result<String> {
    let mut resp = api.lookup_user(client, username, opts)?;
    // if err != nil { ... } — handled by ? above
    let mut user: struct_Data_struct_ID_string__json___id______json___data____ /* todo: struct{Data struct{ID string "json:\"id\""} "json:\"data\""} */ = Default::default();
    let mut err = serde_json::from_str(&resp);
    // if err != nil { ... } — handled by ? above
    if user.data.id == "" {
        return Err((anyhow::anyhow!("user @{} not found", username)).into());
    }
    Ok(user.data.id)
}

/// addCommonFlags adds --auth, --username, --verbose, --trace to a command.
fn add_common_flags(cmd: Box<Command /* todo: cobra.Command */>) {
    cmd.flags().string("auth", "", "Authentication type (oauth1, oauth2, app)");
    cmd.flags().string_p("username", "u", "", "OAuth2 username to act as");
    cmd.flags().bool_p("verbose", "v", false, "Print verbose request/response info");
    cmd.flags().bool_p("trace", "t", false, "Add X-B3-Flags trace header");
}

pub fn create_shortcut_commands(root_cmd: Box<Command /* todo: cobra.Command */>, a: Box<Auth /* todo: auth.Auth */>) {
    root_cmd.add_command(post_cmd(a), reply_cmd(a), quote_cmd(a), delete_cmd(a), read_cmd(a), search_cmd(a), whoami_cmd(a), user_cmd(a), timeline_cmd(a), mentions_cmd(a), like_cmd(a), unlike_cmd(a), repost_cmd(a), unrepost_cmd(a), bookmark_cmd(a), unbookmark_cmd(a), bookmarks_cmd(a), follow_cmd(a), unfollow_cmd(a), following_cmd(a), followers_cmd(a), likes_cmd(a), dm_cmd(a), dms_cmd(a), block_cmd(a), unblock_cmd(a), mute_cmd(a), unmute_cmd(a));
}

fn post_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut media_i_ds: Vec<String> = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: r#"post "TEXT""#.to_string(), short: "Post to X".to_string(), long: r#"Post a new post to X.

Examples:
  xurl post "Hello world!"
  xurl post "Check this out" --media-id 12345
  xurl post "Multiple images" --media-id 111 --media-id 222"#.to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        print_result(api.create_post(client, args[0], media_i_ds, opts));
    }, ..Default::default() });
    cmd.flags().string_array_var(&media_i_ds, "media-id", None, "Media ID(s) to attach (repeatable)");
    add_common_flags(cmd);
    cmd
}

fn reply_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut media_i_ds: Vec<String> = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: r#"reply POST_ID_OR_URL "TEXT""#.to_string(), short: "Reply to a post".to_string(), long: r#"Reply to an existing post. Accepts a post ID or full URL.

Examples:
  xurl reply 1234567890 "Great thread!"
  xurl reply https://x.com/user/status/1234567890 "Nice post!""#.to_string(), args: cobra.exact_args(2), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        print_result(api.reply_to_post(client, args[0], args[1], media_i_ds, opts));
    }, ..Default::default() });
    cmd.flags().string_array_var(&media_i_ds, "media-id", None, "Media ID(s) to attach (repeatable)");
    add_common_flags(cmd);
    cmd
}

fn quote_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: r#"quote POST_ID_OR_URL "TEXT""#.to_string(), short: "Quote a post".to_string(), long: r#"Quote an existing post with your own commentary.

Examples:
  xurl quote 1234567890 "This is so true"
  xurl quote https://x.com/user/status/1234567890 "Interesting take""#.to_string(), args: cobra.exact_args(2), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        print_result(api.quote_post(client, args[0], args[1], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn delete_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "delete POST_ID_OR_URL".to_string(), short: "Delete a post".to_string(), long: r#"Delete one of your posts. Accepts a post ID or full URL.

Examples:
  xurl delete 1234567890
  xurl delete https://x.com/user/status/1234567890"#.to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        print_result(api.delete_post(client, args[0], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn read_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "read POST_ID_OR_URL".to_string(), short: "Read a post".to_string(), long: r#"Fetch and display a single post with author info and metrics.

Examples:
  xurl read 1234567890
  xurl read https://x.com/user/status/1234567890"#.to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        print_result(api.read_post(client, args[0], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn search_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut max_results: i64 = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: r#"search "QUERY""#.to_string(), short: "Search recent posts".to_string(), long: r#"Search recent posts matching a query.

Examples:
  xurl search "golang"
  xurl search "from:elonmusk" -n 20
  xurl search "#buildinpublic" -n 15"#.to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        print_result(api.search_posts(client, args[0], max_results, opts));
    }, ..Default::default() });
    cmd.flags().int_var_p(&max_results, "max-results", "n", 10, "Number of results (min 10, max 100)");
    add_common_flags(cmd);
    cmd
}

fn whoami_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "whoami".to_string(), short: "Show the authenticated user's profile".to_string(), long: r#"Fetch profile information for the currently authenticated user.

Examples:
  xurl whoami"#.to_string(), args: cobra.no_args, run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        print_result(api.get_me(client, opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn user_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "user USERNAME".to_string(), short: "Look up a user by username".to_string(), long: r#"Fetch profile information for any user by their @username.

Examples:
  xurl user elonmusk
  xurl user @XDevelopers"#.to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        print_result(api.lookup_user(client, args[0], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn timeline_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut max_results: i64 = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "timeline".to_string(), short: "Show your home timeline".to_string(), long: r#"Fetch your reverse‑chronological home timeline.

Examples:
  xurl timeline
  xurl timeline -n 25"#.to_string(), args: cobra.no_args, run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.get_timeline(client, user_id, max_results, opts));
    }, ..Default::default() });
    cmd.flags().int_var_p(&max_results, "max-results", "n", 10, "Number of results (1–100)");
    add_common_flags(cmd);
    cmd
}

fn mentions_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut max_results: i64 = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "mentions".to_string(), short: "Show your recent mentions".to_string(), long: r#"Fetch posts that mention the authenticated user.

Examples:
  xurl mentions
  xurl mentions -n 25"#.to_string(), args: cobra.no_args, run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.get_mentions(client, user_id, max_results, opts));
    }, ..Default::default() });
    cmd.flags().int_var_p(&max_results, "max-results", "n", 10, "Number of results (5–100)");
    add_common_flags(cmd);
    cmd
}

fn like_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "like POST_ID_OR_URL".to_string(), short: "Like a post".to_string(), long: r#"Like a post. Accepts a post ID or full URL.

Examples:
  xurl like 1234567890
  xurl like https://x.com/user/status/1234567890"#.to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.like_post(client, user_id, args[0], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn unlike_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "unlike POST_ID_OR_URL".to_string(), short: "Unlike a post".to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.unlike_post(client, user_id, args[0], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn repost_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "repost POST_ID_OR_URL".to_string(), short: "Repost a post".to_string(), long: r#"Repost a post. Accepts a post ID or full URL.

Examples:
  xurl repost 1234567890
  xurl repost https://x.com/user/status/1234567890"#.to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.repost(client, user_id, args[0], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn unrepost_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "unrepost POST_ID_OR_URL".to_string(), short: "Undo a repost".to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.unrepost(client, user_id, args[0], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn bookmark_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "bookmark POST_ID_OR_URL".to_string(), short: "Bookmark a post".to_string(), long: r#"Bookmark a post. Accepts a post ID or full URL.

Examples:
  xurl bookmark 1234567890
  xurl bookmark https://x.com/user/status/1234567890"#.to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.bookmark(client, user_id, args[0], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn unbookmark_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "unbookmark POST_ID_OR_URL".to_string(), short: "Remove a bookmark".to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.unbookmark(client, user_id, args[0], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn bookmarks_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut max_results: i64 = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "bookmarks".to_string(), short: "List your bookmarks".to_string(), long: r#"Fetch your bookmarked posts.

Examples:
  xurl bookmarks
  xurl bookmarks -n 25"#.to_string(), args: cobra.no_args, run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.get_bookmarks(client, user_id, max_results, opts));
    }, ..Default::default() });
    cmd.flags().int_var_p(&max_results, "max-results", "n", 10, "Number of results (1–100)");
    add_common_flags(cmd);
    cmd
}

fn likes_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut max_results: i64 = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "likes".to_string(), short: "List your liked posts".to_string(), long: r#"Fetch posts you have liked.

Examples:
  xurl likes
  xurl likes -n 25"#.to_string(), args: cobra.no_args, run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.get_liked_posts(client, user_id, max_results, opts));
    }, ..Default::default() });
    cmd.flags().int_var_p(&max_results, "max-results", "n", 10, "Number of results (1–100)");
    add_common_flags(cmd);
    cmd
}

fn follow_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "follow USERNAME".to_string(), short: "Follow a user".to_string(), long: r#"Follow a user by their @username.

Examples:
  xurl follow elonmusk
  xurl follow @XDevelopers"#.to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut my_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        let mut target_id = resolve_user_id(client, args[0], opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.follow_user(client, my_id, target_id, opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn unfollow_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "unfollow USERNAME".to_string(), short: "Unfollow a user".to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut my_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        let mut target_id = resolve_user_id(client, args[0], opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.unfollow_user(client, my_id, target_id, opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn following_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut max_results: i64 = Default::default();
    let mut target_user: String = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "following".to_string(), short: "List users you follow".to_string(), long: r#"Fetch the list of users you (or another user) follow.

Examples:
  xurl following
  xurl following --of elonmusk -n 50"#.to_string(), args: cobra.no_args, run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id: String = Default::default();
        let mut err: anyhow::Error = Default::default();
        if target_user != "" {
            user_id = resolve_user_id(client, target_user, opts).unwrap();
        } else {
            user_id = resolve_my_user_id(client, opts).unwrap();
        }
        // if err != nil { ... } — handled by ? above
        print_result(api.get_following(client, user_id, max_results, opts));
    }, ..Default::default() });
    cmd.flags().int_var_p(&max_results, "max-results", "n", 10, "Number of results (1–1000)");
    cmd.flags().string_var(&target_user, "of", "", "Username to list following for (default: you)");
    add_common_flags(cmd);
    cmd
}

fn followers_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut max_results: i64 = Default::default();
    let mut target_user: String = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "followers".to_string(), short: "List your followers".to_string(), long: r#"Fetch the list of your (or another user's) followers.

Examples:
  xurl followers
  xurl followers --of elonmusk -n 50"#.to_string(), args: cobra.no_args, run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut user_id: String = Default::default();
        let mut err: anyhow::Error = Default::default();
        if target_user != "" {
            user_id = resolve_user_id(client, target_user, opts).unwrap();
        } else {
            user_id = resolve_my_user_id(client, opts).unwrap();
        }
        // if err != nil { ... } — handled by ? above
        print_result(api.get_followers(client, user_id, max_results, opts));
    }, ..Default::default() });
    cmd.flags().int_var_p(&max_results, "max-results", "n", 10, "Number of results (1–1000)");
    cmd.flags().string_var(&target_user, "of", "", "Username to list followers for (default: you)");
    add_common_flags(cmd);
    cmd
}

fn block_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "block USERNAME".to_string(), short: "Block a user".to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut my_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        let mut target_id = resolve_user_id(client, args[0], opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.block_user(client, my_id, target_id, opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn unblock_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "unblock USERNAME".to_string(), short: "Unblock a user".to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut my_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        let mut target_id = resolve_user_id(client, args[0], opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.unblock_user(client, my_id, target_id, opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn mute_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "mute USERNAME".to_string(), short: "Mute a user".to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut my_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        let mut target_id = resolve_user_id(client, args[0], opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.mute_user(client, my_id, target_id, opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn unmute_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "unmute USERNAME".to_string(), short: "Unmute a user".to_string(), args: cobra.exact_args(1), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut my_id = resolve_my_user_id(client, opts).unwrap();
        // if err != nil { ... } — handled by ? above
        let mut target_id = resolve_user_id(client, args[0], opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.unmute_user(client, my_id, target_id, opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn dm_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: r#"dm USERNAME "TEXT""#.to_string(), short: "Send a direct message".to_string(), long: r#"Send a direct message to a user.

Examples:
  xurl dm @elonmusk "Hey, great post!"
  xurl dm someuser "Hello there""#.to_string(), args: cobra.exact_args(2), run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        let mut target_id = resolve_user_id(client, args[0], opts).unwrap();
        // if err != nil { ... } — handled by ? above
        print_result(api.send_dm(client, target_id, args[1], opts));
    }, ..Default::default() });
    add_common_flags(cmd);
    cmd
}

fn dms_cmd(a: Box<Auth /* todo: auth.Auth */>) -> Box<Command /* todo: cobra.Command */> {
    let mut max_results: i64 = Default::default();
    let mut cmd = Box::new(Command /* todo: cobra.Command */ { r#use: "dms".to_string(), short: "List recent direct messages".to_string(), long: r#"Fetch your recent direct message events.

Examples:
  xurl dms
  xurl dms -n 25"#.to_string(), args: cobra.no_args, run: |cmd: Box<Command /* todo: cobra.Command */>, args: Vec<String>| {
        let mut client = new_client(a);
        let mut opts = base_opts(cmd);
        print_result(api.get_dm_events(client, max_results, opts));
    }, ..Default::default() });
    cmd.flags().int_var_p(&max_results, "max-results", "n", 10, "Number of results (1–100)");
    add_common_flags(cmd);
    cmd
}

