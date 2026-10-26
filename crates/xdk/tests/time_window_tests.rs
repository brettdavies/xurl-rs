//! `validate_time_window` takes each bound in the RFC 3339 form a call time
//! takes and compares the two as moments, so a start written in one zone
//! and an end in another order by the instant they name.

use xdk::api::shortcuts::validate_time_window;

#[test]
fn a_window_whose_start_precedes_its_end_passes() {
    for (start, end) in [
        (Some("2026-10-25T00:00:00Z"), Some("2026-10-26T00:00:00Z")),
        (
            Some("2026-10-26T01:00:00+02:00"),
            Some("2026-10-26T00:00:00Z"),
        ),
        (
            Some("2026-10-25T23:59:59.5Z"),
            Some("2026-10-25T23:59:59.75Z"),
        ),
        (
            Some("2026-12-31T23:00:00-05:00"),
            Some("2027-01-01T05:00:00Z"),
        ),
        (Some("2026-10-25T00:00:00Z"), None),
        (None, Some("2026-10-25T00:00:00Z")),
        (None, None),
    ] {
        assert_eq!(
            validate_time_window(start, end),
            Ok(()),
            "{start:?} {end:?}"
        );
    }
}

#[test]
fn a_bound_outside_the_form_or_an_end_at_or_before_the_start_is_refused() {
    for (start, end) in [
        (Some("2026-10-26T00:00:00Z"), Some("2026-10-25T00:00:00Z")),
        (Some("2026-10-26T00:00:00Z"), Some("2026-10-26T00:00:00Z")),
        (
            Some("2026-10-26T00:00:00Z"),
            Some("2026-10-26T01:00:00+02:00"),
        ),
        (
            Some("2027-01-01T05:00:00Z"),
            Some("2026-12-31T23:00:00-05:00"),
        ),
        (Some("2026-10-25T00:00:00"), None),
        (None, Some("2026-10-25")),
        (Some(""), None),
    ] {
        assert_eq!(
            validate_time_window(start, end),
            Err("invalid-time-window"),
            "{start:?} {end:?}"
        );
    }
}
