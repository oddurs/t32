// Runs with Node's built-in test runner: `npm test` (tsx --test).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
	parse, format, display, fromBiasedFx, truncate, contains, startFx, endFx,
	utcToFx, parseUtc, fxToUtcString, fxToUnixNs, unixNsToFx, deltaAtNs, taiNsToFx, formatDuration, T32RangeError
} from './t32.ts';

const stamp = (utc: string, len: number) => format(fromBiasedFx(parseUtc(utc), len));

test('encoding vectors from SPEC.md', () => {
	const vectors: [string, number, string][] = [
		['1999-12-31T23:59:28', 8, 'G0000000'],
		['1970-01-01T00:00:00', 8, 'FZ3S5F48'],
		['1969-07-20T20:17:40', 8, 'FZ3BMMKB'],
		['1972-01-01T00:00:00', 8, 'FZ5NA8WA'],
		['2016-12-31T23:59:59', 8, 'G0FZP0S3'],
		['2016-12-31T23:59:60', 8, 'G0FZP0S4'],
		['2017-01-01T00:00:00', 8, 'G0FZP0S5'],
		['2026-10-31T03:13:00', 8, 'G0S7G6B1'],
		['2026-10-31T03:13:00.5', 10, 'G0S7G6B1G0'],
		['2026-10-31T03:13:00.123', 12, 'G0S7G6B13XYE'],
		['1675-08-20T00:00:00', 8, 'FPEY6N00']
	];
	for (const [utc, len, want] of vectors) assert.equal(stamp(utc, len), want, utc);
});

test('decoding vectors from SPEC.md', () => {
	const vectors: [string, string, string][] = [
		['00000000', '-15422-12-06T11:41:52Z', '-15422-12-06T11:41:53Z'],
		['ZZZZZZZZ', '+19421-01-25T12:17:30Z', '+19421-01-25T12:17:31Z'],
		['G', '1999-12-31T23:59:28Z', '3088-10-25T03:45:31Z'],
		['G0S7', '2026-10-24T23:46:35Z', '2026-11-06T03:02:51Z'],
		['G0S7G6', '2026-10-31T03:07:07Z', '2026-10-31T03:24:11Z']
	];
	for (const [s, start, end] of vectors) {
		const st = parse(s);
		assert.equal(fxToUtcString(startFx(st)), start, s);
		assert.equal(fxToUtcString(endFx(st)), end, s);
	}
});

test('parsing vectors from SPEC.md', () => {
	assert.equal(format(parse('g0s7-g6bi')), 'G0S7G6B1');
	assert.equal(format(parse('t32:G0S7G6B1')), 'G0S7G6B1');
	assert.equal(format(parse('oooo')), '0000');
	assert.throws(() => parse('G0SU'), (e: T32RangeError) => e.code === 'bad-char');
	assert.throws(() => parse(''), (e: T32RangeError) => e.code === 'empty');
	assert.throws(() => parse('-'), (e: T32RangeError) => e.code === 'empty');
	assert.throws(() => parseUtc('2016-12-30T23:59:60'), (e: T32RangeError) => e.code === 'not-a-leap-second');
	assert.equal(display(parse('G0S7G6B13XYE')), 'G0S7-G6B1-3XYE');
});

test('the leap second displays as 23:59:60', () => {
	const fx = parseUtc('2016-12-31T23:59:60.25');
	assert.equal(fxToUtcString(fx, 2), '2016-12-31T23:59:60.25Z');
	assert.equal(fxToUnixNs(fx).leap, true);
});

test('truncation equals coarse encoding', () => {
	const fx = utcToFx(2026n, 10, 31, 3, 13, 0, 123_456_789n);
	const full = fromBiasedFx(fx, 16);
	for (let n = 1; n <= 16; n++) assert.deepEqual(truncate(full, n), fromBiasedFx(fx, n));
});

test('text order is time order', () => {
	const items = Array.from({ length: 2000 }, (_, i) => {
		const fx = taiNsToFx(BigInt(i - 1000) * 977_000_000_123n);
		const s = fromBiasedFx(fx, (i % 16) + 1);
		return { key: startFx(s) * 32n + BigInt(s.len), text: format(s) };
	});
	const byTime = [...items].sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0)).map((x) => x.text);
	const byText = [...items].map((x) => x.text).sort();
	assert.deepEqual(byText, byTime);
});

test('containment is prefix', () => {
	assert.ok(contains(parse('G0S7'), parse('G0S7G6B1')));
	assert.ok(!contains(parse('G0S7'), parse('G0S8')));
});

test('unix round trip, including the rubber era', () => {
	assert.equal(deltaAtNs(0n), 8_000_082_000n);
	for (const u of [-5_000_000_000n, -300_000_000n, 0n, 63_071_999n, 63_072_000n, 1_483_228_799n, 1_483_228_800n, 1_793_416_380n]) {
		const back = fxToUnixNs(unixNsToFx(u * 1_000_000_000n)).unixNs;
		assert.ok(back - u * 1_000_000_000n <= 1n && u * 1_000_000_000n - back <= 1n, String(u));
	}
});

test('duration vectors from SPEC.md', () => {
	const s = 1_000_000_000n;
	assert.equal(formatDuration(60n * s), '+1W');
	assert.equal(formatDuration(3600n * s), '+3GG');
	assert.equal(formatDuration(86_400n * s), '+2MC0');
	assert.equal(formatDuration(604_800n * s), '+JEM0');
	assert.equal(formatDuration(s / 2n), '+0.G');
	assert.equal(formatDuration(0n), '+0');
	assert.equal(formatDuration(-3600n * s), '-3GG');
});

test('range limits', () => {
	assert.equal(format(fromBiasedFx(0n, 8)), '00000000');
	assert.throws(() => fromBiasedFx(1n << 80n, 8));
	assert.throws(() => taiNsToFx(-(1n << 39n) * 1_000_000_000n - 1n));
});

test('rubber-era boundaries are shown exactly', async () => {
	const { fxToUtcStringExact } = await import('./t32.ts');
	const s = parse('FZ3BMMKB');
	assert.equal(fxToUtcStringExact(startFx(s)), '1969-07-20T20:17:39.425406218Z');
	assert.equal(fxToUtcStringExact(startFx(parse('G0S7G6B1'))), '2026-10-31T03:13:00Z');
	assert.equal(fxToUtcStringExact(startFx(parse('G0S7G6B1G0')), 3), '2026-10-31T03:13:00.500Z');
});
