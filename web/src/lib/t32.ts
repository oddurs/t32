// T32 in TypeScript: a single file with no dependencies. A port of rust/src/lib.rs
// (informative; SPEC.md and its test vectors are normative), using BigInt.
//
// Three exact representations:
// - fx: biased time in units of 2^-40 s (what stamps and durations are made of);
// - scaled TAI: TAI seconds since the epoch times D, where every TAI - UTC value
//   in Appendix A is an integer multiple of 1/D s, so UTC conversion never rounds;
// - Unix nanoseconds: UTC labels as the POSIX formula counts them.
//
// Rounding happens once, at the end, always toward negative infinity.

export const ALPHABET = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
export const CANONICAL_LEN = 8;
export const MAX_LEN = 16;

const FRAC_BITS = 40n;
const FRAC_MASK = (1n << FRAC_BITS) - 1n;
const RANGE_FX = 1n << 80n;
const BIAS_S = 1n << 39n;
const NS = 1_000_000_000n;
const DAY_NS = 86_400n * NS;
/** Denominator of exact TAI - UTC values (rubber-era rates are in 1e-7 s per day). */
const D = 10_000_000n * DAY_NS;
const K = D / NS;
/** The epoch 2000-01-01T00:00:00 TAI as a "TAI label": Unix seconds + TAI - UTC. */
const EPOCH_TAI_LABEL_S = 946_684_800n;
const MJD_UNIX_EPOCH = 40_587n;
/** Comfortably beyond the representable range (2^39 s plus the epoch offset). */
const MAX_LABEL_NS = 600_000_000_000n * NS;
/** Durations stay below 2^40 s in magnitude: 2^80 units of 2^-40 s. */
const MAX_DURATION = 1n << 80n;

export type T32ErrorCode =
	| 'empty'
	| 'too-long'
	| 'bad-char'
	| 'bad-hyphen'
	| 'out-of-range'
	| 'bad-field'
	| 'not-a-leap-second'
	| 'skipped-label'
	| 'too-fine';

export class T32Error extends Error {
	constructor(public code: T32ErrorCode) {
		super(code);
	}
}
const fail = (code: T32ErrorCode): never => {
	throw new T32Error(code);
};

const floorDiv = (a: bigint, b: bigint) => {
	const q = a / b;
	return a % b !== 0n && a < 0n !== b < 0n ? q - 1n : q;
};
const abs = (a: bigint) => (a < 0n ? -a : a);

// ---------------------------------------------------------------- stamps

export interface Stamp {
	readonly value: bigint;
	readonly len: number;
}

/** ASCII only: O reads as 0, I and L as 1, case-insensitive. */
function digit(ch: string): number {
	const c = ch.charCodeAt(0);
	if (ch.length !== 1 || c > 0x7f) return -1;
	const u = c >= 0x61 && c <= 0x7a ? String.fromCharCode(c - 32) : ch;
	if (u === 'O') return 0;
	if (u === 'I' || u === 'L') return 1;
	return ALPHABET.indexOf(u);
}

/** Floor a biased time to a stamp of `len` characters. */
export function fromFx(fx: bigint, len: number): Stamp {
	if (len < 1) fail('empty');
	if (len > MAX_LEN || !Number.isInteger(len)) fail('too-long');
	if (fx < 0n || fx >= RANGE_FX) fail('out-of-range');
	return { value: fx >> BigInt(5 * (MAX_LEN - len)), len };
}

/**
 * Parse the input form (SPEC.md, Text form): ASCII only, case-insensitive, O read
 * as 0 and I, L as 1, a single hyphen allowed only between two digits, at most 16
 * digits. Anything else, including whitespace, is an error.
 */
export function parse(input: string): Stamp {
	let value = 0n;
	let len = 0;
	let prevHyphen = false;
	for (const ch of input) {
		if (ch === '-') {
			if (len === 0 || prevHyphen) fail('bad-hyphen');
			prevHyphen = true;
			continue;
		}
		const d = digit(ch);
		if (d < 0) fail('bad-char');
		prevHyphen = false;
		if (++len > MAX_LEN) fail('too-long');
		value = value * 32n + BigInt(d);
	}
	if (len === 0) fail('empty');
	if (prevHyphen) fail('bad-hyphen');
	return { value, len };
}

