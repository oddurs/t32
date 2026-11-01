// Runs with Node's built-in test runner: `npm test`. Mirrors rust/src/lib.rs tests;
// every vector also comes out of tools/oracle.py (exact rational arithmetic).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	parse, format, display, fromFx, truncate, contains, startFx, endFx, utcToFx, parseUtc,
	fxToUtc, fxToUtcString, fxToUtcStringExact, fxToTaiNs, unixNsToFx, deltaAt, taiNsToFx, STEPS, maxLengthFor,
	fromUnixMs, formatDuration, parseDuration, durationFromNs, durationToNs, durationBetween, addDuration, isInserted,
	upperBound, maxLength, fxIsProvisional, isProvisional, T32Error
} from './t32.ts';

const NS = 1_000_000_000n;
const code = (c: string) => (e: unknown) => e instanceof T32Error && e.code === c;
const stamp = (utc: string, len: number) => format(fromFx(parseUtc(utc), len));

test('encoding vectors', () => {
	const vectors: [string, number, string][] = [
		['1999-12-31T23:59:28Z', 8, 'G0000000'],
		['1970-01-01T00:00:00Z', 8, 'FZ3S5F48'],
		['1969-07-20T20:17:40Z', 8, 'FZ3BMMKB'],
		['1972-01-01T00:00:00Z', 8, 'FZ5NA8WA'],
		['2016-12-31T23:59:59Z', 8, 'G0FZP0S3'],
		['2016-12-31T23:59:60Z', 8, 'G0FZP0S4'],
		['2017-01-01T00:00:00Z', 8, 'G0FZP0S5'],
		['2026-10-31T03:13:00Z', 8, 'G0S7G6B1'],
		['2026-10-31T03:13:00.500Z', 10, 'G0S7G6B1G0'],
		['2026-10-31T03:13:00.123000Z', 12, 'G0S7G6B13XYE'],
		['1966-06-15T12:00:00.123456789Z', 14, 'FZ0EDKA4VPC2A1'],
		['1675-08-20T00:00:00Z', 8, 'FPEY6N00']
	];
	for (const [utc, len, want] of vectors) assert.equal(stamp(utc, len), want, utc);
});

test('decoding vectors', () => {
	const vectors: [string, string, string][] = [
		['00000000', '-15422-12-06T11:41:52Z', '-15422-12-06T11:41:53Z'],
		['FZ3BMMKB', '1969-07-20T20:17:39.425406217Z', '1969-07-20T20:17:40.425406187Z'],
		['G0FZP0S4', '2016-12-31T23:59:60Z', '2017-01-01T00:00:00Z'],
		['G0S7', '2026-10-24T23:46:35Z', '2026-11-06T03:02:51Z'],
		['G0S7G6', '2026-10-31T03:07:07Z', '2026-10-31T03:24:11Z']
	];
	for (const [s, start, end] of vectors) {
		const st = parse(s);
		assert.equal(fxToUtcStringExact(startFx(st)), start, s);
		assert.equal(fxToUtcStringExact(endFx(st)), end, s);
	}
	assert.equal(fxToUtcString(parseUtc('2016-12-31T23:59:60.25Z'), 2), '2016-12-31T23:59:60.25Z');
});

test('a Unix value never names the leap second', () => {
	const midnight = parseUtc('2017-01-01T00:00:00Z');
	assert.equal(unixNsToFx(1_483_228_800n * NS), midnight);
	const u = fxToUtc(parseUtc('2016-12-31T23:59:60.25Z'));
	assert.deepEqual(u, { unixNs: 1_483_228_800n * NS, insertedNs: 250_000_000n, leapSecond: true });
	assert.deepEqual(fxToUtc(midnight), { unixNs: 1_483_228_800n * NS, insertedNs: null, leapSecond: false });
	assert.throws(() => parseUtc('2016-12-30T23:59:60Z'), code('not_a_leap_second'));
});

