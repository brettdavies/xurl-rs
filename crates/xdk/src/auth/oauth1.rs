//! `OAuth1` HMAC-SHA1 signature generation.
//!
//! Implements the full `OAuth1` signature base string construction and
//! HMAC-SHA1 signing as specified by RFC 5849.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use hmac::{Hmac, KeyInit, Mac};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use sha1::Sha1;
use url::Url;

use crate::error::{Error, Result};
use crate::store::OAuth1Token;

type HmacSha1 = Hmac<Sha1>;

/// Builds a complete `OAuth1` Authorization header.
///
/// # Errors
///
/// Returns an error if the URL is invalid or HMAC signature generation fails.
pub fn build_oauth1_header(
    method: &str,
    url_str: &str,
    token: &OAuth1Token,
    additional_params: Option<&BTreeMap<String, String>>,
) -> Result<String> {
    build_oauth1_header_with_nonce_ts(method, url_str, token, additional_params, None, None)
}

/// Builds a complete `OAuth1` Authorization header with injectable nonce and timestamp.
/// Used for deterministic testing.
///
/// # Errors
///
/// Returns an error if the URL is invalid or HMAC signature generation fails.
pub fn build_oauth1_header_with_nonce_ts(
    method: &str,
    url_str: &str,
    token: &OAuth1Token,
    additional_params: Option<&BTreeMap<String, String>>,
    fixed_nonce: Option<&str>,
    fixed_timestamp: Option<&str>,
) -> Result<String> {
    let parsed_url = Url::parse(url_str)
        .map_err(|e| Error::invalid_url(format!("{url_str}: {e}")).with_source(e))?;

    let mut params = BTreeMap::new();

    // Add query parameters
    for (key, value) in parsed_url.query_pairs() {
        params.insert(key.to_string(), value.to_string());
    }

    // Add additional parameters
    if let Some(extra) = additional_params {
        for (key, value) in extra {
            params.insert(key.clone(), value.clone());
        }
    }

    // Add OAuth parameters
    params.insert("oauth_consumer_key".to_string(), token.consumer_key.clone());
    params.insert(
        "oauth_nonce".to_string(),
        fixed_nonce.map_or_else(generate_nonce, str::to_string),
    );
    params.insert(
        "oauth_signature_method".to_string(),
        "HMAC-SHA1".to_string(),
    );
    params.insert(
        "oauth_timestamp".to_string(),
        fixed_timestamp.map_or_else(generate_timestamp, str::to_string),
    );
    params.insert("oauth_token".to_string(), token.access_token.clone());
    params.insert("oauth_version".to_string(), "1.0".to_string());

    let signature = generate_signature(
        method,
        url_str,
        &params,
        &token.consumer_secret,
        &token.token_secret,
    )?;

    let oauth_params = [
        format!("oauth_consumer_key=\"{}\"", encode(&token.consumer_key)),
        format!("oauth_nonce=\"{}\"", encode(&params["oauth_nonce"])),
        format!("oauth_signature=\"{}\"", encode(&signature)),
        format!("oauth_signature_method=\"{}\"", encode("HMAC-SHA1")),
        format!("oauth_timestamp=\"{}\"", encode(&params["oauth_timestamp"])),
        format!("oauth_token=\"{}\"", encode(&token.access_token)),
        format!("oauth_version=\"{}\"", encode("1.0")),
    ];

    Ok(format!("OAuth {}", oauth_params.join(", ")))
}

/// Generates the `OAuth1` signature.
fn generate_signature(
    method: &str,
    url_str: &str,
    params: &BTreeMap<String, String>,
    consumer_secret: &str,
    token_secret: &str,
) -> Result<String> {
    let parsed_url = Url::parse(url_str)
        .map_err(|e| Error::invalid_url(format!("{url_str}: {e}")).with_source(e))?;

    let base_url = format!(
        "{}://{}{}",
        parsed_url.scheme(),
        parsed_url.host_str().unwrap_or(""),
        parsed_url.path()
    );

    let param_pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect();
    let param_string = param_pairs.join("&");

    let signature_base_string = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        encode(&base_url),
        encode(&param_string)
    );

    let signing_key = format!("{}&{}", encode(consumer_secret), encode(token_secret));

    let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
        .map_err(|e| Error::auth_with_cause("SignatureGenerationError", &e).with_source(e))?;
    mac.update(signature_base_string.as_bytes());
    let result = mac.finalize();

    Ok(BASE64_STANDARD.encode(result.into_bytes()))
}

/// Generates a random nonce.
#[must_use]
pub fn generate_nonce() -> String {
    let n: u64 = rand::random_range(0..1_000_000_000);
    n.to_string()
}

/// Generates the current Unix timestamp as a string.
#[must_use]
pub fn generate_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

/// Every byte RFC 5849 section 3.6 encodes: all but the RFC 3986 unreserved
/// set, `A-Z a-z 0-9 - . _ ~`.
const ENCODED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// Percent-encodes a string as RFC 5849 section 3.6 requires, with uppercase
/// hex.
///
/// X recomputes a signature from the request with this encoding, so a query
/// encoder's output (`+` for a space, `%7E` for `~`, a bare `*`) signs a
/// string X does not build and the request fails verification.
#[must_use]
pub fn encode(s: &str) -> String {
    utf8_percent_encode(s, ENCODED).to_string()
}