/** Canonical text: uppercase, exactly `len` digits, no separators. */
export function format(stamp: Stamp): string {
	let v = stamp.value;
	const out: string[] = new Array(stamp.len);
	for (let i = stamp.len - 1; i >= 0; i--) {
		out[i] = ALPHABET[Number(v % 32n)];
		v /= 32n;
	}
	return out.join('');
}

/** Display form: groups of four joined by hyphens. Never store this. */
export const display = (stamp: Stamp): string => format(stamp).replace(/(.{4})(?=.)/g, '$1-');

/** The containing stamp of `len` characters (string truncation). */
export function truncate(stamp: Stamp, len: number): Stamp {
	if (len < 1 || len > stamp.len) fail('bad-field');
	return { value: stamp.value >> BigInt(5 * (stamp.len - len)), len };
}

const shift = (s: Stamp) => BigInt(5 * (MAX_LEN - s.len));
/** Inclusive start of the interval. */
export const startFx = (s: Stamp): bigint => s.value << shift(s);
/** Exclusive end of the interval (2^80 for the last stamp). */
export const endFx = (s: Stamp): bigint => (s.value + 1n) << shift(s);

/** A contains B exactly when A is a prefix of B. */
export const contains = (a: Stamp, b: Stamp): boolean =>
	b.len >= a.len && truncate(b, a.len).value === a.value;

/** Unit of a stamp of length n, in seconds (float, for display). */
export const unitSeconds = (len: number): number => 2 ** (5 * (CANONICAL_LEN - len));

/**
 * The longest length an encoder may use for a source of the given resolution
 * (seconds): the length whose unit is nearest on a log scale, ties to the shorter.
 */
export function maxLengthFor(resolutionSeconds: number): number {
	const x = -Math.log(resolutionSeconds) / Math.log(32);
	return Math.min(MAX_LEN, Math.max(1, CANONICAL_LEN + Math.ceil(x - 0.5 - 1e-9)));
}

// ---------------------------------------------------------------- durations

/** A signed duration in units of 2^-40 s. Its text form is exact. */
export type Duration = bigint;

/** B - A, start to start. Exact. */
export const durationBetween = (a: Stamp, b: Stamp): Duration => startFx(b) - startFx(a);

/** From nanoseconds, truncating toward zero. */
export function durationFromNs(ns: bigint): Duration {
	const mag = (abs(ns) << FRAC_BITS) / NS;
	if (mag >= MAX_DURATION) fail('out-of-range');
	return ns < 0n ? -mag : mag;
}

/** To nanoseconds, truncating toward zero. */
export function durationToNs(d: Duration): bigint {
	const mag = (abs(d) * NS) >> FRAC_BITS;
	return d < 0n ? -mag : mag;
}

/** Canonical: mandatory sign, no leading zeros, no trailing fraction zeros, "+0" for zero. */
export function formatDuration(d: Duration): string {
	const mag = abs(d);
	if (mag >= MAX_DURATION) fail('out-of-range');
	let int = mag >> FRAC_BITS;
	let frac = mag & FRAC_MASK;
	let i = '';
	do {
		i = ALPHABET[Number(int % 32n)] + i;
		int /= 32n;
	} while (int > 0n);
	let f = '';
	while (frac > 0n) {
		frac <<= 5n;
		f += ALPHABET[Number(frac >> FRAC_BITS)];
		frac &= FRAC_MASK;
	}
	return (d < 0n ? '-' : '+') + i + (f ? '.' + f : '');
}

/** Parse `sign 1*8digit ["." 1*8digit]`, with the same digit leniency as stamps. */
export function parseDuration(text: string): Duration {
	const sign = text[0];
	if (sign !== '+' && sign !== '-') fail('bad-field');
	const [int, frac, extra] = text.slice(1).split('.');
	if (extra !== undefined) fail('bad-char');
	const digits = (t: string) => {
		if (t.length === 0) fail('empty');
		if (t.length > 8) fail('too-long');
		let v = 0n;
		for (const ch of t) {
			const d = digit(ch);
			if (d < 0) fail('bad-char');
			v = v * 32n + BigInt(d);
		}
		return v;
	};
	let mag = digits(int) << FRAC_BITS;
	if (frac !== undefined) mag += digits(frac) << (FRAC_BITS - 5n * BigInt(frac.length));
	return sign === '-' ? -mag : mag;
}

