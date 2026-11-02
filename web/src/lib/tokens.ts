// Reads the design tokens out of app.css, so the system page documents the stylesheet itself
// and cannot drift from it, and measures the contrast its text colours claim.

export type Tokens = { light: Map<string, string>; dark: Map<string, string> };

const DECL = /^\s*(--[\w-]+):\s*([^;]+);/gm;

function declarations(block: string): Map<string, string> {
	return new Map([...block.matchAll(DECL)].map(([, name, value]) => [name, value.trim()]));
}

/** The first top-level `:root` block, and the one inside the dark colour-scheme query. Dark inherits light. */
export function readTokens(css: string): Tokens {
	const light = /^:root \{([\s\S]*?)^\}/m.exec(css);
	const dark = /@media \(prefers-color-scheme: dark\) \{\s*:root \{([\s\S]*?)^\t\}/m.exec(css);
	if (!light || !dark) throw new Error('app.css: no :root block, or no dark :root block');
	const l = declarations(light[1]);
	return { light: l, dark: new Map([...l, ...declarations(dark[1])]) };
}

/** A token's value; throws on a name the stylesheet does not define, so a typo fails the build. */
export function value(map: Map<string, string>, name: string): string {
	const v = map.get(name);
	if (v === undefined) throw new Error(`app.css defines no ${name}`);
	return v;
}

function luminance(hex: string): number {
	const m = /^#([0-9a-f]{6})$/i.exec(hex);
	if (!m) throw new Error(`not a #rrggbb colour: ${hex}`);
	const [r, g, b] = [0, 2, 4].map((i) => {
		const c = parseInt(m[1].slice(i, i + 2), 16) / 255;
		return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
	});
	return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG 2 contrast ratio of two #rrggbb colours. */
export function contrast(a: string, b: string): number {
	const [x, y] = [luminance(a), luminance(b)].sort((p, q) => q - p);
	return (x + 0.05) / (y + 0.05);
}

/** The colours text is set in, and the grounds it is set on. */
export const TEXT_COLOURS = ['--ink', '--ink-2', '--ink-3', '--accent', '--err'] as const;
export const GROUNDS = ['--paper', '--wash'] as const;
