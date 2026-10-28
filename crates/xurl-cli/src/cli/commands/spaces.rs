//! The `spaces` family: find X Spaces by keyword, by id, or by the accounts
//! that created them, and read the posts shared in one.

use serde_json::json;

use super::{
    Run, UNRESOLVED_ID, dry_run_or_validate, effective_limit, handles_lookup_call, list_paged,
    make_client, print_typed, resolve_account_ids, send_validated, with_flags,
};
use crate::cli::failure::CommandResult;
use crate::cli::{CommonFlags, SpacesCommands};
use xdk::api::{Client, shortcuts};

pub(super) async fn run(action: SpacesCommands, run: Run<'_>) -> CommandResult<()> {
    match action {
        SpacesCommands::Search {
            query,
            state,
            max_results,
            common,
        } => {
            let ctx = json!({
                "command": "spaces-search",
                "query": query,
                "state": state,
                "max_results": max_results,
            });
            let n = effective_limit(max_results, run.flags.global_limit);
            send_validated(
                run,
                ctx,
                || Ok(()),
                &common,
                |client: &Client| client.search_spaces(&query, state.as_deref(), n),
            )
            .await
        }
        SpacesCommands::Get { space_ids, common } => {
            let ctx = json!({"command": "spaces-get", "space_ids": space_ids});
            let validate = || shortcuts::validate_space_ids(&space_ids);
            send_validated(run, ctx, validate, &common, |client: &Client| {
                client.get_spaces(&space_ids)
            })
            .await
        }
        SpacesCommands::ByCreator { accounts, common } => by_creator(run, &accounts, &common).await,
        SpacesCommands::Posts {
            space_id,
            max_results,
            common,
        } => {
            let ctx = json!({"command": "spaces-posts", "space_id": space_id});
            let validate = || shortcuts::validate_space_id(&space_id);
            list_paged(run, ctx, validate, max_results, &common, |client, n| {
                client.get_space_posts(&space_id, n)
            })
            .await
        }
    }
}

/// `xr spaces by-creator`: numeric ids go out as they are, and handles
/// resolve to ids first, every one of them in one lookup.
async fn by_creator(run: Run<'_>, accounts: &[String], common: &CommonFlags) -> CommandResult<()> {
    let Run {
        cfg,
        auth,
        flags,
        out,
        stdout,
        ..
    } = run;
    let ctx = json!({"command": "spaces-by-creator", "accounts": accounts});
    let dry_run = dry_run_or_validate(flags.dry_run, ctx, || {
        shortcuts::validate_accounts(accounts)
    })?;
    let client = make_client(cfg, auth)?;
    if let Some(dry_run) = dry_run {
        let mut credentials = Vec::new();
        if let Some(lookup) = handles_lookup_call(&client, accounts, common) {
            credentials.push(lookup.auth_preflight().await);
        }
        let spaces = client.get_spaces_by_creator(&[UNRESOLVED_ID]);
        credentials.push(with_flags(spaces, common, None).auth_preflight().await);
        return dry_run.answer(out, stdout, credentials);
    }
    let user_ids = resolve_account_ids(&client, accounts, common).await?;
    let response = with_flags(client.get_spaces_by_creator(&user_ids), common, None)
        .send()
        .await?;
    print_typed(out, stdout, &response)?;
    Ok(())
}
