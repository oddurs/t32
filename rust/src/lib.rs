//! T32 reference implementation (informative; SPEC.md and its test vectors are normative).
//!
//! Standard library only, integer arithmetic only. Three exact representations:
//! - `Fx`: biased time in units of 2^-40 s (what stamps and durations are made of);
//! - scaled TAI: TAI seconds since the epoch times `D`, where every TAI - UTC value
//!   in Appendix A is an integer multiple of 1/`D` s, so UTC conversion never rounds;
//! - Unix nanoseconds: UTC labels as the POSIX formula counts them.
//!
//! Rounding happens once, at the end, always toward negative infinity.
use std::fmt;

pub const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
pub const CANONICAL_LEN: u8 = 8;
pub const MAX_LEN: u8 = 16;

/// Biased time in units of 2^-40 s. Valid values are 0 <= fx < 2^80.
pub type Fx = u128;
const FRAC_BITS: u32 = 40;
const RANGE_FX: u128 = 1 << 80;
const BIAS_S: i128 = 1 << 39;

const NS: i128 = 1_000_000_000;
const DAY_NS: i128 = 86_400 * NS;
/// Denominator of exact TAI - UTC values: rubber-era coefficients are in 1e-7 s
/// per day, so every offset is a multiple of 1 / (1e7 * DAY_NS) s.
const D: i128 = 10_000_000 * DAY_NS;
/// Scaled units per nanosecond of a UTC label.
const K: i128 = D / NS;
/// The epoch 2000-01-01T00:00:00 TAI as a "TAI label": Unix seconds + TAI - UTC.
const EPOCH_TAI_LABEL_S: i128 = 946_684_800;
const MJD_UNIX_EPOCH: i128 = 40_587;
/// Comfortably beyond the representable range (2^39 s plus the epoch offset),
/// small enough that no intermediate product can overflow.
const MAX_LABEL_NS: i128 = 600_000_000_000 * NS;
/// Durations and stamp arithmetic stay below 2^40 s in magnitude.
const MAX_DURATION: i128 = 1 << 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Empty,
    TooLong,
    BadChar(char),
    BadHyphen,
    OutOfRange,
    BadField,
    NotALeapSecond,
    SkippedLabel,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

// ---------------------------------------------------------------- stamps

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stamp {
    value: u128,
    len: u8,
}

fn digit(c: u8) -> Option<u128> {
    let u = c.to_ascii_uppercase(); // ASCII only: non-ASCII bytes never match
    match u {
        b'O' => Some(0),
        b'I' | b'L' => Some(1),
        _ => ALPHABET.iter().position(|&a| a == u).map(|p| p as u128),
    }
}

impl Stamp {
    /// Floor a biased time to a stamp of `len` characters.
    pub fn from_fx(fx: Fx, len: u8) -> Result<Self, Error> {
        if len == 0 {
            return Err(Error::Empty);
        }
        if len > MAX_LEN {
            return Err(Error::TooLong);
        }
        if fx >= RANGE_FX {
            return Err(Error::OutOfRange);
        }
        Ok(Stamp {
            value: fx >> (5 * (MAX_LEN - len) as u32),
            len,
        })
    }

