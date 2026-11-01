use std::fmt;
use std::ops::{Add, Sub};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utc::{self, UtcDateTime, D, K, MAX_LABEL_NS, NS};
use crate::{Duration, Error, Stamp};

/// Bits after the binary point: an [`Instant`] counts units of 2⁻⁴⁰ s.
pub(crate) const FRAC_BITS: u32 = 40;
const FRAC_MASK: u128 = (1 << FRAC_BITS) - 1;
/// Instants are below 2⁸⁰ units: 2⁴⁰ s, the whole range of T32.
pub(crate) const RANGE: u128 = 1 << 80;
/// The bias B = 2³⁹ s that centres the range on the epoch.
const BIAS_S: i128 = 1 << 39;

/// A point on the T32 time scale, exact to 2⁻⁴⁰ s (about 0.9 ps).
///
/// The time scale is TAI from 1961 and Universal Time before, so from 1961 on,
/// subtracting two instants gives true elapsed SI seconds. Every instant from
/// 15,423 BCE to 19,421 CE is representable, and every representable instant
/// can be written as a [`Stamp`] of any length.
///
/// An instant made from nanoseconds (TAI, Unix or UTC) reads back as the same
/// nanoseconds: readings in nanoseconds give the latest nanosecond whose
/// instant is this one, falling back to the floor when no nanosecond is.
///
/// ```
/// use t32::Instant;
///
/// // 2016-12-31T23:59:59 to 2017-01-01T00:00:00 spans the last leap second.
/// let a = Instant::from_utc("2016-12-31T23:59:59Z".parse()?)?;
/// let b = Instant::from_utc("2017-01-01T00:00:00Z".parse()?)?;
/// assert_eq!((b - a).as_nanos(), 2_000_000_000);
///
/// // Instants convert back to UTC, leap second included.
/// let leap = Instant::from_utc("2016-12-31T23:59:60.25Z".parse()?)?;
/// assert_eq!(leap.to_utc().to_string(), "2016-12-31T23:59:60.25Z");
/// # Ok::<(), t32::Error>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(pub(crate) u128);

impl Instant {
    /// The earliest representable instant, the start of stamp `00000000`
    /// (−15422-12-06T11:41:52 UT, that is, 15423 BCE).
    pub const MIN: Instant = Instant(0);
    /// The latest representable instant, the last 2⁻⁴⁰ s of stamp `ZZZZZZZZ`,
    /// which ends at 19421-01-25T12:17:31 UTC.
    pub const MAX: Instant = Instant(RANGE - 1);
    /// The T32 epoch, 2000-01-01T00:00:00 TAI (1999-12-31T23:59:28 UTC): stamp `G0000000`.
    pub const EPOCH: Instant = Instant((BIAS_S as u128) << FRAC_BITS);

    /// The instant `units` × 2⁻⁴⁰ s after [`Instant::MIN`]: the biased time of
    /// the spec, as fixed point. Fails unless `units < 2⁸⁰`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] unless `units < 2⁸⁰`.
    pub const fn from_units(units: u128) -> Result<Self, Error> {
        if units < RANGE {
            Ok(Instant(units))
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// The biased time in units of 2⁻⁴⁰ s. Its top 5·n bits are the value of
    /// the length-n stamp containing this instant.
    #[must_use]
    pub const fn units(self) -> u128 {
        self.0
    }

    /// The instant `nanos` TAI nanoseconds after (or before) the epoch.
    ///
    /// ```
    /// use t32::Instant;
    ///
    /// assert_eq!(Instant::from_tai_nanos(0)?, Instant::EPOCH);
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] outside the representable range.
    pub fn from_tai_nanos(nanos: i128) -> Result<Self, Error> {
        if nanos.unsigned_abs() > MAX_LABEL_NS as u128 {
            return Err(Error::OutOfRange);
        }
        Self::from_scaled(nanos * K)
    }

    /// TAI nanoseconds since the epoch.
    ///
    /// ```
    /// use t32::Instant;
    ///
    /// for n in [-1, 0, 1, 123_456_789] {
    ///     assert_eq!(Instant::from_tai_nanos(n)?.tai_nanos(), n);
    /// }
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn tai_nanos(self) -> i128 {
        self.reading_scaled().div_euclid(K)
    }

    /// The instant of a Unix timestamp in nanoseconds, a UTC label counted by
    /// the POSIX formula. A Unix value never names a leap second: the value
    /// that repeats is read as the second after it, 00:00:00.
    ///
    /// ```
    /// use t32::{Instant, Stamp};
    ///
    /// let t = Instant::from_unix_nanos(1_793_416_380_000_000_000)?;
    /// assert_eq!(Stamp::new(t, 8)?.to_string(), "G0S7G6B1");
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] outside the representable range, and
    /// [`Error::RemovedTime`] for a label UTC skipped in 1961 or 1968.
    pub fn from_unix_nanos(nanos: i128) -> Result<Self, Error> {
        Self::from_scaled(utc::unix_to_scaled(nanos)?)
    }

