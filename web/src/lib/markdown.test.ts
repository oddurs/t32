import { test } from 'node:test';
import assert from 'node:assert/strict';
import { renderDoc } from './markdown.ts';

test('a column of plain numbers is right-aligned; dates, stamps and text are not', () => {
	const { html } = renderDoc('| From | a (s) | Stamp |\n| --- | --- | --- |\n| 1961-01-01 | 1.4228180 | `G0` |\n| 1968-02-01 | −0.1 | `G1` |\n');
	assert.match(html, /<th class="num">a \(s\)<\/th>/);
	assert.match(html, /<td class="num">−0.1<\/td>/);
	assert.match(html, /<th>From<\/th>/);
	assert.match(html, /<th>Stamp<\/th>/);
});

test('an alignment written in the table wins', () => {
	const { html } = renderDoc('| n |\n| :-: |\n| 1 |\n');
	assert.match(html, /<td style="text-align:center">1<\/td>/);
});

test('emphasis does not stop a number being a number', () => {
	const { html } = renderDoc('| Length |\n| --- |\n| 1 |\n| **8** |\n');
	assert.match(html, /<th class="num">Length<\/th>/);
});
