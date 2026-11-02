import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { DRAFT, DRAFT_DATE, DRAFT_PDF, VECTOR_COUNT } from './site.ts';

const root = new URL('../../../', import.meta.url);

test('the site presents the current draft and vector count', () => {
	const spec = readFileSync(new URL('SPEC.md', root), 'utf8');
	assert.match(spec, new RegExp(`\\| Document \\| T32 specification, draft ${DRAFT.replace('.', '\\.')} \\|`));
	assert.match(spec, new RegExp(`\\| Date \\| ${DRAFT_DATE} \\|`));
	const vectors = JSON.parse(readFileSync(new URL('vectors/t32-vectors.json', root), 'utf8'));
	const count = Object.entries(vectors)
		.filter(([k, v]) => Array.isArray(v) && k !== 'conventions')
		.reduce((n, [, v]) => n + (v as unknown[]).length, 0);
	assert.equal(count, VECTOR_COUNT);
});

test('the PDF the site links to is the one ietf/ builds', () => {
	const source = DRAFT_PDF.replace(/\.pdf$/, '.tex');
	assert.ok(readFileSync(new URL(`ietf/${source}`, root), 'utf8').includes('\\documentclass'));
});
