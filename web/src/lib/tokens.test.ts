import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { readTokens, value, contrast, TEXT_COLOURS, GROUNDS } from './tokens.ts';

const tokens = readTokens(readFileSync(new URL('../app.css', import.meta.url), 'utf8'));

test('the type scale is 16px × 2^(n/5), to the rounding its comments state', () => {
	const steps = { m2: -2, m1: -1, '0': 0, '1': 1, '2': 2, '3': 3, '4': 4, '5': 5, '6': 6, '7': 7, '8': 8 };
	for (const [name, n] of Object.entries(steps)) {
		const rem = parseFloat(value(tokens.light, `--fs-${name}`));
		assert.ok(Math.abs(rem - 2 ** (n / 5)) < 0.0006, `--fs-${name} is ${rem}rem, not 2^(${n}/5)`);
	}
});

test('every text colour holds 4.5:1 on every ground, light and dark', () => {
	for (const [scheme, map] of Object.entries(tokens)) {
		for (const fg of TEXT_COLOURS) {
			for (const bg of GROUNDS) {
				const ratio = contrast(value(map, fg), value(map, bg));
				assert.ok(ratio >= 4.5, `${scheme}: ${fg} on ${bg} is ${ratio.toFixed(2)}:1`);
			}
		}
	}
});

test('the space scale stays on the 4px grid', () => {
	for (let i = 1; i <= 9; i++) {
		const px = parseFloat(value(tokens.light, `--s${i}`)) * 16;
		assert.equal(px % 4, 0, `--s${i} is ${px}px`);
	}
});

test('contrast matches the WCAG definition at its extremes', () => {
	assert.equal(contrast('#000000', '#ffffff'), 21);
	assert.equal(contrast('#777777', '#777777'), 1);
});

test('the drawn mark and favicon use only the stylesheet’s ink, paper and accent', () => {
	const allowed = new Set(['--ink', '--paper', '--accent'].flatMap((n) => [value(tokens.light, n), value(tokens.dark, n)]));
	for (const file of ['mark.svg', 'favicon.svg']) {
		const svg = readFileSync(new URL(`../../static/${file}`, import.meta.url), 'utf8');
		for (const [hex] of svg.matchAll(/#[0-9a-f]{6}/gi)) assert.ok(allowed.has(hex), `${file} uses ${hex}`);
	}
});
