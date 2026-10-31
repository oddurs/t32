// T32 in TypeScript. A port of rust/src/lib.rs using BigInt; integer arithmetic only.
// See SPEC.md. Biased time is fixed point with 40 fractional bits (unit 2^-40 s).

export const ALPHABET = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
export const CANONICAL_LEN = 8;
export const MAX_LEN = 16;

const FRAC_BITS = 40n;
const NS = 1_000_000_000n;
const BIAS_S = 1n << 39n;
/** The TAI epoch 2000-01-01T00:00:00 as Unix seconds + TAI-UTC (a "TAI label"). */
const EPOCH_TAI_LABEL_S = 946_684_800n;
const RANGE_FX = 1n << 80n; // 2^40 s
const DAY_NS = 86_400n * NS;
const MJD_UNIX_EPOCH = 40587n;

export type T32Error =
	| 'empty'
	| 'too-long'
	| 'bad-char'
	| 'out-of-range'
	| 'bad-field'
	| 'not-a-leap-second'
	| 'skipped-second';

export class T32RangeError extends Error {
	constructor(public code: T32Error) {
		super(code);
	}
}

export interface Stamp {
	readonly value: bigint;
	readonly len: number;
}

const fail = (code: T32Error): never => {
	throw new T32RangeError(code);
};

function floorDiv(a: bigint, b: bigint): bigint {
	const q = a / b;
	return a % b !== 0n && a < 0n !== b < 0n ? q - 1n : q;
}

// ---------- stamps ----------

/** Floor a biased fixed-point time to a stamp of `len` characters. */
export function fromBiasedFx(fx: bigint, len: number): Stamp {
	if (len < 1) fail('empty');
	if (len > MAX_LEN) fail('too-long');
	if (fx < 0n || fx >= RANGE_FX) fail('out-of-range');
	return { value: fx >> BigInt(5 * (MAX_LEN - len)), len };
}

