//! The Rust example from the site's Docs page, kept compiling and passing.
use t32::{unix_ns_to_fx, utc_to_fx, Duration, Error, Stamp};

#[test]
fn docs_example() -> Result<(), Error> {
    let s = Stamp::from_fx(utc_to_fx(2026, 10, 31, 3, 13, 0, 0)?, 8)?;
    assert_eq!(s.to_string(), "G0S7G6B1");
    assert!(Stamp::parse("G0S7")?.contains(&s));
    assert_eq!(
        Duration::between(&Stamp::parse("G0S7")?, &s).to_string(),
        "+G6B1"
    );

    let now = Stamp::from_fx(unix_ns_to_fx(1_793_416_380_000_000_000)?, 8)?; // from a Unix time
    assert_eq!(now, s);
    Ok(())
}
