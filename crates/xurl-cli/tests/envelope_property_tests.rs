//! The error envelope's body survives its own serialization: whatever the
//! fields hold, the JSON an emitter writes deserializes to an equal body.
//! The schema drift guard in `schema_tests.rs` ties the committed schema to
//! the type, so this covers the values.

use proptest::prelude::*;
use serde_json::Value;
use xurl::cli::envelope::{ErrorBody, Reason};
use xurl::cli::hints::{NextAction, NextStep};

/// Every value the `Reason` type holds, read from its generated schema.
fn reasons() -> Vec<Reason> {
    let schema = serde_json::to_value(schemars::schema_for!(Reason)).expect("schema serializes");
    schema["oneOf"]
        .as_array()
        .expect("one branch per variant")
        .iter()
        .map(|branch| serde_json::from_value(branch["const"].clone()).expect("a reason"))
        .collect()
}

fn text() -> impl Strategy<Value = Option<String>> {
    prop::option::of(any::<String>())
}

fn flag() -> impl Strategy<Value = Option<bool>> {
    prop::option::of(any::<bool>())
}

fn names() -> impl Strategy<Value = Option<Vec<String>>> {
    prop::option::of(prop::collection::vec(any::<String>(), 0..4))
}

/// A field that distinguishes an absent key from a present `null`.
fn nullable_text() -> impl Strategy<Value = Option<Value>> {
    prop::option::of(prop_oneof![
        Just(Value::Null),
        any::<String>().prop_map(Value::String),
    ])
}

fn next_step() -> impl Strategy<Value = Option<NextStep>> {
    let action = prop::sample::select(vec![
        NextAction::RegisterApp,
        NextAction::SignIn,
        NextAction::SelectApp,
        NextAction::InspectStore,
        NextAction::EnrollApp,
        NextAction::ShowHelp,
        NextAction::ResumeWait,
        NextAction::WaitAndRetry,
        NextAction::Retry,
        NextAction::FixInput,
        NextAction::ReportIssue,
        NextAction::Confirm,
        NextAction::RunCommand,
    ]);
    prop::option::of((action, text(), text(), text()).prop_map(
        |(action, command, template, docs)| NextStep {
            action,
            command,
            template,
            docs,
        },
    ))
}

fn error_body() -> impl Strategy<Value = ErrorBody> {
    let core = (
        prop::sample::select(reasons()),
        any::<i32>(),
        text(),
        next_step(),
        text(),
        text(),
    );
    let mismatch = (
        text(),
        text(),
        text(),
        nullable_text(),
        names(),
        names(),
        text(),
        names(),
    );
    let waits = (text(), prop::option::of(any::<u64>()), text());
    let confirmation = (text(), text(), flag(), flag(), nullable_text(), flag());
    let verb_local = (
        text(),
        names(),
        flag(),
        text(),
        names(),
        text(),
        text(),
        flag(),
    );
    (core, mismatch, waits, confirmation, verb_local).prop_map(
        |(core, mismatch, waits, confirmation, verb_local)| {
            let mut body = ErrorBody::default();
            (
                body.reason,
                body.exit_code,
                body.message,
                body.next_step,
                body.command,
                body.suggestion,
            ) = core;
            (
                body.endpoint,
                body.rendered_url,
                body.method,
                body.requested,
                body.supported,
                body.available_in_app,
                body.app,
                body.other_apps_with_creds,
            ) = mismatch;
            (body.media_id, body.retry_after_secs, body.retry_at) = waits;
            (
                body.post_id,
                body.name,
                body.all,
                body.oauth1,
                body.oauth2_username,
                body.bearer,
            ) = confirmation;
            (
                body.schema,
                body.known_schemas,
                body.valid,
                body.action,
                body.known_hosts,
                body.host,
                body.install_dir,
                body.would_succeed,
            ) = verb_local;
            body
        },
    )
}

proptest! {
    #[test]
    fn an_error_body_round_trips_through_its_json(body in error_body()) {
        let written = serde_json::to_value(&body).expect("the body serializes");
        let read: ErrorBody = serde_json::from_value(written.clone())
            .map_err(|e| TestCaseError::fail(format!("{e}: {written}")))?;
        prop_assert_eq!(read, body);
    }
}

/// `requested` and `oauth2_username` document `null` as a value of its own,
/// distinct from the key being absent, so a key written as `null` is still
/// there when the body is read back.
#[test]
fn a_null_field_stays_present_through_a_round_trip() {
    for field in ["requested", "oauth2_username"] {
        let written = serde_json::json!({
            "reason": "auth-method-mismatch",
            "exit_code": 2,
            field: null,
        });
        let read: ErrorBody =
            serde_json::from_value(written.clone()).expect("the body deserializes");
        assert_eq!(
            serde_json::to_value(&read).expect("the body serializes"),
            written,
            "{field} was written as null and read back absent"
        );
    }
}
