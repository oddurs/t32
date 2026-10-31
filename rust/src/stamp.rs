use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use crate::instant::{FRAC_BITS, RANGE};
use crate::{Duration, Error, Instant, ALPHABET};

/// The value of an input character: a digit of [`ALPHABET`] in either case, or
/// one of the aliases O (0), I and L (1). ASCII only.
pub(crate) fn digit(c: char) -> Option<u8> {
    if !c.is_ascii() {
        return None;
    }
    match c.to_ascii_uppercase() {
        'O' => Some(0),
        'I' | 'L' => Some(1),
        u => ALPHABET.iter().position(|&a| a == u as u8).map(|p| p as u8),
    }
}

/// A T32 stamp: an interval of time named by 1 to 16 base32 digits.
///
/// The stamp of length n is the top 5·n bits of an [`Instant`], so its
/// interval is 32⁸⁻ⁿ seconds wide: one second at length 8, 32 seconds at 7,
/// 1/32 s at 9. Shortening a stamp gives the one that contains it.
///
/// Stamps order by the start of their interval, and a containing stamp sorts
/// immediately before everything it contains. That is exactly the byte order
/// of their text, so sorted strings are in time order.
///
/// ```
/// use t32::Stamp;
///
/// let mut stamps: Vec<Stamp> = ["G0S7G6B1", "G0S8", "G0S7", "FZ3BMMKB"]
///     .iter()
///     .map(|s| s.parse())
///     .collect::<Result<_, _>>()?;
/// stamps.sort();
/// assert_eq!(stamps.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
///            ["FZ3BMMKB", "G0S7", "G0S7G6B1", "G0S8"]);
///
/// let fortnight = stamps[1];
/// assert!(fortnight.contains(stamps[2]));
/// # Ok::<(), t32::Error>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stamp {
    value: u128,
    len: u8,
}

impl Stamp {
    /// The canonical length: 8 characters, one second.
    pub const CANONICAL_LEN: u8 = 8;
    /// The longest stamp: 16 characters, 2⁻⁴⁰ s (about 0.9 ps).
    pub const MAX_LEN: u8 = 16;

    /// The stamp of `len` characters (1 to 16) that contains `instant`.
    ///
    /// Use no more characters than the source resolves; [`Stamp::max_len`]
    /// gives the limit.
    ///
    /// ```
    /// use t32::{Instant, Stamp};
    ///
    /// let t = Instant::from_utc("2026-10-31T03:13:00.500Z".parse()?)?;
    /// assert_eq!(Stamp::new(t, 8)?.to_string(), "G0S7G6B1");
    /// assert_eq!(Stamp::new(t, 10)?.to_string(), "G0S7G6B1G0");
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidLength`] unless `len` is 1 to 16.
    pub const fn new(instant: Instant, len: u8) -> Result<Self, Error> {
        if len == 0 || len > Self::MAX_LEN {
            return Err(Error::InvalidLength(len));
        }
        Ok(Stamp {
            value: instant.units() >> shift_for(len),
            len,
        })
    }

    /// The canonical (one-second) stamp for the current time.
    ///
    /// # Errors
    ///
    /// As [`Instant::now`].
    pub fn now() -> Result<Self, Error> {
        Stamp::new(Instant::now()?, Self::CANONICAL_LEN)
    }

    /// The longest length a source of this resolution may use: the length
    /// whose unit is nearest the resolution on a logarithmic scale, ties going
    /// to the shorter (SPEC.md, Length and resolution).
    ///
    /// ```
    /// use t32::{Duration, Stamp};
    ///
    /// assert_eq!(Stamp::max_len(Duration::from_secs(1)?), 8);
    /// assert_eq!(Stamp::max_len(Duration::from_nanos(1_000_000)?), 10);   // 1 ms
    /// assert_eq!(Stamp::max_len(Duration::from_nanos(1)?), 14);           // 1 ns
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn max_len(resolution: Duration) -> u8 {
        let units = resolution.units();
        if units <= 0 {
            return Self::MAX_LEN;
        }
        // The unit of length n is 2^(5·(16 − n)) units. Exact ties cannot occur:
        // they would need a whole number of units equal to 2^(5k − 2.5).
        let log32 = (units as f64).log2() / 5.0;
        let n = f64::from(Self::MAX_LEN) - (log32 - 0.5).ceil();
        n.clamp(1.0, f64::from(Self::MAX_LEN)) as u8
    }