    /// Parse the input form (SPEC.md, Text form): ASCII only, case-insensitive,
    /// O read as 0 and I, L as 1, a single hyphen allowed only between two digits,
    /// at most 16 digits. Anything else, including whitespace, is an error.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let (mut value, mut len) = (0u128, 0u8);
        let mut prev_hyphen = false;
        for ch in s.chars() {
            if ch == '-' {
                if len == 0 || prev_hyphen {
                    return Err(Error::BadHyphen);
                }
                prev_hyphen = true;
                continue;
            }
            let d = if ch.is_ascii() { digit(ch as u8) } else { None }.ok_or(Error::BadChar(ch))?;
            prev_hyphen = false;
            len += 1;
            if len > MAX_LEN {
                return Err(Error::TooLong);
            }
            value = value * 32 + d;
        }
        if len == 0 {
            return Err(Error::Empty);
        }
        if prev_hyphen {
            return Err(Error::BadHyphen);
        }
        Ok(Stamp { value, len })
    }

    #[allow(clippy::len_without_is_empty)] // a stamp always has 1 to 16 digits
    pub fn len(&self) -> u8 {
        self.len
    }
    pub fn value(&self) -> u128 {
        self.value
    }
    /// The containing stamp of `len` characters (string truncation).
    pub fn truncate(&self, len: u8) -> Result<Self, Error> {
        if len == 0 || len > self.len {
            return Err(Error::BadField);
        }
        Ok(Stamp {
            value: self.value >> (5 * (self.len - len) as u32),
            len,
        })
    }
    fn shift(&self) -> u32 {
        5 * (MAX_LEN - self.len) as u32
    }
    /// Inclusive start of the interval.
    pub fn start(&self) -> Fx {
        self.value << self.shift()
    }
    /// Exclusive end of the interval (may equal 2^80 for the last stamp).
    pub fn end(&self) -> Fx {
        (self.value + 1) << self.shift()
    }
    /// A contains B exactly when A is a prefix of B.
    pub fn contains(&self, other: &Stamp) -> bool {
        other.len >= self.len && other.truncate(self.len) == Ok(*self)
    }
    /// Display form: groups of four joined by hyphens. Never store this.
    pub fn display(&self) -> String {
        let s = self.to_string();
        let groups: Vec<&str> = s
            .as_bytes()
            .chunks(4)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect();
        groups.join("-")
    }
    /// Shift the start by a duration, then floor to this stamp's own length.
    pub fn add(&self, d: Duration) -> Result<Self, Error> {
        let t = self.start() as i128 + d.0;
        if t < 0 {
            return Err(Error::OutOfRange);
        }
        Stamp::from_fx(t as u128, self.len)
    }
}

impl fmt::Display for Stamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = vec![b'0'; self.len as usize];
        let mut v = self.value;
        for i in (0..self.len as usize).rev() {
            out[i] = ALPHABET[(v % 32) as usize];
            v /= 32;
        }
        f.write_str(std::str::from_utf8(&out).unwrap())
    }
}

// ---------------------------------------------------------------- durations

/// A signed duration in units of 2^-40 s. Text form is exact (SPEC.md, Durations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(i128);

impl Duration {
    /// A duration of `units` x 2^-40 s; its magnitude must be below 2^40 s.
    pub fn new(units: i128) -> Result<Self, Error> {
        if units.abs() >= MAX_DURATION {
            return Err(Error::OutOfRange);
        }
        Ok(Duration(units))
    }
    /// The duration in units of 2^-40 s.
    pub fn units(&self) -> i128 {
        self.0
    }
    /// Difference of two starts, B - A. Exact, and always in range.
    pub fn between(a: &Stamp, b: &Stamp) -> Self {
        Duration(b.start() as i128 - a.start() as i128)
    }
    /// From nanoseconds, truncating toward zero.
    pub fn from_ns(ns: i128) -> Result<Self, Error> {
        if ns.unsigned_abs() >= (MAX_DURATION as u128 >> FRAC_BITS) * NS as u128 {
            return Err(Error::OutOfRange);
        }
        let mag = (ns.unsigned_abs() << FRAC_BITS) / NS as u128;
        Duration::new(if ns < 0 { -(mag as i128) } else { mag as i128 })
    }
    /// To nanoseconds, truncating toward zero.
    pub fn to_ns(&self) -> i128 {
        let mag = (self.0.unsigned_abs() * NS as u128) >> FRAC_BITS;
        if self.0 < 0 {
            -(mag as i128)
        } else {
            mag as i128
        }
    }
    /// Parse `sign 1*8digit ["." 1*8digit]` with the same digit leniency as stamps.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let b = s.as_bytes();
        let neg = match b.first() {
            Some(b'+') => false,
            Some(b'-') => true,
            _ => return Err(Error::BadField),
        };
        let rest = &s[1..];
        let (int, frac) = match rest.split_once('.') {
            Some((i, f)) => (i, Some(f)),
            None => (rest, None),
        };
        let digits = |t: &str, max: usize| -> Result<(u128, usize), Error> {
            if t.is_empty() {
                return Err(Error::Empty);
            }
            if t.len() > max {
                return Err(Error::TooLong);
            }
            let mut v = 0u128;
            for &c in t.as_bytes() {
                v = v * 32 + digit(c).ok_or(Error::BadChar(c as char))?;
            }
            Ok((v, t.len()))
        };
        let (iv, _) = digits(int, 8)?;
        let mut mag = iv << FRAC_BITS;
        if let Some(f) = frac {
            let (fv, n) = digits(f, 8)?;
            mag += fv << (FRAC_BITS - 5 * n as u32);
        }
        Duration::new(if neg { -(mag as i128) } else { mag as i128 })
    }
}

