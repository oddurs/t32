use std::fmt::{self, Write};
use std::iter::Sum;
use std::ops::{Add, Neg, Sub};
use std::str::FromStr;

use crate::instant::FRAC_BITS;
use crate::stamp;
use crate::utc::NS;
use crate::{Error, ALPHABET};

/// Durations stay below 2⁴⁰ s in magnitude: 2⁸⁰ units of 2⁻⁴⁰ s.
const LIMIT: i128 = 1 << 80;
const FRAC_MASK: u128 = (1 << FRAC_BITS) - 1;

/// A signed span of SI seconds, exact to 2⁻⁴⁰ s, below 2⁴⁰ s in magnitude.
///
/// Its text is exact too: a sign, base32 seconds and optional base32 fraction
/// digits. `+2MC0` is one day; `-0.G` is minus half a second.
///
/// ```
/// use t32::{Duration, Stamp};
///
/// let day = Duration::from_secs(86_400)?;
/// assert_eq!(day.to_string(), "+2MC0");
/// assert_eq!("+2MC0".parse::<Duration>()?, day);
///
/// // The exact distance between two stamps' starts.
/// let a: Stamp = "G0S7".parse()?;
/// let b: Stamp = "G0S7G6B1".parse()?;
/// assert_eq!((b.start() - a.start()).to_string(), "+G6B1");
/// # Ok::<(), t32::Error>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration(pub(crate) i128);

impl Duration {
    /// No time at all, written `+0`.
    pub const ZERO: Duration = Duration(0);

