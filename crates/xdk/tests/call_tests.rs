//! The call shortcuts send the requests the spec gives their endpoints and
//! decode their responses; the offline validators hold the spec's bounds on
//! what a scheduled call is sent with.

use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xdk::api::Client;
use xdk::api::shortcuts::{
    NewScheduledCall, OccurrenceScope, Occurrences, RsvpAnswer, ScheduledCallChanges,
    validate_call_time, validate_new_scheduled_call, validate_occurrences,
    validate_scheduled_call_changes,
};
use xdk::auth::OAuth2Credential;

const CALL_ID: &str = "8a6f1c2e-call";
const SCHEDULED_ID: &str = "1849302018273645";
const START: &str = "2026-10-23T15:00:00Z";

fn user_client(base_url: String) -> Client {
    Client::builder()
        .base_url(base_url)
        .oauth2(OAuth2Credential {
            client_id: "client-id".into(),
            client_secret: "client-secret".into(),
            access_token: "user-at".into(),
            refresh_token: None,
            expires_at: None,
        })
        .build()
        .expect("client builds")
}

fn scheduled_call() -> serde_json::Value {
    json!({
        "id": SCHEDULED_ID,
        "call_id": CALL_ID,
        "title": "Standup",
        "start_time": START,
        "end_time": "2026-10-23T15:30:00Z",
        "status": "scheduled",
        "calendar": "x",
        "organizer_id": "2244994945",
        "invitees": [{"id": "783214", "response": "accepted"}]
    })
}

#[tokio::test]
async fn get_call_reads_the_call_with_its_host() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/calls/{CALL_ID}")))
        .and(query_param(
            "call.fields",
            "conversation_id,created_at,id,join_policy,join_url,participant_count,title,viewer_may_join",
        ))
        .and(query_param("expansions", "host_id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"id": CALL_ID, "title": "Standup", "join_policy": "open", "participant_count": 3, "viewer_may_join": true, "host_id": "2244994945"},
            "includes": {"users": [{"id": "2244994945", "name": "Host", "username": "host"}]}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .get_call(CALL_ID)
        .send()
        .await
        .expect("read");

    assert_eq!(resp.data.title.as_deref(), Some("Standup"));
    assert_eq!(resp.data.participant_count, Some(3));
    assert_eq!(resp.data.viewer_may_join, Some(true));
}

#[tokio::test]
async fn get_scheduled_calls_sends_the_window_and_reads_the_invitees() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/calls/scheduled"))
        .and(query_param("start_time", START))
        .and(query_param("end_time", "2026-10-24T15:00:00Z"))
        .and(query_param("max_results", "50"))
        .and(query_param("expansions", "call_id,invitees,organizer_id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [scheduled_call()]})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .get_scheduled_calls(Some(START), Some("2026-10-24T15:00:00Z"), 50)
        .send()
        .await
        .expect("listed");

    assert_eq!(resp.data[0].id.as_deref(), Some(SCHEDULED_ID));
    let invitees = resp.data[0].invitees.as_ref().expect("invitees");
    assert_eq!(invitees[0].id, "783214");
    assert_eq!(invitees[0].response, "accepted");
}

#[tokio::test]
async fn get_scheduled_calls_leaves_an_unset_window_to_x() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/2/calls/scheduled"))
        .and(query_param_is_missing("start_time"))
        .and(query_param_is_missing("end_time"))
        .and(query_param("max_results", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .get_scheduled_calls(None, None, 500)
        .send()
        .await
        .expect("listed");

    assert!(resp.data.is_empty());
}

#[tokio::test]
async fn schedule_call_posts_only_the_fields_given() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/2/calls/scheduled"))
        .and(body_json(json!({
            "start_time": START,
            "title": "Standup",
            "duration_minutes": 15,
            "invitee_ids": ["783214"],
            "notify_invitees": false
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({"data": scheduled_call()})))
        .expect(1)
        .mount(&server)
        .await;
    let call = NewScheduledCall {
        start_time: START.to_string(),
        title: Some("Standup".to_string()),
        duration_minutes: Some(15),
        invitee_ids: vec!["783214".to_string()],
        notify_invitees: Some(false),
        ..NewScheduledCall::default()
    };

    let resp = user_client(server.uri())
        .schedule_call(&call)
        .send()
        .await
        .expect("scheduled");

    assert_eq!(resp.data.call_id.as_deref(), Some(CALL_ID));
}

#[tokio::test]
async fn get_scheduled_call_reads_one_by_id() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}")))
        .and(query_param("expansions", "call_id,invitees,organizer_id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": scheduled_call()})))
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .get_scheduled_call(SCHEDULED_ID)
        .send()
        .await
        .expect("read");

    assert_eq!(resp.data.status.as_deref(), Some("scheduled"));
}