/** Shift a stamp's start by a duration, then floor to the stamp's own length. */
export function addDuration(stamp: Stamp, d: Duration): Stamp {
	const t = startFx(stamp) + d;
	if (t < 0n) fail('out-of-range');
	return fromFx(t, stamp.len);
}

// ---------------------------------------------------------------- TAI - UTC

/** One period of Appendix A: TAI - UTC = (a * DAY_NS + (u - m) * r) / D seconds at label u. */
interface Period {
	start: bigint; // first UTC label, Unix ns
	a: bigint; // 1e-7 s
	m: bigint; // reference label, Unix ns
	r: bigint; // 1e-7 s per day
}
const mjdNs = (mjd: number) => (BigInt(mjd) - MJD_UNIX_EPOCH) * DAY_NS;
const rubber = (start: number, a: number, m: number, r: number): Period => ({
	start: mjdNs(start), a: BigInt(a), m: mjdNs(m), r: BigInt(r)
});
const whole = (unixS: number, seconds: number): Period => ({
	start: BigInt(unixS) * NS, a: BigInt(seconds) * 10_000_000n, m: 0n, r: 0n
});

/** Unix second after which the leap-second table is not known to be valid (2027-06-28). */
export const LEAP_TABLE_EXPIRES_UNIX = 1_814_140_800n;

/** Appendix A. Before the first period TAI - UTC is defined as zero. */
const PERIODS: readonly Period[] = [
	rubber(37300, 14_228_180, 37300, 12_960), rubber(37512, 13_728_180, 37300, 12_960),
	rubber(37665, 18_458_580, 37665, 11_232), rubber(38334, 19_458_580, 37665, 11_232),
	rubber(38395, 32_401_300, 38761, 12_960), rubber(38486, 33_401_300, 38761, 12_960),
	rubber(38639, 34_401_300, 38761, 12_960), rubber(38761, 35_401_300, 38761, 12_960),
	rubber(38820, 36_401_300, 38761, 12_960), rubber(38942, 37_401_300, 38761, 12_960),
	rubber(39004, 38_401_300, 38761, 12_960), rubber(39126, 43_131_700, 39126, 25_920),
	rubber(39887, 42_131_700, 39126, 25_920),
	...([
		[63072000, 10], [78796800, 11], [94694400, 12], [126230400, 13], [157766400, 14],
		[189302400, 15], [220924800, 16], [252460800, 17], [283996800, 18], [315532800, 19],
		[362793600, 20], [394329600, 21], [425865600, 22], [489024000, 23], [567993600, 24],
		[631152000, 25], [662688000, 26], [709948800, 27], [741484800, 28], [773020800, 29],
		[820454400, 30], [867715200, 31], [915148800, 32], [1136073600, 33], [1230768000, 34],
		[1341100800, 35], [1435708800, 36], [1483228800, 37]
	] as const).map(([t, d]) => whole(t, d))
];
const ZERO: Period = { start: -(1n << 120n), a: 0n, m: 0n, r: 0n };
const period = (i: number): Period => (i === 0 ? ZERO : PERIODS[i - 1]);

const delta = (p: Period, u: bigint) => p.a * DAY_NS + (u - p.m) * p.r;
const tai = (p: Period, u: bigint) => (u - EPOCH_TAI_LABEL_S * NS) * K + delta(p, u);
/** Inverse of tai(): exact, because tai(u) = u * (K + r) + c. */
const label = (p: Period, t: bigint) =>
	floorDiv(t - (-EPOCH_TAI_LABEL_S * NS * K + p.a * DAY_NS - p.m * p.r), K + p.r);

function periodIndexAt(u: bigint): number {
	for (let i = PERIODS.length; i >= 1; i--) if (u >= PERIODS[i - 1].start) return i;
	return 0;
}
/** The step in TAI - UTC at boundary i (start of period i), in 1/D s. */
const step = (i: number) => delta(period(i), period(i).start) - delta(period(i - 1), period(i).start);

/** TAI - UTC at a UTC label, as an exact fraction of seconds. */
export function deltaAt(unixNs: bigint): { num: bigint; den: bigint } {
	return { num: delta(period(periodIndexAt(unixNs)), unixNs), den: D };
}

/** The boundaries where TAI - UTC steps, with the step in seconds (float, for display). */
export const STEPS = Array.from({ length: PERIODS.length }, (_, k) => k + 1)
	.filter((i) => step(i) !== 0n)
	.map((i) => ({ unixNs: period(i).start, seconds: Number(step(i)) / Number(D), wholeSecond: step(i) === D }));

