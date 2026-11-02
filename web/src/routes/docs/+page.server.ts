import { resolve } from '$app/paths';

// The deploy base ('' or '/t32'), for site-absolute links in the markdown.
const base = resolve('/').replace(/\/$/, '');
import docs from '../../content/docs.md?raw';
import { renderDoc, body } from '$lib/markdown';

export function load() {
	return renderDoc(body(docs), { number: true, base });
}
