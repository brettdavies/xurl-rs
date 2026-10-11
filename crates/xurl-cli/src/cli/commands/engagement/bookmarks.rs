//! Bookmark export commands: `bookmarks` with --all, --folder, --expand,
//! --until-known, --archive; and `bookmark-folders`.

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write as _};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cli::commands::{
    CommonFlags, Run, effective_limit, make_client, print_typed, resolve_my_user_id, with_flags,
};
use crate::cli::failure::CommandResult;
use xdk::Error;
use xdk::api::{ApiResponse, Client};

/// Runs the `xr bookmarks` command with full export capabilities.
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_bookmarks(
    run: Run<'_>,
    max_results: Option<i32>,
    all: bool,
    max_pages: Option<u32>,
    folder: Option<&str>,
    expand: &[String],
    until_known: Option<&str>,
    archive: Option<&str>,
    common: &CommonFlags,
) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;

    let n = effective_limit(max_results, flags.global_limit);
    let client = make_client(cfg, auth)?;
    let user_id = resolve_my_user_id(&client, common).await?;

    // Build field/expansion parameters from --expand flags
    let (post_fields, expansions, media_fields, user_fields) = build_field_params(expand);

    // Simple path: no --all, no --folder, no incremental sync
    if !all && folder.is_none() && until_known.is_none() && archive.is_none() {
        // Backward-compatible: one page, standard output
        let call = client.get_bookmarks(&user_id, n);
        let response = with_flags(call, common, flags.cursor.as_deref())
            .send()
            .await?;
        print_typed(out, stdout, &response)?;
        return Ok(());
    }

    // Complex path: --all, --folder, or incremental sync
    if let Some(folder_id) = folder {
        fetch_folder_bookmarks(
            &client,
            &user_id,
            folder_id,
            n,
            all,
            max_pages,
            &post_fields,
            &expansions,
            &media_fields,
            &user_fields,
            until_known,
            archive,
            common,
            flags.cursor.as_deref(),
            out,
            stdout,
        )
        .await
    } else {
        fetch_all_bookmarks(
            &client,
            &user_id,
            n,
            all,
            max_pages,
            &post_fields,
            &expansions,
            &media_fields,
            &user_fields,
            until_known,
            archive,
            common,
            flags.cursor.as_deref(),
            out,
            stdout,
        )
        .await
    }
}

/// Runs the `xr bookmark-folders` command.
pub(super) async fn run_bookmark_folders(
    run: Run<'_>,
    max_results: Option<i32>,
    all: bool,
    common: &CommonFlags,
) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;

    let n = effective_limit(max_results, flags.global_limit);
    let client = make_client(cfg, auth)?;
    let user_id = resolve_my_user_id(&client, common).await?;

    if !all {
        // One page
        let call = client.get_bookmark_folders(&user_id, n);
        let response = with_flags(call, common, flags.cursor.as_deref())
            .send()
            .await?;
        print_typed(out, stdout, &response)?;
        Ok(())
    } else {
        // Paginate to the end
        let mut all_folders = Vec::new();
        let mut cursor = flags.cursor.clone();

        loop {
            let call = client.get_bookmark_folders(&user_id, n);
            let response = with_flags(call, common, cursor.as_deref()).send().await?;

            all_folders.extend_from_slice(&response.data);

            match response.meta.as_ref().and_then(|m| m.next_token.as_deref()) {
                Some(next) => cursor = Some(next.to_string()),
                None => break,
            }
        }

        // Print merged response
        let merged = ApiResponse {
            data: Some(all_folders),
            includes: None,
            meta: None,
            errors: None,
            extra: Default::default(),
        };
        print_typed(out, stdout, &merged)?;
        Ok(())
    }
}

