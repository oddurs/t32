//! T32 reference implementation. std only, integer arithmetic only.
use std::fmt;

pub const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
pub const CANONICAL_LEN: u8 = 8;
pub const MAX_LEN: u8 = 16;
/// Biased time is held as fixed point with 40 fractional bits (unit 2^-40 s).
const FRAC_BITS: u32 = 40;
const NS: i128 = 1_000_000_000;
const BIAS_S: i128 = 1 << 39;
/// TAI epoch 2000-01-01T00:00:00 expressed as Unix seconds + TAI-UTC (a "TAI label").
const EPOCH_TAI_LABEL_S: i128 = 946_684_800;
const RANGE_FX: u128 = 1 << 80; // 2^40 s

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error { Empty, TooLong, BadChar(char), OutOfRange, BadField, NotALeapSecond, SkippedSecond }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stamp { value: u128, len: u8 }

impl Stamp {
    /// Floor a biased fixed-point time to a stamp of `len` characters.
    pub fn from_biased_fx(fx: u128, len: u8) -> Result<Self, Error> {
        if len == 0 { return Err(Error::Empty) }
        if len > MAX_LEN { return Err(Error::TooLong) }
        if fx >= RANGE_FX { return Err(Error::OutOfRange) }
        Ok(Stamp { value: fx >> (5 * (MAX_LEN - len) as u32), len })
    }
    /// Lenient parse: optional t32: prefix, case-insensitive, O->0, I/L->1,
    /// hyphens ignored, U rejected.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let s = match s.get(..4) { Some(p) if p.eq_ignore_ascii_case("t32:") => &s[4..], _ => s };
        let (mut value, mut len) = (0u128, 0u8);
        for c in s.chars() {
            let d = match c.to_ascii_uppercase() {
                '-' => continue,
                'O' => 0,
                'I' | 'L' => 1,
                u => match ALPHABET.iter().position(|&a| a as char == u) {
                    Some(p) => p as u128,
                    None => return Err(Error::BadChar(c)),
                },
            };
            len += 1;
            if len > MAX_LEN { return Err(Error::TooLong) }
            value = value * 32 + d;
        }
        if len == 0 { return Err(Error::Empty) }
        Ok(Stamp { value, len })
    }
    pub fn len(&self) -> u8 { self.len }
    pub fn truncate(&self, len: u8) -> Self {
        assert!(len >= 1 && len <= self.len);
        Stamp { value: self.value >> (5 * (self.len - len) as u32), len }
    }
    fn shift(&self) -> u32 { 5 * (MAX_LEN - self.len) as u32 }
    /// Inclusive start of the interval, biased fixed point.
    pub fn start_fx(&self) -> u128 { self.value << self.shift() }
    /// Exclusive end of the interval, biased fixed point.
    pub fn end_fx(&self) -> u128 { (self.value + 1) << self.shift() }
    pub fn contains(&self, other: &Stamp) -> bool {
        other.len >= self.len && other.truncate(self.len) == *self
    }
}

impl fmt::Display for Stamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = vec![b'0'; self.len as usize];
        let mut v = self.value;
        for i in (0..self.len as usize).rev() { out[i] = ALPHABET[(v % 32) as usize]; v /= 32; }
        f.write_str(std::str::from_utf8(&out).unwrap())
    }
}

// ---------- TAI - UTC ----------

/// 1972 onward: (Unix second the value takes effect, TAI-UTC in seconds).
pub const LEAP_TABLE: [(i64, i64); 28] = [
    (63072000, 10), (78796800, 11), (94694400, 12), (126230400, 13), (157766400, 14),
    (189302400, 15), (220924800, 16), (252460800, 17), (283996800, 18), (315532800, 19),
    (362793600, 20), (394329600, 21), (425865600, 22), (489024000, 23), (567993600, 24),
    (631152000, 25), (662688000, 26), (709948800, 27), (741484800, 28), (773020800, 29),
    (820454400, 30), (867715200, 31), (915148800, 32), (1136073600, 33), (1230768000, 34),
    (1341100800, 35), (1435708800, 36), (1483228800, 37),
];
/// Unix second after which LEAP_TABLE is not known to be valid (2027-06-28).
pub const LEAP_TABLE_EXPIRES: i64 = 1_814_140_800;