// ---------------------------------------------------------------- conversions

function scaledToFx(t: bigint): bigint {
	const total = t + BIAS_S * D;
	if (total < 0n) fail('out-of-range');
	const fx = ((total / D) << FRAC_BITS) + ((total % D) << FRAC_BITS) / D;
	if (fx >= RANGE_FX) fail('out-of-range');
	return fx;
}

const fxToScaled = (fx: bigint) => (fx >> FRAC_BITS) * D + (((fx & FRAC_MASK) * D) >> FRAC_BITS) - BIAS_S * D;

/**
 * UTC label (Unix nanoseconds, POSIX) -> biased time. Rejects labels that UTC
 * skipped (where TAI - UTC decreased). A Unix value never names 23:59:60.
 */
export function unixNsToFx(u: bigint): bigint {
	if (abs(u) > MAX_LABEL_NS) fail('out-of-range');
	const i = periodIndexAt(u);
	const t = tai(period(i), u);
	// Where TAI - UTC fell, labels whose TAI instant would reach the start of the
	// next period (in TAI) never occurred.
	if (i < PERIODS.length && step(i + 1) < 0n && t >= tai(period(i + 1), period(i + 1).start)) {
		fail('skipped-label');
	}
	return scaledToFx(t);
}

/** TAI nanoseconds since the epoch -> biased time. */
export const taiNsToFx = (taiNs: bigint): bigint =>
	abs(taiNs) > MAX_LABEL_NS ? fail('out-of-range') : scaledToFx(taiNs * K);

/** Biased time -> TAI nanoseconds since the epoch (floored). */
export const fxToTaiNs = (fx: bigint): bigint => floorDiv(fxToScaled(fx), K);

export interface Utc {
	/** The UTC label as Unix nanoseconds (floored). */
	unixNs: bigint;
	/**
	 * For an instant in time UTC inserted (a leap second, or a fractional step
	 * before 1972): nanoseconds into the insertion; unixNs is then the label at
	 * the end of the insertion, e.g. 00:00:00 after 23:59:60.
	 */
	insertedNs: bigint | null;
	/** True when the insertion is a whole leap second, written 23:59:60. */
	leapSecond: boolean;
}

/** Biased time -> UTC label. */
export function fxToUtc(fx: bigint): Utc {
	const t = fxToScaled(fx);
	let i = 0;
	for (let k = PERIODS.length; k >= 1; k--) {
		if (t >= tai(period(k), period(k).start)) {
			i = k;
			break;
		}
	}
	if (i < PERIODS.length) {
		const b = period(i + 1).start;
		const left = tai(period(i), b);
		const s = step(i + 1);
		if (s > 0n && t >= left) return { unixNs: b, insertedNs: floorDiv(t - left, K), leapSecond: s === D };
	}
	return { unixNs: label(period(i), t), insertedNs: null, leapSecond: false };
}

export function daysFromCivil(y: bigint, m: number, d: number): bigint {
	const yy = m <= 2 ? y - 1n : y;
	const era = (yy >= 0n ? yy : yy - 399n) / 400n;
	const yoe = yy - era * 400n;
	const mp = BigInt((m + 9) % 12);
	const doy = (153n * mp + 2n) / 5n + BigInt(d) - 1n;
	const doe = yoe * 365n + yoe / 4n - yoe / 100n + doy;
	return era * 146097n + doe - 719468n;
}

export function civilFromDays(days: bigint): [bigint, number, number] {
	const z = days + 719468n;
	const era = (z >= 0n ? z : z - 146096n) / 146097n;
	const doe = z - era * 146097n;
	const yoe = (doe - doe / 1460n + doe / 36524n - doe / 146096n) / 365n;
	const doy = doe - (365n * yoe + yoe / 4n - yoe / 100n);
	const mp = (5n * doy + 2n) / 153n;
	const d = Number(doy - (153n * mp + 2n) / 5n + 1n);
	const m = Number(mp < 10n ? mp + 3n : mp - 9n);
	return [yoe + era * 400n + (m <= 2 ? 1n : 0n), m, d];
}