/** Lenient parse: optional t32: prefix, case-insensitive, O->0, I/L->1, hyphens ignored, U rejected. */
export function parse(input: string): Stamp {
	let s = input.trim();
	if (s.slice(0, 4).toLowerCase() === 't32:') s = s.slice(4);
	let value = 0n;
	let len = 0;
	for (const c of s) {
		const u = c.toUpperCase();
		let d: number;
		if (u === '-') continue;
		else if (u === 'O') d = 0;
		else if (u === 'I' || u === 'L') d = 1;
		else {
			d = ALPHABET.indexOf(u);
			if (d < 0 || u.length !== 1) fail('bad-char');
		}
		len += 1;
		if (len > MAX_LEN) fail('too-long');
		value = value * 32n + BigInt(d);
	}
	if (len === 0) fail('empty');
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

/** Display form: groups of four separated by hyphens. Never store this. */
export function display(stamp: Stamp): string {
	return format(stamp).replace(/(.{4})(?=.)/g, '$1-');
}

export function truncate(stamp: Stamp, len: number): Stamp {
	if (len < 1 || len > stamp.len) fail('bad-field');
	return { value: stamp.value >> BigInt(5 * (stamp.len - len)), len };
}

const shift = (s: Stamp) => BigInt(5 * (MAX_LEN - s.len));
/** Inclusive start of the interval, biased fixed point. */
export const startFx = (s: Stamp): bigint => s.value << shift(s);
/** Exclusive end of the interval, biased fixed point. */
export const endFx = (s: Stamp): bigint => (s.value + 1n) << shift(s);

export function contains(a: Stamp, b: Stamp): boolean {
	return b.len >= a.len && truncate(b, a.len).value === a.value;
}

/** Unit of a stamp of length n, in seconds, as a float for display. */
export const unitSeconds = (len: number): number => 2 ** (5 * (CANONICAL_LEN - len));

// ---------- TAI - UTC ----------

/** 1972 onward: [Unix second the value takes effect, TAI-UTC in seconds]. */
export const LEAP_TABLE: ReadonlyArray<readonly [bigint, bigint]> = (
	[
		[63072000, 10], [78796800, 11], [94694400, 12], [126230400, 13], [157766400, 14],
		[189302400, 15], [220924800, 16], [252460800, 17], [283996800, 18], [315532800, 19],
		[362793600, 20], [394329600, 21], [425865600, 22], [489024000, 23], [567993600, 24],
		[631152000, 25], [662688000, 26], [709948800, 27], [741484800, 28], [773020800, 29],
		[820454400, 30], [867715200, 31], [915148800, 32], [1136073600, 33], [1230768000, 34],
		[1341100800, 35], [1435708800, 36], [1483228800, 37]
	] as const
).map(([t, d]) => [BigInt(t), BigInt(d)] as const);

/** Unix second after which LEAP_TABLE is not known to be valid (2027-06-28). */
export const LEAP_TABLE_EXPIRES = 1_814_140_800n;

/** 1961-1971: [start MJD, offset a in 1e-7 s, reference MJD m, rate r in 1e-7 s/day]. */
const RUBBER: ReadonlyArray<readonly [bigint, bigint, bigint, bigint]> = (
	[
		[37300, 14_228_180, 37300, 12_960], [37512, 13_728_180, 37300, 12_960],
		[37665, 18_458_580, 37665, 11_232], [38334, 19_458_580, 37665, 11_232],
		[38395, 32_401_300, 38761, 12_960], [38486, 33_401_300, 38761, 12_960],
		[38639, 34_401_300, 38761, 12_960], [38761, 35_401_300, 38761, 12_960],
		[38820, 36_401_300, 38761, 12_960], [38942, 37_401_300, 38761, 12_960],
		[39004, 38_401_300, 38761, 12_960], [39126, 43_131_700, 39126, 25_920],
		[39887, 42_131_700, 39126, 25_920]
	] as const
).map((r) => r.map(BigInt) as unknown as readonly [bigint, bigint, bigint, bigint]);

function findLast<T>(xs: ReadonlyArray<T>, pred: (x: T) => boolean): T | undefined {
	for (let i = xs.length - 1; i >= 0; i--) if (pred(xs[i])) return xs[i];
	return undefined;
}

/** TAI-UTC in nanoseconds at a UTC instant given as Unix nanoseconds. */
export function deltaAtNs(unixNs: bigint): bigint {
	if (unixNs >= LEAP_TABLE[0][0] * NS) {
		return findLast(LEAP_TABLE, ([t]) => unixNs >= t * NS)![1] * NS;
	}
	if (unixNs < (RUBBER[0][0] - MJD_UNIX_EPOCH) * DAY_NS) return 0n; // before 1961: zero
	const [, a, m, r] = findLast(RUBBER, ([s]) => unixNs >= (s - MJD_UNIX_EPOCH) * DAY_NS)!;
	const sinceM = unixNs - (m - MJD_UNIX_EPOCH) * DAY_NS;
	return a * 100n + floorDiv(sinceM * r * 100n, DAY_NS);
}

/** TAI nanoseconds since the T32 epoch -> biased fixed point. */
export function taiNsToFx(taiNs: bigint): bigint {
	const total = taiNs + BIAS_S * NS;
	if (total < 0n) fail('out-of-range');
	const fx = (total << FRAC_BITS) / NS;
	if (fx >= RANGE_FX) fail('out-of-range');
	return fx;
}

/** Biased fixed point -> TAI nanoseconds since epoch (floored). */
export function fxToTaiNs(fx: bigint): bigint {
	return floorDiv(fx * NS, 1n << FRAC_BITS) - BIAS_S * NS;
}

/** Unix time (UTC) in nanoseconds -> biased fixed point. */
export function unixNsToFx(unixNs: bigint): bigint {
	return taiNsToFx(unixNs + deltaAtNs(unixNs) - EPOCH_TAI_LABEL_S * NS);
}

/**
 * Biased fixed point -> UTC as Unix nanoseconds, plus whether the instant
 * falls inside an inserted leap second (in which case unixNs is the start of
 * the following day, the Unix second that repeats).
 */
export function fxToUnixNs(fx: bigint): { unixNs: bigint; leap: boolean } {
	const label = fxToTaiNs(fx) + EPOCH_TAI_LABEL_S * NS;
	// Inside an inserted leap second: the TAI second just before a step.
	for (let i = 1; i < LEAP_TABLE.length; i++) {
		const [t, d] = LEAP_TABLE[i];
		const start = (t + d - 1n) * NS;
		if (label >= start && label < start + NS) return { unixNs: t * NS + (label - start), leap: true };
	}
	const hit = findLast(LEAP_TABLE, ([t, d]) => label >= (t + d) * NS);
	if (hit) return { unixNs: label - hit[1] * NS, leap: false };
	// Rubber era and earlier: solve u = label - deltaAt(u) by fixed-point iteration.
	let u = label;
	for (let i = 0; i < 6; i++) u = label - deltaAtNs(u);
	// The 0.107758 s inserted at 1972-01-01 maps to that instant.
	const t0 = LEAP_TABLE[0][0] * NS;
	return { unixNs: u >= t0 ? t0 : u, leap: false };
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

/**
 * Proleptic Gregorian UTC fields (astronomical years) -> biased fixed point.
 * Accepts second = 60 only during a real inserted leap second.
 */
export function utcToFx(
	y: bigint, mo: number, d: number, h: number, mi: number, s: number, ns = 0n
): bigint {
	const ints = [mo, d, h, mi, s].every(Number.isInteger);
	if (!ints || mo < 1 || mo > 12 || d < 1 || d > 31 || h < 0 || h > 23 || mi < 0 || mi > 59 ||
		s < 0 || s > 60 || ns < 0n || ns >= NS) fail('bad-field');
	const days = daysFromCivil(y, mo, d);
	const [cy, cm, cd] = civilFromDays(days);
	if (cy !== y || cm !== mo || cd !== d) fail('bad-field');
	const nextMidnight = (days + 1n) * DAY_NS;
	const step = deltaAtNs(nextMidnight) - deltaAtNs(nextMidnight - 1n);
	const atEnd = h === 23 && mi === 59;
	const base = days * DAY_NS + BigInt(h * 3600 + mi * 60 + Math.min(s, 59)) * NS + ns;
	if (s === 60) {
		if (!(atEnd && step === NS)) fail('not-a-leap-second');
		return taiNsToFx(base + deltaAtNs(base) + NS - EPOCH_TAI_LABEL_S * NS);
	}
	if (atEnd && s === 59 && step === -NS) fail('skipped-second');
	return unixNsToFx(base);
}

// ---------- conveniences for display ----------

const pad = (n: number | bigint, w: number) => String(n).padStart(w, '0');

/** Format biased fixed point as UTC, e.g. 2016-12-31T23:59:60.000Z. Astronomical years. */
export function fxToUtcString(fx: bigint, fractionDigits = 0): string {
	const { unixNs, leap } = fxToUnixNs(fx);
	let secs = floorDiv(unixNs, NS);
	const frac = unixNs - secs * NS;
	if (leap) secs -= 1n; // show as 23:59:60 of the previous day
	const days = floorDiv(secs, 86400n);
	const rem = Number(secs - days * 86400n);
	const [y, m, d] = civilFromDays(days);
	const year = y < 0n ? '-' + pad(-y, 4) : y > 9999n ? '+' + String(y) : pad(y, 4);
	const sec = leap ? 60 : rem % 60;
	let out = `${year}-${pad(m, 2)}-${pad(d, 2)}T${pad(Math.floor(rem / 3600), 2)}:${pad(Math.floor((rem % 3600) / 60), 2)}:${pad(sec, 2)}`;
	if (fractionDigits > 0) out += '.' + pad(frac, 9).slice(0, fractionDigits);
	return out + 'Z';
}

/**
 * Like fxToUtcString, but shows as many fraction digits as the instant needs
 * (at least `minDigits`), so a boundary that is not on a whole UTC second,
 * as in the 1961-1971 rubber-second era, is never shown rounded.
 */
export function fxToUtcStringExact(fx: bigint, minDigits = 0): string {
	const full = fxToUtcString(fx, 9);
	const m = /^(.*\.)(\d{9})Z$/.exec(full)!;
	let frac = m[2].replace(/0+$/, '');
	if (frac.length < minDigits) frac = frac.padEnd(minDigits, '0');
	return frac ? `${m[1]}${frac}Z` : `${m[1].slice(0, -1)}Z`;
}

/** Encode a JavaScript millisecond timestamp (Unix time) at the given length. */
export function fromUnixMs(ms: number, len = CANONICAL_LEN): Stamp {
	return fromBiasedFx(unixNsToFx(BigInt(Math.floor(ms)) * 1_000_000n), len);
}

/** Parse an ISO-8601-like UTC string, allowing second 60 and astronomical years. */
export function parseUtc(input: string): bigint {
	const m = /^\s*([+-]?\d{1,6})-(\d{2})-(\d{2})(?:[T ](\d{2}):(\d{2})(?::(\d{2})(?:\.(\d{1,9}))?)?)?\s*Z?\s*$/i.exec(input);
	if (!m) fail('bad-field');
	const [, y, mo, d, h = '0', mi = '0', s = '0', f = ''] = m!;
	return utcToFx(BigInt(y), +mo, +d, +h, +mi, +s, BigInt(f.padEnd(9, '0') || '0'));
}

/**
 * A duration in nanoseconds as a T32 duration: sign, base32 seconds, and up to
 * `fractionDigits` base32 fraction digits (floored, trailing zeros dropped).
 */
export function formatDuration(ns: bigint, fractionDigits = 2): string {
	const sign = ns < 0n ? '-' : '+';
	const abs = ns < 0n ? -ns : ns;
	let secs = abs / NS;
	let frac = abs % NS;
	let int = '';
	do {
		int = ALPHABET[Number(secs % 32n)] + int;
		secs /= 32n;
	} while (secs > 0n);
	let f = '';
	for (let i = 0; i < fractionDigits && frac > 0n; i++) {
		frac *= 32n;
		f += ALPHABET[Number(frac / NS)];
		frac %= NS;
	}
	f = f.replace(/0+$/, '');
	return sign + int + (f ? '.' + f : '');
}

export const isProvisional =(unixMs: number) => BigInt(Math.floor(unixMs / 1000)) >= LEAP_TABLE_EXPIRES;
