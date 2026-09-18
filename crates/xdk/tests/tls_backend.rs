//! One TLS handshake with the real X API, through the client, under whichever
//! backend the build selected.
//!
//! The mocked suite runs over plain http, so nothing in it proves that a
//! configured backend can actually negotiate TLS. This test sends one request
//! with a bearer token X will reject; the rejection arrives only after the
//! handshake and costs nothing, while a backend that cannot connect surfaces
//! as a transport error instead. It reaches the network, so it is ignored by
//! default and run by hand once per backend:
//!
//! ```sh
//! cargo test -p xdk-rs --test tls_backend -- --ignored
//! cargo test -p xdk-rs --no-default-features --features native-tls --test tls_backend -- --ignored
//! ```

use xdk::Error;
use xdk::api::Client;

#[tokio::test]
#[ignore = "reaches api.x.com over TLS; run by hand once per backend"]
async fn the_selected_backend_completes_a_handshake_with_x() {
    let client = Client::builder()
        .bearer("not-a-real-token")
        .build()
        .expect("client builds");

    let error = client
        .read_post("20")
        .send()
        .await
        .expect_err("a bogus bearer is rejected");
    assert!(
        matches!(error, Error::Api { status: 401, .. }),
        "expected X to answer 401 after the handshake, got: {error}"
    );
}
