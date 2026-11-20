//! `xr media amplify`: the Amplify programs a publisher can add posts to, the
//! posts in each, and publishing one of your posts into programs.

use serde_json::json;

use super::{Invalid, Run, no_checks, send_validated, valid_post_id, without_unset};
use crate::cli::AmplifyCommands;
use crate::cli::failure::CommandResult;
use xdk::api::Client;
use xdk::api::shortcuts::{
    AmplifyPromotion, resolve_post_id, validate_advertiser_id, validate_amplify_promotion,
    validate_program_id,
};

/// The refusal of `promotion` under `rule`, naming the program, advertiser, or
/// time that broke it: a repeated flag leaves the caller guessing which entry
/// failed.
fn refused(rule: &'static str, promotion: &AmplifyPromotion) -> Invalid {
    let value = match rule {
        "invalid-program-id" => promotion
            .program_ids
            .iter()
            .find(|id| validate_program_id(id).is_err()),
        "invalid-advertiser-id" => promotion
            .blocked_advertiser_ids
            .iter()
            .flatten()
            .find(|id| validate_advertiser_id(id).is_err()),
        "invalid-promotion-end" => promotion.promotion_ends_at.as_ref(),
        _ => None,
    };
    value.map_or_else(|| Invalid::from(rule), |value| Invalid::naming(rule, value))
}

/// The check of a program id, naming it when refused.
fn checked_program(program_id: &str) -> std::result::Result<(), Invalid> {
    validate_program_id(program_id).map_err(|rule| Invalid::naming(rule, program_id))
}

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
            let validate = || checked_program(&program_id);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_amplify_program(&program_id)
            })
            .await
        }
        AmplifyCommands::Posts { program_id, common } => {
            let ctx = json!({"command": "media-amplify-posts", "program_id": program_id});
            let validate = || checked_program(&program_id);
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
            let ctx = without_unset(json!({
                "command": "media-amplify-add",
                "post_id": resolve_post_id(&post_id),
                "program_ids": promotion.program_ids,
                "blocked_advertiser_ids": promotion.blocked_advertiser_ids,
                "promotion_ends_at": promotion.promotion_ends_at,
            }));
            let validate = || {
                valid_post_id(&post_id)?;
                validate_amplify_promotion(&promotion).map_err(|rule| refused(rule, &promotion))
            };
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.add_post_to_amplify_programs(&post_id, &promotion)
            })
            .await
        }
    }
}
