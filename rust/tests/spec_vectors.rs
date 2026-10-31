//! Every test vector in SPEC.md, through the public API only.

use t32::{Duration, Error, Instant, Stamp, UtcDateTime};

fn utc(s: &str) -> UtcDateTime {
    s.parse().unwrap_or_else(|e| panic!("{s}: {e}"))
}

fn stamp(s: &str) -> Stamp {
    s.parse().unwrap_or_else(|e| panic!("{s}: {e}"))
}

#[test]
fn encoder_and_converter() {
    let vectors = [
        ("1999-12-31T23:59:28Z", 8, "G0000000"),
        ("1970-01-01T00:00:00Z", 8, "FZ3S5F48"),
        ("1969-07-20T20:17:40Z", 8, "FZ3BMMKB"),
        ("1966-06-15T12:00:00.123456789Z", 16, "FZ0EDKA4VPC2A1FX"),
        ("1972-01-01T00:00:00Z", 8, "FZ5NA8WA"),
        ("2016-12-31T23:59:59Z", 8, "G0FZP0S3"),
        ("2016-12-31T23:59:60Z", 8, "G0FZP0S4"),
        ("2017-01-01T00:00:00Z", 8, "G0FZP0S5"),
        ("2026-10-31T03:13:00Z", 8, "G0S7G6B1"),
        ("2026-10-31T03:13:00.500Z", 10, "G0S7G6B1G0"),
        ("2026-10-31T03:13:00.123000Z", 12, "G0S7G6B13XYE"),
        ("1675-08-20T00:00:00Z", 8, "FPEY6N00"),
    ];
    for (time, len, want) in vectors {
        let got = Stamp::new(Instant::from_utc(utc(time)).unwrap(), len).unwrap();
        assert_eq!(got.to_string(), want, "{time}");
    }
    let unix = Instant::from_unix_nanos(1_483_228_800_000_000_000).unwrap();
    assert_eq!(Stamp::new(unix, 8).unwrap().to_string(), "G0FZP0S5");
}

#[test]
fn converter_errors() {
    assert!("1961-07-31T23:59:59.95Z".parse::<UtcDateTime>().is_ok());
    assert_eq!(
        "1961-07-31T23:59:59.950000001Z".parse::<UtcDateTime>(),
        Err(Error::RemovedTime)
    );
    assert!("1968-01-31T23:59:59.900000002Z"
        .parse::<UtcDateTime>()
        .is_ok());
    assert_eq!(
        "1968-01-31T23:59:59.900000003Z".parse::<UtcDateTime>(),
        Err(Error::RemovedTime)
    );
    assert_eq!(
        "2016-12-30T23:59:60Z".parse::<UtcDateTime>(),
        Err(Error::NotALeapSecond)
    );
}

#[test]
fn decoder_in_tai() {
    let vectors = [
        ("00000000", -549_755_813_888, 1),
        ("FZ3BMMKB", -960_867_733, 1),
        ("G0000000", 0, 1),
        ("G0FZP0S4", 536_544_036, 1),
        ("G0S7", 846_200_832, 1_048_576),
    ];
    for (s, start, width) in vectors {
        let st = stamp(s);
        assert_eq!(st.start().tai_nanos(), start * 1_000_000_000, "{s}");
        assert_eq!(st.width(), Duration::from_secs(width).unwrap(), "{s}");
    }
}

#[test]
fn decoder_in_utc() {
    let vectors = [
        (
            "00000000",
            "-15422-12-06T11:41:52Z",
            "-15422-12-06T11:41:53Z",
        ),
        (
            "FZ3BMMKB",
            "1969-07-20T20:17:39.425406217Z",
            "1969-07-20T20:17:40.425406187Z",
        ),
        ("G0FZP0S4", "2016-12-31T23:59:60Z", "2017-01-01T00:00:00Z"),
        ("G0S7", "2026-10-24T23:46:35Z", "2026-11-06T03:02:51Z"),
        ("G0S7G6", "2026-10-31T03:07:07Z", "2026-10-31T03:24:11Z"),
    ];
    for (s, start, end) in vectors {
        let st = stamp(s);
        assert_eq!(st.start().to_utc().to_string(), start, "{s}");
        assert_eq!(st.end().unwrap().to_utc().to_string(), end, "{s}");
    }
    // The spec's G0FZP0S4 start is inserted time; its Unix label is the next midnight.
    assert!(stamp("G0FZP0S4").start().in_inserted_time());
    assert_eq!(
        stamp("G0FZP0S4").start().unix_nanos(),
        1_483_228_800_000_000_000
    );
}

#[test]
fn parser() {
    for (input, want) in [
        ("g0s7-g6bi", "G0S7G6B1"),
        ("oooo", "0000"),
        ("G0S7G6B1-3XYE", "G0S7G6B13XYE"),
    ] {
        assert_eq!(stamp(input).to_string(), want);
    }
    let errors = [
        ("G0SU", Error::InvalidChar('U')),
        (" G0S7", Error::InvalidChar(' ')),
        ("\u{131}", Error::InvalidChar('\u{131}')),
        ("-1W", Error::MisplacedHyphen),
        ("1W-", Error::MisplacedHyphen),
        ("G0S7--G6B1", Error::MisplacedHyphen),
        ("00000000000000000", Error::TooLong),
        ("", Error::Empty),
    ];
    for (input, want) in errors {
        assert_eq!(input.parse::<Stamp>(), Err(want), "{input:?}");
    }
}

#[test]
fn durations() {
    assert_eq!(Duration::from_secs(86_400).unwrap().to_string(), "+2MC0");
    assert_eq!(
        Duration::from_nanos(-1_500_000_000).unwrap().to_string(),
        "-1.G"
    );
    assert_eq!(Duration::from_nanos(-1).unwrap().to_string(), "-0.0000012B");
    assert_eq!(Duration::ZERO.to_string(), "+0");
    assert_eq!(
        (stamp("G0S8").start() - stamp("G0S7").start()).to_string(),
        "+10000"
    );
    assert_eq!(
        (stamp("G0S7G6B1").start() - stamp("G0S7").start()).to_string(),
        "+G6B1"
    );
    assert_eq!("1W".parse::<Duration>(), Err(Error::MissingSign));
}
