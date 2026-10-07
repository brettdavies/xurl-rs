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
    let signature_base_string = signature_base_string(method, &parsed_url, params);

    let signing_key = format!("{}&{}", encode(consumer_secret), encode(token_secret));

    let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
        .map_err(|e| Error::auth_with_cause("SignatureGenerationError", &e).with_source(e))?;
    mac.update(signature_base_string.as_bytes());
    let result = mac.finalize();

    Ok(BASE64_STANDARD.encode(result.into_bytes()))
}

/// The signature base string of RFC 5849 section 3.4.1: the method, the URL
/// without its query, and the sorted parameters, each percent-encoded and
/// joined by `&`.
fn signature_base_string(method: &str, url: &Url, params: &BTreeMap<String, String>) -> String {
    let base_url = format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap_or(""),
        url.path()
    );

    let param_pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect();
    let param_string = param_pairs.join("&");

    format!(
        "{}&{}&{}",
        method.to_uppercase(),
        encode(&base_url),
        encode(&param_string)
    )
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

#[cfg(test)]
mod tests {
    use percent_encoding::percent_decode_str;
    use proptest::prelude::*;

    use super::*;

    fn decode(encoded: &str) -> String {
        percent_decode_str(encoded)
            .decode_utf8()
            .expect("the encoder writes UTF-8")
            .into_owned()
    }

    proptest! {
        /// The base string carries the parameters without loss: its third
        /// part decodes to the sorted `key=value` pairs, and each side of a
        /// pair decodes to the string that went in. A separator that reached
        /// the base string unencoded would split a pair in the wrong place.
        #[test]
        fn the_base_string_decodes_to_the_parameters_that_built_it(
            params in prop::collection::btree_map("\\PC{0,12}", "\\PC{0,24}", 0..8),
        ) {
            let url = Url::parse("https://api.x.com/2/tweets/search/recent").unwrap();
            let base = signature_base_string("get", &url, &params);

            let parts: Vec<&str> = base.split('&').collect();
            prop_assert_eq!(parts.len(), 3, "method, URL, and parameters: {}", base);
            prop_assert_eq!(parts[0], "GET");
            prop_assert_eq!(decode(parts[1]), "https://api.x.com/2/tweets/search/recent");

            let pairs = decode(parts[2]);
            let decoded: BTreeMap<String, String> = pairs
                .split('&')
                .filter(|pair| !pair.is_empty())
                .map(|pair| {
                    let (key, value) = pair.split_once('=').expect("each pair has one separator");
                    (decode(key), decode(value))
                })
                .collect();
            prop_assert_eq!(decoded, params);
        }
    }
}
