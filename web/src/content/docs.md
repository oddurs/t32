# Guide

## Quick start

A T32 stamp is a time written as base32 digits. Eight digits is one second; each extra digit is 32 times finer, each digit removed 32 times coarser.

```
G0S7G6B1        2026-10-31 03:13:00 UTC, to the second
G0S7G6B1G0      … to about a millisecond (03:13:00.500)
G0S7G6          the 17-minute span 03:07:07 – 03:24:11 that contains it
```

To try one, open the [playground](/playground/). To implement one, read the [specification](/spec/); it is short, and the test vectors at the end are the definition of correct.

## Reading a stamp

- **The first character is the era.** `F` is 911 to 2000, `G` is 2000 to 3089. Almost everything you will see starts with `G`.
- **The second character is the generation** (about 34 years). `G0` runs from 2000 to 2034.
- **The third is the year-scale bucket** (about 388 days), and so on down. The [tables](/tables/) list the prefixes for recent years.
- **Length tells you resolution.** `G0S7` means "some time in these 12 days", not "the instant at the start of them".

## Choosing a length

| Use | Length | Resolution |
| --- | --- | --- |
| Human-facing dates, file names, URLs | 8 | 1 s |
| Logs, application events | 10 | ≈ 1 ms |
| Databases, distributed tracing | 12 | ≈ 1 µs |
| Hardware and network timestamps | 14 | ≈ 1 ns |

Never use more characters than your source resolves: a millisecond clock gives at most 10, a nanosecond clock at most 14. Pick one length per field and keep it fixed. Mixed lengths still sort correctly, but fixed width makes equality and indexing simple.

## Storing

As text, compare bytes, not locale rules. For a millisecond source, 10 characters:

```sql
CREATE TABLE events (
  t32  text COLLATE "C" NOT NULL CHECK (t32 ~ '^[0-9A-HJKMNP-TV-Z]{10}$'),
  body jsonb
);
CREATE INDEX ON events (t32);
```

As a number, store the stamp's value in a 64-bit integer and declare its length, for example **T32-10** for milliseconds or **T32-12** for microseconds. Integer order is time order, and it decodes without a lookup table.

## Querying ranges

A prefix is a range. Everything in a 9-hour window:

```sql
SELECT * FROM events WHERE t32 >= 'G0S7G' AND t32 < 'G0S7H';
```

The upper bound is the prefix with its last digit incremented, carrying left. Bucketing for a histogram is a substring:

```sql
SELECT left(t32, 6) AS bucket, count(*)   -- 17-minute buckets
FROM events GROUP BY 1 ORDER BY 1;
```

## Using the TypeScript implementation