    /// The Unix timestamp in nanoseconds. An instant inside a leap second
    /// gives the value of the second after it, as POSIX clocks do.
    ///
    /// ```
    /// use t32::Instant;
    ///
    /// // 1970 is in the rubber-second era, and the round trip is still exact.
    /// assert_eq!(Instant::from_unix_nanos(0)?.unix_nanos(), 0);
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn unix_nanos(self) -> i128 {
        self.reading().unix_ns
    }

    /// The instant of a UTC date-time.
    ///
    /// ```
    /// use t32::{Instant, UtcDateTime};
    ///
    /// let t = Instant::from_utc(UtcDateTime::new(2000, 1, 1, 0, 0, 0)?)?;
    /// assert_eq!(t.tai_nanos(), 32_000_000_000);   // TAI − UTC was 32 s
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the date-time lies outside the representable range.
    pub fn from_utc(datetime: UtcDateTime) -> Result<Self, Error> {
        Self::from_scaled(datetime.to_scaled()?)
    }

    /// The UTC date-time of this instant, to the nanosecond.
    ///
    /// Inside a leap second this is 23:59:60.f. Before 1972, UTC also inserted
    /// fractions of a second that have no label of their own; an instant in one
    /// reads as the label at its end, and [`Instant::in_inserted_time`] is true.
    #[must_use]
    pub fn to_utc(self) -> UtcDateTime {
        UtcDateTime::from_reading(&self.reading())
    }

    /// Whether this instant falls in time UTC inserted: a leap second, or one
    /// of the fractional steps before 1972. The spec requires converters to
    /// report this alongside the reading.
    ///
    /// A label's instant is rarely a whole number of 2⁻⁴⁰ s units, so the
    /// instant made from a label is its floor. Just after a fractional
    /// insertion before 1972, that floor can lie inside the insertion: the
    /// instant then reports `true`, and still reads back as the label.
    ///
    /// ```
    /// use t32::Instant;
    ///
    /// assert!(Instant::from_utc("2016-12-31T23:59:60.5Z".parse()?)?.in_inserted_time());
    /// assert!(!Instant::from_utc("2016-12-31T23:59:59.5Z".parse()?)?.in_inserted_time());
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn in_inserted_time(self) -> bool {
        // The instant itself decides, not the end of its unit.
        utc::scaled_to_reading(self.floor_scaled())
            .inserted_ns
            .is_some()
    }

    /// Whether reading this instant as UTC is provisional: its label is at or
    /// after the expiry of the leap-second table, 2027-06-28T00:00:00Z
    /// ([`UtcDateTime::TABLE_EXPIRES`]). The spec requires converters to
    /// report this with the result, whichever way they convert; an instant made
    /// from a Unix time or the system clock reports it here.
    ///
    /// ```
    /// use t32::Instant;
    ///
    /// assert!(!Instant::from_unix_nanos(1_793_416_380_000_000_000)?.is_provisional());
    /// assert!(Instant::from_unix_nanos(1_900_000_000_000_000_000)?.is_provisional());
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn is_provisional(self) -> bool {
        self.to_utc().is_provisional()
    }

    /// The current instant, from the system clock.
    ///
    /// The system clock follows UTC, so this converts through the leap-second
    /// table. Around a past leap second, a clock that smeared it may be up to
    /// 1 s off.
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the system clock reads outside the
    /// representable range, which a working clock never does.
    pub fn now() -> Result<Self, Error> {
        SystemTime::now().try_into()
    }

