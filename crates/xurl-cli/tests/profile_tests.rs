//! `xr profile` reaches `/2/users/me` and its image and banner through the
//! runner: `update` sends only the fields given, `image set` and
//! `banner set` name an uploaded media id, `banner remove` asks before it
//! deletes, and inputs the spec bounds are refused before any request.

mod common;

use tempfile::TempDir;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{oauth2_store, run_in_process as run};

const MEDIA_ID: &str = "1146654567674912769";

fn json_out(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("JSON on stdout")
}

async fn run_json(server: &MockServer, args: &[&str]) -> (i32, String, String) {
    let tmp = TempDir::new().expect("tempdir");
    let store = oauth2_store(tmp.path());
    let mut argv = args.to_vec();
    argv.extend(["--output", "json"]);
    run(&store, &server.uri(), &argv).await
}

fn updated() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"updated": true}}))
}

#[tokio::test]
async fn update_sends_only_the_fields_given_and_an_empty_one_clears() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/2/users/me"))
        .and(body_json(
            serde_json::json!({"name": "X Dev", "location": ""}),
        ))
        .respond_with(updated())
        .expect(1)
        .mount(&server)
        .await;

    let (code, stdout, stderr) = run_json(
        &server,
        &["profile", "update", "--name", "X Dev", "--location", ""],
    )
    .await;

    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["updated"], true);
}

#[tokio::test]
async fn image_set_and_banner_set_name_the_uploaded_media() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/2/users/me/profile_image"))
        .and(body_json(serde_json::json!({"media_id": MEDIA_ID})))
        .respond_with(updated())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/2/users/me/profile_banner"))
        .and(body_json(serde_json::json!({
            "media_id": MEDIA_ID,
            "crop": {"width": 1500, "height": 500, "offset_left": 0, "offset_top": 120}
        })))
        .respond_with(updated())
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(&server, &["profile", "image", "set", MEDIA_ID]).await;
    assert_eq!(code, 0, "stderr: {stderr}");

    let (code, stdout, stderr) = run_json(
        &server,
        &[
            "profile",
            "banner",
            "set",
            MEDIA_ID,
            "--crop",
            "1500x500+0+120",
        ],
    )
    .await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["updated"], true);
}

#[tokio::test]
async fn banner_remove_asks_first_and_deletes_with_force() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/2/users/me/profile_banner"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let (code, _stdout, stderr) = run_json(
        &server,
        &["--no-interactive", "profile", "banner", "remove"],
    )
    .await;
    assert_eq!(code, 1, "stderr: {stderr}");
    let envelope: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("a JSON envelope on stderr");
    assert_eq!(envelope["reason"], "confirmation-required");

    let (code, stdout, stderr) =
        run_json(&server, &["profile", "banner", "remove", "--force"]).await;
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(json_out(&stdout)["data"]["deleted"], true);
}

#[tokio::test]
async fn inputs_outside_the_spec_bounds_are_refused_before_any_request() {
    let server = MockServer::start().await;
    let long_name = "n".repeat(51);

    for (args, reason) in [
        (vec!["profile", "update"], "no-profile-changes"),
        (
            vec!["profile", "update", "--name", &long_name],
            "profile-name-too-long",
        ),
        (
            vec!["profile", "image", "set", "photo.png"],
            "invalid-media-id",
        ),
        (
            vec!["profile", "banner", "set", "photo.png"],
            "invalid-media-id",
        ),
        (
            vec!["profile", "banner", "set", MEDIA_ID, "--crop", "0x500+0+0"],
            "invalid-banner-crop",
        ),
    ] {
        let mut argv = vec!["--dry-run"];
        argv.extend(&args);
        let (code, stdout, stderr) = run_json(&server, &argv).await;

        assert_eq!(code, 0, "{args:?}: {stderr}");
        let body = json_out(&stdout);
        assert_eq!(body["would_succeed"], false, "{args:?}");
        assert_eq!(body["reason"], reason, "{args:?}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}

#[tokio::test]
async fn a_crop_is_a_geometry() {
    let server = MockServer::start().await;

    for crop in ["1500x500", "1500x500+-1+0", "wide"] {
        let (code, _stdout, stderr) = run_json(
            &server,
            &["profile", "banner", "set", MEDIA_ID, "--crop", crop],
        )
        .await;

        assert_eq!(code, 2, "{crop}: {stderr}");
        assert!(stderr.contains("--crop"), "{crop}: {stderr}");
    }
    assert_eq!(server.received_requests().await.expect("recorded").len(), 0);
}
