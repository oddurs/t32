// Shared by the build-time markdown renderer and by pages that format units.

/**
 * Sets plain-text powers as superscripts: 2^40 → 2<sup>40</sup>, 32^(8−n) → 32<sup>8−n</sup>.
 * Code and preformatted text keep the caret, since there it is literal.
 */
export function exponents(html: string): string {
	return html
		.split(/(<pre[\s\S]*?<\/pre>|<code[\s\S]*?<\/code>)/)
		.map((part, i) =>
			i % 2
				? part
				: part.replace(/(\d+|\b[a-z])\^(?:\(([^()<>]+)\)|([−-]?[0-9a-z]+))/g, (_, b, paren, plainExp) => `${b}<sup>${paren ?? plainExp}</sup>`)
		)
		.join('');
}
