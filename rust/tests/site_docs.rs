//! The Rust example on the site's Docs page (web/src/content/docs.md), kept compiling.
use t32::{Duration, Instant, Stamp, UtcDateTime};

#[test]
fn docs_page_example() -> Result<(), t32::Error> {
    let utc: UtcDateTime = "2026-10-31T03:13:00Z".parse()?;
    let stamp = Stamp::new(Instant::from_utc(utc)?, 8)?;
    assert_eq!(stamp.to_string(), "G0S7G6B1");

    let fortnight: Stamp = "G0S7".parse()?;
    assert!(fortnight.contains(stamp) && fortnight < stamp);

    let gap: Duration = stamp.start() - fortnight.start();
    assert_eq!(gap.to_string(), "+G6B1");

    let now = Stamp::now()?;
    let fine = Instant::now()?.stamp(14)?;
    assert!(fine.prefix(8)? >= now);
    Ok(())
}
