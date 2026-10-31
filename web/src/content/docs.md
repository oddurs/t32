# Docs

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
- **Length tells you precision.** `G0S7` means "some time in these 12 days", not "the instant at the start of them".

## Choosing a length

| Use | Length | Resolution |
| --- | --- | --- |
| Human-facing dates, file names, URLs | 8 | 1 s |
| Logs, application events | 10 | ≈ 1 ms |
| Databases, distributed tracing | 12 | ≈ 1 µs |
| Hardware and network timestamps | 14 | ≈ 1 ns |

Pick one length per field and keep it fixed. Mixed lengths still sort correctly, but fixed width makes equality and indexing simple.

## Storing

As text, compare bytes, not locale rules:

```sql
CREATE TABLE events (
  t32  text COLLATE "C" NOT NULL CHECK (t32 ~ '^[0-9A-HJKMNP-TV-Z]{12}$'),
  body jsonb
);
CREATE INDEX ON events (t32);
```

As a number, use the **T32/64** form: the value of a 12-character stamp in a 64-bit integer. Integer order is time order, and it decodes without a lookup table.

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
import { fromUnixMs, format, parse, startFx, endFx, fxToUtcString } from './t32';

format(fromUnixMs(Date.now()));       // 'G0S7G6B1'  (8 characters, 1 s)
format(fromUnixMs(Date.now(), 12));   // 'G0S7G6B13XYE'  (≈ 1 µs)

const s = parse('g0s7-g6bi');         // lenient: case, hyphens, I→1
format(s);                            // 'G0S7G6B1'
fxToUtcString(startFx(s));            // '2026-10-31T03:13:00Z'
fxToUtcString(endFx(s));              // '2026-10-31T03:13:01Z'
```

`Date.now()` is Unix time, so `fromUnixMs` applies the leap-second table for you.

## Using the Rust crate

The [reference crate](https://github.com/oddurs/t32/tree/main/rust) uses only the standard library. Until it is published, depend on it from Git:

```toml
[dependencies]
t32 = { git = "https://github.com/oddurs/t32" }
```

```rust
use t32::{utc_to_fx, Stamp};

let fx = utc_to_fx(2026, 10, 31, 3, 13, 0, 0)?;
let s = Stamp::from_biased_fx(fx, 8)?;
assert_eq!(s.to_string(), "G0S7G6B1");
assert!(Stamp::parse("G0S7")?.contains(&s));
```

## Getting TAI from a system clock

Most clocks give Unix time, which follows UTC. Convert with the TAI − UTC table, which both implementations include.

On Linux, `CLOCK_TAI` reads TAI directly, but only if something has told the kernel the current offset. chrony (`leapsectz right/UTC`) and ntpd (a `leapfile`) do; otherwise the kernel offset is 0 and `CLOCK_TAI` silently equals `CLOCK_REALTIME`. Check before relying on it.

Cloud time services that smear leap seconds are accurate to length 7 (32 s) near a leap second, and to length 8 at all other times.

## Converting for display

People read dates in their own calendar and zone. Decode the stamp to an interval, convert its start (or midpoint) to UTC, then let the platform format local time. Show the precision honestly: a length-6 stamp is "about 3:15 pm", not "3:07:07 pm".

## FAQ

### Is a T32 stamp a hash?

No. It is an exact, reversible encoding of a number of seconds. Nothing is lost except precision you chose to drop.

### Why not ISO 8601?

ISO 8601 is readable, but it has many valid spellings of one instant, sorts correctly only when every value uses the same format and zone, and cannot say how precise a value is. It is also 20 characters for one second.

### Why not Unix time?

Unix time pretends leap seconds do not exist, so one second repeats each time one is inserted, and intervals that span it are a second short. It also has no notion of precision, and its text form sorts correctly only when zero-padded.

### Why TAI instead of UTC?

TAI is a plain count of SI seconds. UTC is TAI minus an offset that changes when a leap second is announced, which is why software that does arithmetic on UTC has had outages. GPS and PTP already use TAI-based time for the same reason. The cost is a 28-row conversion table at the edge.

### Why base32 and not base62 or base64?

Base32 is case-insensitive, so it survives file systems, URLs and people reading it aloud; the Crockford alphabet drops the confusable letters. Its characters are in ASCII order, so text order equals numeric order. And 5 bits per character gives clean steps of 32.

### Why does every stamp start with `G`?

The first character covers about 1,089 years, so everything from 2000 to 3089 shares it. That is the price of a fixed width; writing "2026" repeats "20" for the same reason.

### Why aren't the buckets days and years?

A day is not a fixed number of 32nds of anything. Calendar-aligned buckets would need a mixed radix, which breaks the property that a prefix is a range of exactly one-thirty-second the size. Calendars are for display.

### What about time zones?

A stamp names one instant everywhere. Zones apply only when converting for display.

### Will my stamps change if leap seconds change?

Past stamps never change. A UTC time beyond the table's expiry converts with the last known offset and is flagged provisional; leap seconds are due to end by 2035, after which the table freezes.

### What about dates before 15,420 BCE?

They are out of range. Astronomy uses Julian Dates for deep time, and so should you.
