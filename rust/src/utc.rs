//! UTC date-times and TAI − UTC (SPEC.md: Converting from UTC; Appendix A).
//!
//! # Exact arithmetic
//!
//! Before 1972, TAI − UTC was a linear formula with coefficients in 10⁻⁷ s per
//! day, so every value it takes at a nanosecond label is an integer multiple of
//! 1/[`D`] s. Conversions here run in those units ("scaled TAI") and never round;
//! the one floor the spec allows happens when the result becomes an
//! [`Instant`](crate::Instant).
//!
//! Every row of Appendix A has the same shape, `a + (label − m) × r`, with
//! `r = 0` from 1972. Converting a label is therefore linear, inverting it is
//! one exact division, and each step between rows (inserted or removed time,
//! leap seconds included) falls out of the table rather than being special-cased.

use std::fmt;
use std::str::FromStr;

use crate::civil;
use crate::Error;

pub(crate) const NS: i128 = 1_000_000_000;
const DAY_NS: i128 = 86_400 * NS;
/// Scaled-TAI units per second: every TAI − UTC value is a multiple of 1/D s.
pub(crate) const D: i128 = 10_000_000 * DAY_NS;
/// Scaled-TAI units per nanosecond.
pub(crate) const K: i128 = D / NS;
/// The epoch 2000-01-01T00:00:00 TAI as a Unix-style label plus TAI − UTC.
const EPOCH_TAI_LABEL_S: i128 = 946_684_800;
const MJD_UNIX_EPOCH: i128 = 40_587;
/// Beyond the representable range, and small enough that nothing overflows.
pub(crate) const MAX_LABEL_NS: i128 = 600_000_000_000 * NS;
/// Date-times may be built for years up to this magnitude.
const MAX_YEAR: i64 = 100_000;

// ---------------------------------------------------------------- the table

/// One row of Appendix A: TAI − UTC = `a·DAY_NS + (u − m)·r` in units of 1/D s
/// at label `u` (Unix nanoseconds).
#[derive(Clone, Copy)]
struct Period {
    start: i128, // first label of the row, Unix ns
    a: i128,     // offset, 1e-7 s
    m: i128,     // reference label, Unix ns
    r: i128,     // rate, 1e-7 s per day
}

const fn mjd(day: i128) -> i128 {
    (day - MJD_UNIX_EPOCH) * DAY_NS
}
const fn rubber(start: i128, a: i128, m: i128, r: i128) -> Period {
    Period {
        start: mjd(start),
        a,
        m: mjd(m),
        r,
    }
}
const fn whole(unix_s: i128, seconds: i128) -> Period {
    Period {
        start: unix_s * NS,
        a: seconds * 10_000_000,
        m: 0,
        r: 0,
    }
}

/// Appendix A. Before the first row, TAI − UTC is defined as zero.
#[rustfmt::skip]
const PERIODS: [Period; 41] = [
    rubber(37300, 14_228_180, 37300, 12_960), rubber(37512, 13_728_180, 37300, 12_960),
    rubber(37665, 18_458_580, 37665, 11_232), rubber(38334, 19_458_580, 37665, 11_232),
    rubber(38395, 32_401_300, 38761, 12_960), rubber(38486, 33_401_300, 38761, 12_960),
    rubber(38639, 34_401_300, 38761, 12_960), rubber(38761, 35_401_300, 38761, 12_960),
    rubber(38820, 36_401_300, 38761, 12_960), rubber(38942, 37_401_300, 38761, 12_960),
    rubber(39004, 38_401_300, 38761, 12_960), rubber(39126, 43_131_700, 39126, 25_920),
    rubber(39887, 42_131_700, 39126, 25_920),
    whole(63_072_000, 10),    whole(78_796_800, 11),    whole(94_694_400, 12),
    whole(126_230_400, 13),   whole(157_766_400, 14),   whole(189_302_400, 15),
    whole(220_924_800, 16),   whole(252_460_800, 17),   whole(283_996_800, 18),
    whole(315_532_800, 19),   whole(362_793_600, 20),   whole(394_329_600, 21),
    whole(425_865_600, 22),   whole(489_024_000, 23),   whole(567_993_600, 24),
    whole(631_152_000, 25),   whole(662_688_000, 26),   whole(709_948_800, 27),
    whole(741_484_800, 28),   whole(773_020_800, 29),   whole(820_454_400, 30),
    whole(867_715_200, 31),   whole(915_148_800, 32),   whole(1_136_073_600, 33),
    whole(1_230_768_000, 34), whole(1_341_100_800, 35), whole(1_435_708_800, 36),
    whole(1_483_228_800, 37),
];
const BEFORE_1961: Period = Period {
    start: i128::MIN,
    a: 0,
    m: 0,
    r: 0,
};

