#!/usr/bin/env python3
"""Independent check of T32 conversions with exact rational arithmetic.

Written directly from SPEC.md (Definitions, Encoding, Converting from UTC,
Appendix A), sharing no code with the implementations. Recomputes the spec's
encoder, converter and decoder vectors and exits non-zero on any mismatch.

    python3 tools/oracle.py                                  # check the spec's vectors
    python3 tools/oracle.py --write vectors/t32-vectors.json # regenerate the vectors file
    python3 tools/oracle.py --check vectors/t32-vectors.json # and verify it is current
"""
from fractions import Fraction as F
from datetime import date

ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
EPOCH = 946684800 - 32          # 2000-01-01T00:00:00 TAI as a UTC label (Unix s); TAI - UTC was 32 s
BIAS = 2**39

# Appendix A, 1961-1971: (start, a, m, r) with TAI - UTC = a + (MJD - m) * r seconds.
RUBBER = [
    ("1961-01-01", "1.4228180", 37300, "0.0012960"), ("1961-08-01", "1.3728180", 37300, "0.0012960"),
    ("1962-01-01", "1.8458580", 37665, "0.0011232"), ("1963-11-01", "1.9458580", 37665, "0.0011232"),
    ("1964-01-01", "3.2401300", 38761, "0.0012960"), ("1964-04-01", "3.3401300", 38761, "0.0012960"),
    ("1964-09-01", "3.4401300", 38761, "0.0012960"), ("1965-01-01", "3.5401300", 38761, "0.0012960"),
    ("1965-03-01", "3.6401300", 38761, "0.0012960"), ("1965-07-01", "3.7401300", 38761, "0.0012960"),
    ("1965-09-01", "3.8401300", 38761, "0.0012960"), ("1966-01-01", "4.3131700", 39126, "0.0025920"),
    ("1968-02-01", "4.2131700", 39126, "0.0025920"),
]
# Appendix A, 1972 onward: (start, whole seconds).
LEAPS = [("1972-01-01", 10), ("1972-07-01", 11), ("1973-01-01", 12), ("1974-01-01", 13),
         ("1975-01-01", 14), ("1976-01-01", 15), ("1977-01-01", 16), ("1978-01-01", 17),
         ("1979-01-01", 18), ("1980-01-01", 19), ("1981-07-01", 20), ("1982-07-01", 21),
         ("1983-07-01", 22), ("1985-07-01", 23), ("1988-01-01", 24), ("1990-01-01", 25),
         ("1991-01-01", 26), ("1992-07-01", 27), ("1993-07-01", 28), ("1994-07-01", 29),
         ("1996-01-01", 30), ("1997-07-01", 31), ("1999-01-01", 32), ("2006-01-01", 33),
         ("2009-01-01", 34), ("2012-07-01", 35), ("2015-07-01", 36), ("2017-01-01", 37)]


def days(iso):  # proleptic Gregorian days since 1970-01-01
    y, m, d = map(int, iso.split("-"))
    return (date(y, m, d) - date(1970, 1, 1)).days


def label(y, mo, d, h, mi, s, frac="0"):
    """A UTC label in seconds since 1970 (POSIX), exact. Years 1..9999 only."""
    return (date(y, mo, d) - date(1970, 1, 1)).days * 86400 + h * 3600 + mi * 60 + s + F(frac)


PERIODS = [(days(s) * 86400, lambda u, a=F(a), m=m, r=F(r): a + (F(u) / 86400 + 40587 - m) * r)
           for s, a, m, r in RUBBER]
PERIODS += [(days(s) * 86400, lambda u, n=n: F(n)) for s, n in LEAPS]


def dat(u, i=None):
    """TAI - UTC at label u (seconds), using period i if given."""
    if i is None:
        i = max((k for k, (b, _) in enumerate(PERIODS) if u >= b), default=-1)
    return F(0) if i < 0 else PERIODS[i][1](u)


