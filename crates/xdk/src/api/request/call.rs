//! One request in flight: a shortcut's endpoint plus the per-call options,
//! sent once.

use std::marker::PhantomData;
use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::api::auth_matrix::{WireScheme, endpoints};
use crate::api::response::types::{ApiResponse, User, decode};
use crate::error::{Error, Result};
use crate::store::STORE_TARGET;

use super::{AuthPreflight, CallOptions, Client, RequestOptions, RequestTarget};

/// A shortcut request that has not been sent.
///
/// Every shortcut on [`Client`] returns one of these; the per-call options
/// are set through its methods and [`Call::send`] performs the request.
/// `send` consumes the call, so a call goes out at most once:
///
/// ```compile_fail
/// # async fn run(client: xdk::api::Client) -> xdk::Result<()> {
/// let call = client.get_me();
/// call.send().await?;
/// call.send().await?; // error[E0382]: use of moved value: `call`
/// # Ok(()) }
/// ```
///
/// The call owns a clone of its client (an `Arc` increment), so it moves
/// into a spawned task with no lifetime parameter.
#[must_use = "a Call does nothing until it is sent"]
pub struct Call<T> {
    client: Client,
    options: CallOptions,
    request: RequestOptions,
    paginated: bool,
    failed: Option<Error>,
    response: PhantomData<fn() -> T>,
}

// A `T` that is neither `Send` nor `Sync` is the only one that can catch the
// bound starting to depend on `T`.
crate::assert_send_sync!(Call<std::rc::Rc<()>>);

impl<T> std::fmt::Debug for Call<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Call")
            .field("method", &self.request.method)
            .field("target", &self.request.target)
            .field("paginated", &self.paginated)
            .field("failed", &self.failed)
            .finish_non_exhaustive()
    }
}

/// The stored login a `/2/users/me` request goes out with.
enum StoredLogin {
    /// An `OAuth2` token in `app`, under `stored_as` or unnamed.
    OAuth2 {
        app: String,
        stored_as: Option<String>,
    },
    /// The `OAuth1` access pair of `app`.
    OAuth1 { app: String },
}

impl Call<ApiResponse<User>> {
    /// Sends the call, and stores what the answer says of the login it went
    /// out with: when this is `GET /2/users/me` under a user credential from
    /// the store, the account's id is stored beside that credential, and an
    /// `OAuth2` token stored without a username is stored under the one X
    /// answered with.
    ///
    /// Any other call is sent as [`Call::send`] sends it. A lookup of
    /// another user says nothing about the stored login, so it stores
    /// nothing. A store that cannot be written is a warning on `xdk::auth`,
    /// not a failure: the answer is still X's.
    ///
    /// # Errors
    ///
    /// Everything [`Call::send`] returns.
    // `xr` is the caller: `whoami`, and the commands that resolve the
    // caller's id before they act (`crates/xurl-cli/src/cli/commands/`).
    #[doc(hidden)]
    pub async fn send_saving_identity(self) -> Result<ApiResponse<User>> {
        let asks_who_i_am = self.request.method == endpoints::GET_ME.method
            && matches!(
                &self.request.target,
                RequestTarget::Template { path, .. } if path == endpoints::GET_ME.path
            );
        let login = if asks_who_i_am {
            match self.auth_preflight().await {
                Ok(Some(AuthPreflight {
                    scheme: WireScheme::OAuth2,
                    app: Some(app),
                    username,
                    ..
                })) => Some(StoredLogin::OAuth2 {
                    app,
                    stored_as: username,
                }),
                Ok(Some(AuthPreflight {
                    scheme: WireScheme::OAuth1,
                    app: Some(app),
                    ..
                })) => Some(StoredLogin::OAuth1 { app }),
                _ => None,
            }
        } else {
            None
        };
        let client = self.client.clone();
        let response = self.send().await?;
        if let Some(login) = login
            && let Err(error) = client.store_identity(&login, &response.data).await
        {
            tracing::warn!(
                target: "xdk::auth",
                "the answer could not be stored with the login: {error}"
            );
        }
        Ok(response)
    }
}

impl Client {
    /// Stores `user` as the account behind `login`.
    async fn store_identity(&self, login: &StoredLogin, user: &User) -> Result<()> {
        let mut auth = self.auth().await?;
        let store = &mut auth.token_store;
        match login {
            StoredLogin::OAuth1 { app } => {
                store.set_oauth1_user_id_for_app(app, &user.id)?;
            }
            StoredLogin::OAuth2 {
                app,
                stored_as: Some(name),
            } => {
                store.set_oauth2_user_id_for_app(app, name, &user.id)?;
            }
            StoredLogin::OAuth2 {
                app,
                stored_as: None,
            } => {
                // A token already stored under the username is the newer
                // one, written by a refresh that learned the name; the
                // unnamed one is then not the login this answer is about.
                if store.name_unnamed_oauth2_token_for_app(app, &user.username)? {
                    tracing::info!(
                        target: STORE_TARGET,
                        kind = "login-named",
                        name = %user.username,
                        "stored the unnamed OAuth2 login under its username"
                    );
                    store.set_oauth2_user_id_for_app(app, &user.username, &user.id)?;
                }
            }
        }
        Ok(())
    }
}