#[tokio::test]
async fn update_scheduled_call_puts_the_changes_and_the_occurrences() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}")))
        .and(body_json(json!({
            "start_time": "2026-10-23T16:00:00Z",
            "scope": "this_and_following",
            "occurrence_start_time": START
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"updated": true}})))
        .expect(1)
        .mount(&server)
        .await;
    let changes = ScheduledCallChanges {
        start_time: Some("2026-10-23T16:00:00Z".to_string()),
        occurrences: Occurrences {
            scope: Some(OccurrenceScope::ThisAndFollowing),
            occurrence_start_time: Some(START.to_string()),
        },
        ..ScheduledCallChanges::default()
    };

    let resp = user_client(server.uri())
        .update_scheduled_call(SCHEDULED_ID, &changes)
        .send()
        .await
        .expect("updated");

    assert!(resp.data.updated);
}

#[tokio::test]
async fn cancel_scheduled_call_names_the_occurrences_in_the_query() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}")))
        .and(query_param("scope", "this"))
        .and(query_param("occurrence_start_time", START))
        .and(query_param("notify_invitees", "false"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": {"cancelled": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let occurrences = Occurrences {
        scope: Some(OccurrenceScope::This),
        occurrence_start_time: Some(START.to_string()),
    };

    let resp = user_client(server.uri())
        .cancel_scheduled_call(SCHEDULED_ID, &occurrences, Some(false))
        .send()
        .await
        .expect("cancelled");

    assert!(resp.data.cancelled);
}

