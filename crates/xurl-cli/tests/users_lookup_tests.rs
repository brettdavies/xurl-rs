//! `xr users` looks up 1 to 100 accounts in one request: all-digit arguments
//! go to `/2/users` as ids, handles to `/2/users/by` as usernames. X answers
//! each form from a different endpoint, so a mix is refused before any
//! request, as are more than 100 arguments and a handle outside the spec's
//! pattern.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn users(names: &[(&str, &str)]) -> ResponseTemplate {
    let data: Vec<serde_json::Value> = names
        .iter()
        .map(|(id, username)| serde_json::json!({"id": id, "name": username, "username": username}))
        .collect();
    ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": data}))
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let mut argv = args.to_vec();
    argv.extend(["--output", "json"]);
    run(&store, &server.uri(), &argv).await
}

#[tokio::test]
async fn ids_go_to_the_ids_endpoint_in_one_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users"))
        .and(query_param("ids", "12,783214"))
        .respond_with(users(&[("12", "jack"), ("783214", "X")]))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(&server, &["users", "12", "783214"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][1]["username"], "X");
}

#[tokio::test]
async fn handles_go_to_the_usernames_endpoint_with_or_without_the_at() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by"))
        .and(query_param("usernames", "jack,XDevelopers"))
        .respond_with(users(&[("12", "jack"), ("2244994945", "XDevelopers")]))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(&server, &["users", "@jack", "XDevelopers"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["id"], "12");
}

#[tokio::test]
async fn inputs_outside_the_spec_are_refused_before_any_request() {
    let server = MockServer::start().await;
    let ids: Vec<String> = (1..=101).map(|n| n.to_string()).collect();
    let too_many: Vec<&str> = ["users"]
        .into_iter()
        .chain(ids.iter().map(String::as_str))
        .collect();

    for (args, reason) in [
        (vec!["users", "12", "@jack"], "mixed-ids-and-handles"),
        (vec!["users", "@not-a-handle!"], "invalid-username"),
        (
            vec!["users", "@this_handle_is_too_long"],
            "invalid-username",
        ),
        (too_many, "too-many-ids"),
    ] {
        let mut argv = vec!["--dry-run"];
        argv.extend(&args);
        let (code, stdout, stderr) = run_json(&server, &argv).await;

        assert_eq!(code, 0, "{:?}: {stderr}", &args[..2]);
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{:?}", &args[..2]);
        assert_eq!(body["rule"], reason, "{:?}", &args[..2]);
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

/// X refuses a lookup of one account with a 200 that carries `errors` and no
/// `data`. The refusal reads by X's problem `type`, never as `xr`'s own bug:
/// no `rule`, X's body in `message`, and no `report-issue` step. A handle X
/// finds no account for is the one answer named otherwise, in `account`.
#[rstest::rstest]
#[case::not_authorized("not-authorized-for-resource", "forbidden", 1, None)]
#[case::other("invalid-request", "invalid-request", 1, Some("fix-input"))]
#[tokio::test]
async fn an_errors_only_answer_reads_as_its_problem_type(
    #[case] problem: &str,
    #[case] reason: &str,
    #[case] exit_code: i32,
    #[case] action: Option<&str>,
) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users/by/username/ghost"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errors": [{
                "value": "ghost",
                "detail": "Could not find user with username: [ghost].",
                "title": "Not Found Error",
                "resource_type": "user",
                "parameter": "username",
                "type": format!("https://api.x.com/2/problems/{problem}")
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(&server, &["user", "ghost"]).await;

    assert_eq!(code, exit_code, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], reason, "{envelope}");
    assert_eq!(
        envelope["next_step"]["action"].as_str(),
        action,
        "{envelope}"
    );
    assert!(envelope.get("rule").is_none(), "{envelope}");
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("Could not find user")),
        "{envelope}"
    );
}

/// An id lookup that finds nothing keeps X's reading: `not-found` with X's
/// body in `message` and no `account`, which names handles alone.
#[tokio::test]
async fn an_id_lookup_that_finds_nothing_reads_as_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/users"))
        .and(query_param("ids", "12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errors": [{
                "value": "12",
                "detail": "Could not find user with ids: [12].",
                "title": "Not Found Error",
                "resource_type": "user",
                "parameter": "ids",
                "type": "https://api.x.com/2/problems/resource-not-found"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(&server, &["users", "12"]).await;

    assert_eq!(code, 4, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "not-found", "{envelope}");
    assert!(envelope.get("account").is_none(), "{envelope}");
    assert!(
        envelope["message"]
            .as_str()
            .is_some_and(|m| m.contains("Could not find user with ids")),
        "{envelope}"
    );
}
