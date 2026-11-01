// Helpers that turn stamps into human-readable rows for the playground and tables.
import {
	CANONICAL_LEN, MAX_LEN, parse, fromFx, format, display, startFx, endFx, fxToUtc,
	fxToUtcStringExact, deltaAt, unitSeconds, maxLengthFor, isProvisional, unixNsToFx, utcToFx,
	daysFromCivil, civilFromDays, T32Error, type Stamp
} from './t32';

// A UTC date, to the day, minute, second or a fraction, with an optional Z or numeric offset.
const DATE_RE =
	/^([+-]?\d{1,6})-(\d{2})-(\d{2})(?:[Tt ](\d{2}):(\d{2})(?::(\d{2})(?:\.(\d{1,9}))?)?)?\s*(?:([Zz])|([+-])(\d{2}):?(\d{2}))?$/;
// Unix time, written as in `date -d @…`: seconds with an optional fraction.
const UNIX_RE = /^@\s*([+-]?\d{1,16})(?:\.(\d{1,9}))?$/;

// A date or "now" can also be valid stamp text: "now" is N0W and 2026-10-31 is
// 20261031. Such input is read as what it looks like, and alsoStamp carries the
// stamp reading so the page can offer it.
export type Input =
	| { kind: 'stamp'; stamp: Stamp; notADate?: true }
	| { kind: 'date'; fx: bigint; maxLen: number; precision: string; unix: boolean; alsoStamp: Stamp | null }
	| { kind: 'now'; alsoStamp: Stamp | null };

function precisionOf(fractionDigits: number | null, unit: 'day' | 'minute' | 'second') {
	if (fractionDigits === null) {
		const seconds = { day: 86400, minute: 60, second: 1 }[unit];
		return { maxLen: maxLengthFor(seconds), precision: `the ${unit}` };
	}
	const name = ['', 'a tenth of a second', 'a hundredth of a second', 'the millisecond', '', '', 'the microsecond', '', '', 'the nanosecond'][fractionDigits];
	return { maxLen: maxLengthFor(10 ** -fractionDigits), precision: name || `${fractionDigits} fraction digits` };
}

const fail = (code: 'invalid_date'): never => {
	throw new T32Error(code);
};

const stampOrNull = (t: string): Stamp | null => {
	try {
		return parse(t);
	} catch {
		return null;
	}
};

/**
 * Accepts a T32 stamp; a UTC date such as 2016-12-31T23:59:60Z, to the day,
 * minute, second or a fraction, with a Z or a numeric offset such as +02:00;
 * Unix time as @1793416380.5; or "now". Surrounding whitespace is stripped
 * here, by the caller, as the spec asks.
 */
export function readInput(text: string): Input {
	// ASCII whitespace only: the spec has callers strip whitespace, not other characters.
	const t = text.replace(/^[ \t\n\r]+|[ \t\n\r]+$/g, '');
	const asStamp = stampOrNull(t);
	if (/^now$/i.test(t)) return { kind: 'now', alsoStamp: asStamp };

	const u = UNIX_RE.exec(t);
	if (u) {
		const [, sec, frac] = u;
		const ns = BigInt(sec) * 1_000_000_000n + (sec.startsWith('-') ? -1n : 1n) * BigInt((frac ?? '').padEnd(9, '0') || '0');
		return { kind: 'date', fx: unixNsToFx(ns), unix: true, alsoStamp: null, ...precisionOf(frac ? frac.length : null, 'second') };
	}

	const m = DATE_RE.exec(t);
	if (m) {
		try {
			return readDate(m, asStamp);
		} catch (e) {
			// Not a real date, but valid stamp text (1234-56-78 is 12345678): a stamp.
			if (asStamp) return { kind: 'stamp', stamp: asStamp, notADate: true };
			throw e;
		}
	}

	// Dates that almost match: a date error, unless the text is also a valid stamp.
	if (/^[+-]?\d{1,6}-\d{1,2}-\d/.test(t) || t.startsWith('@')) {
		if (asStamp) return { kind: 'stamp', stamp: asStamp, notADate: true };
		fail('invalid_date');
	}
	return { kind: 'stamp', stamp: parse(t) };
}