/// 1961-1971: (start MJD, offset a in 1e-7 s, reference MJD m, rate r in 1e-7 s/day).
/// TAI-UTC = a + (MJD - m) * r.
const RUBBER: [(i64, i128, i64, i128); 13] = [
    (37300, 14_228_180, 37300, 12_960), (37512, 13_728_180, 37300, 12_960),
    (37665, 18_458_580, 37665, 11_232), (38334, 19_458_580, 37665, 11_232),
    (38395, 32_401_300, 38761, 12_960), (38486, 33_401_300, 38761, 12_960),
    (38639, 34_401_300, 38761, 12_960), (38761, 35_401_300, 38761, 12_960),
    (38820, 36_401_300, 38761, 12_960), (38942, 37_401_300, 38761, 12_960),
    (39004, 38_401_300, 38761, 12_960), (39126, 43_131_700, 39126, 25_920),
    (39887, 42_131_700, 39126, 25_920),
];
const MJD_UNIX_EPOCH: i64 = 40587;
const DAY_NS: i128 = 86_400 * NS;

fn floor_div(a: i128, b: i128) -> i128 { let q = a / b; if (a % b != 0) && ((a < 0) != (b < 0)) { q - 1 } else { q } }

/// TAI-UTC in nanoseconds at a UTC instant given as Unix nanoseconds.
pub fn delta_at_ns(unix_ns: i128) -> i128 {
    if unix_ns >= LEAP_TABLE[0].0 as i128 * NS {
        let d = LEAP_TABLE.iter().rev().find(|(t, _)| unix_ns >= *t as i128 * NS).unwrap().1;
        return d as i128 * NS;
    }
    let first = (RUBBER[0].0 - MJD_UNIX_EPOCH) as i128 * DAY_NS;
    if unix_ns < first { return 0 } // before 1961: defined as zero
    let &(_, a, m, r) = RUBBER.iter().rev()
        .find(|(s, ..)| unix_ns >= (*s - MJD_UNIX_EPOCH) as i128 * DAY_NS).unwrap();
    let since_m_ns = unix_ns - (m - MJD_UNIX_EPOCH) as i128 * DAY_NS;
    // (a + since_m_ns/DAY_NS * r) in 1e-7 s, converted to ns (x100), floored.
    a * 100 + floor_div(since_m_ns * r * 100, DAY_NS)
}

/// TAI nanoseconds since the T32 epoch -> biased fixed point.
pub fn tai_ns_to_fx(tai_ns: i128) -> Result<u128, Error> {
    let total = tai_ns + BIAS_S * NS;
    if total < 0 { return Err(Error::OutOfRange) }
    let fx = ((total as u128) << FRAC_BITS) / NS as u128; // floor
    if fx >= RANGE_FX { return Err(Error::OutOfRange) }
    Ok(fx)
}

/// Biased fixed point -> TAI nanoseconds since epoch (floored).
pub fn fx_to_tai_ns(fx: u128) -> i128 {
    floor_div(fx as i128 * NS, 1i128 << FRAC_BITS) - BIAS_S * NS
}

/// Unix time (UTC) in nanoseconds -> biased fixed point. A repeated Unix second
/// during a leap second is taken as its first occurrence.
pub fn unix_ns_to_fx(unix_ns: i128) -> Result<u128, Error> {
    tai_ns_to_fx(unix_ns + delta_at_ns(unix_ns) - EPOCH_TAI_LABEL_S * NS)
}