    /// The stamp of `len` characters (1 to 16) that contains this instant.
    /// The same as [`Stamp::new`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidLength`] unless `len` is 1 to 16.
    pub const fn stamp(self, len: u8) -> Result<Stamp, Error> {
        Stamp::new(self, len)
    }

    /// This instant moved by a duration, failing outside the representable range.
    ///
    /// ```
    /// use t32::{Duration, Instant};
    ///
    /// let later = Instant::EPOCH.checked_add(Duration::from_secs(86_400)?)?;
    /// assert_eq!(later.tai_nanos(), 86_400_000_000_000);
    /// assert!(Instant::MAX.checked_add(Duration::from_nanos(1)?).is_err());
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the result leaves the representable range.
    pub const fn checked_add(self, duration: Duration) -> Result<Self, Error> {
        let units = self.0 as i128 + duration.units();
        if units < 0 {
            return Err(Error::OutOfRange);
        }
        Self::from_units(units as u128)
    }

    /// This instant moved back by a duration, failing outside the representable range.
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the result leaves the representable range.
    pub fn checked_sub(self, duration: Duration) -> Result<Self, Error> {
        self.checked_add(-duration)
    }

    /// Scaled TAI (units of 1/D s since the epoch) to an instant, floored.
    fn from_scaled(t: i128) -> Result<Self, Error> {
        let total = t + BIAS_S * D;
        if total < 0 {
            return Err(Error::OutOfRange);
        }
        let (q, r) = ((total / D) as u128, (total % D) as u128);
        Self::from_units((q << FRAC_BITS) + (r << FRAC_BITS) / D as u128)
    }

    /// This instant in scaled TAI, floored.
    fn floor_scaled(self) -> i128 {
        let (q, r) = ((self.0 >> FRAC_BITS) as i128, (self.0 & FRAC_MASK) as i128);
        q * D + ((r * D) >> FRAC_BITS) - BIAS_S * D
    }

    /// The last scaled-TAI value inside this instant's 2⁻⁴⁰ s unit: the largest
    /// integer below the start of the next unit.
    fn reading_scaled(self) -> i128 {
        let next = self.0 + 1;
        let (q, r) = ((next >> FRAC_BITS) as i128, (next & FRAC_MASK) as i128);
        q * D + (r * D - 1).div_euclid(1 << FRAC_BITS) - BIAS_S * D
    }

    /// The UTC reading (SPEC.md, Decoding and interval semantics).
    ///
    /// An instant in inserted time reads as the label where the insertion ends,
    /// and inside a leap second as 23:59:60.f. Any other instant reads as the
    /// latest nanosecond label whose instant encodes, at full resolution, no
    /// later than this one: the floor of the reading at the end of its unit.
    /// A unit is narrower than a nanosecond, so at most one label encodes to
    /// it, and nanosecond round trips are exact.
    fn reading(self) -> utc::Reading {
        let own = utc::scaled_to_reading(self.floor_scaled());
        let top = utc::scaled_to_reading(self.reading_scaled());
        match (own.inserted_ns, top.inserted_ns) {
            // A leap second is whole TAI seconds, so its units never straddle its ends.
            (Some(_), _) if own.leap_second => top,
            (Some(_), _) => own,
            // The unit runs into inserted time: the last label before it.
            (None, Some(_)) => utc::Reading {
                unix_ns: top.unix_ns - 1,
                inserted_ns: None,
                leap_second: false,
            },
            (None, None) => top,
        }
    }
}

impl Sub for Instant {
    type Output = Duration;