impl fmt::Display for Duration {
    /// Canonical: mandatory sign, no leading zeros, no trailing fraction zeros, "+0" for zero.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mag = self.0.unsigned_abs();
        let sign = if self.0 < 0 { '-' } else { '+' };
        let (mut int, mut frac) = (mag >> FRAC_BITS, mag & ((1 << FRAC_BITS) - 1));
        let mut is = Vec::new();
        loop {
            is.push(ALPHABET[(int % 32) as usize]);
            int /= 32;
            if int == 0 {
                break;
            }
        }
        is.reverse();
        let mut fs = Vec::new();
        while frac > 0 {
            frac <<= 5;
            fs.push(ALPHABET[(frac >> FRAC_BITS) as usize]);
            frac &= (1 << FRAC_BITS) - 1;
        }
        write!(f, "{sign}{}", std::str::from_utf8(&is).unwrap())?;
        if !fs.is_empty() {
            write!(f, ".{}", std::str::from_utf8(&fs).unwrap())?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- TAI - UTC

/// One period of Appendix A. TAI - UTC within it, as a multiple of 1/D s, is
/// `a * DAY_NS + (unix_ns - m_ns) * r` (with r = 0 for whole-second periods).
#[derive(Clone, Copy)]
struct Period {
    start_ns: i128, // first UTC label of the period, Unix ns
    a: i128,        // offset in 1e-7 s
    m_ns: i128,     // reference instant, Unix ns
    r: i128,        // rate in 1e-7 s per day
}

const fn mjd_ns(mjd: i128) -> i128 {
    (mjd - MJD_UNIX_EPOCH) * DAY_NS
}
const fn rubber(start: i128, a: i128, m: i128, r: i128) -> Period {
    Period {
        start_ns: mjd_ns(start),
        a,
        m_ns: mjd_ns(m),
        r,
    }
}
const fn whole(unix_s: i128, seconds: i128) -> Period {
    Period {
        start_ns: unix_s * NS,
        a: seconds * 10_000_000,
        m_ns: 0,
        r: 0,
    }
}

/// Appendix A. Before the first period, TAI - UTC is defined as zero.
pub const LEAP_TABLE_EXPIRES_UNIX: i64 = 1_814_140_800; // 2027-06-28
const PERIODS: [Period; 41] = [
    rubber(37300, 14_228_180, 37300, 12_960),
    rubber(37512, 13_728_180, 37300, 12_960),
    rubber(37665, 18_458_580, 37665, 11_232),
    rubber(38334, 19_458_580, 37665, 11_232),
    rubber(38395, 32_401_300, 38761, 12_960),
    rubber(38486, 33_401_300, 38761, 12_960),
    rubber(38639, 34_401_300, 38761, 12_960),
    rubber(38761, 35_401_300, 38761, 12_960),
    rubber(38820, 36_401_300, 38761, 12_960),
    rubber(38942, 37_401_300, 38761, 12_960),
    rubber(39004, 38_401_300, 38761, 12_960),
    rubber(39126, 43_131_700, 39126, 25_920),
    rubber(39887, 42_131_700, 39126, 25_920),
    whole(63072000, 10),
    whole(78796800, 11),
    whole(94694400, 12),
    whole(126230400, 13),
    whole(157766400, 14),
    whole(189302400, 15),
    whole(220924800, 16),
    whole(252460800, 17),
    whole(283996800, 18),
    whole(315532800, 19),
    whole(362793600, 20),
    whole(394329600, 21),
    whole(425865600, 22),
    whole(489024000, 23),
    whole(567993600, 24),
    whole(631152000, 25),
    whole(662688000, 26),
    whole(709948800, 27),
    whole(741484800, 28),
    whole(773020800, 29),
    whole(820454400, 30),
    whole(867715200, 31),
    whole(915148800, 32),
    whole(1136073600, 33),
    whole(1230768000, 34),
    whole(1341100800, 35),
    whole(1435708800, 36),
    whole(1483228800, 37),
];
const ZERO: Period = Period {
    start_ns: i128::MIN,
    a: 0,
    m_ns: 0,
    r: 0,
};

impl Period {
    /// TAI - UTC at label `u`, as a multiple of 1/D s.
    fn delta(&self, u: i128) -> i128 {
        self.a * DAY_NS + (u - self.m_ns) * self.r
    }
    /// Scaled TAI seconds since the epoch for label `u` under this period.
    fn tai(&self, u: i128) -> i128 {
        (u - EPOCH_TAI_LABEL_S * NS) * K + self.delta(u)
    }
    /// Inverse of `tai`: the label (floored to 1 ns) of scaled TAI `t`.
    /// tai(u) = u * (K + r) + c is linear in u, so this is exact.
    fn label(&self, t: i128) -> i128 {
        let c = -EPOCH_TAI_LABEL_S * NS * K + self.a * DAY_NS - self.m_ns * self.r;
        (t - c).div_euclid(K + self.r)
    }
}

fn period_at(u: i128) -> (usize, &'static Period) {
    match PERIODS.iter().rposition(|p| u >= p.start_ns) {
        Some(i) => (i + 1, &PERIODS[i]),
        None => (0, &ZERO),
    }
}
fn period(i: usize) -> &'static Period {
    if i == 0 {
        &ZERO
    } else {
        &PERIODS[i - 1]
    }
}

/// Whether converting this UTC label relies on the table beyond its expiry date.
pub fn is_provisional(unix_ns: i128) -> bool {
    unix_ns >= LEAP_TABLE_EXPIRES_UNIX as i128 * NS
}

/// TAI - UTC at a UTC label, in seconds, as an exact fraction (numerator, denominator).
pub fn delta_at(unix_ns: i128) -> (i128, i128) {
    (period_at(unix_ns).1.delta(unix_ns), D)
}

/// A step in TAI - UTC at boundary `i` (the start of PERIODS[i - 1]), in 1/D s.
fn step(i: usize) -> i128 {
    let b = period(i).start_ns;
    period(i).delta(b) - period(i - 1).delta(b)
}

// ---------------------------------------------------------------- conversions

fn scaled_to_fx(t: i128) -> Result<Fx, Error> {
    let total = t + BIAS_S * D;
    if total < 0 {
        return Err(Error::OutOfRange);
    }
    let (q, r) = (total / D, total % D);
    let fx = ((q as u128) << FRAC_BITS) + ((r as u128) << FRAC_BITS) / D as u128;
    if fx >= RANGE_FX {
        Err(Error::OutOfRange)
    } else {
        Ok(fx)
    }
}

fn fx_to_scaled(fx: Fx) -> i128 {
    let (q, r) = (
        (fx >> FRAC_BITS) as i128,
        (fx & ((1 << FRAC_BITS) - 1)) as i128,
    );
    q * D + ((r * D) >> FRAC_BITS) - BIAS_S * D
}

/// UTC label (Unix nanoseconds, POSIX) -> biased time. Rejects labels that
/// UTC skipped (where TAI - UTC decreased). A Unix value never names 23:59:60.
pub fn unix_ns_to_fx(u: i128) -> Result<Fx, Error> {
    if u.abs() > MAX_LABEL_NS {
        return Err(Error::OutOfRange);
    }
    let (i, p) = period_at(u);
    let t = p.tai(u);
    // Where TAI - UTC fell, labels whose TAI instant would reach the start of
    // the next period (in TAI) never occurred.
    if i < PERIODS.len() && step(i + 1) < 0 {
        let b = period(i + 1).start_ns;
        if t >= period(i + 1).tai(b) {
            return Err(Error::SkippedLabel);
        }
    }
    scaled_to_fx(t)
}

/// TAI nanoseconds since the epoch -> biased time.
pub fn tai_ns_to_fx(tai_ns: i128) -> Result<Fx, Error> {
    if tai_ns.abs() > MAX_LABEL_NS {
        return Err(Error::OutOfRange);
    }
    scaled_to_fx(tai_ns * K)
}

/// Biased time -> TAI nanoseconds since the epoch (floored).
pub fn fx_to_tai_ns(fx: Fx) -> i128 {
    fx_to_scaled(fx).div_euclid(K)
}

/// The UTC reading of an instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Utc {
    /// The UTC label as Unix nanoseconds (floored).
    pub unix_ns: i128,
    /// For an instant in time UTC inserted (a leap second, or a fractional step
    /// before 1972): nanoseconds into the insertion. `unix_ns` is then the label
    /// at the end of the insertion, e.g. 00:00:00 after 23:59:60.
    pub inserted_ns: Option<i128>,
}

