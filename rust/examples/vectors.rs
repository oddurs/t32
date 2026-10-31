//! Prints the spec's encoding vectors: `cargo run --example vectors`.
use t32::{Instant, Stamp, UtcDateTime};

fn main() -> Result<(), t32::Error> {
    let cases = [
        ("T32 epoch", "1999-12-31T23:59:28Z", 8),
        ("Unix epoch", "1970-01-01T00:00:00Z", 8),
        ("Apollo 11 landing", "1969-07-20T20:17:40Z", 8),
        ("Last leap second", "2016-12-31T23:59:60Z", 8),
        ("Spec date", "2026-10-31T03:13:00Z", 8),
        (
            "Spec date + 0.123 s (µs clock)",
            "2026-10-31T03:13:00.123000Z",
            12,
        ),
    ];
    for (name, utc, len) in cases {
        let stamp = Stamp::new(Instant::from_utc(utc.parse::<UtcDateTime>()?)?, len)?;
        println!("{name:<20} {utc:<26} {stamp}");
    }
    Ok(())
}
