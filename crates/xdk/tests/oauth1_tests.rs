//! OAuth1 signature verification tests.
//!
//! Tests deterministic OAuth1 HMAC-SHA1 signature generation against
//! known test vectors from RFC 5849, X's published signature example, and
//! pre-captured reference values.

use std::collections::BTreeMap;

use xdk::auth::oauth1::{build_oauth1_header_with_nonce_ts, encode};
use xdk::store::OAuth1Token;

// ═══════════════════════════════════════════════════════════════════════════
// RFC 5849 Section 3.4 — Percent-Encoding Test Vectors
// ═══════════════════════════════════════════════════════════════════════════

// RFC 5849 Section 3.6: only the RFC 3986 unreserved characters (letters,
// digits, `-`, `.`, `_`, `~`) stay bare, and every other byte is `%XX` in
// uppercase hex. X recomputes the signature with this encoding, so a query
// encoder's `+` for a space or `%7E` for a tilde signs a different string.

#[test]
fn test_percent_encoding_unreserved_chars() {
    // Letters and digits are never encoded
    assert_eq!(
        encode("abcdefghijklmnopqrstuvwxyz"),
        "abcdefghijklmnopqrstuvwxyz"
    );
    assert_eq!(
        encode("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    );
    assert_eq!(encode("0123456789"), "0123456789");
    assert_eq!(encode("-"), "-");
    assert_eq!(encode("."), ".");
    assert_eq!(encode("_"), "_");
    assert_eq!(encode("~"), "~");
}

#[test]
fn test_percent_encoding_reserved_chars() {
    // Reserved characters MUST be encoded
    assert_eq!(encode(":"), "%3A");
    assert_eq!(encode("/"), "%2F");
    assert_eq!(encode("?"), "%3F");
    assert_eq!(encode("#"), "%23");
    assert_eq!(encode("["), "%5B");
    assert_eq!(encode("]"), "%5D");
    assert_eq!(encode("@"), "%40");
    assert_eq!(encode("!"), "%21");
    assert_eq!(encode("$"), "%24");
    assert_eq!(encode("&"), "%26");
    assert_eq!(encode("'"), "%27");
    assert_eq!(encode("("), "%28");
    assert_eq!(encode(")"), "%29");
    assert_eq!(encode("*"), "%2A");
    assert_eq!(encode("+"), "%2B");
    assert_eq!(encode(","), "%2C");
    assert_eq!(encode(";"), "%3B");
    assert_eq!(encode("="), "%3D");
}

#[test]
fn test_percent_encoding_special_chars() {
    assert_eq!(encode("%"), "%25");
    assert_eq!(encode(" "), "%20");
    // A non-ASCII character is encoded byte by byte from its UTF-8 form.
    assert_eq!(encode("é"), "%C3%A9");
}

// ═══════════════════════════════════════════════════════════════════════════
// Deterministic signature generation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_oauth1_deterministic_signature() {
    // Using fixed nonce and timestamp, the signature should be deterministic
    let token = OAuth1Token {
        consumer_key: "dpf43f3p2l4k3l03".to_string(),
        consumer_secret: "kd94hf93k423kf44".to_string(),
        access_token: "nnch734d00sl2jdk".to_string(),
        token_secret: "pfkkdhi9sl3r4s00".to_string(),
        user_id: None,
    };

    let fixed_nonce = "kllo9940pd9333jh";
    let fixed_timestamp = "1191242096";

    let header1 = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/users/me",
        &token,
        None,
        Some(fixed_nonce),
        Some(fixed_timestamp),
    )
    .unwrap();

    let header2 = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/users/me",
        &token,
        None,
        Some(fixed_nonce),
        Some(fixed_timestamp),
    )
    .unwrap();

    // Same inputs → same output
    assert_eq!(
        header1, header2,
        "Deterministic: same inputs should produce identical headers"
    );

    // Verify the header format
    assert!(
        header1.starts_with("OAuth "),
        "Header should start with 'OAuth '"
    );
    assert!(header1.contains("oauth_consumer_key=\"dpf43f3p2l4k3l03\""));
    assert!(header1.contains("oauth_token=\"nnch734d00sl2jdk\""));
    assert!(header1.contains(&format!("oauth_nonce=\"{fixed_nonce}\"")));
    assert!(header1.contains(&format!("oauth_timestamp=\"{fixed_timestamp}\"")));
    assert!(header1.contains("oauth_signature_method=\"HMAC-SHA1\""));
    assert!(header1.contains("oauth_version=\"1.0\""));
    assert!(header1.contains("oauth_signature="));
}

