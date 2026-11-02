import css from '../../app.css?raw';
import { readTokens, value, contrast, TEXT_COLOURS } from '$lib/tokens';
import { tex } from '$lib/server/math';

// Everything the page states about a token is read from the stylesheet when the site is built.
export function load() {
	const { light, dark } = readTokens(css);
	const names = [...light.keys()];
	// A value that refers to another token is shown resolved, with the name it came from.
	const shown = (n: string) => {
		const v = value(light, n);
		const ref = /^var\((--[\w-]+)\)$/.exec(v);
		return ref ? `${value(light, ref[1])}, ${ref[1]}` : v;
	};
	const pick = (prefix: RegExp) =>
		names.filter((n) => prefix.test(n)).map((n) => ({ name: n, value: shown(n) }));

	const colours = names
		.filter((n) => /^#[0-9a-f]{6}$/i.test(value(light, n)))
		.map((n) => {
			const text = (TEXT_COLOURS as readonly string[]).includes(n);
			return {
				name: n,
				light: value(light, n),
				dark: value(dark, n),
				// Contrast on paper, for the colours text is set in.
				ratio: text ? [contrast(value(light, n), value(light, '--paper')), contrast(value(dark, n), value(dark, '--paper'))] : null
			};
		});

	const px = (v: string) => Math.round(parseFloat(v) * 16 * 100) / 100;
	return {
		colours,
		scale: pick(/^--fs-/).map((t) => ({ ...t, step: t.name.slice(5).replace('m', '−'), px: px(t.value) })),
		space: pick(/^--s\d$/).map((t) => ({ ...t, px: px(t.value) })),
		frame: pick(/^--(measure|page|rail|gutter|section|radius)$/),
		weights: { light: pick(/^--w-/), dark: pick(/^--w-/).map((t) => ({ ...t, value: value(dark, t.name) })) },
		leading: pick(/^--lh-/),
		tracking: pick(/^--tr-/),
		math: { step: tex('2^{1/5}') }
	};
}
