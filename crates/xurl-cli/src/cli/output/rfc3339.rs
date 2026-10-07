//! UTC timestamps in RFC 3339 form, for the one place `xr` prints a time:
//! the `retry_at` an agent schedules a rate-limited retry against.

const SECS_PER_DAY: u64 = 86_400;

/// Seconds since the Unix epoch as `YYYY-MM-DDTHH:MM:SSZ`.
pub(crate) fn utc_rfc3339(epoch_secs: u64) -> String {
    let (year, month, day) = civil_from_days(epoch_secs / SECS_PER_DAY);
    let secs = epoch_secs % SECS_PER_DAY;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        secs / 3_600,
        secs % 3_600 / 60,
        secs % 60
    )
}

/// The Gregorian date `days` after 1970-01-01, as year, month, and day.
///
/// Howard Hinnant's `civil_from_days`, narrowed to dates on or after the
/// epoch: <https://howardhinnant.github.io/date_algorithms.html#civil_from_days>.
/// It counts in 400-year eras that begin on 1 March, which puts the leap day
/// last in its year, so no month table and no leap-year branch is needed.
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    // 719,468 days separate 0000-03-01 from 1970-01-01.
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let day_of_era = shifted % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::utc_rfc3339;

    #[test]
    fn epoch_seconds_format_as_utc_timestamps() {
        for (epoch_secs, timestamp) in [
            (0, "1970-01-01T00:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (1_709_208_000, "2024-02-29T12:00:00Z"),
            (1_767_225_599, "2025-12-31T23:59:59Z"),
            (2_147_483_648, "2038-01-19T03:14:08Z"),
            (4_102_444_800, "2100-01-01T00:00:00Z"),
        ] {
            assert_eq!(utc_rfc3339(epoch_secs), timestamp, "epoch {epoch_secs}");
        }
    }
}
