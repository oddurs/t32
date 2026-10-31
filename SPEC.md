# T32: A Compact Universal Timestamp

Draft specification, 2026-10-30.

## Abstract

T32 writes any moment as 8 base32 characters: a count of atomic (TAI) seconds since 2000, offset so the past encodes too. 2026-10-31 03:13:00 UTC is `G0S7G6B1`.

A stamp's length is its precision. Drop characters for a coarser span, append characters for finer time; every stamp names an interval, never a false instant. Plain byte-wise sorting is time order, and a prefix is a range query.

The canonical 8-character form covers roughly 15,420 BCE to 19,420 CE at one-second resolution.

## Goals and non-goals

T32 aims to be the shortest unambiguous, sortable, physically uniform name for a moment.

**Goals**

1. **Uniform.** Every second has the same length; subtracting two stamps gives true elapsed time.
2. **Compact.** 8 characters at one second, versus 20 for ISO 8601 (`2026-10-31T03:13:00Z`).
3. **Sortable.** Byte-wise order of the text is time order, with no parsing.
4. **Honest precision.** The length of a stamp states how precisely the moment is known.
5. **Hierarchical.** Every prefix is a valid, coarser stamp that contains the longer one.
6. **Human-safe.** No ambiguous glyphs, no case sensitivity on input, no accidental words.
7. **Stable.** A stamp, once issued, never needs to change.

**Non-goals**

- Replacing the civil calendar. People still read local dates; T32 sits underneath them.
- Time zones. A stamp names an instant everywhere at once; local time is a display concern.
- Unique IDs. T32 carries no randomness; pair it with a random suffix if you need uniqueness.

## Definitions

Five constants define T32; nothing else is configurable.

| Term | Value | Why |
| --- | --- | --- |
| Timescale | TAI (International Atomic Time), SI seconds | No leap seconds, so every second is equal and arithmetic is exact |
| Epoch E | 2000-01-01T00:00:00 TAI (= 1999-12-31T23:59:28 UTC, Unix 946684768) | Near J2000, the astronomical reference epoch; keeps present-day values small |
| Bias B | 2^39 s = 549,755,813,888 s (about 17,421 years) | Offset binary: centres the range on the epoch so past dates stay non-negative and sortable |
| Alphabet | `0123456789ABCDEFGHJKMNPQRSTVWXYZ` (Crockford base32) | Omits I, L, O, U; ASCII-ascending, so text order equals numeric order |
| Canonical length | 8 characters | 40 bits, one-second resolution |

The **unit** of a stamp of length n is u(n) = 32^(8−n) seconds. Length 8 has a unit of 1 s; length 7, 32 s; length 9, 1/32 s.

The **biased time** of an instant t is its TAI seconds since E, plus B. It runs from 0 to 2^40 over the whole representable range.

## Encoding

To encode instant t at length n: take biased time, divide by the unit, floor, and write the result as n base32 digits, most significant first, zero-padded.

```
V(n) = floor( ((t − E) in TAI seconds + B) / 32^(8−n) ),   0 ≤ V(n) < 32^n
```