#[test]
fn test_oauth1_signature_changes_with_nonce() {
    let token = OAuth1Token {
        consumer_key: "consumer-key".to_string(),
        consumer_secret: "consumer-secret".to_string(),
        access_token: "access-token".to_string(),
        token_secret: "token-secret".to_string(),
        user_id: None,
    };

    let header1 = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/users/me",
        &token,
        None,
        Some("nonce1"),
        Some("1000000000"),
    )
    .unwrap();

    let header2 = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/users/me",
        &token,
        None,
        Some("nonce2"),
        Some("1000000000"),
    )
    .unwrap();

    // Different nonce → different signature
    assert_ne!(
        header1, header2,
        "Different nonces should produce different headers"
    );
}

#[test]
fn test_oauth1_signature_changes_with_method() {
    let token = OAuth1Token {
        consumer_key: "ck".to_string(),
        consumer_secret: "cs".to_string(),
        access_token: "at".to_string(),
        token_secret: "ts".to_string(),
        user_id: None,
    };

    let header_get = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/tweets",
        &token,
        None,
        Some("fixed_nonce"),
        Some("1000000000"),
    )
    .unwrap();

    let header_post = build_oauth1_header_with_nonce_ts(
        "POST",
        "https://api.x.com/2/tweets",
        &token,
        None,
        Some("fixed_nonce"),
        Some("1000000000"),
    )
    .unwrap();

    assert_ne!(
        header_get, header_post,
        "Different methods should produce different signatures"
    );
}

#[test]
fn test_oauth1_signature_with_query_params() {
    let token = OAuth1Token {
        consumer_key: "ck".to_string(),
        consumer_secret: "cs".to_string(),
        access_token: "at".to_string(),
        token_secret: "ts".to_string(),
        user_id: None,
    };

    // Query params should be included in the signature base string
    let header = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/tweets?query=hello&max_results=10",
        &token,
        None,
        Some("fixed_nonce"),
        Some("1000000000"),
    )
    .unwrap();

    assert!(header.starts_with("OAuth "));

    // Without query params should produce a different signature
    let header_no_params = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/tweets",
        &token,
        None,
        Some("fixed_nonce"),
        Some("1000000000"),
    )
    .unwrap();

    assert_ne!(
        header, header_no_params,
        "Query params should affect the signature"
    );
}