test('removed labels are rejected', () => {
	// Removed labels are those whose TAI instant reaches the next period's start.
	assert.throws(() => parseUtc('1961-07-31T23:59:59.950000001Z'), code('removed_time'));
	assert.doesNotThrow(() => parseUtc('1961-07-31T23:59:59.95Z'));
	assert.throws(() => parseUtc('1968-01-31T23:59:59.900000003Z'), code('removed_time'));
	assert.doesNotThrow(() => parseUtc('1968-01-31T23:59:59.900000002Z'));
	// No gap: every TAI instant near the steps decodes to a label that converts back.
	for (const label of ['1961-08-01T00:00:00Z', '1968-02-01T00:00:00Z']) {
		const fx0 = parseUtc(label);
		for (let k = -200n; k < 200n; k++) {
			const u = fxToUtc(fx0 + (k << 40n) / 1_000_000_000n);
			assert.equal(u.insertedNs, null);
			assert.doesNotThrow(() => unixNsToFx(u.unixNs), String(k));
		}
	}
});

test('UTC to TAI is monotonic at every step', () => {
	assert.equal(STEPS.length, 38); // 1961-01-01, 9 fractional steps to 1968, 0.107758 s at 1972, 27 leap seconds
	for (const { unixNs: b } of STEPS) {
		let last: bigint | null = null;
		for (let u = b - 2n * NS; u <= b + 2n * NS; u += 10_000_000n) {
			let fx: bigint;
			try {
				fx = unixNsToFx(u);
			} catch {
				continue;
			}
			if (last !== null) assert.ok(fx > last, `boundary ${b}, label ${u}`);
			last = fx;
		}
	}
});

test('decoding inverts encoding, and inserted time decodes to its end', () => {
	for (let u = -400_000_000n * NS; u < 1_900_000_000n * NS; u += 7_777_777_777_777n) {
		let fx: bigint;
		try {
			fx = unixNsToFx(u);
		} catch {
			continue;
		}
		const back = fxToUtc(fx);
		assert.equal(back.insertedNs, null);
		assert.equal(back.unixNs, u);
	}
	const midnight1972 = 63_072_000n * NS;
	const inside = unixNsToFx(midnight1972) - (1n << 40n) / 20n; // 50 ms before, inside the 0.107758 s
	const utc = fxToUtc(inside);
	assert.equal(utc.unixNs, midnight1972);
	assert.ok(utc.insertedNs !== null && !utc.leapSecond);
	assert.equal(deltaAt(0n).num * 1_000_000n, 8_000_082n * deltaAt(0n).den);
});

test('nanosecond round trips are exact', () => {
	for (let n = -2000n; n < 2000n; n++) {
		assert.equal(fxToUtc(unixNsToFx(n)).unixNs, n);
		assert.equal(fxToTaiNs(taiNsToFx(n)), n);
	}
});

test('truncation equals coarse encoding', () => {
	const fx = parseUtc('2026-10-31T03:13:00.123456789Z');
	const full = fromFx(fx, 16);
	for (let n = 1; n <= 16; n++) assert.deepEqual(truncate(full, n), fromFx(fx, n));
});

test('parsing', () => {
	assert.equal(format(parse('g0s7-g6bi')), 'G0S7G6B1');
	assert.equal(format(parse('oooo')), '0000');
	assert.equal(format(parse('G0S7G6B1-3XYE')), 'G0S7G6B13XYE');
	assert.throws(() => parse('G0SU'), code('invalid_char'));
	assert.throws(() => parse(''), code('empty'));
	assert.throws(() => parse('-1W'), code('misplaced_hyphen'));
	assert.throws(() => parse('1W-'), code('misplaced_hyphen'));
	assert.throws(() => parse('G0S7--G6B1'), code('misplaced_hyphen'));
	assert.throws(() => parse(' G0S7'), code('invalid_char'));
	assert.throws(() => parse('ı'), code('invalid_char'));
	assert.throws(() => parse('ſ'), code('invalid_char'));
	assert.throws(() => parse('G0S7G6B13XYE00000'), code('too_long'));
	assert.throws(() => parse('t32:G0S7'), code('invalid_char'));
	assert.equal(display(parse('G0S7G6B13XYE')), 'G0S7-G6B1-3XYE');
});

test('resolution limits', () => {
	assert.equal(maxLengthFor(1), 8);
	assert.equal(maxLengthFor(1e-3), 10);
	assert.equal(maxLengthFor(1e-6), 12);
	assert.equal(maxLengthFor(1e-9), 14);
	assert.equal(maxLengthFor(0.1), 9);
	assert.throws(() => fromUnixMs(0, 11), code('too_fine'));
	// Exact near a midpoint between lengths, where floating-point logarithms decide wrongly.
	const midpoint = 2 ** -7.5; // between the units of lengths 9 and 10
	assert.equal(maxLengthFor(midpoint * (1 + 1e-15)), 9);
	assert.equal(maxLengthFor(midpoint * (1 - 1e-15)), 10);
	assert.equal(maxLengthFor(-1e-3), 10, "a resolution's sign is ignored");
});