impl Period {
    /// TAI − UTC at label `u`, in units of 1/D s.
    fn delta(&self, u: i128) -> i128 {
        self.a * DAY_NS + (u - self.m) * self.r
    }
    /// Scaled TAI since the epoch for label `u` under this row.
    fn tai(&self, u: i128) -> i128 {
        (u - EPOCH_TAI_LABEL_S * NS) * K + self.delta(u)
    }
    /// The inverse of `tai`, floored to 1 ns. Exact: tai(u) = u·(K + r) + c.
    fn label(&self, t: i128) -> i128 {
        let c = -EPOCH_TAI_LABEL_S * NS * K + self.a * DAY_NS - self.m * self.r;
        (t - c).div_euclid(K + self.r)
    }
    /// Scaled TAI of this row's first label.
    fn first_tai(&self) -> i128 {
        self.tai(self.start)
    }
}

/// Row `i`, counting [`BEFORE_1961`] as row 0.
fn row(i: usize) -> &'static Period {
    if i == 0 {
        &BEFORE_1961
    } else {
        &PERIODS[i - 1]
    }
}

/// The row that applies to label `u`, and its index.
fn row_at(u: i128) -> (usize, &'static Period) {
    let i = PERIODS.partition_point(|p| p.start <= u);
    (i, row(i))
}

/// The step in TAI − UTC at the start of row `i` (1/D s): positive steps
/// insert time, negative ones remove labels.
fn step(i: usize) -> i128 {
    let b = row(i).start;
    row(i).delta(b) - row(i - 1).delta(b)
}

/// Whether label `u` fell in time UTC removed: under its own row it would
/// reach the TAI instant where the next row begins.
fn is_removed(u: i128) -> bool {
    let (i, p) = row_at(u);
    i < PERIODS.len() && step(i + 1) < 0 && p.tai(u) >= row(i + 1).first_tai()
}

/// Whether a leap second was inserted at the end of the day starting at `days`.
fn has_leap_second(days: i64) -> bool {
    let next = (days as i128 + 1) * DAY_NS;
    (1..=PERIODS.len()).any(|i| row(i).start == next && step(i) == D)
}

// ---------------------------------------------------------------- conversions

/// Scaled TAI for a UTC label in Unix nanoseconds. A Unix value never names
/// 23:59:60; labels in removed time are rejected.
pub(crate) fn unix_to_scaled(u: i128) -> Result<i128, Error> {
    if u.unsigned_abs() > MAX_LABEL_NS as u128 {
        return Err(Error::OutOfRange);
    }
    if is_removed(u) {
        return Err(Error::RemovedTime);
    }
    Ok(row_at(u).1.tai(u))
}

/// The UTC reading of a scaled-TAI instant.
pub(crate) struct Reading {
    /// The label, floored to 1 ns. Inside inserted time, the label at its end.
    pub unix_ns: i128,
    /// Nanoseconds into inserted time, if the instant falls in it.
    pub inserted_ns: Option<i128>,
    /// Whether that inserted time is a whole leap second (written 23:59:60).
    pub leap_second: bool,
}

pub(crate) fn scaled_to_reading(t: i128) -> Reading {
    // The last row whose first label maps at or before t.
    let i = (1..=PERIODS.len())
        .rev()
        .find(|&i| t >= row(i).first_tai())
        .unwrap_or(0);
    if i < PERIODS.len() {
        let b = row(i + 1).start;
        let end_of_row = row(i).tai(b);
        if step(i + 1) > 0 && t >= end_of_row {
            return Reading {
                unix_ns: b,
                inserted_ns: Some((t - end_of_row).div_euclid(K)),
                leap_second: step(i + 1) == D,
            };
        }
    }
    Reading {
        unix_ns: row(i).label(t),
        inserted_ns: None,
        leap_second: false,
    }
}

