T32 writes any moment as 8 base32 characters: a count of atomic (TAI) seconds
since the start of 2000, offset so the past encodes too. A stamp's length is
its resolution, every stamp names an interval, text order is time order, and a
prefix is a range query.

This crate is the reference implementation of the
[T32 specification](https://github.com/oddurs/t32/blob/main/SPEC.md) (draft 0.2).
It has no dependencies, uses no `unsafe`, and does all of its arithmetic exactly
in integers, rounding once, where the spec says to.

```rust
use t32::{Instant, Stamp, UtcDateTime};

fn main() -> Result<(), t32::Error> {
    // Encode: a UTC time becomes an instant, and an instant a stamp of any length.
    let utc: UtcDateTime = "2026-10-31T03:13:00Z".parse()?;
    let instant = Instant::from_utc(utc)?;
    let stamp = Stamp::new(instant, 8)?;
    assert_eq!(stamp.to_string(), "G0S7G6B1");

    // Length is resolution: a prefix is the stamp that contains it,
    // and sorting stamps sorts them in time.
    let fortnight: Stamp = "G0S7".parse()?;
    assert!(fortnight.contains(stamp));
    assert!(fortnight < stamp && stamp < "G0S8".parse()?);

    // Decode: a stamp is an interval of time.
    assert_eq!(stamp.start().to_utc().to_string(), "2026-10-31T03:13:00Z");
    assert_eq!(stamp.width().as_nanos(), 1_000_000_000);

    // TAI has no leap seconds, so elapsed time is exact across them.
    let before = Instant::from_utc("2016-12-31T23:59:59Z".parse()?)?;
    let after = Instant::from_utc("2017-01-01T00:00:00Z".parse()?)?;
    assert_eq!((after - before).as_nanos(), 2_000_000_000);
    Ok(())
}
```

The types:

| Type | What it is |
| --- | --- |
| [`Instant`](https://docs.rs/t32/latest/t32/struct.Instant.html) | A point on the T32 time scale, exact to 2⁻⁴⁰ s. Converts to and from UTC, Unix time, TAI and `SystemTime`. |
| [`Stamp`](https://docs.rs/t32/latest/t32/struct.Stamp.html) | 1 to 16 base32 digits naming an interval. Parses, prints, orders and nests. |
| [`Duration`](https://docs.rs/t32/latest/t32/struct.Duration.html) | A signed span of SI seconds, exact to 2⁻⁴⁰ s, with an exact text form (`+2MC0` is a day). |
| [`UtcDateTime`](https://docs.rs/t32/latest/t32/struct.UtcDateTime.html) | A UTC label that actually occurred, leap seconds included, in RFC 3339 text. |
| [`Error`](https://docs.rs/t32/latest/t32/enum.Error.html) | The one error type. |

Converting between UTC and TAI uses the full TAI − UTC history from 1961,
including the drifting "rubber seconds" of 1961–1971. With UTC continuous from
2027-05-20, the table is expected to be final.

Try it on the command line: `cargo run --example t32 -- 2026-10-31T03:13:00.500Z`,
`cargo run --example t32 -- G0S7`, or with no argument for the time now.

Licensed under either of [Apache License, Version 2.0](https://github.com/oddurs/t32/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/oddurs/t32/blob/main/LICENSE-MIT), at your option.
