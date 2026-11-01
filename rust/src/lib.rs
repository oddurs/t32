#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc(test(attr(deny(warnings))))]

mod civil;
mod duration;
mod error;
mod instant;
#[cfg(feature = "serde")]
pub mod serde;
mod stamp;
mod utc;

pub use duration::Duration;
pub use error::Error;
pub use instant::Instant;
pub use stamp::Stamp;
pub use utc::UtcDateTime;

// The site's Guide is compiled as doctests, so its Rust example cannot drift
// from the crate. Only under rustdoc: the file is outside the published package.
#[cfg(doctest)]
#[doc = include_str!("../../web/src/content/docs.md")]
struct SiteGuide;

/// The T32 digits, Crockford base32: `0-9` and `A-Z` without I, L, O and U.
/// They are in ASCII order, so text order is numeric order.
pub const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