test('nanosecond counts survive the round trip through a duration', () => {
	for (let n = -3000n; n <= 3000n; n++) assert.equal(durationToNs(durationFromNs(n)), n);
	const a = unixNsToFx(1_700_000_000n * NS);
	for (const gap of [1n, 7n, 999n, 1_000_000_007n]) assert.equal(durationToNs(unixNsToFx(1_700_000_000n * NS + gap) - a), gap);
});

test('arguments are checked, not wrapped', () => {
	assert.throws(() => format({ value: 32n ** 8n, len: 8 }), code('out_of_range'));
	assert.throws(() => format({ value: 0n, len: 2.5 }), code('invalid_length'));
	assert.throws(() => fxToUtc(-1n), code('out_of_range'));
	assert.throws(() => fxToUtc(1n << 80n), code('out_of_range'));
	assert.throws(() => fromUnixMs(NaN), code('out_of_range'));
	assert.throws(() => truncate(parse('G0S7'), 2.5), code('invalid_length'));
	assert.throws(() => fromFx(0n, 0), code('invalid_length'));
	assert.throws(() => maxLengthFor(NaN), code('out_of_range'));
});

test('durations are exact', () => {
	const s = (x: bigint) => formatDuration(durationFromNs(x * NS));
	assert.equal(s(60n), '+1W');
	assert.equal(s(3600n), '+3GG');
	assert.equal(s(86_400n), '+2MC0');
	assert.equal(s(604_800n), '+JEM0');
	assert.equal(formatDuration(durationFromNs(NS / 2n)), '+0.G');
	assert.equal(formatDuration(durationFromNs((-3n * NS) / 2n)), '-1.G');
	assert.equal(formatDuration(0n), '+0');
	assert.equal(formatDuration(durationFromNs(-1n)), '-0.0000012B');
	for (const t of ['+0', '+2MC0', '-1.G', '+0.0000012B', '+ZZZZZZZZ.ZZZZZZZZ']) assert.equal(formatDuration(parseDuration(t)), t);
	assert.throws(() => parseDuration('1W'), code('missing_sign'));
	const a = parse('G0S7');
	const b = parse('G0S7G6B1');
	assert.equal(formatDuration(durationBetween(a, b)), '+G6B1');
	assert.equal(format(addDuration(a, parseDuration('+10000'))), 'G0S8');
	assert.throws(() => addDuration(parse('ZZZZZZZZ'), parseDuration('+1')), code('out_of_range'));
});

test('text order is time order', () => {
	const items = Array.from({ length: 2000 }, (_, i) => {
		const s = fromFx(taiNsToFx(BigInt(i - 1000) * 977_000_000_123n), (i % 16) + 1);
		return { key: startFx(s) * 32n + BigInt(s.len), text: format(s) };
	});
	const byTime = [...items].sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0)).map((x) => x.text);
	assert.deepEqual([...items].map((x) => x.text).sort(), byTime);
});

test('containment is prefix; range limits', () => {
	assert.ok(contains(parse('G0S7'), parse('G0S7G6B1')));
	assert.ok(!contains(parse('G0S7'), parse('G0S8')));
	assert.equal(format(fromFx(0n, 8)), '00000000');
	assert.throws(() => fromFx(1n << 80n, 8), code('out_of_range'));
	assert.throws(() => taiNsToFx(-(1n << 39n) * NS - 1n), code('out_of_range'));
	assert.throws(() => utcToFx(5_443_752_762_932_606n, 1, 1, 0, 0, 0), code('out_of_range'));
	assert.throws(() => unixNsToFx(1n << 120n), code('out_of_range'));
	assert.throws(() => durationFromNs(1n << 100n), code('out_of_range'));
	assert.throws(() => formatDuration(1n << 81n), code('out_of_range'));
});

// ---------------------------------------------------------------- shared vectors

import vectors from '../../../vectors/t32-vectors.json' with { type: 'json' };