    /// The stamp with value V of `len` characters: the binary form, as a field
    /// of type T32-n would hold it.
    ///
    /// ```
    /// use t32::Stamp;
    ///
    /// let s: Stamp = "G0S7G6B1".parse()?;
    /// assert_eq!(Stamp::from_value(s.value(), 8)?, s);
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidLength`] unless `len` is 1 to 16, and
    /// [`Error::OutOfRange`] unless `value < 32^len`.
    pub const fn from_value(value: u128, len: u8) -> Result<Self, Error> {
        if len == 0 || len > Self::MAX_LEN {
            return Err(Error::InvalidLength(len));
        }
        if value >> (5 * len as u32) != 0 {
            return Err(Error::OutOfRange);
        }
        Ok(Stamp { value, len })
    }

    /// Parses a stamp: the same as `s.parse::<Stamp>()`.
    ///
    /// Input is ASCII and case-insensitive; O reads as 0 and I or L as 1; a
    /// hyphen may stand between two digits. Anything else, whitespace included,
    /// is an error.
    ///
    /// ```
    /// use t32::{Error, Stamp};
    ///
    /// assert_eq!(Stamp::parse("g0s7-g6bi")?.to_string(), "G0S7G6B1");
    /// assert_eq!(Stamp::parse("-1W"), Err(Error::MisplacedHyphen));
    /// assert_eq!(Stamp::parse(" G0S7"), Err(Error::InvalidChar(' ')));
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Empty`], [`Error::TooLong`], [`Error::InvalidChar`] or
    /// [`Error::MisplacedHyphen`] for text that is not a stamp.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let (mut value, mut len, mut after_hyphen) = (0u128, 0u8, false);
        for c in s.chars() {
            if c == '-' {
                if len == 0 || after_hyphen {
                    return Err(Error::MisplacedHyphen);
                }
                after_hyphen = true;
                continue;
            }
            let d = digit(c).ok_or(Error::InvalidChar(c))?;
            after_hyphen = false;
            len += 1;
            if len > Self::MAX_LEN {
                return Err(Error::TooLong);
            }
            value = value * 32 + u128::from(d);
        }
        match (len, after_hyphen) {
            (0, _) => Err(Error::Empty),
            (_, true) => Err(Error::MisplacedHyphen),
            _ => Ok(Stamp { value, len }),
        }
    }

    /// The number of characters, 1 to 16.
    #[allow(clippy::len_without_is_empty)] // a stamp is never empty
    #[must_use]
    pub const fn len(self) -> u8 {
        self.len
    }

    /// The value V: the digits read as a base-32 number.
    #[must_use]
    pub const fn value(self) -> u128 {
        self.value
    }

    /// The containing stamp of `len` characters: the first `len` digits.
    ///
    /// ```
    /// use t32::Stamp;
    ///
    /// let s: Stamp = "G0S7G6B1".parse()?;
    /// assert_eq!(s.prefix(4)?.to_string(), "G0S7");
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidLength`] if `len` is 0 or longer than this stamp.
    pub const fn prefix(self, len: u8) -> Result<Self, Error> {
        if len == 0 || len > self.len {
            return Err(Error::InvalidLength(len));
        }
        Ok(Stamp {
            value: self.value >> (5 * (self.len - len) as u32),
            len,
        })
    }

    /// The first instant of the interval.
    #[must_use]
    pub const fn start(self) -> Instant {
        Instant(self.value << shift_for(self.len))
    }

    /// The instant just after the interval, or `None` for a stamp of all Z,
    /// whose interval runs to the end of the representable range.
    ///
    /// ```
    /// use t32::Stamp;
    ///
    /// let s: Stamp = "G0S7G6B1".parse()?;
    /// assert_eq!(s.start().to_utc().to_string(), "2026-10-31T03:13:00Z");
    /// assert_eq!(s.end().unwrap().to_utc().to_string(), "2026-10-31T03:13:01Z");
    /// assert_eq!("ZZZZ".parse::<Stamp>()?.end(), None);
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn end(self) -> Option<Instant> {
        Instant::from_units((self.value + 1) << shift_for(self.len)).ok()
    }

