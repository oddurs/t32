import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readInput, humanSpan } from './describe.ts';
import { format, fromFx, parseUtc, unitSeconds } from './t32.ts';

const stampOf = (text: string, len?: number) => {
	const r = readInput(text);
	if (r.kind === 'stamp') return format(r.stamp);
	if (r.kind === 'now') return 'now';
	return format(fromFx(r.fx, len ?? r.maxLen));
};

test('dates are read to the precision written', () => {
	const len = (t: string) => {
		const r = readInput(t);
		return r.kind === 'date' ? r.maxLen : -1;
	};
	assert.equal(len('2026-10-31'), 5); // a day: nearest unit is 9.1 hours
	assert.equal(len('2026-10-31T03:13'), 7); // a minute: nearest unit is 32 s
	assert.equal(len('2026-10-31T03:13:00Z'), 8);
	assert.equal(len('2026-10-31T03:13:00.500'), 10);
	assert.equal(len('2026-10-31 03:13:00.123456789'), 14);
});

test('offsets move the label to UTC, keeping second 60', () => {
	assert.equal(stampOf('2026-10-31T05:13:00+02:00'), 'G0S7G6B1');
	assert.equal(stampOf('2026-10-30T23:13:00-04:00'), 'G0S7G6B1');
	assert.equal(stampOf('2026-10-31T08:43:00+0530'), 'G0S7G6B1');
	assert.equal(stampOf('2017-01-01T00:59:60+01:00'), 'G0FZP0S4'); // the 2016 leap second
	assert.equal(stampOf('2017-01-01T00:59:60+01:00'), format(fromFx(parseUtc('2016-12-31T23:59:60Z'), 8)));
	assert.throws(() => readInput('2026-02-30T10:00+01:00'));
	assert.throws(() => readInput('2026-10-31+01:00'));
});

test('Unix time, now, and stamps', () => {
	assert.equal(stampOf('@1793416380'), 'G0S7G6B1');
	assert.equal(stampOf('@1793416380.5'), 'G0S7G6B1G'); // a tenth of a second: length 9
	assert.equal(stampOf('@1793416380.500'), 'G0S7G6B1G0');
	assert.equal(readInput('NOW').kind, 'now');
	assert.equal(stampOf(' g0s7-g6bi '), 'G0S7G6B1');
	assert.equal(stampOf('1793416380'), '1793416380'); // digits alone are a stamp
	assert.throws(() => readInput('@abc'));
	assert.throws(() => readInput('2026-1-31'));
});

test('spans are named as the specification names them', () => {
	const spans = [1, 2, 3, 4, 5, 6, 7, 8].map((n) => humanSpan(unitSeconds(n)));
	assert.deepEqual(spans, [
		'≈ 1,089 years', '≈ 34 years', '≈ 388 days', '≈ 12.1 days',
		'≈ 9.1 hours', '≈ 17.1 minutes', '32 seconds', '1 second'
	]);
});
