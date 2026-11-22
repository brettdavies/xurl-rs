//! What the `OAuth2` token endpoint answers that is no OAuth refusal, and the
//! envelope each answer earns, for the tests of the code exchange and of the
//! refresh to share.

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

/// X's page on response codes, which a 404 or a 408 points at.
pub const RESPONSE_CODES_DOCS: &str =
    "https://docs.x.com/x-api/fundamentals/response-codes-and-errors";

/// The README section on the token endpoint, `TOKEN_URL`, and proxies, which
/// a 403 points at.
pub const TOKEN_ENDPOINT_DOCS: &str =
    "https://github.com/brettdavies/xurl-rs#the-oauth2-token-endpoint";

/// One answer from the token endpoint and the envelope it earns.
pub struct NoRefusal {
    pub why: &'static str,
    pub response: ResponseTemplate,
    /// The HTTP status, as the warning that names it spells it.
    pub status: &'static str,
    pub reason: &'static str,
    pub exit_code: i32,
    pub action: &'static str,
    pub docs: &'static str,
}

/// RFC 6749 section 5.2 refuses with a 400 or a 401, or with an `error`
/// string. Any other answer came from something that did not process the
/// grant, such as a proxy or a token URL with the wrong path.
pub fn no_refusal_answers() -> Vec<NoRefusal> {
    let general = xdk::error::EXIT_GENERAL_ERROR;
    let not_found = xdk::error::EXIT_NOT_FOUND;
    vec![
        NoRefusal {
            why: "a 408 with no JSON",
            response: ResponseTemplate::new(408).set_body_string("<html>Request Timeout</html>"),
            status: "408",
            reason: "api-error",
            exit_code: general,
            action: "retry",
            docs: RESPONSE_CODES_DOCS,
        },
        NoRefusal {
            why: "a 404 with no JSON",
            response: ResponseTemplate::new(404).set_body_string("<html>Not Found</html>"),
            status: "404",
            reason: "not-found",
            exit_code: not_found,
            action: "fix-input",
            docs: RESPONSE_CODES_DOCS,
        },
        NoRefusal {
            why: "a 403 with no JSON",
            response: ResponseTemplate::new(403).set_body_string("<html>Forbidden</html>"),
            status: "403",
            reason: "forbidden",
            exit_code: general,
            action: "fix-input",
            docs: TOKEN_ENDPOINT_DOCS,
        },
        NoRefusal {
            why: "a 403 with JSON that holds no `error`",
            response: ResponseTemplate::new(403).set_body_json(
                serde_json::json!({"title": "Forbidden", "detail": "blocked by policy"}),
            ),
            status: "403",
            reason: "forbidden",
            exit_code: general,
            action: "fix-input",
            docs: TOKEN_ENDPOINT_DOCS,
        },
        NoRefusal {
            why: "a 404 with JSON whose `error` is an object",
            response: ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({"error": {"code": 404, "message": "Not Found"}})),
            status: "404",
            reason: "not-found",
            exit_code: not_found,
            action: "fix-input",
            docs: RESPONSE_CODES_DOCS,
        },
    ]
}

/// Answers with `status` and the body of the request it got back, as a debug
/// page or a proxy's block page does.
pub struct Echo(pub u16);

impl Respond for Echo {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        ResponseTemplate::new(self.0)
            .set_body_string(String::from_utf8_lossy(&request.body).into_owned())
    }
}

/// An HTML error page of about 69 KB.
pub fn big_page() -> String {
    format!("<html>{}</html>", "blocked ".repeat(8_600))
}

/// A token endpoint that answers `status` with a `Location` on another server,
/// and that server, which would answer a valid grant on any method. Dropping it
/// checks that the endpoint was asked once and the other server never was.
pub struct Redirecting {
    pub base: String,
    _redirecting: MockServer,
    _target: MockServer,
}

impl Redirecting {
    pub async fn start(status: u16) -> Self {
        let target = MockServer::start().await;
        Mock::given(path("/2/oauth2/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "REDIRECTED-AT",
                "refresh_token": "REDIRECTED-RT",
                "expires_in": 7200,
                "token_type": "bearer",
            })))
            .expect(0)
            .mount(&target)
            .await;
        let redirecting = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/2/oauth2/token"))
            .respond_with(
                ResponseTemplate::new(status)
                    .insert_header("Location", format!("{}/2/oauth2/token", target.uri())),
            )
            .expect(1)
            .mount(&redirecting)
            .await;
        Self {
            base: redirecting.uri(),
            _redirecting: redirecting,
            _target: target,
        }
    }
}
