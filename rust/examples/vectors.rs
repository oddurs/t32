//! Prints encoding vectors and range ends: `cargo run --example vectors`.
use t32::*;

fn utc(fx: Fx) -> String {
    let u = fx_to_utc(fx);
    let s = u.unix_ns.div_euclid(1_000_000_000) as i64;
    let (y, m, d) = civil_from_days(s.div_euclid(86_400));
    let r = s.rem_euclid(86_400);
    let tag = if u.inserted_ns.is_some() {
        " (inserted)"
    } else {
        ""
    };
    format!(
        "{y:05}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z{tag}",
        r / 3600,
        r % 3600 / 60,
        r % 60
    )
}

fn main() -> Result<(), Error> {
    type Fields = (i64, u32, u32, u32, u32, u32, u32);
    let cases: &[(&str, Fields, u8)] = &[
        ("T32 epoch", (1999, 12, 31, 23, 59, 28, 0), 8),
        ("Unix epoch", (1970, 1, 1, 0, 0, 0, 0), 8),
        ("Apollo 11 landing", (1969, 7, 20, 20, 17, 40, 0), 8),
        ("Last leap second", (2016, 12, 31, 23, 59, 60, 0), 8),
        ("Spec date", (2026, 10, 31, 3, 13, 0, 0), 8),
    ];
    for (name, (y, mo, d, h, mi, s, ns), n) in cases {
        let st = Stamp::from_fx(utc_to_fx(*y, *mo, *d, *h, *mi, *s, *ns)?, *n)?;
        println!("{name:<20} {st}");
    }
    for s in ["00000000", "ZZZZZZZZ", "G", "G0S7"] {
        let st = Stamp::parse(s)?;
        println!("{s:<20} {} .. {}", utc(st.start()), utc(st.end()));
    }
    Ok(())
}