[`t32.ts`](https://github.com/oddurs/t32/blob/main/web/src/lib/t32.ts) is a single file with no dependencies. Copy it into your project.

```ts
import { fromUnixMs, format, parse, startFx, endFx, fxToUtcString, durationBetween, formatDuration } from './t32';

format(fromUnixMs(Date.now()));       // 'G0S7G6B1'    (8 characters, 1 s)
format(fromUnixMs(Date.now(), 10));   // 'G0S7G6B1G0'  (≈ 1 ms, the most a 1 ms clock may claim)

const s = parse('g0s7-g6bi');         // case, hyphens between digits, I → 1
format(s);                            // 'G0S7G6B1'
fxToUtcString(startFx(s));            // '2026-10-31T03:13:00Z'
fxToUtcString(endFx(s));              // '2026-10-31T03:13:01Z'

formatDuration(durationBetween(parse('G0S7'), s));   // '+G6B1' (exact)
```

`Date.now()` is Unix time, so `fromUnixMs` applies the leap-second table for you. `parse` is strict about everything else: trim whitespace before calling it.

## Using the Rust crate

The [`t32` crate](https://crates.io/crates/t32) has no required dependencies and no `unsafe`, and builds on Rust 1.70 and later. Add `features = ["serde"]` to serialize its types: text in JSON and other human-readable formats, exact integers in compact ones.

```toml
[dependencies]
t32 = "0.2"
```

```rust
use t32::{Duration, Instant, Stamp, UtcDateTime};

let utc: UtcDateTime = "2026-10-31T03:13:00Z".parse()?;
let stamp = Stamp::new(Instant::from_utc(utc)?, 8)?;           // "G0S7G6B1"

let fortnight: Stamp = "G0S7".parse()?;
assert!(fortnight.contains(stamp) && fortnight < stamp);       // Ord is time order

let gap: Duration = stamp.start() - fortnight.start();          // exact
assert_eq!(gap.to_string(), "+G6B1");

let now = Stamp::now()?;                                        // from the system clock
let fine = Instant::now()?.stamp(14)?;                          // a nanosecond clock allows 14
```

Four types carry everything: `Instant` (a point in time, exact to 2⁻⁴⁰ s), `Stamp`, `Duration` and `UtcDateTime` (RFC 3339 text, leap seconds included). `cargo run --example t32 -- G0S7` prints a stamp's interval.

## Getting TAI from a system clock

Most clocks give Unix time, which follows UTC. Convert with the TAI − UTC table, which both implementations include. A Unix value never names a leap second: the second that repeats is read as 00:00:00 of the next day.

On Linux, `CLOCK_TAI` reads TAI directly (subtract 946,684,800 s for seconds since the T32 epoch), but only if something has told the kernel the current offset. chrony (`leapsectz right/UTC`) and ntpd (a `leapfile`) do. Otherwise the offset is 0 and `CLOCK_TAI` silently equals `CLOCK_REALTIME`, 37 s wrong. Check that `adjtimex()` reports `tai` equal to 37 before relying on it; an unset offset is 0, and a stale one is wrong by whole seconds.

Around a past leap second, clocks that smeared it (some cloud providers did, over many hours) or repeated a second can be up to 1 s off. Undo a known smear before encoding; otherwise treat stamps from that window as approximate. With UTC continuous from 2027-05-20, this only matters for historical data.

## Converting for display

People read dates in their own calendar and zone. Decode the stamp to an interval, convert its start (or midpoint) to UTC, then let the platform format local time. Show the resolution honestly: a length-6 stamp is "about 3:15 pm", not "3:07:07 pm".

## FAQ

### Can I add a random suffix for uniqueness?

Not with alphabet characters: they would read as more resolution. Keep randomness in a separate field, or after a character outside the alphabet, such as `G0S7G6B1_k3j9`.

### Is a T32 stamp a hash?

No. It is an exact, reversible encoding of a number of seconds. Nothing is lost except resolution you chose to drop.

### Why not ISO 8601?

ISO 8601 is readable, but it has many valid spellings of one instant, sorts correctly only when every value uses the same format and zone, and cannot say how precise a value is. It is also 20 characters for one second.

### Why not Unix time?

Unix time pretends leap seconds do not exist, so one second repeats each time one is inserted, and intervals that span it are a second short. It also has no notion of precision, and its text form sorts correctly only when zero-padded.

### Why TAI instead of UTC?

TAI is a plain count of SI seconds. UTC is TAI minus an offset that changes when a leap second is announced, which is why software that does arithmetic on UTC has had outages. GPS and PTP already use TAI-based time for the same reason. The cost is a small conversion table at the edge: 28 whole-second rows, plus 13 formula rows for 1961–1971.

### Why not TAI64?

TAI64 got the important part right: TAI with an offset-binary bias, which T32 shares. But its text form is 17 hexadecimal characters at a precision fixed by the variant (TAI64, TAI64N, TAI64NA). T32 is 8 characters at one second, and its length states its resolution, so one format covers eras to picoseconds and every prefix is a meaningful range.

### Why not ULID or UUIDv7?

They are identifiers: a millisecond Unix timestamp plus randomness. T32 is a time. It uses TAI rather than Unix time, has no fixed resolution, and makes no promise of uniqueness. Use a ULID when you need a key, and T32 when you need to say when.

### Why base32 and not base62 or base64?

Base32 is case-insensitive, so it survives file systems, URLs and people reading it aloud; the Crockford alphabet drops the confusable letters. Its characters are in ASCII order, so text order equals numeric order. And 5 bits per character gives clean steps of 32.

### Why does every stamp start with `G`?

The first character covers about 1,089 years, so everything from 2000 to 3089 shares it. That is the price of a fixed width; writing "2026" repeats "20" for the same reason.

### Why not start at J2000?

J2000 is 2000-01-01T12:00:00 in Terrestrial Time, an astronomical convention that is 32.184 s ahead of TAI and starts at noon. A T32 epoch at midnight TAI makes the era boundary exactly the start of 2000, which is easier to explain and changes nothing else.

### Why aren't the buckets days and years?

A day is not a fixed number of 32nds of anything. Calendar-aligned buckets would need a mixed radix, which breaks the property that a prefix is a range of exactly one-thirty-second the size. Calendars are for display.

### What about time zones?

A stamp names one instant everywhere. Zones apply only when converting for display.

### Will my stamps change if leap seconds change?

No stamp ever changes meaning. What could change is the conversion of a future UTC label, if the leap-second table changed. In October 2026 the CGPM made UTC continuous from 2027-05-20, and no leap second was announced for the end of 2026, so TAI − UTC is expected to stay 37 s. Converters still report a conversion past their table's expiry date as provisional.

### What about dates before 15,420 BCE?

They are out of range. Astronomy uses Julian Dates for deep time, and so should you.
