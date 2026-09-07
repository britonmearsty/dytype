/// Calendar helpers built on Unix timestamps without pulling in a date crate.
/// Day keys are days since the Unix epoch (UTC), which keeps streak counting
/// and daily bucketing simple and deterministic.
/// Days since the Unix epoch for the given Unix timestamp (seconds, UTC).
pub fn day_key(timestamp_secs: u64) -> i64 {
    (timestamp_secs / 86_400) as i64
}

/// Converts days since the Unix epoch into a civil date `(year, month, day)`.
/// Implementation follows Howard Hinnant's `civil_from_days` algorithm.
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let month_prime = (5 * doy + 2) / 153;
    let day = (doy - (153 * month_prime + 2) / 5 + 1) as u32;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    } as u32;
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

/// Compact `MM/DD` label for a timestamp.
pub fn short_date(timestamp_secs: u64) -> String {
    let (_, month, day) = civil_from_days(day_key(timestamp_secs));
    format!("{month:02}/{day:02}")
}

/// First day of the week `ago` weeks back from `today`, used for weekly views.
pub fn week_ago(today_day: i64, weeks: i64) -> i64 {
    today_day - weeks * 7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_epoch_is_1970() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(day_key(0), 0);
        assert_eq!(day_key(86_399), 0);
        assert_eq!(day_key(86_400), 1);
    }

    #[test]
    fn known_modern_dates() {
        assert_eq!(civil_from_days(11_017), (2000, 3, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(20_000), (2024, 10, 4));
        assert_eq!(short_date(1_709_164_800), "02/29");
    }

    #[test]
    fn week_ago_math() {
        assert_eq!(week_ago(10_000, 1), 9_993);
        assert_eq!(week_ago(10_000, 0), 10_000);
    }
}