/// Fetches all bookmarks (non-folder mode) with pagination.
#[allow(clippy::too_many_arguments)]
async fn fetch_all_bookmarks(
    client: &Client,
    user_id: &str,
    n: i32,
    all: bool,
    max_pages: Option<u32>,
    post_fields: &Option<String>,
    expansions: &Option<String>,
    media_fields: &Option<String>,
    user_fields: &Option<String>,
    until_known: Option<&str>,
    archive: Option<&str>,
    common: &CommonFlags,
    initial_cursor: Option<&str>,
    out: &crate::cli::output::OutputConfig,
    stdout: &mut dyn std::io::Write,
) -> CommandResult<()> {
    let mut all_posts = Vec::new();
    let mut cursor = initial_cursor.map(String::from);
    let mut pages = 0u32;
    let max = max_pages.unwrap_or(u32::MAX);

    // Load known IDs if incremental sync
    let known_ids = if let Some(state_path) = until_known {
        load_known_ids(state_path, archive)?
    } else {
        HashSet::new()
    };

    loop {
        pages += 1;
        if pages > max {
            break;
        }

        let call = if post_fields.is_some() || expansions.is_some() {
            client.get_bookmarks_with_fields(
                user_id,
                n,
                post_fields.as_deref(),
                expansions.as_deref(),
                media_fields.as_deref(),
                user_fields.as_deref(),
            )
        } else {
            client.get_bookmarks(user_id, n)
        };

        let response = with_flags(call, common, cursor.as_deref()).send().await?;

        // Check for incremental stop
        let mut stopped_early = false;
        for post in &response.data {
            if !known_ids.is_empty() && known_ids.contains(&post.id) {
                stopped_early = true;
                break;
            }
            all_posts.push(post.clone());
        }

        if stopped_early {
            break;
        }

        if !all {
            // Single page
            print_typed(out, stdout, &response)?;
            return Ok(());
        }

        match response.meta.as_ref().and_then(|m| m.next_token.as_deref()) {
            Some(next) => cursor = Some(next.to_string()),
            None => break,
        }
    }

    // Archive if requested
    if let Some(archive_dir) = archive {
        archive_posts(&all_posts, archive_dir).map_err(|e| Error::io(e.to_string()))?;
    }

    // Update state if requested
    if let Some(state_path) = until_known {
        update_state(state_path, &all_posts, user_id, None)
            .map_err(|e| Error::io(e.to_string()))?;
    }

    // Print merged response
    let merged = ApiResponse {
        data: Some(all_posts.clone()),
        includes: None,
        meta: None,
        errors: None,
        extra: Default::default(),
    };

    // Warn if we hit the 800 cap
    if all_posts.len() >= 800 {
        eprintln!(
            "Warning: fetched {} items; X API only exposes the most recent 800 bookmarks",
            all_posts.len()
        );
    }

    print_typed(out, stdout, &merged)?;
    Ok(())
}

/// Fetches bookmarks from a specific folder, then hydrates the post IDs.
#[allow(clippy::too_many_arguments)]
async fn fetch_folder_bookmarks(
    client: &Client,
    user_id: &str,
    folder_id: &str,
    n: i32,
    all: bool,
    max_pages: Option<u32>,
    post_fields: &Option<String>,
    expansions: &Option<String>,
    media_fields: &Option<String>,
    user_fields: &Option<String>,
    until_known: Option<&str>,
    archive: Option<&str>,
    common: &CommonFlags,
    initial_cursor: Option<&str>,
    out: &crate::cli::output::OutputConfig,
    stdout: &mut dyn std::io::Write,
) -> CommandResult<()> {
    let mut post_ids = Vec::new();
    let mut cursor = initial_cursor.map(String::from);
    let mut pages = 0u32;
    let max = max_pages.unwrap_or(u32::MAX);

    // Fetch folder contents (IDs only)
    loop {
        pages += 1;
        if pages > max {
            break;
        }

        let call = client.get_bookmarks_by_folder(user_id, folder_id, n);
        let response = with_flags(call, common, cursor.as_deref()).send().await?;

        post_ids.extend(response.data.iter().map(|r| r.id.clone()));

        if !all {
            // Single page of IDs - can't hydrate here without --all, so just return the IDs
            print_typed(out, stdout, &response)?;
            return Ok(());
        }

        match response.meta.as_ref().and_then(|m| m.next_token.as_deref()) {
            Some(next) => cursor = Some(next.to_string()),
            None => break,
        }
    }

    // Hydrate post IDs in batches of 100
    let mut all_posts = Vec::new();
    for chunk in post_ids.chunks(100) {
        let call = client.get_posts_by_ids(
            &chunk.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            post_fields.as_deref(),
            expansions.as_deref(),
            media_fields.as_deref(),
            user_fields.as_deref(),
        );
        let response = with_flags(call, common, None).send().await?;
        all_posts.extend(response.data);
    }

    // Archive if requested
    if let Some(archive_dir) = archive {
        archive_posts(&all_posts, archive_dir).map_err(|e| Error::io(e.to_string()))?;
    }

    // Update state if requested
    if let Some(state_path) = until_known {
        update_state(state_path, &all_posts, user_id, Some(folder_id))
            .map_err(|e| Error::io(e.to_string()))?;
    }

    // Print merged response
    let merged = ApiResponse {
        data: Some(all_posts),
        includes: None,
        meta: None,
        errors: None,
        extra: Default::default(),
    };

    print_typed(out, stdout, &merged)?;
    Ok(())
}