/// Biased time -> UTC label.
pub fn fx_to_utc(fx: Fx) -> Utc {
    let t = fx_to_scaled(fx);
    // The last period whose first label maps at or before t.
    let i = (1..=PERIODS.len())
        .rev()
        .find(|&i| t >= period(i).tai(period(i).start_ns))
        .unwrap_or(0);
    if i < PERIODS.len() {
        let next = i + 1;
        let b = period(next).start_ns;
        let left = period(i).tai(b);
        if step(next) > 0 && t >= left {
            return Utc {
                unix_ns: b,
                inserted_ns: Some((t - left).div_euclid(K)),
            };
        }
    }
    Utc {
        unix_ns: period(i).label(t),
        inserted_ns: None,
    }
}

pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (yoe + era * 400 + if m <= 2 { 1 } else { 0 }, m, d)
}

/// Proleptic Gregorian UTC fields (astronomical years) -> biased time.
/// Second 60 is accepted only during an inserted leap second.
pub fn utc_to_fx(y: i64, mo: u32, d: u32, h: u32, mi: u32, s: u32, ns: u32) -> Result<Fx, Error> {
    if !(1..=12).contains(&mo)
        || d == 0
        || d > 31
        || h > 23
        || mi > 59
        || s > 60
        || ns >= 1_000_000_000
    {
        return Err(Error::BadField);
    }
    if y.abs() > 100_000 {
        return Err(Error::OutOfRange);
    }
    let days = days_from_civil(y, mo, d);
    if civil_from_days(days) != (y, mo, d) {
        return Err(Error::BadField);
    }
    let label =
        |sec: u32| (days as i128 * 86_400 + (h * 3600 + mi * 60 + sec) as i128) * NS + ns as i128;
    if s == 60 {
        let next = (days as i128 + 1) * DAY_NS;
        let leap = (1..=PERIODS.len()).any(|i| period(i).start_ns == next && step(i) == D);
        if !(leap && h == 23 && mi == 59) {
            return Err(Error::NotALeapSecond);
        }
        let u = label(59);
        return scaled_to_fx(period_at(u).1.tai(u) + NS * K);
    }
    unix_ns_to_fx(label(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    type Fields = (i64, u32, u32, u32, u32, u32, u32);

    fn st(f: Fields, n: u8) -> String {
        let (y, mo, d, h, mi, s, ns) = f;
        Stamp::from_fx(utc_to_fx(y, mo, d, h, mi, s, ns).unwrap(), n)
            .unwrap()
            .to_string()
    }

    #[test]
    fn encoding_vectors() {
        let v: &[(Fields, u8, &str)] = &[
            ((1999, 12, 31, 23, 59, 28, 0), 8, "G0000000"),
            ((1970, 1, 1, 0, 0, 0, 0), 8, "FZ3S5F48"),
            ((1969, 7, 20, 20, 17, 40, 0), 8, "FZ3BMMKB"),
            ((1972, 1, 1, 0, 0, 0, 0), 8, "FZ5NA8WA"),
            ((2016, 12, 31, 23, 59, 59, 0), 8, "G0FZP0S3"),
            ((2016, 12, 31, 23, 59, 60, 0), 8, "G0FZP0S4"),
            ((2017, 1, 1, 0, 0, 0, 0), 8, "G0FZP0S5"),
            ((2026, 10, 31, 3, 13, 0, 0), 8, "G0S7G6B1"),
            ((2026, 10, 31, 3, 13, 0, 500_000_000), 10, "G0S7G6B1G0"),
            ((2026, 10, 31, 3, 13, 0, 123_000_000), 12, "G0S7G6B13XYE"),
            ((1675, 8, 20, 0, 0, 0, 0), 8, "FPEY6N00"),
        ];
        for &(f, n, want) in v {
            assert_eq!(st(f, n), want, "{f:?}");
        }
    }

    #[test]
    fn rubber_era_is_exact() {
        // Cross-checked against exact rational arithmetic (tools/oracle.py).
        assert_eq!(
            st((1966, 6, 15, 12, 0, 0, 123_456_789), 16),
            "FZ0EDKA4VPC2A1FX"
        );
        // Unix epoch: TAI - UTC = 4.2131700 + (40587 - 39126) * 0.002592 = 8.000082 s
        assert_eq!(delta_at(0), (8_000_082 * D / 1_000_000, D));
    }

    #[test]
    fn unix_never_names_the_leap_second() {
        let midnight = utc_to_fx(2017, 1, 1, 0, 0, 0, 0).unwrap();
        assert_eq!(unix_ns_to_fx(1_483_228_800 * NS).unwrap(), midnight);
        let leap = utc_to_fx(2016, 12, 31, 23, 59, 60, 250_000_000).unwrap();
        let u = fx_to_utc(leap);
        assert_eq!(u.unix_ns, 1_483_228_800 * NS);
        assert_eq!(u.inserted_ns, Some(250_000_000));
        assert_eq!(
            fx_to_utc(midnight),
            Utc {
                unix_ns: 1_483_228_800 * NS,
                inserted_ns: None
            }
        );
        assert_eq!(
            utc_to_fx(2016, 12, 30, 23, 59, 60, 0),
            Err(Error::NotALeapSecond)
        );
    }

    #[test]
    fn skipped_labels_are_rejected() {
        // TAI - UTC fell by 0.05 s on 1961-08-01 and by 0.1 s on 1968-02-01. The
        // removed labels are those whose TAI instant reaches the next period's start.
        let skipped = Err(Error::SkippedLabel);
        assert_eq!(utc_to_fx(1961, 7, 31, 23, 59, 59, 950_000_001), skipped);
        assert!(utc_to_fx(1961, 7, 31, 23, 59, 59, 950_000_000).is_ok());
        assert_eq!(utc_to_fx(1968, 1, 31, 23, 59, 59, 900_000_003), skipped);
        assert!(utc_to_fx(1968, 1, 31, 23, 59, 59, 900_000_002).is_ok());
        // Every TAI instant near those steps decodes to a label that converts back.
        for b in [mjd_ns(37512), mjd_ns(39887)] {
            let t0 = period_at(b).1.tai(b);
            for k in -200..200i128 {
                let fx = scaled_to_fx(t0 + k * K).unwrap();
                let u = fx_to_utc(fx);
                assert!(u.inserted_ns.is_none());
                assert!(
                    unix_ns_to_fx(u.unix_ns).is_ok(),
                    "gap at {k} ns from boundary"
                );
            }
        }
    }

    #[test]
    fn utc_to_tai_is_monotonic_at_every_step() {
        for i in 1..=PERIODS.len() {
            let b = period(i).start_ns;
            let mut last: Option<Fx> = None;
            let mut u = b - 2 * NS;
            while u <= b + 2 * NS {
                if let Ok(fx) = unix_ns_to_fx(u) {
                    if let Some(l) = last {
                        assert!(fx > l, "not monotonic near boundary {i} at {u}");
                    }
                    last = Some(fx);
                }
                u += 10_000_000;
            }
        }
    }

    #[test]
    fn decode_inverts_encode() {
        let mut u = -400_000_000 * NS;
        while u < 1_900_000_000 * NS {
            if let Ok(fx) = unix_ns_to_fx(u) {
                let back = fx_to_utc(fx);
                assert_eq!(back.inserted_ns, None);
                assert!((back.unix_ns - u).abs() <= 1, "u={u} back={}", back.unix_ns);
            }
            u += 7_777_777_777_777;
        }
        // The Apollo 11 stamp spans a rubber second: 0.99999997 UTC seconds.
        let apollo = Stamp::parse("FZ3BMMKB").unwrap();
        assert_eq!(fx_to_utc(apollo.start()).unix_ns, -14_182_940_574_593_783);
        assert_eq!(fx_to_utc(apollo.end()).unix_ns, -14_182_939_574_593_813);
        // Inside the 0.107758 s inserted at 1972-01-01: decodes to that midnight.
        let t = period(14).tai(63_072_000 * NS) - 50_000_000 * K;
        let utc = fx_to_utc(scaled_to_fx(t).unwrap());
        assert_eq!(utc.unix_ns, 63_072_000 * NS);
        assert!(utc.inserted_ns.is_some());
    }

    #[test]
    fn truncation_equals_coarse_encoding() {
        let fx = utc_to_fx(2026, 10, 31, 3, 13, 0, 123_456_789).unwrap();
        let full = Stamp::from_fx(fx, 16).unwrap();
        for n in 1..=16 {
            assert_eq!(full.truncate(n).unwrap(), Stamp::from_fx(fx, n).unwrap());
        }
    }

    #[test]
    fn parsing() {
        let ok = |s: &str| Stamp::parse(s).unwrap().to_string();
        assert_eq!(ok("g0s7-g6bi"), "G0S7G6B1");
        assert_eq!(ok("oooo"), "0000");
        assert_eq!(ok("G0S7G6B1-3XYE"), "G0S7G6B13XYE");
        assert_eq!(Stamp::parse("G0SU"), Err(Error::BadChar('U')));
        assert_eq!(Stamp::parse(""), Err(Error::Empty));
        assert_eq!(Stamp::parse("-1W"), Err(Error::BadHyphen));
        assert_eq!(Stamp::parse("1W-"), Err(Error::BadHyphen));
        assert_eq!(Stamp::parse("G0S7--G6B1"), Err(Error::BadHyphen));
        assert_eq!(Stamp::parse(" G0S7"), Err(Error::BadChar(' ')));
        assert_eq!(Stamp::parse("\u{131}"), Err(Error::BadChar('\u{131}')));
        assert_eq!(Stamp::parse("G0S7G6B13XYE00000"), Err(Error::TooLong));
        assert_eq!(Stamp::parse("G0S7G6B1").unwrap().display(), "G0S7-G6B1");
    }

    #[test]
    fn durations_are_exact() {
        let s = |x: i128| Duration::from_ns(x * NS).unwrap().to_string();
        assert_eq!(s(60), "+1W");
        assert_eq!(s(3600), "+3GG");
        assert_eq!(s(86_400), "+2MC0");
        assert_eq!(s(604_800), "+JEM0");
        assert_eq!(Duration::from_ns(NS / 2).unwrap().to_string(), "+0.G");
        assert_eq!(Duration::from_ns(-3 * NS / 2).unwrap().to_string(), "-1.G");
        assert_eq!(Duration::new(0).unwrap().to_string(), "+0");
        assert_eq!(Duration::from_ns(-1).unwrap().to_string(), "-0.0000012B");
        for t in ["+0", "+2MC0", "-1.G", "+0.0000012B", "+ZZZZZZZZ.ZZZZZZZZ"] {
            assert_eq!(Duration::parse(t).unwrap().to_string(), t);
        }
        assert_eq!(Duration::parse("1W"), Err(Error::BadField));
        let a = Stamp::parse("G0S7").unwrap();
        let b = Stamp::parse("G0S7G6B1").unwrap();
        assert_eq!(Duration::between(&a, &b).to_string(), "+G6B1");
        assert_eq!(
            a.add(Duration::parse("+10000").unwrap())
                .unwrap()
                .to_string(),
            "G0S8"
        );
        assert_eq!(
            Stamp::parse("ZZZZZZZZ")
                .unwrap()
                .add(Duration::parse("+1").unwrap()),
            Err(Error::OutOfRange)
        );
    }

    #[test]
    fn sort_order_is_time_order() {
        let mut v: Vec<(u128, String)> = (0..2000)
            .map(|i| {
                let fx = tai_ns_to_fx((i as i128 - 1000) * 977_000_000_123).unwrap();
                let s = Stamp::from_fx(fx, (i % 16 + 1) as u8).unwrap();
                (s.start() * 32 + s.len() as u128, s.to_string())
            })
            .collect();
        let mut by_text = v.clone();
        by_text.sort_by(|a, b| a.1.cmp(&b.1));
        v.sort();
        assert_eq!(v, by_text);
    }

    #[test]
    fn range_limits() {
        assert_eq!(
            utc_to_fx(5_443_752_762_932_606, 1, 1, 0, 0, 0, 0),
            Err(Error::OutOfRange)
        );
        assert_eq!(unix_ns_to_fx(i128::MAX / 1000), Err(Error::OutOfRange));
        assert_eq!(Duration::from_ns(1 << 100), Err(Error::OutOfRange));
        assert_eq!(Duration::new(1 << 80), Err(Error::OutOfRange));
        assert!(is_provisional(1_900_000_000 * NS) && !is_provisional(0));
        assert_eq!(
            Stamp::from_fx(tai_ns_to_fx(-BIAS_S * NS).unwrap(), 8)
                .unwrap()
                .to_string(),
            "00000000"
        );
        assert!(tai_ns_to_fx(-BIAS_S * NS - 1).is_err());
        assert!(tai_ns_to_fx(BIAS_S * NS).is_err());
    }
}