    /// The duration of `units` × 2⁻⁴⁰ s. Fails at 2⁴⁰ s or more in magnitude.
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the magnitude is 2⁴⁰ s or more.
    pub const fn from_units(units: i128) -> Result<Self, Error> {
        if units.unsigned_abs() < LIMIT as u128 {
            Ok(Duration(units))
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// The duration in units of 2⁻⁴⁰ s.
    #[must_use]
    pub const fn units(self) -> i128 {
        self.0
    }

    /// A whole number of seconds.
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the magnitude is 2⁴⁰ s or more.
    ///
    /// ```
    /// use t32::Duration;
    ///
    /// const DAY: Duration = match Duration::from_secs(86_400) {
    ///     Ok(d) => d,
    ///     Err(_) => panic!(),
    /// };
    /// assert_eq!(DAY.to_string(), "+2MC0");
    /// ```
    pub const fn from_secs(secs: i64) -> Result<Self, Error> {
        Self::from_units((secs as i128) << FRAC_BITS)
    }

    /// From nanoseconds, truncated toward zero (so negation commutes with it).
    ///
    /// ```
    /// use t32::Duration;
    ///
    /// assert_eq!(Duration::from_nanos(500_000_000)?.to_string(), "+0.G");
    /// assert_eq!(Duration::from_nanos(-1)?.to_string(), "-0.0000012B");
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the magnitude is 2⁴⁰ s or more.
    pub const fn from_nanos(nanos: i128) -> Result<Self, Error> {
        let mag = nanos.unsigned_abs();
        if mag >= (LIMIT as u128 >> FRAC_BITS) * NS as u128 {
            return Err(Error::OutOfRange);
        }
        let units = ((mag << FRAC_BITS) / NS as u128) as i128;
        Self::from_units(if nanos < 0 { -units } else { units })
    }

    /// In nanoseconds, to the nearest (halves away from zero).
    ///
    /// A nanosecond is not a whole number of 2⁻⁴⁰ s units, so
    /// [`Duration::from_nanos`] truncates; rounding back to the nearest makes
    /// every nanosecond count survive the round trip, and the difference of
    /// two instants made from nanoseconds reads as the exact difference.
    ///
    /// ```
    /// use t32::Duration;
    ///
    /// for n in [1, -1, 7, 999_999_999] {
    ///     assert_eq!(Duration::from_nanos(n)?.as_nanos(), n);
    /// }
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub const fn as_nanos(self) -> i128 {
        let half = 1 << (FRAC_BITS - 1);
        let mag = ((self.0.unsigned_abs() * NS as u128 + half) >> FRAC_BITS) as i128;
        if self.0 < 0 {
            -mag
        } else {
            mag
        }
    }

    /// Whether the duration is below zero.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// The magnitude. Always in range: the bound is symmetric.
    #[must_use]
    pub const fn abs(self) -> Duration {
        Duration(self.0.abs())
    }

    /// The sum, failing at 2⁴⁰ s or more in magnitude.
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the sum is 2⁴⁰ s or more in magnitude.
    pub const fn checked_add(self, other: Duration) -> Result<Self, Error> {
        Self::from_units(self.0 + other.0)
    }

    /// The difference, failing at 2⁴⁰ s or more in magnitude.
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the difference is 2⁴⁰ s or more in magnitude.
    pub const fn checked_sub(self, other: Duration) -> Result<Self, Error> {
        Self::from_units(self.0 - other.0)
    }
}

impl Add for Duration {
    type Output = Duration;

    /// # Panics
    ///
    /// Panics at 2⁴⁰ s or more in magnitude; [`Duration::checked_add`]
    /// returns an error instead.
    fn add(self, other: Duration) -> Duration {
        self.checked_add(other).expect("duration out of range")
    }
}

impl Sub for Duration {
    type Output = Duration;

    /// # Panics
    ///
    /// Panics at 2⁴⁰ s or more in magnitude; [`Duration::checked_sub`]
    /// returns an error instead.
    fn sub(self, other: Duration) -> Duration {
        self.checked_sub(other).expect("duration out of range")
    }
}

impl Sum for Duration {
    /// # Panics
    ///
    /// Panics if the total reaches 2⁴⁰ s in magnitude. Partial sums may
    /// exceed it on the way: only the total is checked.
    fn sum<I: Iterator<Item = Duration>>(iter: I) -> Duration {
        // Each term is below 2⁸⁰ units, so i128 holds 2⁴⁷ of them before overflowing.
        let total = iter.fold(0i128, |a, d| {
            a.checked_add(d.0)
                .expect("sum of durations overflowed i128")
        });
        Duration::from_units(total).expect("duration out of range")
    }
}

impl Neg for Duration {
    type Output = Duration;

    /// Always in range: the bound is symmetric.
    fn neg(self) -> Duration {
        Duration(-self.0)
    }
}

impl TryFrom<std::time::Duration> for Duration {
    type Error = Error;

    /// Converts a standard-library duration, exact to the nanosecond.
    fn try_from(d: std::time::Duration) -> Result<Self, Error> {
        Duration::from_nanos(i128::try_from(d.as_nanos()).map_err(|_| Error::OutOfRange)?)
    }
}

impl TryFrom<Duration> for std::time::Duration {
    type Error = Error;

    /// The standard-library duration, to the nearest nanosecond. Fails for
    /// negative durations, which `std::time::Duration` cannot hold.
    fn try_from(d: Duration) -> Result<Self, Error> {
        if d.is_negative() {
            return Err(Error::OutOfRange);
        }
        let nanos = d.as_nanos().unsigned_abs();
        let secs = u64::try_from(nanos / NS as u128).map_err(|_| Error::OutOfRange)?;
        Ok(std::time::Duration::new(secs, (nanos % NS as u128) as u32))
    }
}

impl fmt::Display for Duration {
    /// The canonical form: a sign, no leading zeros, no trailing fraction
    /// zeros, `+0` for zero.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mag = self.0.unsigned_abs();
        let (int, mut frac) = (mag >> FRAC_BITS, mag & FRAC_MASK);
        f.write_char(if self.0 < 0 { '-' } else { '+' })?;
        // Below 2⁴⁰ s, the integer part has at most 8 digits.
        let places = (1..8).take_while(|&k| int >> (5 * k) > 0).count();
        for k in (0..=places).rev() {
            f.write_char(ALPHABET[(int >> (5 * k) & 31) as usize].into())?;
        }
        if frac > 0 {
            f.write_char('.')?;
            while frac > 0 {
                frac <<= 5;
                f.write_char(ALPHABET[(frac >> FRAC_BITS) as usize].into())?;
                frac &= FRAC_MASK;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Duration({self})")
    }
}

impl FromStr for Duration {
    type Err = Error;

    /// Parses `sign 1*8digit ["." 1*8digit]`, with the same digit leniency as
    /// stamps (either case, O for 0, I and L for 1).
    fn from_str(s: &str) -> Result<Self, Error> {
        let (negative, rest) = match s.chars().next() {
            Some('+') => (false, &s[1..]),
            Some('-') => (true, &s[1..]),
            _ => return Err(Error::MissingSign),
        };
        let (int, frac) = match rest.split_once('.') {
            Some((i, f)) => (i, Some(f)),
            None => (rest, None),
        };
        let mut units = digits(int)? << FRAC_BITS;
        if let Some(f) = frac {
            let n = f.chars().count() as u32;
            units += digits(f)? << (FRAC_BITS - 5 * n);
        }
        Duration::from_units(if negative {
            -(units as i128)
        } else {
            units as i128
        })
    }
}

/// 1 to 8 base32 digits as a number.
fn digits(t: &str) -> Result<u128, Error> {
    let mut v = 0u128;
    for (i, c) in t.chars().enumerate() {
        // A character is checked before it is counted (SPEC.md, Text form).
        let d = stamp::digit(c).ok_or(Error::InvalidChar(c))?;
        if i == 8 {
            return Err(Error::TooLong);
        }
        v = v * 32 + u128::from(d);
    }
    if t.is_empty() {
        Err(Error::Empty)
    } else {
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_exact_and_canonical() {
        for s in [
            "+0",
            "+2MC0",
            "-1.G",
            "+0.0000012B",
            "+ZZZZZZZZ.ZZZZZZZZ",
            "-JEM0",
        ] {
            assert_eq!(s.parse::<Duration>().unwrap().to_string(), s);
        }
        assert_eq!(
            Duration::from_nanos(-1_500_000_000).unwrap().to_string(),
            "-1.G"
        );
        assert_eq!((-Duration::ZERO).to_string(), "+0");
        assert_eq!("+o.g".parse::<Duration>().unwrap().to_string(), "+0.G");
    }

    #[test]
    fn errors() {
        assert_eq!("1W".parse::<Duration>(), Err(Error::MissingSign));
        assert_eq!("+".parse::<Duration>(), Err(Error::Empty));
        assert_eq!("+1.".parse::<Duration>(), Err(Error::Empty));
        assert_eq!("+123456789".parse::<Duration>(), Err(Error::TooLong));
        assert_eq!("+é".parse::<Duration>(), Err(Error::InvalidChar('é')));
        assert_eq!("+1.2.3".parse::<Duration>(), Err(Error::InvalidChar('.')));
        assert_eq!(Duration::from_nanos(1 << 100), Err(Error::OutOfRange));
        assert_eq!(Duration::from_units(LIMIT), Err(Error::OutOfRange));
        let max = Duration::from_units(LIMIT - 1).unwrap();
        assert!(max.checked_add(Duration::from_units(1).unwrap()).is_err());
    }

    #[test]
    fn operators() {
        let s = Duration::from_secs(1).unwrap();
        assert_eq!(s + s - s, s);
        assert_eq!((-s).abs(), s);
        assert_eq!(
            [s, s, s].into_iter().sum::<Duration>(),
            Duration::from_secs(3).unwrap()
        );
        assert!(Duration::from_units(i128::MIN).is_err());
        assert!(Duration::from_units(i128::MAX).is_err());
        assert!(std::time::Duration::try_from(-s).is_err());
        let tiny = Duration::from_units(-1).unwrap();
        assert!(
            std::time::Duration::try_from(tiny).is_err(),
            "negative, however small"
        );
        assert_eq!(std::time::Duration::try_from(s).unwrap().as_secs(), 1);
        let max = Duration::from_units(LIMIT - 1).unwrap();
        assert!(std::panic::catch_unwind(|| max + s).is_err());
        assert_eq!(
            [max, max, -max].into_iter().sum::<Duration>(),
            max,
            "only the total is checked"
        );
    }

    #[test]
    fn std_and_nanos() {
        let d = Duration::try_from(std::time::Duration::from_millis(1500)).unwrap();
        assert_eq!(d.as_nanos(), 1_500_000_000);
        // A nanosecond is not exact in 2⁻⁴⁰ s, yet every count reads back.
        for n in (-3_000..3_000).chain([i64::MAX as i128, -(i64::MAX as i128)]) {
            assert_eq!(Duration::from_nanos(n).unwrap().as_nanos(), n, "{n} ns");
        }
        let seven = std::time::Duration::from_nanos(7);
        let back = std::time::Duration::try_from(Duration::try_from(seven).unwrap()).unwrap();
        assert_eq!(back, seven);
        let long = Duration::from_secs(1 << 39).unwrap(); // beyond u64 nanoseconds
        assert_eq!(
            std::time::Duration::try_from(long).unwrap().as_secs(),
            1 << 39
        );
    }
}