1. Convert t to TAI seconds since E (see [Converting from UTC](#converting-from-utc)).
2. Add B.
3. Divide by u(n) and **floor** toward negative infinity. Never round: rounding would let a stamp name an interval that does not contain t.
4. If the result is outside [0, 32^n), the instant is out of range; fail, never wrap or clamp.
5. Emit exactly n digits from the alphabet, uppercase, no separators.

Because of the floor, encoding at length n and then truncating to length k gives the same stamp as encoding at length k directly. Precision can be reduced with string slicing alone.

## Decoding and interval semantics

A stamp is an interval, not an instant: a stamp of length n with value V names the half-open span below, in TAI.

```
[ E − B + V · 32^(8−n),  E − B + (V+1) · 32^(8−n) )
```

- **Start** is the earliest instant the stamp could mean. Use it for sorting and range queries.
- **Midpoint** is the best single estimate. Use it when one instant is required.
- **Containment:** stamp A contains stamp B exactly when A is a prefix of B.
- **No partial overlap:** two stamps are either nested (one is a prefix of the other) or disjoint. There is no third case.
- **Equality:** `G0S7G6B1` and `G0S7G6B10` are different stamps. The second is one thirty-second of the first, not a synonym. Trailing zeros carry meaning.

Length limits: n ≥ 1. Implementations must accept lengths 1 to 16 (unit down to 2^−40 s, about 0.9 ps) and may reject longer stamps.

## Ordering and prefix properties

Byte-wise comparison of canonical stamps orders them by start time, and on equal starts puts the coarser (containing) stamp first.

- **Same length:** text order is exactly time order.
- **Mixed lengths:** still sorted by start; `G0S7` sorts immediately before everything inside it, such as `G0S7G6B1`.
- **Collation:** compare bytes (ASCII, or SQL `COLLATE "C"`). Locale-aware collation can break the order.
- **Range query by prefix:** everything inside `G0S7` matches `LIKE 'G0S7%'`, or, index-friendly, `t >= 'G0S7' AND t < 'G0S8'`.
- **Successor:** the exclusive upper bound of a prefix is the prefix with its last digit incremented, carrying left (`G0SZ` → `G0T0`). The all-`Z` prefix has no successor; use the open range `t >= 'ZZ…'`.

## Precision by length

Each character multiplies resolution by 32 (5 bits). Units do not align with calendar days or decimal fractions; that is the cost of a single uniform radix.

| Length | Unit (exact) | Unit (approx.) | Example | Typical use |
| --- | --- | --- | --- | --- |
| 1 | 2^35 s | 1,089 years | `G` | Era |
| 2 | 2^30 s | 34.0 years | `G0` | Generation |
| 3 | 2^25 s | 388 days | `G0S` | Year-scale bucket |
| 4 | 2^20 s | 12.1 days | `G0S7` | Fortnight bucket |
| 5 | 2^15 s | 9.10 hours | `G0S7G` | Shift-scale bucket |
| 6 | 2^10 s | 17.1 minutes | `G0S7G6` | Meeting-scale |
| 7 | 2^5 s | 32 seconds | `G0S7G6B` | Coarse log time |
| **8** | **1 s** | **1 second** | **`G0S7G6B1`** | **Canonical** |
| 10 | 2^−10 s | 0.977 ms | | Application events |
| 12 | 2^−20 s | 0.954 µs | | Databases, tracing |
| 14 | 2^−30 s | 0.931 ns | | Hardware timestamps |
| 16 | 2^−40 s | 0.909 ps | | Metrology |

Odd lengths above 8 (9, 11, 13, 15) are valid; even lengths are listed because they match common precision tiers.

## Durations

A duration is a sign, then TAI seconds in the same base32 digits, then optionally a dot and base32 fraction digits: `+2MC0` is one day.

| Duration | Seconds | T32 duration |
| --- | --- | --- |
| 1 minute | 60 | `+1W` |
| 1 hour | 3,600 | `+3GG` |
| 1 day | 86,400 | `+2MC0` |
| 1 week | 604,800 | `+JEM0` |
| Half a second | 0.5 | `+0.G` |

Rules:

- The sign is mandatory (`+` or `-`), so a duration can never be mistaken for a stamp. Leading zeros are dropped; zero is `+0`.
- Durations need the dot because, unlike stamps, they have no fixed width to imply the unit.
- Durations are SI seconds. `+2MC0` is always 86,400 s, even across a UTC leap second; a civil day is a calendar concept, not a T32 one.
- **Stamp − stamp:** the difference of the two starts, uncertain by up to the coarser stamp's unit.
- **Stamp + duration:** shift the start, then floor to the stamp's own length. The result is never more precise than its input.

## Converting from UTC

T32 is TAI-native; UTC (and Unix time, which follows UTC) converts by adding TAI − UTC for that date, which is 37 s for every instant since 2017-01-01.

From a Unix timestamp:

```
biased = unix + ΔAT(unix) − 946684800 + 2^39
```

**1972 onward: the leap-second table.** ΔAT is a whole number of seconds that steps by one at each leap second. [Appendix A](#appendix-a-tai--utc) lists all 28 values. Only this table ever needs updating, and only when IERS Bulletin C announces a change.

**1961 to 1971: the rubber-second era.** UTC ran at a slightly adjusted rate, so ΔAT is a linear formula in MJD (Modified Julian Date) within each period, also in Appendix A. At 1972-01-01, ΔAT steps to exactly 10 s.

**Before 1961: recorded time is taken as is.** ΔAT is defined as 0, and the date is read as proleptic Gregorian civil time at Greenwich, with astronomical year numbering (year 0 = 1 BCE). Convert Julian-calendar dates first.

- This makes pre-1961 stamps **stable**: they never change when models of Earth's rotation are revised.
- The cost: pre-1961 time follows Earth's rotation, not atomic seconds. Elapsed time between such stamps is off from true physical time by the change in ΔT (TT − UT): under about a minute across the last three centuries, but hours in antiquity.
- Applications that need physical elapsed time in the deep past apply a ΔT model to the decoded value when they read it, never when they write the stamp.
- At 1961-01-01 there is a 1.42 s discontinuity where the definition switches.

**The leap second itself.** UTC 23:59:60 is a real second. Encode it as the 23:59:59 of that day plus one second, using that day's ΔAT. Unix time cannot represent it (the Unix second repeats); a converter given a repeated Unix second takes the first occurrence. Where available, read `CLOCK_TAI` directly and skip conversion entirely.

**Negative leap seconds.** None has ever happened, but the table allows them. On such a day UTC 23:59:59 does not exist, and a converter given it must reject it.

**Future instants.** The published table carries an expiry date (currently 2027-06-28). A UTC time past the expiry converts with the last known offset and must be flagged **provisional**. Store future events as their UTC label and convert when they happen, or recompute after each Bulletin C. Leap seconds are due to be discontinued by 2035, after which the table should freeze.

**Smeared clocks.** Some cloud providers spread a leap second across a day instead of inserting it. Timestamps read from such clocks can be up to 1 s off near a leap second; treat them as accurate to length 7, not 8, in that window.

## Text form, display and parsing

Parse leniently, emit strictly: output is always canonical, input forgives the mistakes people make when reading or typing.

**Canonical output**

- Uppercase alphabet characters only. No separators, no prefix, no padding beyond the stamp's own length.
- Canonical form is the only form that may be stored, compared or indexed.

**Display (optional, never stored)**

- Group in fours with a hyphen: `G0S7-G6B1`, `G0S7-G6B1-3XYE`.
- Where the type is not obvious from context, prefix `t32:` (`t32:G0S7G6B1`). The prefix is not part of the stamp.

**Parsing**

1. Strip an optional `t32:` prefix (case-insensitive).
2. Ignore hyphens.
3. Uppercase. Read `O` as `0`, and `I` or `L` as `1`.
4. Reject `U`, any other character, an empty result, or more than the supported length.

**No check character.** Crockford base32 defines optional check symbols, but in T32 length carries meaning, so a trailing check digit would be indistinguishable from one more digit of precision. Integrity belongs to the transport.

## Binary form

In binary, a stamp is its value V as an unsigned big-endian integer, with the length fixed by the field type rather than stored.

| Field type | Length | Bits used | Bytes | Resolution |
| --- | --- | --- | --- | --- |
| T32/40 | 8 | 40 | 5 | 1 s |
| T32/64 | 12 | 60 (top 4 bits zero) | 8 (u64) | 0.954 µs |
| T32/80 | 16 | 80 | 10 | 0.909 ps |

- Big-endian bytes sort in time order, the same as the text.
- 8 characters pack into exactly 5 bytes and 16 into exactly 10, so text and binary convert without padding.
- T32/64 is the recommended database column: one machine word, full range, sub-microsecond.

## Out of scope and known limits

These limits are deliberate; each would cost one of the goals to remove.

- **Calendar alignment.** Buckets do not fall on days, months or years. Every era begins on the epoch: `F` covers about 911 to 2000, `G` about 2000 to 3089, so dates before and after 2000 differ in the first character.
- **Relativity.** TAI is defined on Earth's geoid. Clocks in orbit, on the Moon or on Mars must convert through a defined relativistic transformation (as GPS already does) before encoding.
- **Pre-1961 physics.** Stamps before 1961 track Earth's rotation, not atomic seconds (see [Converting from UTC](#converting-from-utc)).
- **Range.** The canonical form ends at about 19,420 CE. Extending it would add a character to every stamp in use today.
- **Uniqueness.** Two events in the same second share a stamp. Add precision or a random suffix; T32 is a time, not an ID.

## Test vectors

An implementation is conforming when it reproduces every row below. All were generated by the reference implementation in [`rust/`](rust/src/lib.rs), and the TypeScript implementation in [`web/`](web/src/lib/t32.ts) is tested against the same rows.

**Encoding (UTC input, proleptic Gregorian)**

| Case | UTC | Length | Stamp |
| --- | --- | --- | --- |
| T32 epoch | 1999-12-31T23:59:28 | 8 | `G0000000` |
| Unix epoch (rubber era) | 1970-01-01T00:00:00 | 8 | `FZ3S5F48` |
| Apollo 11 landing | 1969-07-20T20:17:40 | 8 | `FZ3BMMKB` |
| UTC becomes integer | 1972-01-01T00:00:00 | 8 | `FZ5NA8WA` |
| Before last leap second | 2016-12-31T23:59:59 | 8 | `G0FZP0S3` |
| Last leap second | 2016-12-31T23:59:60 | 8 | `G0FZP0S4` |
| After last leap second | 2017-01-01T00:00:00 | 8 | `G0FZP0S5` |
| Spec date | 2026-10-31T03:13:00 | 8 | `G0S7G6B1` |
| Spec date + 0.5 s | 2026-10-31T03:13:00.5 | 10 | `G0S7G6B1G0` |
| Spec date + 0.123 s | 2026-10-31T03:13:00.123 | 12 | `G0S7G6B13XYE` |
| Pre-1961, taken as is | 1675-08-20T00:00:00 | 8 | `FPEY6N00` |

**Decoding (interval start and end, UTC)**

| Stamp | Start | End (exclusive) |
| --- | --- | --- |
| `00000000` | −15422-12-06T11:41:52 | −15422-12-06T11:41:53 |
| `ZZZZZZZZ` | 19421-01-25T12:17:30 | 19421-01-25T12:17:31 |
| `G` | 1999-12-31T23:59:28 | 3088-10-25T03:45:31 |
| `G0S7` | 2026-10-24T23:46:35 | 2026-11-06T03:02:51 |
| `G0S7G6` | 2026-10-31T03:07:07 | 2026-10-31T03:24:11 |

Years are astronomical (−15422 = 15423 BCE).

**Parsing and errors**

| Input | Result |
| --- | --- |
| `g0s7-g6bi` | `G0S7G6B1` |
| `t32:G0S7G6B1` | `G0S7G6B1` |
| `oooo` | `0000` |
| `G0SU` | Error: invalid character |
| empty, or `-` | Error: empty |
| 2016-12-30T23:59:60 | Error: not a leap second |

## Appendix A: TAI − UTC

TAI − UTC has been 37 s since 2017-01-01; IERS Bulletin C 72 confirmed no leap second at the end of 2026. Source: `leap-seconds.list` (IERS/NIST, as shipped in tzdata, expires 2027-06-28) and the USNO `tai-utc.dat` history.

**1972 onward (whole seconds; value applies from 00:00:00 UTC on the date)**

| From | TAI − UTC (s) | From | TAI − UTC (s) |
| --- | --- | --- | --- |
| 1972-01-01 | 10 | 1985-07-01 | 23 |
| 1972-07-01 | 11 | 1988-01-01 | 24 |
| 1973-01-01 | 12 | 1990-01-01 | 25 |
| 1974-01-01 | 13 | 1991-01-01 | 26 |
| 1975-01-01 | 14 | 1992-07-01 | 27 |
| 1976-01-01 | 15 | 1993-07-01 | 28 |
| 1977-01-01 | 16 | 1994-07-01 | 29 |
| 1978-01-01 | 17 | 1996-01-01 | 30 |
| 1979-01-01 | 18 | 1997-07-01 | 31 |
| 1980-01-01 | 19 | 1999-01-01 | 32 |
| 1981-07-01 | 20 | 2006-01-01 | 33 |
| 1982-07-01 | 21 | 2009-01-01 | 34 |
| 1983-07-01 | 22 | 2012-07-01 | 35 |
| | | 2015-07-01 | 36 |
| | | 2017-01-01 | 37 |

**1961 to 1971 (TAI − UTC = a + (MJD − m) × r seconds, MJD of the UTC instant)**

| From | a (s) | m (MJD) | r (s/day) |
| --- | --- | --- | --- |
| 1961-01-01 | 1.4228180 | 37300 | 0.0012960 |
| 1961-08-01 | 1.3728180 | 37300 | 0.0012960 |
| 1962-01-01 | 1.8458580 | 37665 | 0.0011232 |
| 1963-11-01 | 1.9458580 | 37665 | 0.0011232 |
| 1964-01-01 | 3.2401300 | 38761 | 0.0012960 |
| 1964-04-01 | 3.3401300 | 38761 | 0.0012960 |
| 1964-09-01 | 3.4401300 | 38761 | 0.0012960 |
| 1965-01-01 | 3.5401300 | 38761 | 0.0012960 |
| 1965-03-01 | 3.6401300 | 38761 | 0.0012960 |
| 1965-07-01 | 3.7401300 | 38761 | 0.0012960 |
| 1965-09-01 | 3.8401300 | 38761 | 0.0012960 |
| 1966-01-01 | 4.3131700 | 39126 | 0.0025920 |
| 1968-02-01 | 4.2131700 | 39126 | 0.0025920 |

Before 1961-01-01, TAI − UTC is defined as 0 for T32.

## Appendix B: reference implementation

The Rust library in [`rust/src/lib.rs`](rust/src/lib.rs) is normative where this text is ambiguous: standard library only, integer arithmetic only, and its tests cover every vector above. Biased time is held as fixed point with 40 fractional bits, enough for every length up to 16.
