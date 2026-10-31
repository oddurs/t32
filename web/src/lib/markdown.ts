// Renders a GFM document with GitHub-style heading ids, optional ISO-style
// section numbers, and a table of contents. Runs at build time only.
import { Marked, type Tokens } from 'marked';

export interface TocEntry {
	depth: number;
	id: string;
	text: string;
	label: string | null;
}

export interface RenderedDoc {
	html: string;
	toc: TocEntry[];
}

const REPO_BLOB = 'https://github.com/oddurs/t32/blob/main/';

/** GitHub's heading slug: lowercase, drop punctuation, spaces to hyphens. */
function slug(text: string): string {
	return text
		.toLowerCase()
		.replace(/<[^>]*>/g, '')
		.replace(/[^\p{L}\p{N}\s_-]/gu, '')
		.trim()
		.replace(/\s/g, '-');
}

const plain = (s: string) => s.replace(/[`*_]/g, '');
const escapeAttr = (s: string) => s.replace(/&/g, '&amp;').replace(/"/g, '&quot;');

export function renderDoc(src: string, opts: { number?: boolean; base?: string } = {}): RenderedDoc {
	const toc: TocEntry[] = [];
	const seen = new Map<string, number>();
	let section = 0;
	let subsection = 0;

	const md = new Marked({ gfm: true });
	md.use({
		renderer: {
			heading(this: { parser: { parseInline: (t: Tokens.Generic[]) => string } }, { tokens, depth, text }: Tokens.Heading) {
				const inner = this.parser.parseInline(tokens);
				const base = slug(plain(text));
				const n = seen.get(base) ?? 0;
				seen.set(base, n + 1);
				const id = n === 0 ? base : `${base}-${n}`;

				let label: string | null = null;
				if (opts.number && depth === 2) {
					if (/^appendix\b/i.test(text)) {
						subsection = 0;
					} else {
						section += 1;
						subsection = 0;
						label = String(section);
					}
				} else if (opts.number && depth === 3 && section > 0) {
					subsection += 1;
					label = `${section}.${subsection}`;
				}
				if (depth >= 2 && depth <= 3) toc.push({ depth, id, text: plain(text), label });

				const no = label ? `<span class="secno">${label}</span>` : '';
				return `<h${depth} id="${id}">${no}${inner}<a class="anchor" href="#${id}" aria-label="Link to this section">#</a></h${depth}>\n`;
			},
			link(this: { parser: { parseInline: (t: Tokens.Generic[]) => string } }, { href, title, tokens }: Tokens.Link) {
				const inner = this.parser.parseInline(tokens);
				// Site-absolute links get the deploy base; other relative links point at the repo.
				const abs = /^\/[^/]/.test(href)
					? (opts.base ?? '') + href
					: /^(https?:|mailto:|#)/.test(href)
						? href
						: REPO_BLOB + href;
				const t = title ? ` title="${escapeAttr(title)}"` : '';
				return `<a href="${escapeAttr(abs)}"${t}>${inner}</a>`;
			},
			table(this: { parser: { parseInline: (t: Tokens.Generic[]) => string } }, token: Tokens.Table) {
				// Wrap tables so wide ones scroll on phones instead of breaking the page.
				const cell = (c: Tokens.TableCell, tag: 'th' | 'td') => {
					const align = c.align ? ` style="text-align:${c.align}"` : '';
					return `<${tag}${align}>${this.parser.parseInline(c.tokens)}</${tag}>`;
				};
				const head = `<tr>${token.header.map((c) => cell(c, 'th')).join('')}</tr>`;
				const body = token.rows.map((r) => `<tr>${r.map((c) => cell(c, 'td')).join('')}</tr>`).join('\n');
				return `<div class="table-wrap"><table>\n<thead>${head}</thead>\n<tbody>${body}</tbody>\n</table></div>\n`;
			}
		}
	});

	return { html: md.parse(src) as string, toc };
}

/** Drop everything before the first `## ` heading (the title block the page draws itself). */
export function body(src: string): string {
	const i = src.search(/^## /m);
	return i < 0 ? src : src.slice(i);
}

/** The body of the `## heading` section: up to the next `## ` heading, without its own heading line. */
export function section(src: string, heading: string): string {
	const start = src.indexOf(`\n## ${heading}\n`);
	if (start < 0) throw new Error(`no section "${heading}"`);
	const from = start + heading.length + 5;
	const next = src.indexOf('\n## ', from);
	return src.slice(from, next < 0 ? undefined : next).trim();
}

/** Everything after the first `# ` title line. */
export function afterTitle(src: string): string {
	return src.replace(/^# [^\n]*\n/, '').trim();
}

/** Lowers every heading by one level, so a document can sit inside a section. */
export function demote(src: string): string {
	return src.replace(/^(#{2,5}) /gm, '#$1 ');
}