#[test]
fn test_oauth1_signature_with_additional_params() {
    let token = OAuth1Token {
        consumer_key: "ck".to_string(),
        consumer_secret: "cs".to_string(),
        access_token: "at".to_string(),
        token_secret: "ts".to_string(),
        user_id: None,
    };

    let mut extra = BTreeMap::new();
    extra.insert("extra_param".to_string(), "extra_value".to_string());

    let header_with = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/users/me",
        &token,
        Some(&extra),
        Some("fixed_nonce"),
        Some("1000000000"),
    )
    .unwrap();

    let header_without = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/users/me",
        &token,
        None,
        Some("fixed_nonce"),
        Some("1000000000"),
    )
    .unwrap();

    assert_ne!(
        header_with, header_without,
        "Additional params should affect the signature"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// RFC 5849 Section 3.4.1 — Signature Base String construction
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_oauth1_known_vector_signature() {
    // Known test vector: we compute with fixed inputs and verify the output
    // is stable and correct format.
    //
    // Using the RFC 5849 example credentials:
    // Consumer key: dpf43f3p2l4k3l03
    // Consumer secret: kd94hf93k423kf44
    // Token: nnch734d00sl2jdk
    // Token secret: pfkkdhi9sl3r4s00
    // Nonce: kllo9940pd9333jh
    // Timestamp: 1191242096

    let token = OAuth1Token {
        consumer_key: "dpf43f3p2l4k3l03".to_string(),
        consumer_secret: "kd94hf93k423kf44".to_string(),
        access_token: "nnch734d00sl2jdk".to_string(),
        token_secret: "pfkkdhi9sl3r4s00".to_string(),
        user_id: None,
    };

    let header = build_oauth1_header_with_nonce_ts(
        "GET",
        "http://photos.example.net/photos?file=vacation.jpg&size=original",
        &token,
        None,
        Some("kllo9940pd9333jh"),
        Some("1191242096"),
    )
    .unwrap();

    // Extract the signature from the header
    let sig_start = header.find("oauth_signature=\"").unwrap() + 17;
    let sig_end = header[sig_start..].find('"').unwrap() + sig_start;
    let signature = &header[sig_start..sig_end];

    // The signature should be a percent-encoded base64 string
    // We can't easily predict the exact value without reimplementing the algorithm,
    // but we can verify it's non-empty, base64-ish after decoding the percent-encoding
    assert!(!signature.is_empty(), "Signature should not be empty");

    // Verify it's URL-encoded (the base64 = and + chars get encoded)
    // The signature value here is percent-encoded, so decode it
    let decoded: String = percent_decode(signature);
    // The decoded value should be valid base64
    assert!(
        base64_valid(&decoded),
        "Decoded signature should be valid base64, got: {decoded}"
    );

    // Pin the exact signature for regression detection.
    // This was computed by running the test once and capturing the output.
    // If the algorithm changes, this test will catch it.
    let expected_sig = "tR3%2BTy81lMeYAr%2FFid0kMTYa%2FWM%3D";
    assert_eq!(
        signature, expected_sig,
        "Signature should match known value"
    );
}

/// Percent-decode a string (reverse of encode).
fn percent_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

/// Check if a string looks like valid base64.
fn base64_valid(s: &str) -> bool {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s).is_ok()
}

// ═══════════════════════════════════════════════════════════════════════════
// Error cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_oauth1_invalid_url_returns_error() {
    let token = OAuth1Token {
        consumer_key: "ck".to_string(),
        consumer_secret: "cs".to_string(),
        access_token: "at".to_string(),
        token_secret: "ts".to_string(),
        user_id: None,
    };

    let result = build_oauth1_header_with_nonce_ts(
        "GET",
        "not a valid url",
        &token,
        None,
        Some("nonce"),
        Some("12345"),
    );

    assert!(result.is_err(), "Invalid URL should return an error");
}

#[test]
fn test_oauth1_header_contains_all_required_params() {
    let token = OAuth1Token {
        consumer_key: "my-consumer-key".to_string(),
        consumer_secret: "my-consumer-secret".to_string(),
        access_token: "my-access-token".to_string(),
        token_secret: "my-token-secret".to_string(),
        user_id: None,
    };

    let header = build_oauth1_header_with_nonce_ts(
        "POST",
        "https://api.x.com/2/tweets",
        &token,
        None,
        Some("test_nonce_123"),
        Some("1700000000"),
    )
    .unwrap();

    // RFC 5849 Section 3.5.1: Authorization Header
    assert!(header.starts_with("OAuth "), "Must start with 'OAuth '");

    // All required OAuth params must be present
    let required_params = [
        "oauth_consumer_key",
        "oauth_nonce",
        "oauth_signature",
        "oauth_signature_method",
        "oauth_timestamp",
        "oauth_token",
        "oauth_version",
    ];

    for param in &required_params {
        assert!(
            header.contains(param),
            "Header must contain '{param}', got: {header}"
        );
    }

    // Verify values
    assert!(header.contains("oauth_consumer_key=\"my-consumer-key\""));
    assert!(header.contains("oauth_token=\"my-access-token\""));
    assert!(header.contains("oauth_nonce=\"test_nonce_123\""));
    assert!(header.contains("oauth_timestamp=\"1700000000\""));
    assert!(header.contains("oauth_signature_method=\"HMAC-SHA1\""));
    assert!(header.contains("oauth_version=\"1.0\""));
}