// ---------------------------------------------------------------- UtcDateTime

/// A UTC date and time to the nanosecond: a label that actually occurred.
///
/// Dates are proleptic Gregorian with astronomical years (year 0 is 1 BCE).
/// Second 60 exists only during an inserted leap second, and the few labels
/// UTC skipped in 1961 and 1968 do not exist at all; both are rejected on
/// construction. Before 1961, times are read as Universal Time.
///
/// The text form is RFC 3339 in UTC, extended with a sign for years outside
/// 0000 to 9999: `YYYY-MM-DDThh:mm:ss[.fffffffff]Z`.
///
/// ```
/// use t32::{Error, UtcDateTime};
///
/// let t: UtcDateTime = "2016-12-31T23:59:60Z".parse()?;   // the last leap second
/// assert_eq!(t.second(), 60);
///
/// assert_eq!("2016-12-30T23:59:60Z".parse::<UtcDateTime>(), Err(Error::NotALeapSecond));
/// assert_eq!(UtcDateTime::new(2026, 2, 29, 0, 0, 0), Err(Error::InvalidDateTime));
/// # Ok::<(), Error>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcDateTime {
    year: i64,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
}

impl UtcDateTime {
    /// The date-time from which conversions are provisional: the expiry of the
    /// leap-second table built into this crate (`leap-seconds.list`, 2027-06-28).
    pub const TABLE_EXPIRES: UtcDateTime = UtcDateTime {
        year: 2027,
        month: 6,
        day: 28,
        hour: 0,
        minute: 0,
        second: 0,
        nanosecond: 0,
    };

    /// The date-time with these fields, at a whole second.
    ///
    /// ```
    /// use t32::UtcDateTime;
    ///
    /// let t = UtcDateTime::new(2026, 10, 31, 3, 13, 0)?;
    /// assert_eq!(t.to_string(), "2026-10-31T03:13:00Z");
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidDateTime`] for a field out of bounds or a day the
    /// month lacks, [`Error::NotALeapSecond`] for a second 60 that was not
    /// inserted, [`Error::RemovedTime`] for a time UTC skipped, and
    /// [`Error::OutOfRange`] for a year beyond ±100,000.
    pub fn new(
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, Error> {
        UtcDateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond: 0,
        }
        .validated()
    }

    /// The same date-time with the nanosecond field set (0 to 999,999,999).
    ///
    /// ```
    /// use t32::UtcDateTime;
    ///
    /// let t = UtcDateTime::new(2026, 10, 31, 3, 13, 0)?.with_nanosecond(500_000_000)?;
    /// assert_eq!(t.to_string(), "2026-10-31T03:13:00.5Z");
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidDateTime`] for a value of a second or more, and
    /// [`Error::RemovedTime`] if the result is a time UTC skipped.
    pub fn with_nanosecond(self, nanosecond: u32) -> Result<Self, Error> {
        UtcDateTime { nanosecond, ..self }.validated()
    }

    fn validated(self) -> Result<Self, Error> {
        let UtcDateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
        } = self;
        // The spec's order: invalid_date, not_a_leap_second, removed_time, out_of_range.
        if !(1..=12).contains(&month)
            || day == 0
            || u32::from(day) > civil::days_in_month(year, month.into())
            || hour > 23
            || minute > 59
            || second > 60
            || nanosecond >= 1_000_000_000
        {
            return Err(Error::InvalidDateTime);
        }
        if year.unsigned_abs() > MAX_YEAR as u64 {
            // No leap second was inserted out here, and the label cannot be encoded.
            return Err(if second == 60 {
                Error::NotALeapSecond
            } else {
                Error::OutOfRange
            });
        }
        let days = civil::days_from_civil(year, month.into(), day.into());
        if second == 60 {
            if !(hour == 23 && minute == 59 && has_leap_second(days)) {
                return Err(Error::NotALeapSecond);
            }
        } else if is_removed(self.label_ns()) {
            return Err(Error::RemovedTime);
        }
        Ok(self)
    }

    /// The year, astronomical (year 0 is 1 BCE).
    #[must_use]
    pub const fn year(self) -> i64 {
        self.year
    }
    /// The month, 1 to 12.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }
    /// The day of the month, 1 to 31.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }
    /// The hour, 0 to 23.
    #[must_use]
    pub const fn hour(self) -> u8 {
        self.hour
    }
    /// The minute, 0 to 59.
    #[must_use]
    pub const fn minute(self) -> u8 {
        self.minute
    }
    /// The second, 0 to 60 (60 only during a leap second).
    #[must_use]
    pub const fn second(self) -> u8 {
        self.second
    }
    /// The nanosecond, 0 to 999,999,999.
    #[must_use]
    pub const fn nanosecond(self) -> u32 {
        self.nanosecond
    }

    /// Whether converting this date-time relies on the built-in leap-second table
    /// beyond its expiry date. The spec requires converters to report this; with
    /// UTC continuous from 2027-05-20, no further leap seconds are expected.
    ///
    /// ```
    /// use t32::UtcDateTime;
    ///
    /// assert!(!"2026-10-31T03:13:00Z".parse::<UtcDateTime>()?.is_provisional());
    /// assert!("2030-01-01T00:00:00Z".parse::<UtcDateTime>()?.is_provisional());
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn is_provisional(self) -> bool {
        self >= Self::TABLE_EXPIRES
    }