    /// The middle of the interval: the best single estimate of the instant.
    #[must_use]
    pub const fn midpoint(self) -> Instant {
        Instant(self.start().0 + ((1u128 << shift_for(self.len)) >> 1))
    }

    /// The width of the interval, 32⁸⁻ⁿ seconds.
    ///
    /// ```
    /// use t32::Stamp;
    ///
    /// assert_eq!("G0S7G6B1".parse::<Stamp>()?.width().as_nanos(), 1_000_000_000);
    /// assert_eq!("G0S7G6B".parse::<Stamp>()?.width().as_nanos(), 32_000_000_000);
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub const fn width(self) -> Duration {
        Duration(1i128 << shift_for(self.len))
    }

    /// Whether `other` lies inside this stamp: whether this stamp is a prefix of it.
    #[must_use]
    pub fn contains(self, other: Stamp) -> bool {
        other.prefix(self.len) == Ok(self)
    }

    /// Whether `instant` lies inside this stamp's interval.
    #[must_use]
    pub const fn contains_instant(self, instant: Instant) -> bool {
        instant.units() >> shift_for(self.len) == self.value
    }

    /// The next stamp of the same length, or `None` after the last. Its text
    /// is the exclusive upper bound for a prefix range query.
    ///
    /// ```
    /// use t32::Stamp;
    ///
    /// // Everything inside G0SZ is >= "G0SZ" and < "G0T0".
    /// assert_eq!("G0SZ".parse::<Stamp>()?.next().unwrap().to_string(), "G0T0");
    /// assert_eq!("ZZ".parse::<Stamp>()?.next(), None);
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn next(self) -> Option<Stamp> {
        Stamp::from_value(self.value + 1, self.len).ok()
    }

    /// The stamp of the same length that contains this one's start moved by
    /// `duration`.
    ///
    /// ```
    /// use t32::Stamp;
    ///
    /// let s: Stamp = "G0S7".parse()?;
    /// assert_eq!(s.checked_add("+10000".parse()?)?.to_string(), "G0S8");
    /// # Ok::<(), t32::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the result leaves the representable range.
    pub fn checked_add(self, duration: Duration) -> Result<Self, Error> {
        Stamp::new(self.start().checked_add(duration)?, self.len)
    }

    /// As [`Stamp::checked_add`], moving back.
    ///
    /// # Errors
    ///
    /// Returns [`Error::OutOfRange`] if the result leaves the representable range.
    pub fn checked_sub(self, duration: Duration) -> Result<Self, Error> {
        self.checked_add(-duration)
    }

    /// The display form: groups of four joined by hyphens, for people to read.
    /// Never store it; store the canonical form from [`Display`](fmt::Display).
    ///
    /// ```
    /// use t32::Stamp;
    ///
    /// assert_eq!("G0S7G6B13XYE".parse::<Stamp>()?.display_form(), "G0S7-G6B1-3XYE");
    /// # Ok::<(), t32::Error>(())
    /// ```
    #[must_use]
    pub fn display_form(self) -> String {
        let mut out = String::with_capacity(19);
        for (i, c) in self.to_string().chars().enumerate() {
            if i > 0 && i % 4 == 0 {
                out.push('-');
            }
            out.push(c);
        }
        out
    }
}

/// Bits below the value in an instant, for a stamp of `len` characters.
const fn shift_for(len: u8) -> u32 {
    5 * (Stamp::MAX_LEN - len) as u32
}

// The spec's bit layout: 16 digits of 5 bits are exactly an Instant's 80 bits.
const _: () = assert!(5 * Stamp::MAX_LEN as u32 == 2 * FRAC_BITS && RANGE == 1 << 80);

impl Ord for Stamp {
    /// By start, then shorter (containing) first: the byte order of the text.
    fn cmp(&self, other: &Self) -> Ordering {
        (self.start(), self.len).cmp(&(other.start(), other.len))
    }
}

impl PartialOrd for Stamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for Stamp {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Stamp::parse(s)
    }
}

