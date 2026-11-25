//! Requests sent with the credential they go out under kept at hand, so a 401
//! names it.
//!
//! The credential is read before the request goes out, by the scheme
//! selection the send itself runs, and nothing is sent to read it. A request
//! that attaches no credential, or whose credential cannot be read, fails as
//! any other does.

use serde::de::DeserializeOwned;
use xdk::api::{ApiResponse, AuthPreflight, Call, Client, RequestOptions, User};

use crate::cli::failure::{CommandResult, Failure};

/// [`Call::send`], with a 401 carrying the credential the call went out under.
pub(super) async fn send<T: DeserializeOwned>(call: Call<T>) -> CommandResult<T> {
    let credential = call.stored_credential().await.ok().flatten();
    call.send()
        .await
        .map_err(|error| Failure::of_request(error, credential))
}

/// [`Call::send_saving_identity`], with a 401 carrying the credential the call
/// went out under.
pub(super) async fn send_saving_identity(
    call: Call<ApiResponse<User>>,
) -> CommandResult<ApiResponse<User>> {
    let credential = call.stored_credential().await.ok().flatten();
    call.send_saving_identity()
        .await
        .map_err(|error| Failure::of_request(error, credential))
}

/// The credential a raw request goes out under, for [`Failure::of_request`].
pub(super) async fn raw_credential(
    client: &Client,
    options: &RequestOptions,
) -> Option<AuthPreflight> {
    client.stored_credential(options).await.ok().flatten()
}
