//! `xr media amplify` lists Amplify programs and their posts and publishes one
//! of your posts into programs: `add` sends only the flags it is given and
//! refuses what breaks the spec's patterns before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

const PROGRAM: &str = "prog_2026-launch";
const POST_ID: &str = "1346889436626259968";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn program(state: &str) -> serde_json::Value {
    serde_json::json!({
        "id": PROGRAM,
        "name": "Launch pre-rolls",
        "state": state,
        "advertiser_ids": ["1234567890"],
        "publisher_ids": ["2244994945"]
    })
}

#[tokio::test]
async fn the_reads_print_what_x_answers() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/media/amplify/programs"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [program("active")]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/2/media/amplify/programs/{PROGRAM}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": program("inactive")})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/2/media/amplify/programs/{PROGRAM}/posts")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": [
                {"post_id": POST_ID, "added_at": "2026-11-02T16:00:00Z"}
            ]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, listed, stderr) = run(
        &store,
        &server.uri(),
        &["media", "amplify", "list", "--output", "json"],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&listed)["data"][0]["state"], "active");

    let (code, one, stderr) = run(
        &store,
        &server.uri(),
        &["media", "amplify", "get", PROGRAM, "--output", "json"],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&one)["data"]["state"], "inactive");

    let (code, posts, stderr) = run(
        &store,
        &server.uri(),
        &["media", "amplify", "posts", PROGRAM, "--output", "json"],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&posts)["data"][0]["post_id"], POST_ID);
}

#[tokio::test]
async fn add_posts_the_resolved_post_id_the_programs_and_what_it_was_given() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/amplify/posts"))
        .and(body_json(serde_json::json!({
            "post_id": POST_ID,
            "program_ids": [PROGRAM, "other"],
            "blocked_advertiser_ids": ["123"],
            "promotion_ends_at": "2026-12-01T00:00:00Z"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"added": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let post_url = format!("https://x.com/jack/status/{POST_ID}");

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media",
            "amplify",
            "add",
            &post_url,
            "--program",
            PROGRAM,
            "--program",
            "other",
            "--block-advertiser",
            "123",
            "--ends-at",
            "2026-12-01T00:00:00Z",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["added"], true);
}

#[tokio::test]
async fn add_needs_a_program() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &["--output", "json", "media", "amplify", "add", POST_ID],
    )
    .await;

    assert_eq!(code, 2, "{stderr}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_bad_input_is_refused_by_its_rule_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for (args, rule) in [
        (vec!["get", "has space"], "invalid-program-id"),
        (vec!["posts", "dot.ted"], "invalid-program-id"),
        (
            vec!["add", "not-a-post", "--program", PROGRAM],
            "invalid-post-id",
        ),
        (
            vec!["add", POST_ID, "--program", "has space"],
            "invalid-program-id",
        ),
        (
            vec![
                "add",
                POST_ID,
                "--program",
                PROGRAM,
                "--block-advertiser",
                "x1",
            ],
            "invalid-advertiser-id",
        ),
        (
            vec![
                "add",
                POST_ID,
                "--program",
                PROGRAM,
                "--ends-at",
                "next week",
            ],
            "invalid-promotion-end",
        ),
    ] {
        let mut argv = vec!["--dry-run", "--output", "json", "media", "amplify"];
        argv.extend(&args);
        let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["rule"], rule, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn add_under_dry_run_names_the_inputs_and_the_credential() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "media",
            "amplify",
            "add",
            POST_ID,
            "--program",
            PROGRAM,
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "media-amplify-add");
    assert_eq!(body["would_succeed"], true);
    assert_eq!(body["auth"]["scheme"], "oauth1");
    assert_eq!(body["post_id"], POST_ID);
    assert_eq!(body["program_ids"][0], PROGRAM);
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_refusal_names_the_entry_that_broke_the_rule() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for (args, rule, value) in [
        (
            vec!["add", POST_ID, "--program", PROGRAM, "--program", "bad/id"],
            "invalid-program-id",
            "bad/id",
        ),
        (
            vec![
                "add",
                POST_ID,
                "--program",
                PROGRAM,
                "--block-advertiser",
                "1",
                "--block-advertiser",
                "x1",
            ],
            "invalid-advertiser-id",
            "x1",
        ),
        (
            vec![
                "add",
                POST_ID,
                "--program",
                PROGRAM,
                "--ends-at",
                "next week",
            ],
            "invalid-promotion-end",
            "next week",
        ),
        (vec!["get", "has space"], "invalid-program-id", "has space"),
    ] {
        let mut argv = vec!["--dry-run", "--output", "json", "media", "amplify"];
        argv.extend(&args);
        let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["rule"], rule, "{args:?}");
        assert_eq!(body["value"], value, "{args:?}");
    }
}

#[tokio::test]
async fn a_dry_run_leaves_out_what_no_flag_set() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--dry-run",
            "--output",
            "json",
            "media",
            "amplify",
            "add",
            POST_ID,
            "--program",
            PROGRAM,
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert!(body.get("blocked_advertiser_ids").is_none(), "{stdout}");
    assert!(body.get("promotion_ends_at").is_none(), "{stdout}");
}
