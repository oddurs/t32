import spec from '../../../../SPEC.md?raw';
import { renderDoc, body } from '$lib/markdown';

export function load() {
	return renderDoc(body(spec), { number: true });
}