/// Builds field/expansion parameters from --expand flags.
fn build_field_params(
    expand: &[String],
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    if expand.is_empty() {
        return (None, None, None, None);
    }

    let mut post_fields = vec![
        "created_at",
        "author_id",
        "public_metrics",
        "entities",
        "conversation_id",
    ];
    let mut expansions = vec!["author_id"];
    let mut media_fields = Vec::new();
    let user_fields = ["username", "name"];

    for item in expand {
        match item.as_str() {
            "media" => {
                post_fields.push("attachments");
                expansions.push("attachments.media_keys");
                media_fields.extend_from_slice(&["url", "preview_image_url", "alt_text", "type"]);
            }
            "quoted" => {
                post_fields.push("referenced_posts");
                expansions.push("referenced_tweets.id");
                expansions.push("referenced_tweets.id.author_id");
            }
            "urls" => {
                // entities already included
            }
            "note" => {
                post_fields.push("note_tweet");
            }
            "article" => {
                post_fields.push("article");
            }
            _ => {}
        }
    }

    (
        Some(post_fields.join(",")),
        Some(expansions.join(",")),
        if media_fields.is_empty() {
            None
        } else {
            Some(media_fields.join(","))
        },
        Some(user_fields.join(",")),
    )
}

/// State file format for incremental sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StateFile {
    schema: u32,
    user_id: String,
    folder_id: Option<String>,
    newest_id: Option<String>,
    seen_count: usize,
    last_run: String,
    complete: bool,
}

/// Loads known IDs from state file and archive.
fn load_known_ids(_state_path: &str, archive: Option<&str>) -> CommandResult<HashSet<String>> {
    let mut known = HashSet::new();

    // Load from archive if it exists
    if let Some(archive_dir) = archive {
        let archive_file = Path::new(archive_dir).join("bookmarks.jsonl");
        if archive_file.exists() {
            let file = File::open(archive_file).map_err(|e| Error::io(e.to_string()))?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.map_err(|e| Error::io(e.to_string()))?;
                if let Ok(post) = serde_json::from_str::<Value>(&line)
                    && let Some(id) = post.get("id").and_then(|v| v.as_str())
                {
                    known.insert(id.to_string());
                }
            }
        }
    }

    Ok(known)
}

/// Archives posts to bookmarks.jsonl (one per line).
fn archive_posts(
    posts: &[xdk::api::response::types::Post],
    archive_dir: &str,
) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(archive_dir)?;
    let archive_file = Path::new(archive_dir).join("bookmarks.jsonl");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(archive_file)?;

    for post in posts {
        let json = serde_json::to_string(post).map_err(std::io::Error::other)?;
        writeln!(file, "{}", json)?;
    }

    Ok(())
}

/// Updates the state file after a successful fetch.
fn update_state(
    state_path: &str,
    posts: &[xdk::api::response::types::Post],
    user_id: &str,
    folder_id: Option<&str>,
) -> Result<(), std::io::Error> {
    let newest_id = posts.first().map(|p| p.id.clone());

    // Use a simple RFC3339 timestamp without chrono
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(std::io::Error::other)?;
    let timestamp = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        1970 + (now.as_secs() / 31_557_600), // Approximate year
        ((now.as_secs() % 31_557_600) / 2_629_800) + 1, // Approximate month
        ((now.as_secs() % 2_629_800) / 86400) + 1, // Approximate day
        (now.as_secs() % 86400) / 3600,
        (now.as_secs() % 3600) / 60,
        now.as_secs() % 60
    );

    let state = StateFile {
        schema: 1,
        user_id: user_id.to_string(),
        folder_id: folder_id.map(String::from),
        newest_id,
        seen_count: posts.len(),
        last_run: timestamp,
        complete: true,
    };

    let json = serde_json::to_string_pretty(&state).map_err(std::io::Error::other)?;
    let tmp_path = format!("{}.tmp", state_path);
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, state_path)?;

    Ok(())
}