    /// TAI − UTC in force at this date-time, in seconds, as an exact fraction
    /// `(numerator, denominator)`.
    ///
    /// ```
    /// use t32::UtcDateTime;
    ///
    /// let (n, d) = "2026-10-31T03:13:00Z".parse::<UtcDateTime>()?.tai_minus_utc();
    /// assert_eq!(n / d, 37);
    /// let (n, d) = "1970-01-01T00:00:00Z".parse::<UtcDateTime>()?.tai_minus_utc();
    /// assert_eq!(n * 1_000_000, 8_000_082 * d);   // 8.000082 s
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn tai_minus_utc(self) -> (i128, i128) {
        let u = self.label_ns();
        (row_at(u).1.delta(u), D)
    }

    /// The label in Unix nanoseconds, reading second 60 as 59.
    fn label_ns(&self) -> i128 {
        let days = civil::days_from_civil(self.year, self.month.into(), self.day.into()) as i128;
        let secs = days * 86_400
            + i128::from(self.hour) * 3600
            + i128::from(self.minute) * 60
            + i128::from(self.second.min(59));
        secs * NS + i128::from(self.nanosecond)
    }

    /// Scaled TAI of this date-time.
    pub(crate) fn to_scaled(self) -> Result<i128, Error> {
        let t = unix_to_scaled(self.label_ns())?;
        Ok(if self.second == 60 { t + NS * K } else { t })
    }

    /// The date-time of a Unix label (no validation: it comes from a reading).
    fn from_label_ns(u: i128) -> Self {
        let days = u.div_euclid(DAY_NS);
        let in_day = u.rem_euclid(DAY_NS);
        let secs = (in_day / NS) as u32;
        let (year, month, day) = civil::civil_from_days(days as i64);
        UtcDateTime {
            year,
            month: month as u8,
            day: day as u8,
            hour: (secs / 3600) as u8,
            minute: (secs % 3600 / 60) as u8,
            second: (secs % 60) as u8,
            nanosecond: (in_day % NS) as u32,
        }
    }

    /// The date-time of a reading: a leap second as 23:59:60.f; other inserted
    /// time as the label at its end.
    pub(crate) fn from_reading(r: &Reading) -> Self {
        match r.inserted_ns {
            Some(ns) if r.leap_second => UtcDateTime {
                second: 60,
                ..Self::from_label_ns(r.unix_ns - NS + ns)
            },
            _ => Self::from_label_ns(r.unix_ns),
        }
    }
}

