//! One request in flight: a shortcut's endpoint plus the per-call options,
//! sent once.

use std::marker::PhantomData;
use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::api::auth_matrix::WireScheme;
use crate::api::response::types::decode;
use crate::error::{Error, Result};

use super::{CallOptions, Client, RequestOptions, RequestTarget};

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

    /// Sets the scheme from its wire string, unvalidated, so the binary's
    /// `--auth` value reaches scheme selection exactly as typed.
    #[doc(hidden)]
    pub fn auth_wire(mut self, scheme: impl Into<String>) -> Self {
        self.options.auth_type = scheme.into();
        self
    }
}

impl<T: DeserializeOwned> Call<T> {
    /// Sends under `scheme` instead of the auto-detected one.
    pub fn auth(self, scheme: WireScheme) -> Self {
        self.auth_wire(scheme.as_wire())
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