/// Biased fixed point -> Unix nanoseconds. An instant inside an inserted leap
/// second maps to the Unix second that repeats (the start of the next day).
pub fn fx_to_unix_ns(fx: u128) -> i128 {
    let label = fx_to_tai_ns(fx) + EPOCH_TAI_LABEL_S * NS; // TAI as Unix-style label
    if let Some((_, d)) = LEAP_TABLE.iter().rev().find(|(t, d)| label >= (*t + *d) as i128 * NS) {
        return label - *d as i128 * NS;
    }
    if label >= (LEAP_TABLE[0].0 + 10) as i128 * NS { return LEAP_TABLE[0].0 as i128 * NS; }
    let mut u = label;
    for _ in 0..4 { u = label - delta_at_ns(u); }
    u
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

/// Proleptic Gregorian UTC fields (astronomical years) -> biased fixed point.
/// Accepts second = 60 only during a real inserted leap second; rejects a
/// second removed by a negative leap second.
pub fn utc_to_fx(y: i64, mo: u32, d: u32, h: u32, mi: u32, s: u32, ns: u32) -> Result<u128, Error> {
    if !(1..=12).contains(&mo) || d == 0 || d > 31 || h > 23 || mi > 59 || s > 60 || ns >= 1_000_000_000 {
        return Err(Error::BadField);
    }
    let days = days_from_civil(y, mo, d);
    if civil_from_days(days) != (y, mo, d) { return Err(Error::BadField) }
    let next_midnight = (days + 1) as i128 * DAY_NS;
    let step = delta_at_ns(next_midnight) - delta_at_ns(next_midnight - 1);
    let at_end = h == 23 && mi == 59;
    let base = days as i128 * DAY_NS + ((h * 3600 + mi * 60 + s.min(59)) as i128) * NS + ns as i128;
    if s == 60 {
        if !(at_end && step == NS) { return Err(Error::NotALeapSecond) }
        return tai_ns_to_fx(base + delta_at_ns(base) + NS - EPOCH_TAI_LABEL_S * NS);
    }
    if at_end && s == 59 && step == -NS { return Err(Error::SkippedSecond) }
    unix_ns_to_fx(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn st(y: i64, mo: u32, d: u32, h: u32, mi: u32, s: u32, ns: u32, n: u8) -> String {
        Stamp::from_biased_fx(utc_to_fx(y, mo, d, h, mi, s, ns).unwrap(), n).unwrap().to_string()
    }
    #[test] fn epoch_is_g0000000() {
        assert_eq!(Stamp::from_biased_fx(tai_ns_to_fx(0).unwrap(), 8).unwrap().to_string(), "G0000000");
        assert_eq!(st(1999, 12, 31, 23, 59, 28, 0, 8), "G0000000");
    }
    #[test] fn leap_second_is_contiguous() {
        let a = utc_to_fx(2016, 12, 31, 23, 59, 59, 0).unwrap();
        let b = utc_to_fx(2016, 12, 31, 23, 59, 60, 0).unwrap();
        let c = utc_to_fx(2017, 1, 1, 0, 0, 0, 0).unwrap();
        assert_eq!(b - a, 1 << FRAC_BITS);
        assert_eq!(c - b, 1 << FRAC_BITS);
        assert_eq!(utc_to_fx(2016, 12, 30, 23, 59, 60, 0), Err(Error::NotALeapSecond));
    }
    #[test] fn truncation_equals_coarse_encoding() {
        let fx = utc_to_fx(2026, 10, 31, 3, 13, 0, 123_456_789).unwrap();
        let full = Stamp::from_biased_fx(fx, 16).unwrap();
        for n in 1..=16 { assert_eq!(full.truncate(n), Stamp::from_biased_fx(fx, n).unwrap()); }
    }
    #[test] fn parse_is_lenient_output_strict() {
        assert_eq!(Stamp::parse("g0s7-g6ci").unwrap().to_string(), Stamp::parse("G0S7G6C1").unwrap().to_string());
        assert_eq!(Stamp::parse("oooo").unwrap().to_string(), "0000");
        assert_eq!(Stamp::parse("G0SU"), Err(Error::BadChar('U')));
        assert_eq!(Stamp::parse(""), Err(Error::Empty));
        assert_eq!(Stamp::parse("-"), Err(Error::Empty));
    }
    #[test] fn sort_order_is_time_order() {
        let mut v: Vec<(u128, String)> = (0..2000).map(|i| {
            let fx = tai_ns_to_fx((i as i128 - 1000) * 977_000_000_123).unwrap();
            let n = (i % 16 + 1) as u8;
            let s = Stamp::from_biased_fx(fx, n).unwrap();
            (s.start_fx() * 32 + n as u128, s.to_string())
        }).collect();
        let mut by_text = v.clone();
        by_text.sort_by(|a, b| a.1.cmp(&b.1));
        v.sort();
        assert_eq!(v, by_text);
    }
    #[test] fn spec_vectors() {
        let v: &[((i64,u32,u32,u32,u32,u32,u32), u8, &str)] = &[
            ((1999,12,31,23,59,28,0),8,"G0000000"), ((1970,1,1,0,0,0,0),8,"FZ3S5F48"),
            ((1969,7,20,20,17,40,0),8,"FZ3BMMKB"), ((1972,1,1,0,0,0,0),8,"FZ5NA8WA"),
            ((2016,12,31,23,59,59,0),8,"G0FZP0S3"), ((2016,12,31,23,59,60,0),8,"G0FZP0S4"),
            ((2017,1,1,0,0,0,0),8,"G0FZP0S5"), ((2026,10,31,3,13,0,0),8,"G0S7G6B1"),
            ((2026,10,31,3,13,0,500_000_000),10,"G0S7G6B1G0"),
            ((2026,10,31,3,13,0,123_000_000),12,"G0S7G6B13XYE"), ((1675,8,20,0,0,0,0),8,"FPEY6N00"),
        ];
        for &((y,mo,d,h,mi,s,ns), n, want) in v { assert_eq!(st(y,mo,d,h,mi,s,ns,n), want); }
        assert_eq!(Stamp::parse("g0s7-g6bi").unwrap().to_string(), "G0S7G6B1");
        assert_eq!(Stamp::parse("t32:G0S7G6B1").unwrap().to_string(), "G0S7G6B1");
    }
    #[test] fn containment_is_prefix() {
        let a = Stamp::parse("G0S7").unwrap();
        assert!(a.contains(&Stamp::parse("G0S7G6C1").unwrap()));
        assert!(!a.contains(&Stamp::parse("G0S8").unwrap()));
        let b = Stamp::parse("G0S7G6C1").unwrap();
        assert!(a.start_fx() <= b.start_fx() && b.end_fx() <= a.end_fx());
    }
    #[test] fn unix_round_trip() {
        for &u in &[-5_000_000_000i64, -300_000_000, 0, 63_071_999, 63_072_000, 1_483_228_799, 1_483_228_800, 1_793_416_380] {
            let fx = unix_ns_to_fx(u as i128 * NS).unwrap();
            let back = fx_to_unix_ns(fx);
            assert!((back - u as i128 * NS).abs() <= 1, "u={u} back={back}");
        }
    }
    #[test] fn rubber_era_unix_epoch() {
        // 1970-01-01: 4.2131700 + (40587-39126)*0.002592 = 8.000082 s
        assert_eq!(delta_at_ns(0), 8_000_082_000);
    }
    #[test] fn range_limits() {
        assert!(tai_ns_to_fx(-BIAS_S * NS - 1).is_err());
        assert!(tai_ns_to_fx(BIAS_S * NS).is_err());
        assert_eq!(Stamp::from_biased_fx(tai_ns_to_fx(-BIAS_S * NS).unwrap(), 8).unwrap().to_string(), "00000000");
    }
}
