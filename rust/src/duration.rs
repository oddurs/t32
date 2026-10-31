use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Neg, Sub};
use std::str::FromStr;

use crate::instant::FRAC_BITS;
use crate::stamp::digit;
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

    /// In nanoseconds, truncated toward zero.
    #[must_use]
    pub const fn as_nanos(self) -> i128 {
        let mag = ((self.0.unsigned_abs() * NS as u128) >> FRAC_BITS) as i128;
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
    /// Panics if the total reaches 2⁴⁰ s in magnitude.
    fn sum<I: Iterator<Item = Duration>>(iter: I) -> Duration {
        iter.fold(Duration::ZERO, |a, b| a + b)
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

    /// The standard-library duration, truncated to the nanosecond. Fails for
    /// negative durations, which `std::time::Duration` cannot hold.
    fn try_from(d: Duration) -> Result<Self, Error> {
        let nanos = u64::try_from(d.as_nanos()).map_err(|_| Error::OutOfRange)?;
        Ok(std::time::Duration::from_nanos(nanos))
    }
}

impl fmt::Display for Duration {
    /// The canonical form: a sign, no leading zeros, no trailing fraction
    /// zeros, `+0` for zero.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mag = self.0.unsigned_abs();
        let (mut int, mut frac) = (mag >> FRAC_BITS, mag & FRAC_MASK);
        let mut digits = Vec::new();
        loop {
            digits.push(ALPHABET[(int % 32) as usize]);
            int /= 32;
            if int == 0 {
                break;
            }
        }
        digits.reverse();
        if frac > 0 {
            digits.push(b'.');
            while frac > 0 {
                frac <<= 5;
                digits.push(ALPHABET[(frac >> FRAC_BITS) as usize]);
                frac &= FRAC_MASK;
            }
        }
        let sign = if self.0 < 0 { '-' } else { '+' };
        write!(f, "{sign}{}", std::str::from_utf8(&digits).unwrap())
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
        if i == 8 {
            return Err(Error::TooLong);
        }
        v = v * 32 + u128::from(digit(c).ok_or(Error::InvalidChar(c))?);
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
        assert_eq!(std::time::Duration::try_from(s).unwrap().as_secs(), 1);
        let max = Duration::from_units(LIMIT - 1).unwrap();
        assert!(std::panic::catch_unwind(|| max + s).is_err());
    }

    #[test]
    fn std_and_nanos() {
        let d = Duration::try_from(std::time::Duration::from_millis(1500)).unwrap();
        assert_eq!(d.as_nanos(), 1_500_000_000);
        assert_eq!(Duration::from_nanos(-7).unwrap().as_nanos(), -6); // 7 ns is not exact in 2⁻⁴⁰ s
    }
}
