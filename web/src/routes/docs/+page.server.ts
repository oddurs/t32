import { base } from '$app/paths';
import docs from '../../content/docs.md?raw';
import { renderDoc, body } from '$lib/markdown';

export function load() {
	return renderDoc(body(docs), { number: true, base });
}