impl<T> Call<T> {
    pub(crate) fn new(client: &Client, request: RequestOptions) -> Self {
        Self {
            client: client.clone(),
            options: CallOptions::default(),
            request,
            paginated: false,
            failed: None,
            response: PhantomData,
        }
    }

    /// A call whose request could not be built; `send` returns `error`.
    pub(crate) fn failed(client: &Client, error: Error) -> Self {
        let mut call = Self::new(client, RequestOptions::default());
        call.failed = Some(error);
        call
    }

    /// Marks a list endpoint, the only kind that sends
    /// [`Call::pagination_token`].
    pub(crate) fn paginated(mut self) -> Self {
        self.paginated = true;
        self
    }

    /// Sets the scheme from its wire string without validating it.
    ///
    /// Nothing checks that `scheme` names a scheme this client can send. An
    /// unknown string reaches scheme selection as typed and surfaces there as
    /// an auth-method mismatch, which is what `xr --auth <value>` relies on to
    /// report the user's own text back. Prefer [`Call::auth`], which takes a
    /// [`WireScheme`] and cannot be misspelled.
    #[doc(hidden)]
    pub fn auth_wire_unchecked(mut self, scheme: impl Into<String>) -> Self {
        self.options.auth_type = scheme.into();
        self
    }
}

impl<T: DeserializeOwned> Call<T> {
    /// Sends under `scheme` instead of the auto-detected one.
    pub fn auth(self, scheme: WireScheme) -> Self {
        self.auth_wire_unchecked(scheme.as_wire())
    }

    /// Selects which stored `OAuth2` user a store-backed client sends as. A
    /// client built from credentials holds one user and ignores it.
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.options.username = username.into();
        self
    }

    /// Sends the `X-B3-Flags: 1` header that flags the request for upstream
    /// tracing.
    pub fn trace(mut self, on: bool) -> Self {
        self.options.trace = on;
        self
    }

    /// Sends with no `Authorization` header at all, skipping scheme
    /// selection; a probe of what an endpoint answers unauthenticated.
    pub fn no_auth(mut self, on: bool) -> Self {
        self.options.no_auth = on;
        self
    }

    /// Bounds this one request in place of the client-level timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.options.timeout = Some(timeout);
        self
    }

    /// The cursor a list endpoint continues from; single-item endpoints
    /// ignore it.
    pub fn pagination_token(mut self, token: impl Into<String>) -> Self {
        self.options.pagination_token = token.into();
        self
    }

    /// Adds a request header. One the client would set itself
    /// (`Authorization`, `User-Agent`, `Content-Type`, `X-B3-Flags`) is
    /// replaced by yours.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.request
            .headers
            .push(format!("{}: {}", name.into(), value.into()));
        self
    }

    /// Reports the credential this call would be sent with, without sending
    /// it: [`Client::auth_preflight`] for the request as built so far.
    ///
    /// # Errors
    ///
    /// Everything [`Client::auth_preflight`] returns, plus the error of a
    /// request that could not be built.
    pub async fn auth_preflight(&self) -> Result<Option<AuthPreflight>> {
        if let Some(error) = &self.failed {
            // `failed` holds only a body that did not serialize.
            return Err(Error::json(error.to_string()));
        }
        let mut request = self.request.clone();
        request.auth_type.clone_from(&self.options.auth_type);
        request.username.clone_from(&self.options.username);
        request.no_auth = self.options.no_auth;
        self.client.auth_preflight(&request).await
    }

    /// Performs the request and decodes the response.
    ///
    /// # Errors
    ///
    /// Everything [`Client::send_request`] returns, plus [`Error::Json`]
    /// when the body does not decode into `T` and [`Error::Validation`]
    /// when a 200 carries only `errors`.
    pub async fn send(self) -> Result<T> {
        let Self {
            client,
            options,
            mut request,
            paginated,
            failed,
            ..
        } = self;
        if let Some(error) = failed {
            return Err(error);
        }
        request.auth_type = options.auth_type;
        request.username = options.username;
        request.trace = options.trace;
        request.no_auth = options.no_auth;
        if paginated
            && !options.pagination_token.is_empty()
            && let RequestTarget::Template { query, .. } = &mut request.target
        {
            query.push(("pagination_token".to_string(), options.pagination_token));
        }
        let timeout = options.timeout.unwrap_or_else(|| client.request_timeout());
        decode(client.send_request_with(&request, timeout).await?)
    }
}
