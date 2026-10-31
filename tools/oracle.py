#!/usr/bin/env python3
"""Independent check of T32 conversions with exact rational arithmetic.

Written directly from SPEC.md (Definitions, Encoding, Converting from UTC,
Appendix A), sharing no code with the implementations. Recomputes the spec's
encoder, converter and decoder vectors and exits non-zero on any mismatch.

    python3 tools/oracle.py
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


def decode(stamp):
    """Interval start and end of a stamp as UTC labels (seconds since 1970, exact).

    A boundary instant inside inserted time is reported as the label at the end of
    the insertion, marked with '*'."""
    v = 0
    for c in stamp:
        v = v * 32 + ALPHABET.index(c)
    unit = F(32) ** (8 - len(stamp))
    out = []
    for t in (v * unit - BIAS, (v + 1) * unit - BIAS):
        T = t + EPOCH + 32                  # TAI as a Unix-style label
        found = None
        for i in range(len(PERIODS) - 1, -2, -1):
            if i < 0:
                found = (T, "")
                break
            b, f = PERIODS[i]
            a0 = f(0)                       # TAI - UTC is a + k*u, linear in u
            k = f(1) - a0
            u = (T - a0) / (1 + k)
            if u >= b:
                found = (u, "")
                break
            if T >= b + dat(b, i - 1 if i > 0 else None) and i > 0:
                found = (F(b), "*")         # inserted time before period i
                break
        out.append(found)
    return out


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
        ("1966-06-15T12:00:00.123456789", label(1966, 6, 15, 12, 0, 0, "0.123456789"), 16, "FZ0EDKA4VPC2A1FX"),
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

    # Decoder vectors from SPEC.md ('*' marks inserted time).
    for st, want in [
        ("00000000", "-15422-12-06T11:41:52Z .. -15422-12-06T11:41:53Z"),
        ("FZ3BMMKB", "01969-07-20T20:17:39.425406217Z .. 01969-07-20T20:17:40.425406187Z"),
        ("G0FZP0S4", "02017-01-01T00:00:00Z* .. 02017-01-01T00:00:00Z"),
        ("G0S7", "02026-10-24T23:46:35Z .. 02026-11-06T03:02:51Z"),
        ("G0S7G6", "02026-10-31T03:07:07Z .. 02026-10-31T03:24:11Z"),
    ]:
        (a, ma), (b, mb) = decode(st)
        check(st, f"{iso(a)}{ma} .. {iso(b)}{mb}", want)

    print(f"{failures} failure(s)")
    sys.exit(1 if failures else 0)
