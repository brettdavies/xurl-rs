//! Delivering a signed sample event to a webhook receiver under test.

use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::Request;
use hyper::header::HOST;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

use crate::error::{Error, Result};
use crate::webhooks::{SignedSample, SigningSecrets};

/// Posts a [`SignedSample`] signed for `secrets` to the receiver bound at
/// `address`, at `path`, the way X delivers an event, and returns the status
/// the receiver answered with.
///
/// # Example
///
/// ```
/// use tokio_util::sync::CancellationToken;
/// use xdk::webhooks::{DEFAULT_PATH, Receiver, ReceiverConfig, SigningSecrets};
///
/// #[tokio::main]
/// async fn main() -> xdk::Result<()> {
///     let secrets = SigningSecrets {
///         oauth2_client_secret: Some("the app's client secret".to_string()),
///         oauth1_consumer_secret: None,
///     };
///     let bind = "127.0.0.1:0".parse().expect("an address");
///     let config = ReceiverConfig::new(bind, secrets.clone());
///     let mut receiver = Receiver::bind(config, CancellationToken::new()).await?;
///
///     let status =
///         xdk::testing::deliver_signed_sample(receiver.local_addr(), DEFAULT_PATH, &secrets)
///             .await?;
///
///     assert_eq!(status, 200);
///     let event = receiver.next_event().await.expect("the sample");
///     assert!(event.body.starts_with(br#"{"xr_test_event":true"#));
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns a `validation` error when `secrets` holds no secret, an `io`
/// error when nothing accepts a connection at `address`, and a
/// `network-error` when the exchange fails after it connects.
pub async fn deliver_signed_sample(
    address: SocketAddr,
    path: &str,
    secrets: &SigningSecrets,
) -> Result<u16> {
    let sample = SignedSample::new(secrets)?;
    let stream = TcpStream::connect(address)
        .await
        .map_err(|e| Error::io(format!("could not connect to {address}: {e}")).with_source(e))?;
    let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
        .await
        .map_err(|e| Error::http(format!("could not reach {address}: {e}")).with_source(e))?;
    tokio::spawn(connection);
    let mut request = Request::post(path).header(HOST, address.to_string());
    for (name, value) in sample.headers() {
        request = request.header(name, value);
    }
    let request = request
        .body(Full::new(Bytes::from_static(sample.body.as_bytes())))
        .map_err(|_| Error::validation_of("invalid-request-path", path))?;
    let response = sender
        .send_request(request)
        .await
        .map_err(|e| Error::http(format!("{address} did not answer: {e}")).with_source(e))?;
    Ok(response.status().as_u16())
}