/** Proleptic Gregorian UTC fields (astronomical years) -> biased time. */
export function utcToFx(y: bigint, mo: number, d: number, h: number, mi: number, s: number, ns = 0n): bigint {
	const ints = [mo, d, h, mi, s].every(Number.isInteger);
	if (!ints || mo < 1 || mo > 12 || d < 1 || d > 31 || h < 0 || h > 23 || mi < 0 || mi > 59 ||
		s < 0 || s > 60 || ns < 0n || ns >= NS) fail('bad-field');
	if (abs(y) > 100_000n) fail('out-of-range');
	const days = daysFromCivil(y, mo, d);
	const [cy, cm, cd] = civilFromDays(days);
	if (cy !== y || cm !== mo || cd !== d) fail('bad-field');
	const at = (sec: number) => (days * 86_400n + BigInt(h * 3600 + mi * 60 + sec)) * NS + ns;
	if (s === 60) {
		const next = (days + 1n) * DAY_NS;
		const leap = STEPS.some((x) => x.unixNs === next && x.wholeSecond && x.seconds > 0);
		if (!(leap && h === 23 && mi === 59)) fail('not-a-leap-second');
		const u = at(59);
		return scaledToFx(tai(period(periodIndexAt(u)), u) + NS * K);
	}
	return unixNsToFx(at(s));
}

// ---------------------------------------------------------------- text conveniences

const pad = (n: number | bigint, w: number) => String(n).padStart(w, '0');

function utcParts(fx: bigint): { date: string; time: string; frac: string } {
	const utc = fxToUtc(fx);
	let ns = utc.unixNs;
	let secOverride: number | null = null;
	if (utc.insertedNs !== null && utc.leapSecond) {
		ns = utc.unixNs - NS + utc.insertedNs; // 23:59:59.x of the day before, shown as :60
		secOverride = 60;
	}
	const secs = floorDiv(ns, NS);
	const frac = ns - secs * NS;
	const days = floorDiv(secs, 86400n);
	const rem = Number(secs - days * 86400n);
	const [y, m, d] = civilFromDays(days);
	const year = y < 0n ? '-' + pad(-y, 4) : y > 9999n ? '+' + String(y) : pad(y, 4);
	const sec = secOverride ?? rem % 60;
	return {
		date: `${year}-${pad(m, 2)}-${pad(d, 2)}`,
		time: `${pad(Math.floor(rem / 3600), 2)}:${pad(Math.floor((rem % 3600) / 60), 2)}:${pad(sec, 2)}`,
		frac: pad(frac, 9)
	};
}

/** UTC as ISO 8601 with a fixed number of fraction digits (truncated). Astronomical years. */
export function fxToUtcString(fx: bigint, fractionDigits = 0): string {
	const p = utcParts(fx);
	return `${p.date}T${p.time}${fractionDigits > 0 ? '.' + p.frac.slice(0, fractionDigits) : ''}Z`;
}

/**
 * UTC with as many fraction digits as the instant needs (at least minDigits), so
 * a boundary off a whole UTC second (as before 1972) is never shown rounded.
 */
export function fxToUtcStringExact(fx: bigint, minDigits = 0): string {
	const p = utcParts(fx);
	let f = p.frac.replace(/0+$/, '');
	if (f.length < minDigits) f = f.padEnd(minDigits, '0');
	return `${p.date}T${p.time}${f ? '.' + f : ''}Z`;
}

/** Encode a JavaScript millisecond timestamp (Unix time). Lengths above 10 overstate a 1 ms source. */
export function fromUnixMs(ms: number, len = CANONICAL_LEN): Stamp {
	if (len > maxLengthFor(1e-3)) fail('too-fine');
	return fromFx(unixNsToFx(BigInt(Math.floor(ms)) * 1_000_000n), len);
}

/** Parse an ISO 8601 UTC string (astronomical years, second 60, up to 9 fraction digits). */
export function parseUtc(input: string): bigint {
	const m = /^([+-]?\d{1,6})-(\d{2})-(\d{2})(?:[Tt ](\d{2}):(\d{2})(?::(\d{2})(?:\.(\d{1,9}))?)?)?[Zz]?$/.exec(input);
	if (!m) fail('bad-field');
	const [, y, mo, d, h = '0', mi = '0', s = '0', f = ''] = m!;
	return utcToFx(BigInt(y), +mo, +d, +h, +mi, +s, BigInt(f.padEnd(9, '0') || '0'));
}

export const isProvisional = (unixNs: bigint): boolean => unixNs / NS >= LEAP_TABLE_EXPIRES_UNIX;
