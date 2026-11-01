use std::fmt;

/// Everything that can go wrong in T32.
///
/// Every fallible function in this crate returns this one error type, so `?`
/// works across parsing, conversion and arithmetic.
///
/// ```
/// use t32::{Error, Stamp};
///
/// assert_eq!("G0SU".parse::<Stamp>(), Err(Error::InvalidChar('U')));
/// assert_eq!(Error::InvalidChar('U').to_string(), "invalid character 'U' (T32 digits are 0-9 and A-Z except I, L, O, U)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Error {
    /// The text had no digits.
    Empty,
    /// A stamp had more than 16 digits, or a part of a duration more than 8.
    TooLong,
    /// A character that is not a T32 digit (or an accepted alias) appeared.
    InvalidChar(char),
    /// A hyphen appeared somewhere other than between two digits.
    MisplacedHyphen,
    /// A stamp length outside 1 to 16 was asked for, or a prefix longer than
    /// the stamp.
    InvalidLength(u8),
    /// A duration was written without its mandatory `+` or `-`.
    MissingSign,
    /// The value lies outside what T32 can represent: 15,423 BCE to 19,421 CE
    /// for instants, or 2^40 seconds for durations.
    OutOfRange,
    /// A date or time field was out of bounds, or the text was not of the form
    /// `YYYY-MM-DDThh:mm:ss[.fffffffff]Z`.
    InvalidDateTime,
    /// Second 60 was given, but no leap second was inserted at that time.
    NotALeapSecond,
    /// The UTC time never occurred: it fell in a span removed when TAI − UTC
    /// decreased (only on 1961-08-01 and 1968-02-01).
    RemovedTime,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Empty => f.write_str("no digits"),
            Error::TooLong => f.write_str(
                "too many digits (at most 16 in a stamp, 8 in each part of a duration)",
            ),
            Error::InvalidChar(c) => write!(
                f,
                "invalid character {c:?} (T32 digits are 0-9 and A-Z except I, L, O, U)"
            ),
            Error::MisplacedHyphen => {
                f.write_str("a hyphen may only stand between two digits, as in G0S7-G6B1")
            }
            Error::InvalidLength(n) => write!(
                f,
                "invalid stamp length {n} (must be 1 to 16, and no longer than the stamp it shortens)"
            ),
            Error::MissingSign => f.write_str("a duration must start with + or -"),
            Error::OutOfRange => f.write_str("outside the range T32 can represent"),
            Error::InvalidDateTime => {
                f.write_str("invalid date-time (expected YYYY-MM-DDThh:mm:ss[.fffffffff]Z)")
            }
            Error::NotALeapSecond => f.write_str("no leap second was inserted at this time"),
            Error::RemovedTime => {
                f.write_str("this UTC time never occurred: it was removed when TAI - UTC fell")
            }
        }
    }
}

impl std::error::Error for Error {}