    /// The exact duration from `rhs` to `self`.
    fn sub(self, rhs: Instant) -> Duration {
        // Both lie in [0, 2⁸⁰), so the difference is always a valid duration.
        Duration(self.0 as i128 - rhs.0 as i128)
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    /// # Panics
    ///
    /// Panics outside the representable range, like `std::time::Instant`;
    /// [`Instant::checked_add`] returns an error instead.
    fn add(self, duration: Duration) -> Instant {
        self.checked_add(duration).expect("instant out of range")
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    /// # Panics
    ///
    /// Panics outside the representable range; [`Instant::checked_sub`]
    /// returns an error instead.
    fn sub(self, duration: Duration) -> Instant {
        self.checked_sub(duration).expect("instant out of range")
    }
}

impl TryFrom<Instant> for SystemTime {
    type Error = Error;

    /// The system time of an instant, by its Unix reading (an instant inside a
    /// leap second becomes the following 00:00:00, as POSIX clocks show it).
    fn try_from(instant: Instant) -> Result<Self, Error> {
        // Whole seconds and a non-negative fraction, so the full range fits.
        let nanos = instant.unix_nanos();
        let secs =
            u64::try_from(nanos.div_euclid(NS).unsigned_abs()).map_err(|_| Error::OutOfRange)?;
        let whole = std::time::Duration::from_secs(secs);
        let base = if nanos >= 0 {
            UNIX_EPOCH.checked_add(whole)
        } else {
            UNIX_EPOCH.checked_sub(whole)
        };
        base.and_then(|t| {
            t.checked_add(std::time::Duration::from_nanos(nanos.rem_euclid(NS) as u64))
        })
        .ok_or(Error::OutOfRange)
    }
}

impl TryFrom<SystemTime> for Instant {
    type Error = Error;

    /// Converts a system time, which counts Unix time (UTC).
    fn try_from(time: SystemTime) -> Result<Self, Error> {
        let nanos = match time.duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_nanos() as i128,
            Err(e) => -(e.duration().as_nanos() as i128),
        };
        Instant::from_unix_nanos(nanos)
    }
}

impl TryFrom<UtcDateTime> for Instant {
    type Error = Error;

    fn try_from(datetime: UtcDateTime) -> Result<Self, Error> {
        Instant::from_utc(datetime)
    }
}

impl fmt::Debug for Instant {
    /// Shows the full 16-character stamp, which is exact, and the UTC reading:
    /// `Instant(G0S7G6B100000000, 2026-10-31T03:13:00Z)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exact = Stamp::new(*self, Stamp::MAX_LEN).map_err(|_| fmt::Error)?;
        write!(f, "Instant({exact}, {})", self.to_utc())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nanosecond_round_trips_are_exact() {
        let mut u: i128 = -400_000_000 * NS;
        while u < 1_900_000_000 * NS {
            if let Ok(t) = Instant::from_unix_nanos(u) {
                assert!(!t.in_inserted_time());
                assert_eq!(t.unix_nanos(), u);
                assert_eq!(Instant::from_utc(t.to_utc()).unwrap(), t);
            }
            let t = Instant::from_tai_nanos(u).unwrap();
            assert_eq!(t.tai_nanos(), u);
            u += 7_777_777_777_777;
        }
        for u in -2_000..2_000 {
            assert_eq!(Instant::from_unix_nanos(u).unwrap().unix_nanos(), u);
            assert_eq!(Instant::from_tai_nanos(u).unwrap().tai_nanos(), u);
        }
    }

    #[test]
    fn round_trips_are_exact_at_every_boundary() {
        // Every row of Appendix A starts at a midnight; sweep 50 ns either side.
        let midnights = (37_300i128..=57_754).map(|mjd| (mjd - 40_587) * 86_400 * NS);
        for b in midnights.filter(|&b| {
            let t = |u| Instant::from_unix_nanos(u).map(|i| i.tai_nanos());
            // Only days where TAI − UTC changes, to keep the sweep quick.
            matches!((t(b - NS), t(b)), (Ok(x), Ok(y)) if y - x != NS)
        }) {
            for u in b - 50..b + 50 {
                if let Ok(t) = Instant::from_unix_nanos(u) {
                    // (The unit may begin inside inserted time just before the
                    // label, as at 1961-01-01; the label still reads back.)
                    assert_eq!(t.unix_nanos(), u, "label {u}");
                    assert_eq!(Instant::from_utc(t.to_utc()).unwrap(), t, "label {u}");
                }
            }
        }
    }

    #[test]
    fn stamp_boundaries_read_as_their_floor() {
        // Starts of stamps up to 12 characters are exact in scaled TAI.
        let apollo: Stamp = "FZ3BMMKB".parse().unwrap();
        assert_eq!(apollo.start().unix_nanos(), -14_182_940_574_593_783);
        assert_eq!(apollo.end().unwrap().unix_nanos(), -14_182_939_574_593_813);
        assert_eq!(
            apollo.start().to_utc().to_string(),
            "1969-07-20T20:17:39.425406217Z"
        );
    }

