//! `xr media amplify`: the Amplify programs a publisher can add posts to, the
//! posts in each, and publishing one of your posts into programs.

use serde_json::json;

use super::{Invalid, Run, no_checks, send_validated, valid_post_id};
use crate::cli::AmplifyCommands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;
use xdk::api::shortcuts::{
    AmplifyPromotion, resolve_post_id, validate_amplify_promotion, validate_program_id,
};

pub(super) async fn run(action: AmplifyCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        AmplifyCommands::List { common } => {
            let ctx = json!({"command": "media-amplify-list"});
            send_validated(run, ctx, no_checks, &common, |client: &Client| {
                client.get_amplify_programs()
            })
            .await
        }
        AmplifyCommands::Get { program_id, common } => {
            let ctx = json!({"command": "media-amplify-get", "program_id": program_id});
            let validate = || validate_program_id(&program_id);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_amplify_program(&program_id)
            })
            .await
        }
        AmplifyCommands::Posts { program_id, common } => {
            let ctx = json!({"command": "media-amplify-posts", "program_id": program_id});
            let validate = || validate_program_id(&program_id);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_amplify_program_posts(&program_id)
            })
            .await
        }
        AmplifyCommands::Add {
            post_id,
            program,
            block_advertiser,
            ends_at,
            common,
        } => {
            let promotion = AmplifyPromotion {
                program_ids: program,
                blocked_advertiser_ids: (!block_advertiser.is_empty()).then_some(block_advertiser),
                promotion_ends_at: ends_at,
            };
            let ctx = json!({
                "command": "media-amplify-add",
                "post_id": resolve_post_id(&post_id),
                "program_ids": promotion.program_ids,
                "blocked_advertiser_ids": promotion.blocked_advertiser_ids,
                "promotion_ends_at": promotion.promotion_ends_at,
            });
            let validate = || {
                valid_post_id(&post_id)?;
                validate_amplify_promotion(&promotion).map_err(Invalid::from)
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.add_post_to_amplify_programs(&post_id, &promotion)
            })
            .await
        }
    }
}