#[tokio::test]
async fn cancel_scheduled_call_sends_no_query_for_every_occurrence() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}")))
        .and(query_param_is_missing("scope"))
        .and(query_param_is_missing("occurrence_start_time"))
        .and(query_param_is_missing("notify_invitees"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": {"cancelled": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    user_client(server.uri())
        .cancel_scheduled_call(SCHEDULED_ID, &Occurrences::default(), None)
        .send()
        .await
        .expect("cancelled");
}

#[tokio::test]
async fn respond_to_scheduled_call_posts_the_answer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/2/calls/scheduled/{SCHEDULED_ID}/rsvp")))
        .and(body_json(json!({"response": "tentative"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"data": {"response": "tentative"}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = user_client(server.uri())
        .respond_to_scheduled_call(SCHEDULED_ID, RsvpAnswer::Tentative, &Occurrences::default())
        .send()
        .await
        .expect("answered");

    assert_eq!(resp.data.response, "tentative");
}

#[test]
fn a_call_time_is_rfc_3339_with_a_zone() {
    for time in [
        "2026-10-23T15:00:00Z",
        "2026-10-23T15:00:00.123456789Z",
        "2026-12-31T23:59:59+05:30",
        "2026-02-29T00:00:00-08:00",
    ] {
        assert_eq!(validate_call_time(time), Ok(()), "{time}");
    }
    for time in [
        "",
        "2026-10-23",
        "2026-10-23 15:00:00Z",
        "2026-10-23T15:00:00",
        "2026-13-01T00:00:00Z",
        "2026-10-32T00:00:00Z",
        "2026-10-23T24:00:00Z",
        "2026-10-23T15:60:00Z",
        "2026-10-23T15:00:00.Z",
        "2026-10-23T15:00:00.1234567890Z",
        "2026-10-23T15:00:00+24:00",
        "2026-10-23T15:00:00z",
    ] {
        assert_eq!(validate_call_time(time), Err("invalid-call-time"), "{time}");
    }
}

#[test]
fn a_new_call_holds_the_specs_bounds() {
    let base = || NewScheduledCall {
        start_time: START.to_string(),
        ..NewScheduledCall::default()
    };
    assert_eq!(validate_new_scheduled_call(&base()), Ok(()));
    let cases: Vec<(NewScheduledCall, &str)> = vec![
        (
            NewScheduledCall {
                start_time: "tomorrow".to_string(),
                ..base()
            },
            "invalid-call-time",
        ),
        (
            NewScheduledCall {
                title: Some("x".repeat(101)),
                ..base()
            },
            "call-title-too-long",
        ),
        (
            NewScheduledCall {
                description: Some("x".repeat(1001)),
                ..base()
            },
            "call-description-too-long",
        ),
        (
            NewScheduledCall {
                recurrence: Some("x".repeat(501)),
                ..base()
            },
            "call-recurrence-too-long",
        ),
        (
            NewScheduledCall {
                time_zone: Some("x".repeat(65)),
                ..base()
            },
            "call-time-zone-too-long",
        ),
        (
            NewScheduledCall {
                duration_minutes: Some(0),
                ..base()
            },
            "invalid-call-duration",
        ),
        (
            NewScheduledCall {
                duration_minutes: Some(44_641),
                ..base()
            },
            "invalid-call-duration",
        ),
        (
            NewScheduledCall {
                invitee_ids: vec!["@jack".to_string()],
                ..base()
            },
            "invalid-user-id",
        ),
        (
            NewScheduledCall {
                invitee_ids: (0..26).map(|n| n.to_string()).collect(),
                ..base()
            },
            "too-many-invitees",
        ),
    ];
    for (call, reason) in cases {
        assert_eq!(validate_new_scheduled_call(&call), Err(reason));
    }
    let longest = NewScheduledCall {
        title: Some("x".repeat(100)),
        description: Some("x".repeat(1000)),
        duration_minutes: Some(44_640),
        invitee_ids: (0..25).map(|n| n.to_string()).collect(),
        ..base()
    };
    assert_eq!(validate_new_scheduled_call(&longest), Ok(()));
}

#[test]
fn a_change_set_changes_something_within_the_specs_bounds() {
    assert_eq!(
        validate_scheduled_call_changes(&ScheduledCallChanges::default()),
        Err("no-call-changes")
    );
    let occurrence_only = ScheduledCallChanges {
        occurrences: Occurrences {
            scope: Some(OccurrenceScope::This),
            occurrence_start_time: Some(START.to_string()),
        },
        ..ScheduledCallChanges::default()
    };
    assert_eq!(
        validate_scheduled_call_changes(&occurrence_only),
        Err("no-call-changes"),
        "naming the occurrences changes nothing"
    );
    let retitled = ScheduledCallChanges {
        title: Some("Retro".to_string()),
        ..ScheduledCallChanges::default()
    };
    assert_eq!(validate_scheduled_call_changes(&retitled), Ok(()));
    let cases: Vec<(ScheduledCallChanges, &str)> = vec![
        (
            ScheduledCallChanges {
                invitee_ids: Some((0..101).map(|n| n.to_string()).collect()),
                ..ScheduledCallChanges::default()
            },
            "too-many-invitees",
        ),
        (
            ScheduledCallChanges {
                recurrence: Some(String::new()),
                ..ScheduledCallChanges::default()
            },
            "empty-call-recurrence",
        ),
        (
            ScheduledCallChanges {
                title: Some("Retro".to_string()),
                occurrences: Occurrences {
                    scope: None,
                    occurrence_start_time: Some("2026-10-23".to_string()),
                },
                ..ScheduledCallChanges::default()
            },
            "invalid-call-time",
        ),
    ];
    for (changes, reason) in cases {
        assert_eq!(validate_scheduled_call_changes(&changes), Err(reason));
    }
}

#[test]
fn an_occurrence_is_named_by_its_rfc_3339_start() {
    assert_eq!(validate_occurrences(&Occurrences::default()), Ok(()));
    let bad = Occurrences {
        scope: Some(OccurrenceScope::This),
        occurrence_start_time: Some("next monday".to_string()),
    };
    assert_eq!(validate_occurrences(&bad), Err("invalid-call-time"));
}