def encode_label(u, n):
    i = max((k for k, (b, _) in enumerate(PERIODS) if u >= b), default=-1)
    tai = u + dat(u) - (EPOCH + 32)     # TAI seconds since the epoch
    if i + 1 < len(PERIODS):            # removed time before the next boundary
        b = PERIODS[i + 1][0]
        if dat(b, i + 1) < dat(b, i) and tai >= b + dat(b, i + 1) - (EPOCH + 32):
            raise ValueError("removed label")
    v = (tai + BIAS) * F(32) ** (n - 8)
    return digits(v.numerator // v.denominator, n)


def encode_leap(y, mo, d, frac, n):     # 23:59:60.frac
    u = label(y, mo, d, 23, 59, 59)
    tai = u + dat(u) + 1 + F(frac) - (EPOCH + 32)
    v = (tai + BIAS) * F(32) ** (n - 8)
    return digits(v.numerator // v.denominator, n)


UNIT = F(1, 2**40)                      # one unit of biased time, in seconds
TAI_LABEL = EPOCH + 32                  # TAI seconds since the epoch = X - TAI_LABEL, X = u + dat(u)


def first_tai(j):
    """X of the first label of row j."""
    b = PERIODS[j][0]
    return b + dat(b, j)


def left(k):
    """X where row k-1 reaches boundary k: the start of any time inserted there."""
    b = PERIODS[k][0]
    return b + dat(b, k - 1)             # row -1 is before 1961, where TAI - UTC is 0


def step(k):
    b = PERIODS[k][0]
    return dat(b, k) - dat(b, k - 1)


def invert(j, x):
    """The label u of row j (or before 1961, j = -1) with u + dat(u) = x. Exact."""
    if j < 0:
        return x
    a0 = PERIODS[j][1](0)                # TAI - UTC is a0 + k*u within a row
    k = PERIODS[j][1](1) - a0
    return (x - a0) / (1 + k)


def row_before(x, strict):
    """The last row whose first label maps before x (strictly, or at or before)."""
    rows = [j for j in range(len(PERIODS)) if (first_tai(j) < x if strict else first_tai(j) <= x)]
    return rows[-1] if rows else -1


def inserted_at(x, strict):
    """The boundary k if x lies in time inserted there, else None."""
    j = row_before(x, strict)
    k = j + 1
    if k < len(PERIODS) and step(k) > 0 and (x > left(k) if strict else x >= left(k)):
        return k
    return None


def ceil_ns(seconds):
    """The smallest whole nanosecond count not below `seconds`."""
    v = seconds * 10**9
    return -((-v.numerator) // v.denominator)


def reading(t):
    """The UTC reading of an instant t (TAI seconds since the epoch, a multiple of UNIT).

    SPEC.md, Decoding: an inserted instant reads as the label b where its insertion
    ends (a leap second as 23:59:60.f); any other instant reads as the latest
    nanosecond label whose instant encodes, at full resolution, no later than t.
    Returns (kind, ns): ("label", n), ("leap", (b_ns, offset_ns)) or ("inserted", b_ns).
    """
    x = t + TAI_LABEL
    k = inserted_at(x, strict=False)
    if k is not None:
        b = PERIODS[k][0]
        if step(k) == 1:
            offset = ceil_ns(x - left(k) + UNIT) - 1   # within 0 .. 999,999,999
            return ("leap", (b * 10**9, offset))
        return ("inserted", b * 10**9)
    y = x + UNIT                        # the end of t's unit: labels must map below it
    k = inserted_at(y, strict=True)
    if k is not None:                   # the unit runs into inserted time
        return ("label", PERIODS[k][0] * 10**9 - 1)
    return ("label", ceil_ns(invert(row_before(y, strict=True), y)) - 1)


def stamp_value(stamp):
    v = 0
    for c in stamp:
        v = v * 32 + ALPHABET.index(c)
    return v


def stamp_bounds(stamp):
    """Start and end of a stamp as TAI seconds since the epoch (exact)."""
    unit = F(32) ** (8 - len(stamp))
    v = stamp_value(stamp)
    return v * unit - BIAS, (v + 1) * unit - BIAS


def rfc3339(ns, second=None):
    """RFC 3339 text of a label in nanoseconds since 1970, years extended with a sign."""
    s, frac = divmod(ns, 10**9)
    d, r = divmod(s, 86400)
    y, m, dd = civil(d)
    year = f"{y:04d}" if 0 <= y <= 9999 else (f"-{-y:04d}" if y < 0 else f"+{y}")
    sec = r % 60 if second is None else second
    f = f".{frac:09d}".rstrip("0") if frac else ""
    return f"{year}-{m:02d}-{dd:02d}T{r // 3600:02d}:{r % 3600 // 60:02d}:{sec:02d}{f}Z"


def reading_text(t):
    kind, v = reading(t)
    if kind == "leap":
        b, offset = v
        return rfc3339(b - 10**9 + offset, second=60), True
    if kind == "inserted":
        return rfc3339(v), True
    return rfc3339(v), False


def decode(stamp):
    """(start text, start inserted, end text or None) for a stamp."""
    start, end = stamp_bounds(stamp)
    s_text, s_ins = reading_text(start)
    e_text = None if end + BIAS >= 2**40 else reading_text(end)[0]
    return s_text, s_ins, e_text


def iso(u):
    import math
    s = math.floor(u)
    ns = math.floor((u - s) * 10**9)
    d, r = divmod(s, 86400)
    y, m, dd = civil(d)
    frac = f".{ns:09d}".rstrip("0") if ns else ""
    return f"{y:05d}-{m:02d}-{dd:02d}T{r // 3600:02d}:{r % 3600 // 60:02d}:{r % 60:02d}{frac}Z"


def civil(z):  # days since 1970 -> proleptic Gregorian (astronomical years)
    z += 719468
    era = z // 146097                # Python floors already
    doe = z - era * 146097
    yoe = (doe - doe // 1460 + doe // 36524 - doe // 146096) // 365
    doy = doe - (365 * yoe + yoe // 4 - yoe // 100)
    mp = (5 * doy + 2) // 153
    d = doy - (153 * mp + 2) // 5 + 1
    m = mp + 3 if mp < 10 else mp - 9
    return yoe + era * 400 + (1 if m <= 2 else 0), m, d


def parse(text):
    """Parser class: the stamp-input grammar of SPEC.md, returning canonical text."""
    out, prev_hyphen = "", False
    for ch in text:
        if ch == "-":
            if not out or prev_hyphen:
                raise ValueError("misplaced hyphen")
            prev_hyphen = True
            continue
        if ord(ch) > 0x7F:
            raise ValueError("invalid character")
        c = {"O": "0", "I": "1", "L": "1"}.get(ch.upper(), ch.upper())
        if c not in ALPHABET:
            raise ValueError("invalid character")
        out, prev_hyphen = out + c, False
        if len(out) > 16:
            raise ValueError("too long")
    if not out:
        raise ValueError("empty")
    if prev_hyphen:
        raise ValueError("misplaced hyphen")
    return out


def duration(seconds):
    """Canonical duration text for an exact number of seconds (a multiple of 2^-40)."""
    units = seconds * 2**40
    assert units.denominator == 1 and abs(units) < 2**80
    sign, mag = ("-" if units < 0 else "+"), abs(units.numerator)
    i, f = mag >> 40, mag & (2**40 - 1)
    text = ""
    while True:
        text = ALPHABET[i % 32] + text
        i //= 32
        if i == 0:
            break
    frac = ""
    while f:
        f <<= 5
        frac += ALPHABET[f >> 40]
        f &= 2**40 - 1
    return sign + text + ("." + frac if frac else "")


def trunc_units(seconds):
    """Seconds to a whole number of 2^-40 s, truncating toward zero."""
    u = seconds * 2**40
    return F(int(u) if u >= 0 else -int(-u), 2**40)


def digits(v, n):
    out = ""
    for _ in range(n):
        out = ALPHABET[v % 32] + out
        v //= 32
    assert v == 0
    return out



# ------------------------------------------------------------------ conformance vectors

def ns_label(y, mo, d, h=0, mi=0, s=0, ns=0):
    return ((date(y, mo, d) - date(1970, 1, 1)).days * 86400 + h * 3600 + mi * 60 + s) * 10**9 + ns


def encode_ns(n, length):
    """Stamp text for a label in nanoseconds, or an error code."""
    try:
        return {"stamp": encode_label(F(n, 10**9), length)}
    except ValueError:
        return {"error": "removed_time"}


def lcg(seed):
    while True:
        seed = (seed * 6364136223846793005 + 1442695040888963407) % 2**64
        yield seed >> 11


def vectors():
    rng = lcg(32)
    v = {
        "title": "T32 conformance vectors",
        "spec": "SPEC.md, draft 0.2",
        "generator": "tools/oracle.py: exact rational arithmetic written from the spec, sharing no code with the implementations",
        "conventions": [
            "Large integers are decimal strings.",
            "UTC text is RFC 3339 in UTC with astronomical years; years outside 0000-9999 carry a sign. Inputs never use second 60 except during a leap second.",
            "Lengths follow the resolution rule: 8 for whole seconds, 9 for tenths, 14 for nanoseconds.",
            "units are the biased time of the spec in units of 2^-40 s (an Instant's fixed-point value); a stamp's start_units is its value shifted left by 5 * (16 - length).",
            "Decoded UTC readings follow the spec: an inserted instant reads as the label where its insertion ends, a leap second as 23:59:60.f; any other instant as the latest nanosecond label that encodes, at full resolution, no later than it.",
            "Error codes: empty, too_long, invalid_char, misplaced_hyphen, missing_sign, removed_time, not_a_leap_second, invalid_date.",
        ],
    }

    # Parser.
    parse_cases = ["G0S7G6B1", "g0s7g6b1", "g0s7-g6bi", "G0S7-G6B1-3XYE", "oooo", "LIli", "Z", "ZZZZZZZZZZZZZZZZ",
                   "G-0-S-7", "", "-", "-1W", "1W-", "G0S7--G6B1", "G0SU", " G0S7", "G0S7 ", "G0S7\t",
                   "\u0131", "\u017f", "\uff27", "\u00fc", "+1W", "00000000000000000", "t32:G0S7"]
    v["parse"] = []
    for text in parse_cases:
        try:
            v["parse"].append({"input": text, "stamp": parse(text)})
        except ValueError as e:
            code = {"invalid character": "invalid_char", "misplaced hyphen": "misplaced_hyphen",
                    "too long": "too_long", "empty": "empty"}[str(e)]
            v["parse"].append({"input": text, "error": code})

    # Encoder and converter, from UTC text.
    enc = []

    def add(n, length, note=None):
        r = encode_ns(n, length)
        row = {"utc": rfc3339(n) if "stamp" in r else rfc3339(n), "length": length, **r}
        if note:
            row["note"] = note
        enc.append(row)

    add(ns_label(1999, 12, 31, 23, 59, 28), 8, "the T32 epoch")
    add(ns_label(1970, 1, 1), 8, "Unix epoch, rubber-second era")
    add(ns_label(1969, 7, 20, 20, 17, 40), 8, "Apollo 11 landing")
    add(ns_label(1966, 6, 15, 12, 0, 0, 123456789), 14, "rubber era, exact")
    add(ns_label(1972, 1, 1), 8, "UTC becomes TAI - 10 s")
    add(ns_label(2026, 10, 31, 3, 13, 0), 8, "spec date")
    add(ns_label(1675, 8, 20), 8, "before 1961: read as Universal Time")
    t = ns_label(2026, 10, 31, 3, 13, 0, 123456789)
    for length in range(1, 15):
        add(t, length, "lengths 1 to 14 of one instant")

    # Every boundary of Appendix A, at nanosecond resolution.
    for k, (b, _) in enumerate(PERIODS):
        b_ns = int(b) * 10**9
        for n, note in [(b_ns - 10**9, "1 s before a boundary"), (b_ns - 1, "1 ns before a boundary"),
                        (b_ns, "a boundary"), (b_ns + 1, "1 ns after a boundary")]:
            add(n, 14, f"{note} of Appendix A ({rfc3339(b_ns)[:10]})")

    # Removed time, exactly at its edges.
    for y, mo, d, last_ok, first_bad in [(1961, 7, 31, 950000000, 950000001), (1968, 1, 31, 900000002, 900000003)]:
        add(ns_label(y, mo, d, 23, 59, 59, last_ok), 14, "the last label before removed time")
        add(ns_label(y, mo, d, 23, 59, 59, first_bad), 14, "the first removed label")

    # Leap seconds.
    leap_utc = []
    for k, (b, _) in enumerate(PERIODS):
        if k and step(k) == 1:
            y, mo, d = civil(int(b) // 86400 - 1)
            before = ns_label(y, mo, d, 23, 59, 59)
            for second, frac, length in [(60, "0", 8), (60, "0.5", 9)]:
                text = rfc3339(before, second=60)[:-1] + ("" if frac == "0" else frac[1:]) + "Z"
                leap_utc.append({"utc": text, "length": length,
                                 "stamp": encode_leap(y, mo, d, frac, length), "note": "leap second"})
            add(before, 8, "the second before a leap second")
    enc[len(enc):] = leap_utc

    # Pseudo-random instants, 1600 to 2100.
    lo, hi = ns_label(1600, 1, 1), ns_label(2100, 1, 1)
    for _ in range(60):
        n = lo + next(rng) % (hi - lo)
        add(n, 14)
    v["encode_utc"] = enc

    # From Unix time.
    v["encode_unix"] = []
    for n, length in [(0, 14), (1483228800 * 10**9, 8), (1483228799 * 10**9, 8), (-1, 14),
                      (1793416380 * 10**9, 8), (-14182940 * 10**9, 8)] + \
                     [(int(next(rng) % (4 * 10**18)) - 2 * 10**18, 14) for _ in range(20)]:
        r = encode_ns(n, length)
        v["encode_unix"].append({"unix_nanos": str(n), "length": length, **r})

    # Converter errors that are not removed time.
    v["utc_errors"] = [
        {"utc": "2016-12-30T23:59:60Z", "error": "not_a_leap_second"},
        {"utc": "2015-12-31T23:59:60Z", "error": "not_a_leap_second"},
        {"utc": "2016-12-31T23:58:60Z", "error": "not_a_leap_second"},
        {"utc": "2026-02-29T00:00:00Z", "error": "invalid_date"},
        {"utc": "2026-04-31T00:00:00Z", "error": "invalid_date"},
    ]

    # Decoder.
    stamps = ["00000000", "G0000000", "FZ3BMMKB", "G0FZP0S4", "G0S7", "G0S7G6", "G0S7G6B1G0",
              "G", "Z", "ZZZZZZZZ", "ZZZZZZZZZZZZZZZZ", "0000000000000000"]
    for row in leap_utc:
        stamps.append(row["stamp"])
    # Units at every boundary: where each row begins, and where inserted time begins.
    for k in range(len(PERIODS)):
        for x in [first_tai(k)] + ([left(k)] if k and step(k) != 0 else []):
            u = (x - TAI_LABEL + BIAS) / UNIT
            stamps.append(digits(u.numerator // u.denominator, 16))
    for _ in range(40):
        u = next(rng) % 2**80
        length = 1 + next(rng) % 16
        stamps.append(digits(u >> (5 * (16 - length)), length))
    seen, dec = set(), []
    for st in stamps:
        if st in seen:
            continue
        seen.add(st)
        start, end = stamp_bounds(st)
        s_text, s_ins = reading_text(start)
        row = {"stamp": st, "start_units": str(stamp_value(st) << (5 * (16 - len(st)))),
               "start_utc": s_text, "start_inserted": s_ins}
        if end + BIAS >= 2**40:
            row["end_utc"] = None
        else:
            e_text, e_ins = reading_text(end)
            row["end_utc"], row["end_inserted"] = e_text, e_ins
        dec.append(row)
    v["decode"] = dec

    # Durations.
    sec = 2**40
    v["durations"] = []
    for units in [0, sec, -sec, 60 * sec, 3600 * sec, 86400 * sec, 604800 * sec, sec // 2, -3 * sec // 2,
                  1, -1, 2**80 - 1, -(2**80 - 1), 1234567 * sec + 89]:
        v["durations"].append({"units": str(units), "text": duration(F(units, sec))})
    for nanos in [1, -1, 7, -7, 500000000, -1500000000, 86400 * 10**9]:
        units = int(trunc_units(F(nanos, 10**9)) * sec)
        v["durations"].append({"nanos": str(nanos), "units": str(units), "text": duration(F(units, sec))})
    for text, code in [("1W", "missing_sign"), ("", "missing_sign"), ("+", "empty"), ("+1.", "empty"),
                       ("+123456789", "too_long"), ("+1.123456789", "too_long"), ("+1.2.3", "invalid_char"),
                       ("+U", "invalid_char"), ("+ 1", "invalid_char")]:
        v["durations"].append({"text": text, "error": code})
    for text, canonical in [("+o.g", "+0.G"), ("-0", "+0"), ("+00001", "+1"), ("+1.G0", "+1.G")]:
        v["durations"].append({"text": text, "canonical": canonical})
    return v


if __name__ == "__main__":
    import sys
    failures = 0

    def check(name, got, want):
        global failures
        ok = got == want
        failures += not ok
        print(f"{'ok ' if ok else 'BAD'} {name:<34} {got}" + ("" if ok else f"   (spec: {want})"))

    # Encoder and converter vectors from SPEC.md.
    for name, u, n, want in [
        ("1999-12-31T23:59:28", label(1999, 12, 31, 23, 59, 28), 8, "G0000000"),
        ("1970-01-01T00:00:00", label(1970, 1, 1, 0, 0, 0), 8, "FZ3S5F48"),
        ("1969-07-20T20:17:40", label(1969, 7, 20, 20, 17, 40), 8, "FZ3BMMKB"),
        ("1966-06-15T12:00:00.123456789", label(1966, 6, 15, 12, 0, 0, "0.123456789"), 14, "FZ0EDKA4VPC2A1"),
        ("1972-01-01T00:00:00", label(1972, 1, 1, 0, 0, 0), 8, "FZ5NA8WA"),
        ("2016-12-31T23:59:59", label(2016, 12, 31, 23, 59, 59), 8, "G0FZP0S3"),
        ("2017-01-01T00:00:00", label(2017, 1, 1, 0, 0, 0), 8, "G0FZP0S5"),
        ("2026-10-31T03:13:00", label(2026, 10, 31, 3, 13, 0), 8, "G0S7G6B1"),
        ("2026-10-31T03:13:00.500", label(2026, 10, 31, 3, 13, 0, "0.500"), 10, "G0S7G6B1G0"),
        ("2026-10-31T03:13:00.123000", label(2026, 10, 31, 3, 13, 0, "0.123000"), 12, "G0S7G6B13XYE"),
        ("1675-08-20T00:00:00", label(1675, 8, 20, 0, 0, 0), 8, "FPEY6N00"),
    ]:
        check(name, encode_label(u, n), want)
    check("2016-12-31T23:59:60", encode_leap(2016, 12, 31, "0", 8), "G0FZP0S4")

    def outcome(f):
        try:
            f()
            return "accepted"
        except ValueError as e:
            return f"error: {e}"

    check("Unix 1483228800", encode_label(F(1483228800), 8), "G0FZP0S5")
    for name, u, want in [
        ("1961-07-31T23:59:59.950000001", label(1961, 7, 31, 23, 59, 59, "0.950000001"), "error: removed label"),
        ("1961-07-31T23:59:59.95", label(1961, 7, 31, 23, 59, 59, "0.95"), "accepted"),
        ("1968-01-31T23:59:59.900000003", label(1968, 1, 31, 23, 59, 59, "0.900000003"), "error: removed label"),
        ("1968-01-31T23:59:59.900000002", label(1968, 1, 31, 23, 59, 59, "0.900000002"), "accepted"),
    ]:
        check(name, outcome(lambda: encode_label(u, 8)), want)
    # 23:59:60 exists only before a positive whole-second step.
    nxt = days("2016-12-31") * 86400 + 86400
    leap = any(b == nxt and dat(b, i) - dat(b, i - 1) == 1 for i, (b, _) in enumerate(PERIODS) if i)
    check("2016-12-31T23:59:60 exists", leap, True)
    nxt = days("2016-12-30") * 86400 + 86400
    leap = any(b == nxt and dat(b, i) - dat(b, i - 1) == 1 for i, (b, _) in enumerate(PERIODS) if i)
    check("2016-12-30T23:59:60 exists", leap, False)

    # Parser vectors.
    for text, want in [("g0s7-g6bi", "G0S7G6B1"), ("oooo", "0000"), ("G0S7G6B1-3XYE", "G0S7G6B13XYE"),
                       ("G0SU", "error: invalid character"), (" G0S7", "error: invalid character"),
                       ("\u0131", "error: invalid character"), ("-1W", "error: misplaced hyphen"),
                       ("1W-", "error: misplaced hyphen"), ("G0S7--G6B1", "error: misplaced hyphen"),
                       ("0" * 17, "error: too long"), ("", "error: empty")]:
        try:
            got = parse(text)
        except ValueError as e:
            got = f"error: {e}"
        check(f"parse {text!r}", got, want)

    # Duration vectors.
    unit4 = F(32) ** 4
    for name, sec, want in [("86,400 s", F(86400), "+2MC0"), ("-1.5 s", F(-3, 2), "-1.G"),
                            ("-1 ns", trunc_units(F(-1, 10**9)), "-0.0000012B"), ("0", F(0), "+0"),
                            ("G0S8 - G0S7", unit4, "+10000")]:
        check(f"duration {name}", duration(sec), want)
    v = lambda t: sum(ALPHABET.index(c) * 32 ** k for k, c in enumerate(reversed(t)))
    check("duration G0S7G6B1 - G0S7", duration(F(v("G0S7G6B1")) - v("G0S7") * unit4), "+G6B1")

    # Decoder class: interval start in TAI seconds since the epoch (no table needed).
    for st, want in [("00000000", -549755813888), ("G0000000", 0), ("G0S7", 846200832), ("FZ3BMMKB", -960867733), ("G0FZP0S4", 536544036)]:
        check(f"TAI start of {st}", v(st) * F(32) ** (8 - len(st)) - BIAS, want)

    # Decoder vectors from SPEC.md.
    for st, want in [
        ("00000000", ("-15422-12-06T11:41:52Z", False, "-15422-12-06T11:41:53Z")),
        ("FZ3BMMKB", ("1969-07-20T20:17:39.425406217Z", False, "1969-07-20T20:17:40.425406187Z")),
        ("G0FZP0S4", ("2016-12-31T23:59:60Z", True, "2017-01-01T00:00:00Z")),
        ("G0S7", ("2026-10-24T23:46:35Z", False, "2026-11-06T03:02:51Z")),
        ("G0S7G6", ("2026-10-31T03:07:07Z", False, "2026-10-31T03:24:11Z")),
    ]:
        check(st, decode(st), want)

    if "--write" in sys.argv or "--check" in sys.argv:
        import json
        path = sys.argv[-1]
        text = json.dumps(vectors(), indent=1, ensure_ascii=False) + "\n"
        if "--write" in sys.argv:
            with open(path, "w") as f:
                f.write(text)
            print(f"wrote {path}")
        else:
            with open(path) as f:
                current = f.read()
            ok = current == text
            failures += not ok
            print(f"{'ok ' if ok else 'BAD'} {path} {'is current' if ok else 'differs: run tools/oracle.py --write ' + path}")

    print(f"{failures} failure(s)")
    sys.exit(1 if failures else 0)
