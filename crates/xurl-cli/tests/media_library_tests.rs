//! `xr media library` adds uploads to the media library, changes and removes
//! them, and lists the categories a monetization names: each sends only the
//! fields its flags name, and refuses what breaks a bound before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth1_store, run_in_process as run};

const MEDIA_KEY: &str = "7_1146654567674912770";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

#[tokio::test]
async fn add_posts_the_key_the_details_and_the_monetization() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/library"))
        .and(body_json(serde_json::json!({
            "media_key": MEDIA_KEY,
            "title": "Launch day",
            "monetization": {
                "enabled": true,
                "video_category_ids": [4],
                "allowed_countries": ["US", "CA"],
                "blocked_advertiser_ids": ["123"],
                "blocked_advertiser_category_ids": [10]
            }
        })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"data": {"media_key": MEDIA_KEY}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media",
            "library",
            "add",
            MEDIA_KEY,
            "--title",
            "Launch day",
            "--monetize",
            "--video-category",
            "4",
            "--country",
            "US",
            "--country",
            "CA",
            "--block-advertiser",
            "123",
            "--block-advertiser-category",
            "10",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["media_key"], MEDIA_KEY);
}

#[tokio::test]
async fn add_without_a_monetization_flag_sends_no_monetization() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/media/library"))
        .and(body_json(
            serde_json::json!({"media_key": MEDIA_KEY, "name": "Raw cut"}),
        ))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"data": {"media_key": MEDIA_KEY}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media", "library", "add", MEDIA_KEY, "--name", "Raw cut", "--output", "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
}

#[tokio::test]
async fn update_puts_only_the_flags_it_was_given_and_no_monetize_turns_ads_off() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/media/library/{MEDIA_KEY}")))
        .and(body_json(serde_json::json!({
            "label": "promo",
            "monetization": {"enabled": false}
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"updated": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media",
            "library",
            "update",
            MEDIA_KEY,
            "--label",
            "promo",
            "--no-monetize",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["updated"], true);
}

#[tokio::test]
async fn remove_deletes_the_item_with_force() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/media/library/{MEDIA_KEY}")))
        .and(body_json(serde_json::json!({})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media", "library", "remove", MEDIA_KEY, "--force", "--output", "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["deleted"], true);
}

#[tokio::test]
async fn remove_without_force_sends_nothing_when_it_cannot_ask() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--no-interactive",
            "--output",
            "json",
            "media",
            "library",
            "remove",
            MEDIA_KEY,
        ],
    )
    .await;

    assert_ne!(code, 0, "{stdout}");
    assert!(stderr.contains("confirmation-required"), "{stderr}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn categories_sends_the_language_only_when_given() {
    let server = MockServer::start().await;
    let answer = serde_json::json!({"data": {
        "video_categories": [{"id": 1, "description": "Notícias"}],
        "advertiser_categories": []
    }});
    Mock::given(method("GET"))
        .and(path("/2/media/library/categories"))
        .and(query_param("lang", "pt-BR"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&answer))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "media",
            "library",
            "categories",
            "--lang",
            "pt-BR",
            "--output",
            "json",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(
        json_out(&stdout)["data"]["video_categories"][0]["description"],
        "Notícias"
    );
}

#[tokio::test]
async fn a_bad_input_is_refused_by_its_rule_before_any_request() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());
    let long_title = "a".repeat(71);

    for (args, rule) in [
        (vec!["add", "not-a-key"], "invalid-media-key"),
        (vec!["update", MEDIA_KEY], "no-library-changes"),
        (
            vec!["add", MEDIA_KEY, "--title", &long_title],
            "library-title-too-long",
        ),
        (
            vec![
                "add",
                MEDIA_KEY,
                "--video-category",
                "1",
                "--video-category",
                "2",
                "--video-category",
                "3",
            ],
            "too-many-video-categories",
        ),
        (
            vec!["add", MEDIA_KEY, "--country", "USA"],
            "invalid-country-code",
        ),
        (vec!["remove", "not-a-key"], "invalid-media-key"),
        (
            vec!["categories", "--lang", "english-us"],
            "invalid-language-tag",
        ),
    ] {
        let mut argv = vec!["--dry-run", "--output", "json", "media", "library"];
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
async fn monetize_and_no_monetize_exclude_each_other() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    let (code, _stdout, stderr) = run(
        &store,
        &server.uri(),
        &[
            "--output",
            "json",
            "media",
            "library",
            "update",
            MEDIA_KEY,
            "--monetize",
            "--no-monetize",
        ],
    )
    .await;

    assert_eq!(code, 2, "{stderr}");
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn add_under_dry_run_names_the_details_and_the_credential() {
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
            "library",
            "add",
            MEDIA_KEY,
            "--title",
            "Launch day",
            "--monetize",
            "--video-category",
            "4",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    let body = json_out(&stdout);
    assert_eq!(body["command"], "media-library-add");
    assert_eq!(body["would_succeed"], true);
    assert_eq!(body["auth"]["scheme"], "oauth1");
    assert_eq!(body["media_key"], MEDIA_KEY);
    assert_eq!(body["monetization"]["enabled"], true);
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn an_addition_takes_one_category_and_needs_one_for_ads_while_a_change_does_not() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for (args, rule) in [
        (
            vec!["add", MEDIA_KEY, "--monetize"],
            Some("monetization-needs-category"),
        ),
        (
            vec![
                "add",
                MEDIA_KEY,
                "--video-category",
                "1",
                "--video-category",
                "2",
            ],
            Some("too-many-video-categories"),
        ),
        (
            vec!["add", MEDIA_KEY, "--monetize", "--video-category", "1"],
            None,
        ),
        (
            vec!["add", MEDIA_KEY, "--monetize", "--program", "prog"],
            None,
        ),
        (vec!["update", MEDIA_KEY, "--monetize"], None),
        (
            vec![
                "update",
                MEDIA_KEY,
                "--video-category",
                "1",
                "--video-category",
                "2",
            ],
            None,
        ),
    ] {
        let mut argv = vec!["--dry-run", "--output", "json", "media", "library"];
        argv.extend(&args);
        let (code, stdout, stderr) = run(&store, &server.uri(), &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], rule.is_none(), "{args:?}");
        if let Some(rule) = rule {
            assert_eq!(body["rule"], rule, "{args:?}");
        }
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_program_and_a_video_category_exclude_each_other() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for verb in ["add", "update"] {
        let (code, _stdout, stderr) = run(
            &store,
            &server.uri(),
            &[
                "--output",
                "json",
                "media",
                "library",
                verb,
                MEDIA_KEY,
                "--program",
                "prog",
                "--video-category",
                "1",
            ],
        )
        .await;

        assert_eq!(code, 2, "{verb}: {stderr}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_refusal_names_the_entry_that_broke_the_rule() {
    let server = MockServer::start().await;
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth1_store(tmp.path());

    for (args, rule, value) in [
        (
            vec!["add", MEDIA_KEY, "--country", "US", "--country", "USA"],
            "invalid-country-code",
            "USA",
        ),
        (
            vec!["remove", "not-a-key"],
            "invalid-media-key",
            "not-a-key",
        ),
        (
            vec!["categories", "--lang", "english-us"],
            "invalid-language-tag",
            "english-us",
        ),
    ] {
        let mut argv = vec!["--dry-run", "--output", "json", "media", "library"];
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
            "library",
            "categories",
        ],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(json_out(&stdout).get("lang").is_none(), "{stdout}");
}