// ═══════════════════════════════════════════════════════════════════════════
// X's published signature example
// https://docs.x.com/resources/fundamentals/authentication/oauth-1-0a/creating-a-signature
// ═══════════════════════════════════════════════════════════════════════════

/// The `status` value of X's example, whose space, `+`, `,` and `!` are the
/// characters a query-string encoder and RFC 3986 disagree on.
const X_EXAMPLE_STATUS: &str = "Hello Ladies + Gentlemen, a signed OAuth request!";

/// The page's request, keys, nonce, and timestamp produce the page's
/// `oauth_signature`, `Ls93hJiZbQ3akF3HF3x1Bz8/zU4=`.
#[test]
fn test_x_published_example_produces_the_published_signature() {
    let token = OAuth1Token {
        consumer_key: "xvz1evFS4wEEPTGEFPHBog".to_string(),
        consumer_secret: "kAcSOqF21Fu85e7zjz7ZN2U4ZRhfV3WpwPAoE3Z7kBw".to_string(),
        access_token: "370773112-GmHxMAgYyLbNEtIKZeRNFsMKPR9EyMZeS9weJAEb".to_string(),
        token_secret: "LswwdoUaIvS8ltyTt5jkRh4J50vUPVVHtR2YPi5kE".to_string(),
        user_id: None,
    };
    let mut body = BTreeMap::new();
    body.insert("status".to_string(), X_EXAMPLE_STATUS.to_string());

    let header = build_oauth1_header_with_nonce_ts(
        "POST",
        "https://api.x.com/1.1/statuses/update.json?include_entities=true",
        &token,
        Some(&body),
        Some("kYjzVBB8Y0ZFabxSWbWovY3uYSQ2pTgmZeNu2VS4cg"),
        Some("1318622958"),
    )
    .unwrap();

    assert!(
        header.contains("oauth_signature=\"Ls93hJiZbQ3akF3HF3x1Bz8%2FzU4%3D\""),
        "got: {header}"
    );
}

/// The signature base string encodes each value once in the parameter string
/// and the parameter string once more, so the page's base string carries the
/// `status` value as below: `%2520` for a space and `%252B` for the `+`.
#[test]
fn test_x_published_example_status_as_the_base_string_carries_it() {
    assert_eq!(
        encode(&encode(X_EXAMPLE_STATUS)),
        "Hello%2520Ladies%2520%252B%2520Gentlemen%252C%2520a%2520signed%2520OAuth%2520request%2521"
    );
}

/// A request with no query and no body parameters has nothing the two
/// encoders disagree on, so its header is the one the Go-compatible encoder
/// produced.
#[test]
fn test_a_request_without_parameters_signs_as_before() {
    let token = OAuth1Token {
        consumer_key: "dpf43f3p2l4k3l03".to_string(),
        consumer_secret: "kd94hf93k423kf44".to_string(),
        access_token: "nnch734d00sl2jdk".to_string(),
        token_secret: "pfkkdhi9sl3r4s00".to_string(),
        user_id: None,
    };

    let header = build_oauth1_header_with_nonce_ts(
        "GET",
        "https://api.x.com/2/users/me",
        &token,
        None,
        Some("kllo9940pd9333jh"),
        Some("1191242096"),
    )
    .unwrap();

    assert_eq!(
        header,
        "OAuth oauth_consumer_key=\"dpf43f3p2l4k3l03\", oauth_nonce=\"kllo9940pd9333jh\", \
         oauth_signature=\"n4eoY3O3I7bQ6slLqNVCxcXp9Mo%3D\", oauth_signature_method=\"HMAC-SHA1\", \
         oauth_timestamp=\"1191242096\", oauth_token=\"nnch734d00sl2jdk\", oauth_version=\"1.0\""
    );
}