type Row = Record<string, unknown> & { error?: string };
const run = <T>(f: () => T): { ok: T } | { error: string } => {
	try {
		return { ok: f() };
	} catch (e) {
		if (e instanceof T32Error) return { error: e.code };
		throw e;
	}
};
const want = (row: Row, field: string) => ('error' in row ? { error: row.error } : { ok: row[field] });

test('conformance vectors: parse', () => {
	for (const row of vectors.parse) assert.deepEqual(run(() => format(parse(row.input))), want(row, 'stamp'), row.input);
});

test('conformance vectors: encode from UTC, Unix and TAI', () => {
	for (const row of vectors.encode_utc as Row[]) {
		const utc = row.utc as string;
		assert.deepEqual(run(() => format(fromFx(parseUtc(utc), row.length as number))), want(row, 'stamp'), utc);
		if ('provisional' in row) {
			const fx = parseUtc(utc);
			const label = fxToUtc(fx);
			assert.equal(label.leapSecond ? false : isProvisional(label.unixNs), row.provisional, utc);
		}
	}
	for (const row of vectors.encode_unix as Row[]) {
		const n = BigInt(row.unix_nanos as string);
		assert.deepEqual(run(() => format(fromFx(unixNsToFx(n), row.length as number))), want(row, 'stamp'), String(n));
		if ('provisional' in row) assert.equal(isProvisional(n), row.provisional, String(n));
	}
	for (const row of vectors.encode_tai as Row[])
		assert.deepEqual(run(() => format(fromFx(BigInt(row.units as string), row.length as number))), want(row, 'stamp'));
	for (const row of vectors.utc_errors) assert.deepEqual(run(() => parseUtc(row.utc)), want(row, ''), row.utc);
});

test('conformance vectors: decode', () => {
	for (const row of vectors.decode as Row[]) {
		const s = parse(row.stamp as string);
		const id = row.stamp as string;
		assert.equal(startFx(s), BigInt(row.start_units as string), id);
		assert.equal(fxToUtcStringExact(startFx(s)), row.start_utc, id);
		assert.equal(isInserted(startFx(s)), row.start_inserted, id);
		assert.equal(fxIsProvisional(startFx(s)), row.start_provisional, id);
		const end = endFx(s);
		if (row.end_utc === null) assert.equal(end, 1n << 80n, id);
		else {
			assert.equal(fxToUtcStringExact(end), row.end_utc, id);
			assert.equal(isInserted(end), row.end_inserted, id);
			assert.equal(fxIsProvisional(end), row.end_provisional, id);
		}
	}
});

test('conformance vectors: durations', () => {
	for (const row of vectors.durations as Row[]) {
		if (typeof row.text !== 'string') {
			assert.deepEqual(run(() => durationFromNs(BigInt(row.nanos as string))), want(row, ''));
		} else if (row.error) {
			assert.deepEqual(run(() => parseDuration(row.text as string)), want(row, ''), row.text);
		} else if (row.canonical) {
			assert.equal(formatDuration(parseDuration(row.text)), row.canonical);
		} else {
			assert.equal(formatDuration(BigInt(row.units as string)), row.text);
			assert.equal(parseDuration(row.text), BigInt(row.units as string));
			if (row.nanos) assert.equal(durationFromNs(BigInt(row.nanos as string)), BigInt(row.units as string));
		}
	}
});

test('conformance vectors: binary, bounds, resolution, arithmetic', () => {
	for (const row of vectors.binary) {
		const s = parse(row.stamp);
		assert.equal(s.value.toString(16).padStart(2 * Math.ceil((5 * s.len) / 8), '0'), row.bytes, row.stamp);
	}
	for (const row of vectors.upper_bound) {
		const b = upperBound(parse(row.prefix));
		assert.equal(b && format(b), row.bound, row.prefix);
	}
	for (const row of vectors.max_length) assert.equal(maxLength(BigInt(row.units)), row.length, row.units);
	for (const row of vectors.add as Row[]) {
		const got = run(() => format(addDuration(parse(row.stamp as string), parseDuration(row.duration as string))));
		assert.deepEqual(got, want(row, 'result'), `${row.stamp} ${row.duration}`);
	}
	for (const row of vectors.difference)
		assert.equal(formatDuration(durationBetween(parse(row.a), parse(row.b))), row.b_minus_a, `${row.b} - ${row.a}`);
});
