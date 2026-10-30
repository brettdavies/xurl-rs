//! The `xr lists` family: create, change, and delete lists; a list's posts,
//! members, and followers; the caller's own, followed, pinned, and member
//! lists; and following and pinning. The spec's bounds on a list's name,
//! description, and id are checked before any request, a change that names
//! nothing is refused, and deleting a list asks first.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const MY_ID: &str = "2244994945";
const LIST_ID: &str = "1146654567674912769";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

fn error_envelope(stderr: &str) -> serde_json::Value {
    serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr")
}

async fn answer_me(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/2/users/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": MY_ID, "name": "Alice", "username": "alice"}
        })))
        .mount(server)
        .await;
}

async fn answer(server: &MockServer, verb: &str, at: &str, data: serde_json::Value) {
    Mock::given(method(verb))
        .and(path(at))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": data})))
        .expect(1)
        .mount(server)
        .await;
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let mut argv = args.to_vec();
    argv.extend(["--output", "json"]);
    run(&store, &server.uri(), &argv).await
}

#[tokio::test]
async fn add_creates_a_private_list() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/lists"))
        .and(body_json(
            serde_json::json!({"name": "Rustaceans", "private": true}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": LIST_ID, "name": "Rustaceans"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) =
        run_json(&server, &["lists", "add", "Rustaceans", "--private"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["id"], LIST_ID);
}

#[tokio::test]
async fn update_sends_only_what_the_flags_name() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/lists/{LIST_ID}")))
        .and(body_json(
            serde_json::json!({"description": "People who write Rust", "private": false}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"updated": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "lists",
            "update",
            LIST_ID,
            "--description",
            "People who write Rust",
            "--public",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["updated"], true);
}

#[tokio::test]
async fn inputs_outside_the_spec_bounds_are_refused_before_any_request() {
    let server = MockServer::start().await;
    let long_name = "x".repeat(26);
    let long_description = "x".repeat(101);

    for (args, reason) in [
        (
            vec!["lists", "add", long_name.as_str()],
            "list-name-too-long",
        ),
        (vec!["lists", "add", ""], "empty-list-name"),
        (
            vec!["lists", "add", "Rust", "--description", &long_description],
            "list-description-too-long",
        ),
        (vec!["lists", "update", LIST_ID], "no-list-changes"),
        (vec!["lists", "get", "rustaceans"], "invalid-list-id"),
        (vec!["lists", "follow", "rustaceans"], "invalid-list-id"),
    ] {
        let mut argv = vec!["--dry-run"];
        argv.extend(&args);
        let (code, stdout, stderr) = run_json(&server, &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["rule"], reason, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn remove_asks_first_and_deletes_with_force() {
    let server = MockServer::start().await;
    answer(
        &server,
        "DELETE",
        &format!("/2/lists/{LIST_ID}"),
        serde_json::json!({"deleted": true}),
    )
    .await;

    let (code, _stdout, stderr) =
        run_json(&server, &["--no-interactive", "lists", "remove", LIST_ID]).await;
    assert_eq!(code, 1, "stderr: {stderr}");
    assert_eq!(error_envelope(&stderr)["reason"], "confirmation-required");

    let (code, stdout, stderr) = run_json(
        &server,
        &["--no-interactive", "lists", "remove", LIST_ID, "--force"],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["deleted"], true);
}

#[tokio::test]
async fn list_of_reads_the_lists_another_account_owns() {
    let server = MockServer::start().await;
    answer(
        &server,
        "GET",
        "/2/users/by",
        serde_json::json!([{"id": "12", "name": "jack", "username": "jack"}]),
    )
    .await;
    answer(
        &server,
        "GET",
        "/2/users/12/owned_lists",
        serde_json::json!([{"id": LIST_ID, "name": "Rustaceans"}]),
    )
    .await;

    let (code, stdout, stderr) = run_json(&server, &["lists", "list", "--of", "@jack"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"][0]["name"], "Rustaceans");
}

#[tokio::test]
async fn members_add_resolves_the_handle_and_sends_its_id() {
    let server = MockServer::start().await;
    answer(
        &server,
        "GET",
        "/2/users/by",
        serde_json::json!([{"id": "12", "name": "jack", "username": "jack"}]),
    )
    .await;
    Mock::given(method("POST"))
        .and(path(format!("/2/lists/{LIST_ID}/members")))
        .and(body_json(serde_json::json!({"user_id": "12"})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"is_member": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) =
        run_json(&server, &["lists", "members", "add", LIST_ID, "@jack"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["is_member"], true);
}

#[tokio::test]
async fn members_remove_puts_both_ids_in_the_path() {
    let server = MockServer::start().await;
    answer(
        &server,
        "GET",
        "/2/users/by",
        serde_json::json!([{"id": "12", "name": "jack", "username": "jack"}]),
    )
    .await;
    answer(
        &server,
        "DELETE",
        &format!("/2/lists/{LIST_ID}/members/12"),
        serde_json::json!({"is_member": false}),
    )
    .await;

    let (code, _stdout, stderr) =
        run_json(&server, &["lists", "members", "remove", LIST_ID, "jack"]).await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn follow_and_pin_send_the_list_id_from_the_callers_account() {
    let server = MockServer::start().await;
    answer_me(&server).await;
    Mock::given(method("POST"))
        .and(path(format!("/2/users/{MY_ID}/followed_lists")))
        .and(body_json(serde_json::json!({"list_id": LIST_ID})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"following": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/2/users/{MY_ID}/pinned_lists")))
        .and(body_json(serde_json::json!({"list_id": LIST_ID})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"pinned": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    answer(
        &server,
        "DELETE",
        &format!("/2/users/{MY_ID}/followed_lists/{LIST_ID}"),
        serde_json::json!({"following": false}),
    )
    .await;
    answer(
        &server,
        "DELETE",
        &format!("/2/users/{MY_ID}/pinned_lists/{LIST_ID}"),
        serde_json::json!({"pinned": false}),
    )
    .await;

    for verb in ["follow", "pin", "unfollow", "unpin"] {
        let (code, _stdout, stderr) = run_json(&server, &["lists", verb, LIST_ID]).await;
        assert_eq!(code, 0, "{verb}: {stderr}");
    }
}

#[tokio::test]
async fn the_callers_lists_are_read_from_their_own_account() {
    let server = MockServer::start().await;
    answer_me(&server).await;
    let lists = serde_json::json!([{"id": LIST_ID, "name": "Rustaceans"}]);
    for at in [
        "owned_lists",
        "followed_lists",
        "list_memberships",
        "pinned_lists",
    ] {
        answer(
            &server,
            "GET",
            &format!("/2/users/{MY_ID}/{at}"),
            lists.clone(),
        )
        .await;
    }

    for verb in ["list", "followed", "memberships", "pinned"] {
        let (code, stdout, stderr) = run_json(&server, &["lists", verb]).await;
        assert_eq!(code, 0, "{verb}: {stderr}");
        assert_eq!(json_out(&stdout)["data"][0]["name"], "Rustaceans", "{verb}");
    }
}

#[tokio::test]
async fn a_lists_posts_members_and_followers_page_by_list_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/lists/{LIST_ID}/tweets")))
        .and(query_param("max_results", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "20", "text": "just setting up my twttr"}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let users = serde_json::json!([{"id": "12", "name": "jack", "username": "jack"}]);
    answer(
        &server,
        "GET",
        &format!("/2/lists/{LIST_ID}/members"),
        users.clone(),
    )
    .await;
    answer(
        &server,
        "GET",
        &format!("/2/lists/{LIST_ID}/followers"),
        users,
    )
    .await;
    answer(
        &server,
        "GET",
        &format!("/2/lists/{LIST_ID}"),
        serde_json::json!({"id": LIST_ID, "name": "Rustaceans", "member_count": 1}),
    )
    .await;

    for args in [
        &["lists", "posts", LIST_ID, "-n", "25"][..],
        &["lists", "members", "list", LIST_ID][..],
        &["lists", "followers", LIST_ID][..],
        &["lists", "get", LIST_ID][..],
    ] {
        let (code, _stdout, stderr) = run_json(&server, args).await;
        assert_eq!(code, 0, "{args:?}: {stderr}");
    }
}