impl fmt::Display for Stamp {
    /// The canonical form: uppercase digits, no separators.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = [0u8; Stamp::MAX_LEN as usize];
        let mut v = self.value;
        for slot in out[..self.len as usize].iter_mut().rev() {
            *slot = ALPHABET[(v % 32) as usize];
            v /= 32;
        }
        // Every byte comes from ALPHABET, so this is ASCII.
        f.write_str(std::str::from_utf8(&out[..self.len as usize]).map_err(|_| fmt::Error)?)
    }
}

impl fmt::Debug for Stamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stamp({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_equals_coarse_encoding() {
        let t = Instant::from_utc("2026-10-31T03:13:00.123456789Z".parse().unwrap()).unwrap();
        let full = Stamp::new(t, 16).unwrap();
        for n in 1..=16 {
            assert_eq!(full.prefix(n).unwrap(), Stamp::new(t, n).unwrap());
            assert!(Stamp::new(t, n).unwrap().contains_instant(t));
        }
    }

    #[test]
    fn order_is_byte_order() {
        let mut stamps: Vec<Stamp> = (0..3000i128)
            .map(|i| {
                let t = Instant::from_tai_nanos((i - 1500) * 977_000_000_123).unwrap();
                Stamp::new(t, (i % 16 + 1) as u8).unwrap()
            })
            .collect();
        stamps.sort();
        let text: Vec<String> = stamps.iter().map(Stamp::to_string).collect();
        let mut sorted = text.clone();
        sorted.sort();
        assert_eq!(text, sorted);
    }

    #[test]
    fn parsing_errors() {
        assert_eq!(Stamp::parse(""), Err(Error::Empty));
        assert_eq!(Stamp::parse("1W-"), Err(Error::MisplacedHyphen));
        assert_eq!(Stamp::parse("G0S7--G6B1"), Err(Error::MisplacedHyphen));
        assert_eq!(Stamp::parse("\u{131}"), Err(Error::InvalidChar('\u{131}')));
        assert_eq!(Stamp::parse("\u{17F}"), Err(Error::InvalidChar('\u{17F}')));
        assert_eq!(Stamp::parse("G0S7G6B13XYE00000"), Err(Error::TooLong));
        assert_eq!(Stamp::parse("oooo").unwrap().to_string(), "0000");
    }

    #[test]
    fn edges() {
        let last: Stamp = "ZZZZZZZZZZZZZZZZ".parse().unwrap();
        assert_eq!(last.start(), Instant::MAX);
        assert_eq!(last.end(), None);
        assert_eq!(Stamp::new(Instant::MIN, 8).unwrap().to_string(), "00000000");
        assert_eq!(
            Stamp::new(Instant::EPOCH, 8).unwrap().to_string(),
            "G0000000"
        );
        assert_eq!(Stamp::new(Instant::MIN, 0), Err(Error::InvalidLength(0)));
        assert_eq!(Stamp::new(Instant::MIN, 17), Err(Error::InvalidLength(17)));
        assert_eq!(Stamp::from_value(32, 1), Err(Error::OutOfRange));
        assert_eq!(format!("{:?}", last.prefix(2).unwrap()), "Stamp(ZZ)");
        let z: Stamp = "Z".parse().unwrap();
        assert!(z.checked_add("+1".parse().unwrap()).is_ok());
        assert!(z.checked_add(z.width()).is_err());
        assert!(Stamp::parse("0")
            .unwrap()
            .checked_sub("+1".parse().unwrap())
            .is_err());
    }

    #[test]
    fn max_len_follows_the_nearest_unit() {
        let ns = |n: i128| Stamp::max_len(Duration::from_nanos(n).unwrap());
        // Decimal fraction digits 0..=9 of a second.
        let lens: Vec<u8> = (0..=9).map(|d| ns(10i128.pow(9 - d))).collect();
        assert_eq!(lens, [8, 9, 9, 10, 11, 11, 12, 13, 13, 14]);
        assert_eq!(Stamp::max_len(Duration::from_secs(32).unwrap()), 7);
        assert_eq!(Stamp::max_len(Duration::from_units(1).unwrap()), 16);
        assert_eq!(Stamp::max_len(Duration::ZERO), 16);
        assert_eq!(Stamp::max_len(Duration::from_secs(1 << 39).unwrap()), 1);
    }
}
