//! Serde support, behind the `serde` feature.
//!
//! Human-readable formats (JSON, TOML, YAML…) get the T32 text forms, which
//! are exact and sort as time. Compact formats (bincode, postcard…) get exact
//! integers where those are smaller.
//!
//! | Type | Human-readable | Compact |
//! | --- | --- | --- |
//! | [`Stamp`] | canonical text, `"G0S7G6B1"` | the same string |
//! | [`Instant`] | the exact 16-character stamp, `"G0S7G6B100000000"` | `u128` units of 2⁻⁴⁰ s |
//! | [`Duration`] | canonical text, `"+2MC0"` | `i128` units of 2⁻⁴⁰ s |
//! | [`UtcDateTime`] | RFC 3339, `"2016-12-31T23:59:60Z"` | the same string |
//!
//! Deserializing validates exactly as parsing and the constructors do, so a
//! value that arrives is always a valid one.
//!
//! ```
//! use t32::{Duration, Instant, Stamp};
//!
//! #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
//! struct Event {
//!     logged: Stamp,
//!     at: Instant,
//!     took: Duration,
//! }
//!
//! let json = r#"{"logged":"G0S7G6B13XYE","at":"G0S7G6B13XYEW4F0","took":"+0.G"}"#;
//! let event: Event = serde_json::from_str(json)?;
//! assert!(event.logged.contains_instant(event.at));
//! assert_eq!(event.took.as_nanos(), 500_000_000);
//! assert_eq!(serde_json::to_string(&event)?, json);
//! # Ok::<(), serde_json::Error>(())
//! ```

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::{Duration, Error, Instant, Stamp, UtcDateTime};

/// Visits a string and parses it with `FromStr`.
struct TextVisitor<T>(&'static str, PhantomData<T>);

impl<T: FromStr<Err = Error>> Visitor<'_> for TextVisitor<T> {
    type Value = T;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<T, E> {
        s.parse().map_err(E::custom)
    }
}

fn text<'de, D: Deserializer<'de>, T: FromStr<Err = Error>>(
    d: D,
    expecting: &'static str,
) -> Result<T, D::Error> {
    d.deserialize_str(TextVisitor(expecting, PhantomData))
}

// ---------------------------------------------------------------- Stamp

impl Serialize for Stamp {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Stamp {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        text(d, "a T32 stamp such as \"G0S7G6B1\"")
    }
}

// ---------------------------------------------------------------- UtcDateTime

impl Serialize for UtcDateTime {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for UtcDateTime {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        text(
            d,
            "an RFC 3339 UTC date-time such as \"2026-10-31T03:13:00Z\"",
        )
    }
}

// ---------------------------------------------------------------- Duration

impl Serialize for Duration {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            s.collect_str(self)
        } else {
            s.serialize_i128(self.units())
        }
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if d.is_human_readable() {
            text(d, "a T32 duration such as \"+2MC0\"")
        } else {
            Duration::from_units(i128::deserialize(d)?).map_err(de::Error::custom)
        }
    }
}

// ---------------------------------------------------------------- Instant

/// An instant's exact text: its stamp at full length.
struct ExactStamp(Instant);

impl FromStr for ExactStamp {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let stamp: Stamp = s.parse()?;
        if stamp.len() != Stamp::MAX_LEN {
            // A shorter stamp is an interval, not an instant.
            return Err(Error::InvalidLength(stamp.len()));
        }
        Ok(ExactStamp(stamp.start()))
    }
}

impl Serialize for Instant {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            let exact = Stamp::new(*self, Stamp::MAX_LEN).map_err(serde::ser::Error::custom)?;
            s.collect_str(&exact)
        } else {
            s.serialize_u128(self.units())
        }
    }
}

impl<'de> Deserialize<'de> for Instant {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if d.is_human_readable() {
            let ExactStamp(t) = text(d, "a 16-character T32 stamp such as \"G0S7G6B100000000\"")?;
            Ok(t)
        } else {
            Instant::from_units(u128::deserialize(d)?).map_err(de::Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Configure, Token};

    #[test]
    fn human_readable_forms() {
        let stamp: Stamp = "G0S7G6B1".parse().unwrap();
        assert_tokens(&stamp.readable(), &[Token::Str("G0S7G6B1")]);
        let day = Duration::from_secs(86_400).unwrap();
        assert_tokens(&day.readable(), &[Token::Str("+2MC0")]);
        let leap: UtcDateTime = "2016-12-31T23:59:60Z".parse().unwrap();
        assert_tokens(&leap.readable(), &[Token::Str("2016-12-31T23:59:60Z")]);
        assert_tokens(
            &Instant::EPOCH.readable(),
            &[Token::Str("G000000000000000")],
        );
    }

    #[test]
    fn compact_forms() {
        // postcard is a compact (not human-readable) format.
        let stamp: Stamp = "G0S7G6B1".parse().unwrap();
        assert_tokens(&stamp.compact(), &[Token::Str("G0S7G6B1")]);
        let day = Duration::from_secs(-86_400).unwrap();
        let t = Instant::EPOCH;
        let bytes = postcard::to_stdvec(&(day, t, stamp)).unwrap();
        let back: (Duration, Instant, Stamp) = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back, (day, t, stamp));
        // Integers, not text: the instant costs 12 bytes as a varint, not 18 as a string.
        assert!(bytes.len() < 40, "{}", bytes.len());
        let bad = postcard::to_stdvec(&(1u128 << 80)).unwrap();
        assert!(postcard::from_bytes::<Instant>(&bad).is_err());
    }

    #[test]
    fn json_round_trips_and_validates() {
        let t = Instant::from_utc("2016-12-31T23:59:60.25Z".parse().unwrap()).unwrap();
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(serde_json::from_str::<Instant>(&json).unwrap(), t);
        let bad = |s: &str| serde_json::from_str::<Instant>(s).unwrap_err().to_string();
        assert!(bad("\"G0S7\"").contains("invalid stamp length 4"));
        assert!(serde_json::from_str::<UtcDateTime>("\"2016-12-30T23:59:60Z\"").is_err());
        assert!(serde_json::from_str::<Stamp>("\"G0SU\"").is_err());
        assert!(serde_json::from_str::<Duration>("\"1W\"").is_err());
        assert!(serde_json::from_str::<Stamp>("42").is_err());
    }
}
