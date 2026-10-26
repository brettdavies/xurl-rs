//! The `articles` family: save a draft X Article from a plain-text body,
//! and publish one.

use std::io::Read;

use serde_json::json;

use super::{Run, send_confirmed, send_validated};
use crate::cli::ArticlesCommands;
use crate::cli::failure::CommandResult;
use xdk::Error;
use xdk::api::{Client, shortcuts};

pub(super) async fn run(action: ArticlesCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        ArticlesCommands::Draft {
            title,
            body_file,
            cover,
            common,
        } => {
            let body = read_body(&body_file)?;
            let ctx = json!({
                "command": "articles-draft",
                "title": title,
                "body_file": body_file,
                "blocks": shortcuts::article_blocks(&body).len(),
                "cover": cover,
            });
            let validate = || {
                shortcuts::validate_article_title(&title)?;
                shortcuts::validate_article_body(&body)?;
                cover
                    .as_deref()
                    .map_or(Ok(()), shortcuts::validate_media_id)
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.create_article_draft(&title, &body, cover.as_deref())
            })
            .await
        }
        ArticlesCommands::Publish {
            article_id,
            force,
            common,
        } => {
            let ctx = json!({"command": "articles-publish", "article_id": article_id});
            let prompt = format!("Publish article {article_id}?");
            let validate = || shortcuts::validate_article_id(&article_id);
            send_confirmed(run, ctx, &prompt, force, validate, &common, |client| {
                client.publish_article(&article_id)
            })
            .await
        }
    }
}

/// The article body `path` names, or stdin's when it is `-`.
fn read_body(path: &str) -> Result<String, Error> {
    if path == "-" {
        let mut body = String::new();
        std::io::stdin().read_to_string(&mut body).map_err(|e| {
            Error::io(format!("cannot read --body-file from stdin: {e}")).with_source(e)
        })?;
        return Ok(body);
    }
    std::fs::read_to_string(path)
        .map_err(|e| Error::io(format!("cannot read --body-file {path}: {e}")).with_source(e))
}
