//! Proleptic Gregorian calendar arithmetic, after Howard Hinnant's
//! "chrono-Compatible Low-Level Date Algorithms"
//! (<https://howardhinnant.github.io/date_algorithms.html>), in the public domain.
//! Years are astronomical: year 0 is 1 BCE.

/// Days since 1970-01-01 for a valid date.
pub(crate) fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The number of days in a month, for any year (no overflow).
pub(crate) fn days_in_month(y: i64, m: u32) -> u32 {
    let leap = y.rem_euclid(4) == 0 && (y.rem_euclid(100) != 0 || y.rem_euclid(400) == 0);
    match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The date that is `z` days after 1970-01-01.
pub(crate) fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_across_eras() {
        for z in (-8_000_000..8_000_000).step_by(997) {
            let (y, m, d) = civil_from_days(z);
            assert_eq!(days_from_civil(y, m, d), z);
        }
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        assert_eq!(civil_from_days(-719_528), (0, 1, 1));
        for (y, feb) in [
            (2000, 29),
            (1900, 28),
            (2024, 29),
            (0, 29),
            (-4, 29),
            (-1, 28),
            (i64::MAX, 28),
        ] {
            assert_eq!(days_in_month(y, 2), feb, "{y}");
        }
    }
}
