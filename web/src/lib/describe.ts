// Helpers that turn stamps into human-readable rows for the playground and tables.
import {
	CANONICAL_LEN, MAX_LEN, LEAP_TABLE_EXPIRES, parse, parseUtc, fromBiasedFx, format, display,
	startFx, endFx, fxToUnixNs, fxToUtcStringExact, deltaAtNs, unitSeconds, T32RangeError, type Stamp
} from './t32';

const DATE_RE = /^\s*[+-]?\d{1,6}-\d{2}-\d{2}/;

export type Input =
	| { kind: 'stamp'; stamp: Stamp }
	| { kind: 'date'; fx: bigint; fractionDigits: number };

/** Accepts either a T32 stamp or a UTC date such as 2016-12-31T23:59:60. */
export function readInput(text: string): Input {
	if (DATE_RE.test(text)) {
		const frac = /\.(\d+)/.exec(text)?.[1]?.length ?? 0;
		return { kind: 'date', fx: parseUtc(text), fractionDigits: frac };
	}
	return { kind: 'stamp', stamp: parse(text) };
}

/** The length implied by a date's written precision: seconds → 8, ms → 10, µs → 12, ns → 14. */
export function lengthForDate(fractionDigits: number): number {
	if (fractionDigits === 0) return CANONICAL_LEN;
	if (fractionDigits <= 3) return 10;
	if (fractionDigits <= 6) return 12;
	return 14;
}

export function errorText(e: unknown): string {
	const code = e instanceof T32RangeError ? e.code : 'bad-field';
	return {
		empty: 'Enter a stamp or a date.',
		'too-long': `Stamps are at most ${MAX_LEN} characters.`,
		'bad-char': 'Not a valid stamp: use 0–9 and A–Z except U. (A date must look like 2026-10-31T03:13:00.)',
		'out-of-range': 'Outside the range T32 can represent (about 15,420 BCE to 19,420 CE).',
		'bad-field': 'Not a valid date. Use YYYY-MM-DD, optionally with Thh:mm:ss and a fraction.',
		'not-a-leap-second': 'There was no leap second at that moment, so second 60 does not exist.',
		'skipped-second': 'That second was removed by a negative leap second.'
	}[code];
}

/** "32 seconds", "≈ 17.1 minutes", "≈ 0.977 ms", … */
export function humanSpan(seconds: number): string {
	const fmt = (v: number, unit: string) => {
		const exact = Number.isInteger(v);
		const s = exact || v >= 100
			? Math.round(v).toLocaleString('en-US')
			: v.toPrecision(3).replace(/(\.\d*?)0+$/, '$1').replace(/\.$/, '');
		return `${exact ? '' : '≈ '}${s} ${unit}`;
	};
	if (seconds >= 365.2425 * 86400) return fmt(seconds / (365.2425 * 86400), 'years');
	if (seconds >= 86400) return fmt(seconds / 86400, 'days');
	if (seconds >= 3600) return fmt(seconds / 3600, 'hours');
	if (seconds >= 60) return fmt(seconds / 60, 'minutes');
	if (seconds >= 1) return fmt(seconds, seconds === 1 ? 'second' : 'seconds');
	if (seconds >= 1e-3) return fmt(seconds * 1e3, 'ms');
	if (seconds >= 1e-6) return fmt(seconds * 1e6, 'µs');
	if (seconds >= 1e-9) return fmt(seconds * 1e9, 'ns');
	return fmt(seconds * 1e12, 'ps');
}

/** "2^5 s", "1 s", "2^−10 s" */
export function exactUnit(len: number): string {
	const e = 5 * (CANONICAL_LEN - len);
	return e === 0 ? '1 s' : `2^${e < 0 ? '−' + -e : e} s`;
}

/** UTC fraction digits worth showing for a stamp of this length. */
export function digitsFor(len: number): number {
	return Math.max(0, Math.min(9, Math.ceil((len - CANONICAL_LEN) * 1.5052)));
}

export function secondsString(ns: bigint, maxFrac = 9, group = true): string {
	const neg = ns < 0n;
	const a = neg ? -ns : ns;
	const int = (a / 1_000_000_000n).toLocaleString('en-US', { useGrouping: group });
	const frac = (a % 1_000_000_000n).toString().padStart(9, '0').slice(0, maxFrac).replace(/0+$/, '');
	return (neg ? '−' : '') + int + (frac ? '.' + frac : '');
}

export interface Description {
	stamp: Stamp;
	canonical: string;
	display: string;
	unitExact: string;
	unitHuman: string;
	start: string;
	end: string;
	mid: string;
	startUnixMs: number | null;
	unixStart: string;
	taiMinusUtc: string;
	provisional: boolean;
	valueHex: string;
	valueDec: string;
}

export function describe(stamp: Stamp): Description {
	const s = startFx(stamp);
	const e = endFx(stamp);
	const digits = digitsFor(stamp.len);
	const { unixNs } = fxToUnixNs(s);
	const ms = Number(unixNs / 1_000_000n);
	return {
		stamp,
		canonical: format(stamp),
		display: display(stamp),
		unitExact: exactUnit(stamp.len),
		unitHuman: humanSpan(unitSeconds(stamp.len)),
		start: fxToUtcStringExact(s, digits),
		end: fxToUtcStringExact(e, digits),
		mid: fxToUtcStringExact((s + e) / 2n, Math.min(9, digits + 2)),
		startUnixMs: Math.abs(ms) <= 8.64e15 ? ms : null,
		unixStart: secondsString(unixNs, 9, false),
		taiMinusUtc: secondsString(deltaAtNs(unixNs), 7) + ' s',
		provisional: unixNs / 1_000_000_000n >= LEAP_TABLE_EXPIRES,
		valueHex: '0x' + stamp.value.toString(16).toUpperCase(),
		valueDec: stamp.value.toLocaleString('en-US')
	};
}

export function encode(fx: bigint, len: number): Stamp {
	return fromBiasedFx(fx, len);
}