impl fmt::Debug for UtcDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UtcDateTime({self})")
    }
}

impl fmt::Display for UtcDateTime {
    /// RFC 3339, with only as many fraction digits as needed.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.year {
            0..=9999 => write!(f, "{:04}", self.year)?,
            y if y < 0 => write!(f, "-{:04}", -y)?,
            y => write!(f, "+{y}")?,
        }
        write!(
            f,
            "-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.month, self.day, self.hour, self.minute, self.second
        )?;
        if self.nanosecond > 0 {
            let frac = format!("{:09}", self.nanosecond);
            write!(f, ".{}", frac.trim_end_matches('0'))?;
        }
        f.write_str("Z")
    }
}

impl FromStr for UtcDateTime {
    type Err = Error;

    /// Parses `YYYY-MM-DDThh:mm:ss[.fffffffff]Z` (also `t`, a space, or `z`).
    /// An unsigned year has exactly four digits; a signed one (ISO 8601's
    /// expanded form, needed outside 0000 to 9999) has four to six.
    fn from_str(s: &str) -> Result<Self, Error> {
        let bad = Error::InvalidDateTime;
        let (sign, body) = match s.as_bytes().first() {
            Some(b'+') => (1, &s[1..]),
            Some(b'-') => (-1, &s[1..]),
            _ => (1, s),
        };
        let sign_given = body.len() < s.len();
        let body = body.strip_suffix(['Z', 'z']).ok_or(bad)?;
        let (date, time) = body.split_once(['T', 't', ' ']).ok_or(bad)?;
        let (time, frac) = match time.split_once('.') {
            Some((t, f)) => (t, Some(f)),
            None => (time, None),
        };
        let num = |t: &str, len: usize| -> Result<u32, Error> {
            if t.len() == len && t.bytes().all(|b| b.is_ascii_digit()) {
                t.parse().map_err(|_| bad)
            } else {
                Err(bad)
            }
        };
        let mut d = date.rsplitn(3, '-');
        let (dd, mm, yy) = (
            d.next().ok_or(bad)?,
            d.next().ok_or(bad)?,
            d.next().ok_or(bad)?,
        );
        let digits_ok = if sign_given {
            (4..=6).contains(&yy.len())
        } else {
            yy.len() == 4
        };
        if !digits_ok || !yy.bytes().all(|b| b.is_ascii_digit()) {
            return Err(bad);
        }
        let year = sign * yy.parse::<i64>().map_err(|_| bad)?;
        let mut t = time.split(':');
        let mut field = || num(t.next().ok_or(bad)?, 2);
        let (h, mi, sec) = (field()?, field()?, field()?);
        if t.next().is_some() {
            return Err(bad);
        }
        let nanosecond = match frac {
            Some(f) if !f.is_empty() && f.len() <= 9 => {
                num(f, f.len())? * 10u32.pow(9 - f.len() as u32)
            }
            Some(_) => return Err(bad),
            None => 0,
        };
        UtcDateTime {
            year,
            month: num(mm, 2)? as u8,
            day: num(dd, 2)? as u8,
            hour: h as u8,
            minute: mi as u8,
            second: sec as u8,
            nanosecond,
        }
        .validated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> UtcDateTime {
        s.parse().unwrap()
    }

    #[test]
    fn text_round_trips() {
        for s in [
            "2026-10-31T03:13:00Z",
            "2016-12-31T23:59:60.25Z",
            "1966-06-15T12:00:00.123456789Z",
            "0000-01-01T00:00:00Z",
            "-15422-12-06T11:41:52Z",
            "+19421-01-25T12:17:30Z",
        ] {
            assert_eq!(utc(s).to_string(), s);
        }
        assert_eq!(
            utc("2026-10-31 03:13:00.500z").to_string(),
            "2026-10-31T03:13:00.5Z"
        );
        assert_eq!(
            utc("+2026-10-31T03:13:00Z").to_string(),
            "2026-10-31T03:13:00Z"
        );
        assert_eq!(
            format!("{:?}", utc("2026-10-31T03:13:00Z")),
            "UtcDateTime(2026-10-31T03:13:00Z)"
        );
        assert_eq!(
            UtcDateTime::new(i64::MIN, 1, 1, 0, 0, 0),
            Err(Error::OutOfRange)
        );
        assert_eq!(
            UtcDateTime::new(i64::MAX, 1, 1, 0, 0, 0),
            Err(Error::OutOfRange)
        );
        for bad in [
            "2026-10-31T03:13:00",
            "2026-10-31T3:13:00Z",
            "2026-1-31T03:13:00Z",
            "2026-10-31T03:13Z",
            "2026-10-31T03:13:00.Z",
            "2026-10-31T03:13:00.1234567890Z",
            "26-10-31T03:13:00Z",
            "10000-01-01T00:00:00Z",
            "+123-01-01T00:00:00Z",
            "2026-10-31T24:00:00Z",
            "2026-02-29T00:00:00Z",
        ] {
            assert_eq!(
                bad.parse::<UtcDateTime>(),
                Err(Error::InvalidDateTime),
                "{bad}"
            );
        }
    }

    #[test]
    fn leap_seconds_and_removed_time() {
        assert!("2016-12-31T23:59:60Z".parse::<UtcDateTime>().is_ok());
        assert_eq!(
            "2016-12-30T23:59:60Z".parse::<UtcDateTime>(),
            Err(Error::NotALeapSecond)
        );
        assert_eq!(
            "2016-12-31T23:58:60Z".parse::<UtcDateTime>(),
            Err(Error::NotALeapSecond)
        );
        // TAI − UTC fell by 0.05 s on 1961-08-01 and by 0.1 s on 1968-02-01.
        let removed = Err(Error::RemovedTime);
        assert_eq!(
            "1961-07-31T23:59:59.950000001Z".parse::<UtcDateTime>(),
            removed
        );
        assert!("1961-07-31T23:59:59.95Z".parse::<UtcDateTime>().is_ok());
        assert_eq!(
            "1968-01-31T23:59:59.900000003Z".parse::<UtcDateTime>(),
            removed
        );
        assert!("1968-01-31T23:59:59.900000002Z"
            .parse::<UtcDateTime>()
            .is_ok());
    }

    #[test]
    fn steps_are_where_appendix_a_says() {
        let steps: Vec<(i64, i128)> = (1..=PERIODS.len())
            .filter(|&i| step(i) != 0)
            .map(|i| ((row(i).start / DAY_NS) as i64, step(i)))
            .collect();
        assert_eq!(steps.len(), 38); // 1961, 9 fractional to 1968, 1972, 27 leap seconds
        assert_eq!(steps.iter().filter(|s| s.1 == D).count(), 27);
        assert_eq!(steps.iter().filter(|s| s.1 < 0).count(), 2);
    }

    #[test]
    fn conversion_is_monotonic_at_every_step() {
        for i in 1..=PERIODS.len() {
            let b = row(i).start;
            let mut last = None;
            for u in (b - 2 * NS..=b + 2 * NS).step_by(10_000_000) {
                if let Ok(t) = unix_to_scaled(u) {
                    if let Some(l) = last {
                        assert!(t > l, "row {i}, label {u}");
                    }
                    last = Some(t);
                }
            }
        }
    }

    #[test]
    fn removed_time_leaves_no_gap() {
        // Every TAI instant near the removals decodes to a label that converts back.
        for b in [mjd(37512), mjd(39887)] {
            let t0 = row_at(b).1.tai(b);
            for k in -200..200 {
                let r = scaled_to_reading(t0 + k * K);
                assert!(r.inserted_ns.is_none());
                assert!(
                    unix_to_scaled(r.unix_ns).is_ok(),
                    "gap {k} ns from boundary"
                );
            }
        }
    }

    #[test]
    fn inserted_time_reads_as_its_end() {
        // 50 ms into the 0.107758 s inserted at 1972-01-01.
        let midnight = 63_072_000 * NS;
        let t = row(14).first_tai() - 50_000_000 * K;
        let r = scaled_to_reading(t);
        assert_eq!(r.unix_ns, midnight);
        assert!(r.inserted_ns.is_some() && !r.leap_second);
    }
}