function readDate(m: RegExpExecArray, alsoStamp: Stamp | null): Input {
	const [, ys, mos, ds, hs, mis, ss, frac, , sign, oh, om] = m;
	let y = BigInt(ys), mo = +mos, d = +ds, h = hs ? +hs : 0, mi = mis ? +mis : 0;
	const s = ss ? +ss : 0;
	if (sign) {
		// An offset moves the label to UTC. Shift hours and minutes only, so that
		// second 60 survives the move: 00:59:60+01:00 is 23:59:60Z.
		if (hs === undefined || +oh > 23 || +om > 59) fail('invalid_date');
		const [cy, cm, cd] = civilFromDays(daysFromCivil(y, mo, d));
		if (cy !== y || cm !== mo || cd !== d || h > 23 || mi > 59) fail('invalid_date');
		const offset = (sign === '-' ? -1n : 1n) * BigInt(+oh * 60 + +om);
		const total = daysFromCivil(y, mo, d) * 1440n + BigInt(h * 60 + mi) - offset;
		const days = total >= 0n ? total / 1440n : -((-total + 1439n) / 1440n);
		const minute = Number(total - days * 1440n);
		[y, mo, d] = civilFromDays(days);
		h = Math.floor(minute / 60);
		mi = minute % 60;
	}
	const fx = utcToFx(y, mo, d, h, mi, s, BigInt((frac ?? '').padEnd(9, '0') || '0'));
	const unit = hs === undefined ? 'day' : ss === undefined ? 'minute' : 'second';
	return { kind: 'date', fx, unix: false, alsoStamp, ...precisionOf(frac ? frac.length : null, unit) };
}

export function errorText(e: unknown): string {
	const code = e instanceof T32Error ? e.code : 'invalid_date';
	return {
		empty: 'Enter a stamp or a date.',
		too_long: `Stamps are at most ${MAX_LEN} characters.`,
		invalid_char: 'Not a valid stamp: use 0–9 and A–Z except U. (A date looks like 2026-10-31T03:13:00.)',
		misplaced_hyphen: 'A hyphen may only stand between two characters, as in G0S7-G6B1.',
		missing_sign: 'A duration starts with + or −.',
		out_of_range: 'Outside the range T32 can represent (15,423 BCE to 19,421 CE).',
		invalid_date: 'Not a valid date. Write YYYY-MM-DD, optionally with Thh:mm, seconds, a fraction and Z or an offset such as +02:00; or Unix time as @1793416380.',
		not_a_leap_second: 'There was no leap second at that moment, so second 60 does not exist.',
		removed_time: 'UTC skipped that moment: TAI − UTC fell here, so this label never occurred.',
		invalid_length: `A stamp has 1 to ${MAX_LEN} characters.`,
		too_fine: 'That length is finer than the source can resolve.'
	}[code];
}

/** "32 seconds", "≈ 17.1 minutes", "≈ 0.977 ms", … */
export function humanSpan(seconds: number): string {
	const fmt = (v: number, unit: string) => {
		const exact = Number.isInteger(v);
		const s =
			exact || v >= 100
				? Math.round(v).toLocaleString('en-US')
				: v.toPrecision(3).replace(/(\.\d*?)0+$/, '$1').replace(/\.$/, '');
		return `${exact ? '' : '≈ '}${s} ${unit}`;
	};
	// Below two years, days read better: the spec calls 2^25 s "388 days", not "1.06 years".
	if (seconds >= 2 * 365.2425 * 86400) return fmt(seconds / (365.2425 * 86400), 'years');
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
export const digitsFor = (len: number): number =>
	Math.max(0, Math.min(9, Math.ceil((len - CANONICAL_LEN) * 1.5052)));

/** Nanoseconds as decimal seconds, e.g. "−1,234.5". */
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
	inserted: boolean;
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
	const utc = fxToUtc(s);
	const dat = deltaAt(utc.unixNs);
	// Floored, not truncated: a start before 1970 must not move later.
	const ms = Number(utc.unixNs >= 0n ? utc.unixNs / 1_000_000n : -((-utc.unixNs + 999_999n) / 1_000_000n));
	return {
		stamp,
		canonical: format(stamp),
		display: display(stamp),
		unitExact: exactUnit(stamp.len),
		unitHuman: humanSpan(unitSeconds(stamp.len)),
		start: fxToUtcStringExact(s, digits),
		// The last stamps end at 2^80, past the range: a bound with no UTC reading of its own.
		end: e === 1n << 80n ? "the end of T32's range" : fxToUtcStringExact(e, digits),
		mid: fxToUtcStringExact((s + e) / 2n, Math.min(9, digits + 2)),
		inserted: utc.insertedNs !== null,
		startUnixMs: Math.abs(ms) <= 8.64e15 ? ms : null,
		unixStart: secondsString(utc.unixNs, 9, false),
		taiMinusUtc: secondsString((dat.num * 1_000_000_000n) / dat.den, 9) + ' s',
		provisional: isProvisional(utc.unixNs),
		valueHex: '0x' + stamp.value.toString(16).toUpperCase(),
		valueDec: stamp.value.toLocaleString('en-US')
	};
}

export const encode = (fx: bigint, len: number): Stamp => fromFx(fx, len);