    #[test]
    fn a_unix_value_never_names_the_leap_second() {
        let midnight = Instant::from_unix_nanos(1_483_228_800 * NS).unwrap();
        assert_eq!(midnight.to_utc().to_string(), "2017-01-01T00:00:00Z");
        let leap = Instant::from_utc("2016-12-31T23:59:60.25Z".parse().unwrap()).unwrap();
        assert_eq!(leap.unix_nanos(), 1_483_228_800 * NS);
        assert!(leap.in_inserted_time());
    }

    #[test]
    fn range_is_checked() {
        assert_eq!(Instant::from_tai_nanos(-BIAS_S * NS).unwrap(), Instant::MIN);
        assert!(Instant::from_tai_nanos(-BIAS_S * NS - 1).is_err());
        assert!(Instant::from_tai_nanos(BIAS_S * NS).is_err());
        assert!(Instant::from_unix_nanos(i128::MAX / 1000).is_err());
        assert!(Instant::from_units(RANGE).is_err());
        assert!(Instant::from_utc(UtcDateTime::new(30_000, 1, 1, 0, 0, 0).unwrap()).is_err());
        assert_eq!((Instant::MAX - Instant::MIN).units(), (RANGE - 1) as i128);
    }

    #[test]
    fn extreme_integers_are_rejected_not_wrapped() {
        assert!(Instant::from_tai_nanos(i128::MIN).is_err());
        assert!(Instant::from_tai_nanos(i128::MAX).is_err());
        assert!(Instant::from_unix_nanos(i128::MIN).is_err());
        assert!(Instant::from_unix_nanos(i128::MAX).is_err());
        assert!(Instant::from_units(u128::MAX).is_err());
    }

    #[test]
    fn nanosecond_differences_are_exact() {
        for gap in [1, 7, 999, 1_000_000_007] {
            let a = Instant::from_unix_nanos(1_700_000_000 * NS).unwrap();
            let b = Instant::from_unix_nanos(1_700_000_000 * NS + gap).unwrap();
            assert_eq!((b - a).as_nanos(), gap);
        }
    }

    #[test]
    fn provisional_from_every_path() {
        let expiry = 1_814_140_800 * NS;
        assert!(!Instant::from_unix_nanos(expiry - 1)
            .unwrap()
            .is_provisional());
        assert!(Instant::from_unix_nanos(expiry).unwrap().is_provisional());
    }

    #[test]
    fn operators() {
        let day = Duration::from_secs(86_400).unwrap();
        assert_eq!(Instant::EPOCH + day - day, Instant::EPOCH);
        assert_eq!((Instant::EPOCH + day) - Instant::EPOCH, day);
        assert!(std::panic::catch_unwind(|| Instant::MAX + day).is_err());
        assert_eq!(
            format!("{:?}", Instant::EPOCH),
            "Instant(G000000000000000, 1999-12-31T23:59:28Z)"
        );
    }

    #[test]
    fn system_time_is_unix_time() {
        for nanos in [-86_400 * NS, -1, 0, 1, 1_793_416_380 * NS] {
            let t = Instant::from_unix_nanos(nanos).unwrap();
            assert_eq!(
                Instant::try_from(SystemTime::try_from(t).unwrap()).unwrap(),
                t
            );
        }
        let t = Instant::try_from(UNIX_EPOCH).unwrap();
        assert_eq!(t.unix_nanos(), 0);
        let before = UNIX_EPOCH - std::time::Duration::from_secs(86_400);
        assert_eq!(
            Instant::try_from(before).unwrap().unix_nanos(),
            -86_400 * NS
        );
        assert!(Instant::now().is_ok());
        // Centuries beyond u64 nanoseconds convert both ways.
        for secs in [20_000_000_000i128, -21_000_000_000] {
            let t = Instant::from_unix_nanos(secs * NS + 1).unwrap();
            assert_eq!(
                Instant::try_from(SystemTime::try_from(t).unwrap()).unwrap(),
                t
            );
        }
    }
}
