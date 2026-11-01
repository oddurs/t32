# T32: A Compact Universal Timestamp

| | |
| --- | --- |
| Document | T32 specification, draft 0.3 |
| Date | 2026-11-01 |
| Editor | Oddur Sigurdsson ([@oddurs](https://github.com/oddurs)) |
| Latest version | https://oddurs.github.io/t32/spec/ |
| Source and history | https://github.com/oddurs/t32 |
| Comments | [Issues](https://github.com/oddurs/t32/issues) for errata and questions; [Discussions](https://github.com/oddurs/t32/discussions) for proposals |

## Abstract

T32 writes any moment as 8 base32 characters: a count of atomic (TAI) seconds since the start of 2000, offset so the past encodes too. 2026-10-31 03:13:00 UTC is `G0S7G6B1`.

A stamp's length is its resolution. Drop characters for a coarser span, append characters for finer time; every stamp names an interval, never a false instant. Byte-wise sorting is time order, and a prefix is a range query.

The canonical 8-character form covers 15,423 BCE to 19,421 CE at one-second resolution.

## Status of this document

This is draft 0.3 of the T32 specification, published for public review. It is complete enough to implement and to test against, and it is not yet frozen: any part may change before version 1.0. Changes are made in the open, each with its reason, and are listed in Appendix D. Each draft is tagged in the repository as `spec-draft-<number>`, so a draft number always names one fixed text.

Comments are welcome from anyone. They are most useful when specific: a sentence that can be read two ways, a vector that seems wrong, a use the format serves badly. Appendix C lists the questions the editor considers open; views on them are particularly sought. How comments become changes, and the path from this draft to version 1.0 and to a working group, is set out in [GOVERNANCE.md](GOVERNANCE.md).

## Conventions and conformance

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC 2119] [RFC 8174] when, and only when, they appear in all capitals, as shown here.

The grammar uses ABNF [RFC 5234].

This document defines four conformance classes:

| Class | Does | Sections |
| --- | --- | --- |
| Encoder | Instant to stamp | Encoding; Length and resolution; Text form; Binary form |
| Decoder | Stamp to interval | Decoding and interval semantics |
| Parser | Text to stamp or duration | Grammar; Text form; Durations |
| Converter | UTC (or Unix time) to and from TAI | Converting from UTC; Appendix A |

An implementation conforms to a class when it meets every requirement of that class and reproduces every test vector for it. The text, the test vectors and Appendix A are normative; where a vector and the text disagree, the text decides and the disagreement is an erratum. The implementations in Appendix B are informative.

## Goals and non-goals

T32 aims to be the shortest unambiguous, sortable and physically uniform name for a moment.

**Goals**

1. **Uniform.** From 1961 onward, a T32 second is an SI second of TAI, and subtracting two stamps gives elapsed time. (Before 1961, T32 follows Universal Time; see Converting from UTC.)
2. **Compact.** 8 characters at one second, against 20 for RFC 3339 (`2026-10-31T03:13:00Z`).
3. **Sortable.** Byte-wise order of canonical text is time order.
4. **Honest resolution.** A stamp's length states the resolution of the record.
5. **Hierarchical.** Every prefix is a valid, coarser stamp that contains the longer one.
6. **Human-safe.** Case-insensitive, without the most confusable letters, and fewer accidental words.
7. **Stable.** A stamp always denotes the same interval of the T32 time scale. It never needs to change.

**Non-goals**

- Replacing the civil calendar. People read local dates; T32 sits underneath them.
- Time zones. A stamp names an instant everywhere at once.
- Unique identifiers. T32 carries no randomness. Keep any random part in a separate field, or after a character outside the alphabet: appended alphabet characters would read as resolution.

## Definitions

Five constants define T32. Nothing else is configurable.

| Term | Value | Why |
| --- | --- | --- |
| Time scale | TAI (International Atomic Time) from 1961; Universal Time before (see Converting from UTC) | No leap seconds: every second is equal and arithmetic is exact |
| Epoch E | 2000-01-01T00:00:00 TAI (= 1999-12-31T23:59:28 UTC, Unix 946684768) | A round, recent instant that keeps present-day values small. It is not J2000 (2000-01-01T12:00:00 TT) |
| Bias B | 2^39 s = 549,755,813,888 s (about 17,421 years) | Offset binary: centres the range on the epoch, so past instants stay non-negative and sortable |
| Alphabet | `0123456789ABCDEFGHJKMNPQRSTVWXYZ` (Crockford base32) | Omits I, L, O, U; in ASCII order, so text order equals numeric order |
| Lengths | 1 to 16 characters; 8 is canonical | 5 bits per character; 8 characters is 40 bits at 1 s |

A stamp of length n has **unit** u(n) = 32^(8−n) seconds: 1 s at length 8, 32 s at length 7, 1/32 s at length 9.

The **biased time** of an instant t is its TAI seconds since E, plus B. Every representable instant has a biased time in [0, 2^40) seconds.

TAI itself is computed after the fact by the BIPM; in real time it is approximated as UTC(k) + ΔAT. T32 does not distinguish these realizations.

## Grammar

```abnf
stamp          = 1*16digit32                  ; canonical form

digit32        = DIGIT / %x41-48 / %x4A-4B / %x4D-4E
                 / %x50-54 / %x56-5A
                 ; 0-9 A-H J K M N P-T V-Z

stamp-input    = in-digit 0*15( [ "-" ] in-digit )

in-digit       = digit32 / lower32 / alias
lower32        = %x61-68 / %x6A-6B / %x6D-6E / %x70-74 / %x76-7A
alias          = %x4F / %x6F / %x49 / %x69 / %x4C / %x6C
                 ; O o read as 0; I i L l read as 1

duration       = sign 1*8digit32 [ "." 1*8digit32 ]
duration-input = sign 1*8in-digit [ "." 1*8in-digit ]
sign           = "+" / "-"

utc-label      = year "-" month "-" day ( "T" / SP )
                 hour ":" minute ":" second [ "." 1*9DIGIT ] "Z"
year           = 4DIGIT / sign 4*6DIGIT
month          = 2DIGIT          ; 01-12
day            = 2DIGIT          ; 01-31, as the month allows
hour           = 2DIGIT          ; 00-23
minute         = 2DIGIT          ; 00-59
second         = 2DIGIT          ; 00-60, 60 only in a leap second
```

Hyphens in `stamp-input` stand only between two digits, singly. Canonical durations have further constraints (see Durations). `utc-label` is RFC 3339 [RFC 3339] in UTC, with ISO 8601's expanded years for those outside 0000 to 9999; as in all ABNF, `"T"` and `"Z"` also match lowercase. Converters read and write UTC labels in this form, with the fewest fraction digits that are exact.

## Encoding

An encoder writes instant t at length n as the floor of its biased time divided by the unit, in n digits:

```
V = floor( (t − E + B) / 32^(8−n) ),    0 ≤ V < 32^n
```

1. Obtain t − E in TAI seconds, exactly (see Converting from UTC).
2. Add B, divide by u(n), and floor. Encoders MUST floor exactly once, at this step, toward negative infinity; intermediate values MUST NOT be rounded. Rounding would let a stamp name an interval that does not contain t.
3. If V is outside [0, 32^n), the encoder MUST fail. It MUST NOT wrap or clamp.
4. Encoders MUST write V as exactly n digits from the alphabet, most significant first, uppercase.

Because there is one floor, encoding at length n and truncating to length k gives the stamp that encoding at length k gives. Reducing resolution is a string operation.

## Decoding and interval semantics

A stamp of length n and value V denotes the half-open interval of the T32 time scale (TAI from 1961):

```
[ E − B + V · 32^(8−n),   E − B + (V+1) · 32^(8−n) )
```

- **Start** is the earliest instant the stamp can mean. Sorting and range queries use it.
- **Midpoint** is the best single estimate when one instant is required.
- **Containment.** Stamp A contains stamp B exactly when A is a prefix of B.
- **No partial overlap.** Two stamps are nested or disjoint; there is no third case.
- **Trailing zeros carry meaning.** `G0S7G6B1` and `G0S7G6B10` are different stamps; the second is one thirty-second of the first.
- **Nanosecond readings.** A decoder that writes an instant as a nanosecond UTC label MUST write the latest nanosecond label whose length-16 stamp is no later than the instant's length-16 stamp; within a leap second, labels 23:59:60.f count as labels. Every nanosecond label then survives a round trip exactly. A label's instant need not be a multiple of 2⁻⁴⁰ s, so the instant encoded from a label is its floor, which can fall in time inserted just before that label (see Steps in ΔAT); it still reads back as the label.

## Ordering and prefix properties

Byte-wise comparison of canonical stamps orders them by start, and on equal starts puts the shorter (containing) stamp first.

- **Same length:** text order is time order.
- **Mixed lengths:** still ordered by start. `G0S7` sorts immediately before everything it contains.
- **Collation:** comparisons MUST be byte-wise (ASCII, or SQL `COLLATE "C"`). Locale-aware collation can break the order.
- **Range query:** everything inside `G0S7`, at any length, is `t >= 'G0S7' AND t < 'G0S8'`. The upper bound is the prefix with its last digit incremented, carrying left and dropping every digit that carries to zero (`G0SZ` → `G0T`, `G0ZZ` → `G1`); keeping them would admit the shorter stamp `G0T`, which `G0SZ` does not contain. A prefix of all `Z` has no upper bound.

## Length and resolution

Length records the resolution of the record: the width of the interval in which the instant is stated to lie. It does not describe the accuracy of the clock that produced it.

| Length | Unit (exact) | Unit (approx.) | Example | Typical source |
| --- | --- | --- | --- | --- |
| 1 | 2^35 s | 1,089 years | `G` | |
| 2 | 2^30 s | 34.0 years | `G0` | |
| 3 | 2^25 s | 388 days | `G0S` | |
| 4 | 2^20 s | 12.1 days | `G0S7` | |
| 5 | 2^15 s | 9.10 hours | `G0S7G` | |
| 6 | 2^10 s | 17.1 minutes | `G0S7G6` | |
| 7 | 2^5 s | 32 seconds | `G0S7G6B` | |
| **8** | **1 s** | **1 second** | **`G0S7G6B1`** | **Seconds (canonical)** |
| 10 | 2^−10 s | 0.977 ms | | Milliseconds |
| 12 | 2^−20 s | 0.954 µs | | Microseconds |
| 14 | 2^−30 s | 0.931 ns | | Nanoseconds |
| 16 | 2^−40 s | 0.909 ps | | Picoseconds |

Odd lengths are valid. A source's resolution r is the unit in which it reports time, such as 1 ns for a POSIX `timespec`, not the accuracy of the clock behind it. An encoder MUST NOT emit a stamp longer than the longest length n with u(n) > r/√32: the length whose unit is nearest r on a logarithmic scale, the shorter where two are equally near. In integers, n is the longest length with 32 · u(n)² > r². A millisecond clock therefore yields at most 10 characters, a nanosecond clock at most 14. Encoders SHOULD use one fixed length per field.

## Durations

A duration is a signed whole number of 2^−40 s, written exactly as a sign, base32 seconds, and optional base32 fraction digits: `+2MC0` is one day.

| Duration | Seconds | T32 duration |
| --- | --- | --- |
| 1 minute | 60 | `+1W` |
| 1 hour | 3,600 | `+3GG` |
| 1 day | 86,400 | `+2MC0` |
| 1 week | 604,800 | `+JEM0` |
| Half a second | 0.5 | `+0.G` |

- **Value.** sign × (I + F / 32^k) seconds, where I is the integer part and F the k fraction digits. Every duration is exact in this form, and its magnitude is less than 2^40 s.
- **Canonical form.** The sign is REQUIRED. No leading zeros in the integer part (zero is `0`), no trailing zeros in the fraction, no fraction if it is zero. Zero is `+0`, never `-0`.
- **From other units.** Converting a duration from decimal units (nanoseconds, say) MUST truncate toward zero, so negating a duration negates its text.
- **Stamp − stamp.** B − A is start(B) − start(A), exactly. The difference of the instants that B and A stand for lies within (−u(A), +u(B)) of it.
- **Range.** Implementations MUST reject a duration whose magnitude reaches 2^40 s.
- **Stamp + duration.** Add the duration to the stamp's start, then floor to the stamp's own length. If the result leaves [0, 2^40) s, the operation MUST fail.
- Durations are SI seconds. `+2MC0` is 86,400 s even across a leap second; a civil day is a calendar notion.

## Converting from UTC

T32 is TAI-native. UTC converts through ΔAT = TAI − UTC, given in Appendix A as a function of the UTC label. ΔAT has been 37 s since 2017-01-01.

A **UTC label** L is a time written in the proleptic Gregorian calendar with astronomical year numbering (year 0 is 1 BCE), counted in seconds since 1970-01-01T00:00:00 by the POSIX formula: days × 86,400 + seconds of the day. A Unix timestamp is exactly such a label. Then:

```
t − E = L + ΔAT(L) − 946684800 s
```

Converters MUST evaluate ΔAT exactly (every value in Appendix A is an exact decimal) and MUST NOT round before the encoder's single floor.

### Steps in ΔAT

At each boundary b in Appendix A, ΔAT changes by s = ΔAT_after(b) − ΔAT_before(b). Appendix A lists every non-zero step.

- **Removed time (s < 0).** A label before b whose TAI instant, under the old formula, would reach the TAI instant of b under the new one never occurred. Converters MUST reject it. This happened on 1961-08-01 and 1968-02-01, removing just under 0.05 s and 0.1 s of labels (from 23:59:59.950000001 and 23:59:59.900000003 respectively, at nanosecond resolution). Defined this way, every instant of TAI still has a label. A negative leap second would remove 23:59:59.
- **Inserted time (s > 0).** For s seconds of TAI before the instant labelled b, the formula gives no label.
  - From 1972-07-01 each insertion is one leap second, written 23:59:60 on the day before b. Converters MUST accept second 60 then and only then, and convert 23:59:60.f to the TAI instant one second after that of 23:59:59.f. (The insertion of 0.107758 s at 1972-01-01 is fractional and has no second 60.)
  - Decoders MUST write an instant in a leap second as 23:59:60.f, and any other inserted instant as the label b. They MUST report that the instant was inserted.
  - Before 1972, UTC clocks repeated fractions of a second at these steps; T32 does not represent the repetition.

### Unix time

A Unix value is a UTC label and never names 23:59:60. The Unix value at a leap-second boundary therefore converts to 00:00:00 of the following day. By the decoder rule, an instant inside a leap second converts back to that same value.

### Before 1961

Before 1961-01-01, ΔAT is defined as 0 and the label is read as Universal Time (mean solar time at Greenwich), extended proleptically. Labels are Gregorian: a date in the Julian calendar is converted to Gregorian before it is a label.

- Pre-1961 stamps are **stable**: they do not change when models of Earth's rotation are revised.
- Pre-1961 T32 time follows Earth's rotation, not atomic seconds. Elapsed time between such stamps differs from physical time by the change in ΔT (TT − UT): under about a minute across the last three centuries, hours in antiquity. Applications that need physical elapsed time in the deep past apply a ΔT model when they read a stamp, never when they write one.
- The definition changes at 1961-01-01, where ΔAT steps from 0 to 1.4228180 s; the step is inserted time.

### The table and the future

In Resolution 5 of October 2026, the 28th CGPM decided that UTC becomes continuous on 2027-05-20, allowing |UT1 − UTC| to reach 3,600 s [CGPM 2026]. IERS Bulletin C 72 announced no leap second for 2026-12-31. No further step in ΔAT is therefore expected: Appendix A is expected to be final, with ΔAT = 37 s for every label from 2017-01-01.

- A table carries an expiry date: Appendix A expires at 2027-06-28T00:00:00Z, as does the `leap-seconds.list` it reproduces. Converting a label at or after the expiry of the table in use, in either direction, is **provisional**. A converter MUST return that status with the result, and MUST NOT fail a conversion for it alone. The status belongs to the conversion, not to the stamp.
- Converters SHOULD be able to report which table they used, identified by its expiry date.
- If UTC is ever stepped again, Appendix A gains rows in a new version of this document. Stamps already issued keep their meaning; only conversions of later labels change.

### TAI clocks, smeared clocks and stepped clocks

- A converter MAY read TAI directly, for example Linux `CLOCK_TAI`, where t − E = `CLOCK_TAI` − 946684800 s. It MUST first confirm that the kernel's TAI offset equals ΔAT from its own table (on Linux, `adjtimex()` reports it as `tai`, which must be 37). An unset offset is 0, and a stale one is wrong by a whole second or more.
- Around a leap second, some clocks smear it across many hours and others repeat a second. Readings from those windows can be up to 1 s from the label they claim. An encoder SHOULD undo a known smear before encoding. Otherwise, stamps from the window are not guaranteed to contain the true instant, and SHOULD be marked as such by the application.

## Text form, display and parsing

Parse leniently, emit strictly.

- **Canonical form.** Encoders MUST emit `stamp`: uppercase alphabet digits only. Only the canonical form may be stored, compared, hashed or indexed.
- **Display form.** For people, a stamp MAY be shown in groups of four joined by hyphens: `G0S7-G6B1`, `G0S7-G6B1-3XYE`. The display form MUST NOT be stored.
- **Parsing.** Parsers MUST accept exactly `stamp-input` and MUST produce the canonical form.
  - Input is ASCII. Any other character, including whitespace and non-ASCII letters, MUST be rejected. Callers that accept surrounding whitespace strip it before parsing.
  - Letters are case-insensitive. `O` is read as `0`, and `I` or `L` as `1`. `U` MUST be rejected.
  - A hyphen MAY appear only between two digits. Leading, trailing and doubled hyphens MUST be rejected, so a negative duration such as `-1W` is never read as a stamp.
  - More than 16 digits MUST be rejected.
  - A parser reads its input from left to right and reports the first error it meets: at a character, the error that character causes; at the end of the input, a missing sign, a missing part (`empty`) or a trailing hyphen. Converters check a UTC label in this order: `invalid_date`, `not_a_leap_second`, `removed_time`, `out_of_range`. The error codes are listed under Test vectors.
- **No check character.** Length carries meaning, so a trailing check digit would be indistinguishable from one more digit of resolution. Integrity belongs to the transport.

## Binary form

In binary, a stamp is its value V as an unsigned integer. The length is not stored: a binary field declares it, as type **T32-n**, and every value in the field has that length.

| Field type | Bits | Packed bytes | Fits in | Resolution | Typical source |
| --- | --- | --- | --- | --- | --- |
| T32-8 | 40 | 5 | 64-bit integer | 1 s | Seconds |
| T32-10 | 50 | 7 | 64-bit integer | 0.977 ms | Milliseconds |
| T32-12 | 60 | 8 | 64-bit integer | 0.954 µs | Microseconds |
| T32-14 | 70 | 9 | 128-bit integer | 0.931 ns | Nanoseconds |
| T32-16 | 80 | 10 | 128-bit integer | 0.909 ps | Picoseconds |

- Packed, V is written big-endian in the fewest whole bytes, and byte order is time order, like the text. Lengths 8 and 16 pack with no spare bits.
- As an integer, V sorts in time order within the field.
- A field SHOULD declare the length that matches its source, by the rule in Length and resolution. Lengths up to 12 fit a signed 64-bit integer, so T32-10 suits millisecond sources and T32-12 microsecond sources.

## Versioning

This document is draft 0.3. Until version 1.0, any part may change. From 1.0, Definitions, Encoding, Decoding, the grammar and the reading of labels before 1961 are frozen; later versions may only add, for example rows in Appendix A. A stamp carries no version, and its meaning never changes.

## Security and privacy considerations

- **Canonicalize first.** Different input texts can denote the same stamp (`g0s7-g6bi`, `G0S7G6B1`). Compare, hash and use as keys only canonical forms.
- **Bounded input.** A valid input is at most 31 characters (16 digits and 15 hyphens), and a duration at most 18. A parser that reports the first error it meets has met one by the 32nd character, so it never reads further.
- **Confusables.** Parsing is ASCII-only, so look-alike Unicode letters are rejected rather than mapped.
- **Resolution is information.** A long stamp can reveal timing, such as when an action happened to the microsecond. Truncate to the resolution you mean to publish; for T32 that is a string slice.
- **Table integrity.** A tampered leap-second table shifts every conversion after the change. Obtain it from tzdata or the IERS, and check its expiry date and hash (`leap-seconds.list` carries one).
- **Range.** Out-of-range instants and overflowing arithmetic MUST fail. They MUST NOT wrap.

## Prior art

| Format | 2026-10-31T03:13:00Z | Timescale | Characters | Text order is time order | Resolution in the value |
| --- | --- | --- | --- | --- | --- |
| RFC 3339 / ISO 8601 | `2026-10-31T03:13:00Z` | UTC | 20 | Only within one profile and offset | Partly (reduced forms) |
| Unix time | `1793416380` | UTC, POSIX count | 10 | Only when zero-padded | No |
| TAI64 label, hex | `@400000006ae55ce1` | TAI | 17 | Yes | No (fixed per variant) |
| ULID (time part) | `01M67Z4FK0` | Unix time, ms | 10 + 16 random | Yes | No |
| UUIDv7 | 48-bit Unix ms in a UUID | Unix time, ms | 36 | Yes | No |
| **T32** | **`G0S7G6B1`** | **TAI** | **8** | **Yes** | **Yes, by length** |

TAI64 is the closest relative: the same timescale and the same offset-binary idea (its bias is 2^62), written in hexadecimal at a precision fixed by the variant (TAI64, TAI64N, TAI64NA). ULID showed that sortable Crockford base32 works for time-ordered keys. T32 adds three things: length as resolution, with interval semantics and prefix containment; the shortest case-insensitive text that sorts as time; and an exact UTC mapping at every step of the table.

## Out of scope and known limits

These limits are deliberate; removing any of them would cost a goal.

- **Calendar alignment.** Buckets do not fall on days, months or years. Eras are aligned to the epoch: `F` covers about 911 to 2000 and `G` about 2000 to 3089, so dates either side of 2000 differ in the first character.
- **Relativity.** TAI, like Terrestrial Time (TT = TAI + 32.184 s), is a time scale for clocks on Earth. Clocks in orbit, on the Moon or on Mars convert through a defined relativistic transformation, as GPS does, before encoding.
- **Before 1961.** Stamps follow Universal Time, not atomic seconds (see Converting from UTC).
- **Range.** The canonical form ends in 19,421 CE. Extending it would add a character to every stamp in use.
- **Uniqueness.** Two events in the same unit share a stamp. T32 is a time, not an identifier; see Goals and non-goals for where randomness may go.

## Test vectors

A conforming implementation reproduces every vector for its class. The tables below are a readable selection. The complete set, 782 vectors including both sides of every boundary in Appendix A, is [`vectors/t32-vectors.json`](vectors/t32-vectors.json), in a form any implementation can load. Both are normative.

Every vector was computed by an exact rational-arithmetic oracle written directly from this text (Appendix B), including the errors, and is reproduced by both implementations. A disagreement between this text, the vectors and an implementation is a defect, and reports of one are especially welcome.

Errors in the vectors file are named by code:

| Code | Condition |
| --- | --- |
| `empty` | No digit where one is required: the whole stamp, or either part of a duration |
| `too_long` | More than 16 digits in a stamp, or 8 in either part of a duration |
| `invalid_char` | A character outside the input grammar |
| `misplaced_hyphen` | A hyphen not between two digits |
| `missing_sign` | A duration that does not begin with `+` or `-` |
| `invalid_date` | Text that is not a `utc-label`, or a day or time the calendar lacks |
| `not_a_leap_second` | Second 60 where no leap second was inserted |
| `removed_time` | A UTC label in removed time |
| `out_of_range` | An instant, stamp or duration outside its range |

Which code an input gets when it has several errors is fixed by the order under Text form, display and parsing.

**Encoder and converter (UTC labels)**

| Case | UTC | Length | Stamp |
| --- | --- | --- | --- |
| T32 epoch | 1999-12-31T23:59:28 | 8 | `G0000000` |
| Unix epoch (ΔAT = 8.000082 s) | 1970-01-01T00:00:00 | 8 | `FZ3S5F48` |
| Apollo 11 landing | 1969-07-20T20:17:40 | 8 | `FZ3BMMKB` |
| Rubber era, exact (ns clock) | 1966-06-15T12:00:00.123456789 | 14 | `FZ0EDKA4VPC2A1` |
| UTC becomes integer | 1972-01-01T00:00:00 | 8 | `FZ5NA8WA` |
| Before last leap second | 2016-12-31T23:59:59 | 8 | `G0FZP0S3` |
| Last leap second | 2016-12-31T23:59:60 | 8 | `G0FZP0S4` |
| After last leap second | 2017-01-01T00:00:00 | 8 | `G0FZP0S5` |
| Unix 1483228800 | (2017-01-01T00:00:00) | 8 | `G0FZP0S5` |
| Spec date | 2026-10-31T03:13:00 | 8 | `G0S7G6B1` |
| Spec date + 0.5 s (ms clock) | 2026-10-31T03:13:00.500 | 10 | `G0S7G6B1G0` |
| Spec date + 0.123 s (µs clock) | 2026-10-31T03:13:00.123000 | 12 | `G0S7G6B13XYE` |
| Before 1961, taken as UT | 1675-08-20T00:00:00 | 8 | `FPEY6N00` |

**Converter errors**

| UTC | Result |
| --- | --- |
| 1961-07-31T23:59:59.95 | Accepted |
| 1961-07-31T23:59:59.950000001 | Error: removed time |
| 1968-01-31T23:59:59.900000002 | Accepted |
| 1968-01-31T23:59:59.900000003 | Error: removed time |
| 2016-12-30T23:59:60 | Error: not a leap second |

**Decoder (interval start in TAI seconds since E)**

| Stamp | Start (s) | Width (s) |
| --- | --- | --- |
| `00000000` | −549,755,813,888 | 1 |
| `FZ3BMMKB` | −960,867,733 | 1 |
| `G0000000` | 0 | 1 |
| `G0FZP0S4` | 536,544,036 | 1 |
| `G0S7` | 846,200,832 | 1,048,576 |

**Decoder with converter (interval start and end as UTC labels, floored to 1 ns)**

| Stamp | Start | End (exclusive) |
| --- | --- | --- |
| `00000000` | −15422-12-06T11:41:52 | −15422-12-06T11:41:53 |
| `FZ3BMMKB` | 1969-07-20T20:17:39.425406217 | 1969-07-20T20:17:40.425406187 |
| `G0FZP0S4` | 2016-12-31T23:59:60 (inserted; label 2017-01-01T00:00:00) | 2017-01-01T00:00:00 |
| `G0S7` | 2026-10-24T23:46:35 | 2026-11-06T03:02:51 |
| `G0S7G6` | 2026-10-31T03:07:07 | 2026-10-31T03:24:11 |

The `FZ3BMMKB` interval is 0.99999997 UTC seconds wide, because UTC seconds were stretched in 1969. Years are astronomical (−15422 is 15423 BCE).

**Parser**

| Input | Result |
| --- | --- |
| `g0s7-g6bi` | `G0S7G6B1` |
| `oooo` | `0000` |
| `G0S7G6B1-3XYE` | `G0S7G6B13XYE` |
| `G0SU` | Error: invalid character |
| ` G0S7` (leading space) | Error: invalid character |
| `ı` (U+0131) | Error: invalid character |
| `-1W`, `1W-`, `G0S7--G6B1` | Error: misplaced hyphen |
| 17 digits | Error: too long |
| empty | Error: empty |

**Durations**

| Input | Value | Canonical |
| --- | --- | --- |
| 86,400 s | one day | `+2MC0` |
| −1.5 s | | `-1.G` |
| −1 ns (truncated toward zero) | −1,099 × 2^−40 s | `-0.0000012B` |
| 0 | | `+0` |
| `G0S8` start − `G0S7` start | | `+10000` |
| `G0S7G6B1` start − `G0S7` start | | `+G6B1` |
| `1W` (no sign) | | Error |

## References

**Normative**

- [RFC 2119] Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, 1997.
- [RFC 8174] Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, 2017.
- [RFC 5234] Crocker, D., Overell, P., "Augmented BNF for Syntax Specifications: ABNF", STD 68, RFC 5234, 2008.
- [IERS-C] IERS, Bulletin C (announcements of leap seconds), and `leap-seconds.list` as distributed with the IANA tz database.
- [TAI-UTC] USNO, `tai-utc.dat`: TAI − UTC from 1961, as also embodied in the IAU SOFA routine `iauDat`.

**Informative**

- [CROCKFORD] Crockford, D., "Base 32", https://www.crockford.com/base32.html.
- [TAI64] Bernstein, D. J., "TAI64, TAI64N, and TAI64NA", https://cr.yp.to/libtai/tai64.html.
- [ULID] "Universally Unique Lexicographically Sortable Identifier", https://github.com/ulid/spec.
- [RFC 9562] Davis, K., Peabody, B., Leach, P., "Universally Unique IDentifiers (UUIDs)", RFC 9562, 2024.
- [RFC 3339] Klyne, G., Newman, C., "Date and Time on the Internet: Timestamps", RFC 3339, 2002.
- [RFC 9557] Sharma, U., Bormann, C., "Date and Time on the Internet: Timestamps with Additional Information", RFC 9557, 2024.
- [ISO 8601] ISO 8601-1:2019, "Date and time — Representations for information interchange — Part 1: Basic rules".
- [POSIX] IEEE Std 1003.1-2024, Base Definitions, "Seconds Since the Epoch".
- [CGPM 2022] 27th CGPM (2022), Resolution 4, "On the use and future development of UTC".
- [CGPM 2026] 28th CGPM (2026), Resolution 5, "On the technical actions needed to ensure the continuity of UTC", https://www.bipm.org/en/cgpm-2026/resolution-5: UTC continuous from 2027-05-20.

## Appendix A: TAI − UTC

ΔAT = TAI − UTC as a function of the UTC label. It has been 37 s since 2017-01-01, and with UTC continuous from 2027-05-20 it is expected to stay 37 s (see The table and the future). Before 1961-01-01, ΔAT is defined as 0 for T32. This table expires at 2027-06-28T00:00:00Z.

**1972 onward (whole seconds; each value applies from 00:00:00 UTC on its date)**

| From | ΔAT (s) | From | ΔAT (s) |
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

**1961 to 1971 (ΔAT = a + (MJD − m) × r seconds, where MJD is the Modified Julian Date of the UTC label, fraction of the day included)**

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

**Steps (from these tables)**

| Boundary | Step (s) | Kind |
| --- | --- | --- |
| 1961-01-01 | +1.4228180 | Inserted (start of atomic definition) |
| 1961-08-01 | −0.05 | Removed |
| 1963-11-01, 1964-04-01, 1964-09-01, 1965-01-01, 1965-03-01, 1965-07-01, 1965-09-01 | +0.1 each | Inserted |
| 1968-02-01 | −0.1 | Removed |
| 1972-01-01 | +0.107758 | Inserted |
| Each date from 1972-07-01 in the first table | +1 | Inserted (leap second, 23:59:60) |

The boundaries 1962-01-01, 1964-01-01 and 1966-01-01 change the formula but not its value.

## Appendix B: Implementations (informative)

Known implementations, with their conformance, are listed in [IMPLEMENTATIONS.md](IMPLEMENTATIONS.md). Additions are welcome. Those maintained in the specification's repository:

- [`rust/`](rust/src/lib.rs): the reference implementation, published as the [`t32`](https://crates.io/crates/t32) crate. Integer arithmetic only, no required dependencies.
- [`web/src/lib/t32.ts`](web/src/lib/t32.ts): the same algorithm in one TypeScript file, using BigInt.
- [`tools/oracle.py`](tools/oracle.py): an independent check in exact rational arithmetic, written from this text and sharing no code with the implementations.

Both implementations hold biased time as fixed point in units of 2^−40 s, and hold TAI − UTC as exact multiples of 1/(10^7 × 86,400 × 10^9) s, the finest step any coefficient in Appendix A needs. Converting a label is then exact, and inverting it is a single exact division per period.

## Appendix C: Open questions

These are the questions the editor considers open before version 1.0. Each states the current position and the kind of evidence that would change it. Discussion of any of them is welcome, in the [Discussions](https://github.com/oddurs/t32/discussions) of the specification's repository.

**C.1 The name.** "T32" is also the name of Arm's 32-bit Thumb instruction set and the common name of Lauterbach's TRACE32 debugger. The current position is to keep the name, since the contexts rarely meet and the name says what the format is: T for time, 32 for its radix. Evidence of real confusion, or a better name, would reopen it.

**C.2 The epoch.** T32 counts from 2000-01-01T00:00:00 TAI. Alternatives considered were J2000 (2000-01-01T12:00:00 TT), which suits astronomy, and 1972-01-01, when UTC's offset from TAI became whole seconds. The current position favours a round instant in the scale T32 counts. Since the epoch fixes every stamp, this question closes at version 1.0.

**C.3 Before 1961.** Labels before 1961 are read as Universal Time, which keeps stamps stable but makes elapsed time non-physical across centuries. The alternative is to apply a ΔT model, which is physical but changes stamps whenever the model is revised. The current position favours stability. Users of historical or astronomical data are invited to say which they would rather have. The reading of these labels is frozen at version 1.0, so this question closes then.

**C.4 The resolution rule.** An encoder may use the length whose unit is nearest its source's resolution on a logarithmic scale (u(n) > r/√32), so a stamp can claim up to √32 (about 5.7) times more resolution than its source has. The stricter alternative is the longest length whose unit is no finer than the source, which gives 9 for a millisecond clock instead of 10. The current position prefers the familiar lengths 8, 10, 12 and 14 for seconds, milliseconds, microseconds and nanoseconds.

**C.5 Letter case.** Canonical stamps are uppercase. Lowercase reads better in URLs and file names, and some systems fold case. Since parsing is case-insensitive, the question is only which form is canonical. The current position is uppercase, as in Crockford base32.

**C.6 A human rendering.** The specification defines a display form for stamps, but not how to show a stamp's interval to people in their own calendar and time zone. A recommended rendering could make stamps easier to adopt in interfaces. The question is whether it belongs in this document or in a companion one.

**C.7 Identifiers and registration.** T32 defines no URI scheme, media type or registered tag. Candidates include a suffix tag under RFC 9557 and a URN namespace. Registration would follow version 1.0, and only if implementers ask for it.

**C.8 Other time scales.** TAI suits clocks on Earth. Lunar and Martian time scales are being defined, and a T32 profile for one of them would need its own epoch and conversion. The current position is to leave the format TAI-only and to define profiles, if any, in separate documents.

## Appendix D: Change history

**Draft 0.3, 2026-11-01.** Errata and clarifications from a review of draft 0.2; no stamp changes meaning.
- Text and vectors are both normative, and the text decides where they disagree.
- Decoders MUST write an instant in a leap second as 23:59:60.f and MUST report inserted time, as the vectors already required.
- The nanosecond reading rule is exact: the latest nanosecond label whose length-16 stamp is no later than the instant's.
- Appendix A expires at 2027-06-28T00:00:00Z; a provisional conversion is returned with its result, not failed.
- The range-query bound drops digits that carry to zero (`G0SZ` → `G0T`): the old bound admitted shorter stamps outside the prefix.
- The resolution rule is stated in integers, and a source's resolution is the unit it reports.
- B − A names its operands; MJD includes the fraction of the day; second 60 exists from 1972-07-01.
- UTC label text has a grammar, and every error code a definition and an order of precedence. Added `out_of_range`.
- The reading of labels before 1961 joins what version 1.0 freezes.
- Drafts are tagged `spec-draft-<number>`. After draft 0.2 was dated, its 1966 vector changed from length 16 to 14 and the vectors file was added without a new number; both belong to this draft.
- The vectors gained the remaining spec-table rows and categories for binary packing, stamp ± duration, stamp − stamp, range bounds, the resolution rule, TAI-instant encoding and provisional status. The oracle now generates every vector, errors included.
- References cite the CGPM's Resolution 5 by number.

**Draft 0.2, 2026-10-31.**
- Added BCP 14 conformance language, an ABNF grammar, conformance classes, versioning, security and privacy considerations, prior art and references.
- Scoped the goals: TAI from 1961, Universal Time before.
- Defined removed and inserted time at every step in TAI − UTC, including the fractional steps of 1961–1972.
- A Unix value never names 23:59:60.
- Added the resolution rule and binary field types T32-n.
- Durations became exact, in units of 2⁻⁴⁰ s.
- Parsing became ASCII-only, with hyphens only between digits.
- Noted continuous UTC from 2027-05-20.
- Added the nanosecond reading rule, the vectors file and Appendices C and D.

**Draft 0.1, 2026-10-30.** First public draft.
