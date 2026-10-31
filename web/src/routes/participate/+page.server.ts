import { base } from '$app/paths';
import spec from '../../../../SPEC.md?raw';
import implementations from '../../../../IMPLEMENTATIONS.md?raw';
import governance from '../../../../GOVERNANCE.md?raw';
import { renderDoc, section, afterTitle, demote } from '$lib/markdown';

export function load() {
	const md = (src: string) => renderDoc(src, { base }).html;
	return {
		questions: md(section(spec, 'Appendix C: Open questions')),
		implementations: md(demote(afterTitle(implementations))),
		governance: md(demote(afterTitle(governance)))
	};
}
