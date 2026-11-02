// LaTeX to MathML, when the site is built. Browsers lay out MathML natively and screen readers
// read it, so pages carry no math script. Server-only, so the renderer never reaches a client bundle.
import temml from 'temml';

/** Typesets a LaTeX expression. Throws on a mistake, so a broken formula fails the build. */
export function tex(src: string, display = false): string {
	return temml.renderToString(src, { displayMode: display, throwOnError: true });
}

/** A unit as exactUnit() writes it ("2^−10 s", "1 s"), with the power typeset. */
export function unit(exact: string): string {
	const m = /^2\^([−-]?\d+) s$/.exec(exact);
	return m ? `${tex(`2^{${m[1].replace('−', '-')}}`)} s` : exact;
}
