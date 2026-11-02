import { untrack } from 'svelte';

/** True when the reader has asked for less motion; transitions then take no time. */
export function reducedMotion(): boolean {
	return typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/**
 * Delays that make characters, or rows, arrive one after another when a length grows, and
 * leave from the end when it shrinks. Call during component setup; `length` is read in a
 * pre-effect, so the delays see the old length and the new one while the transitions start.
 */
export function stagger(length: () => number, step = 60) {
	let from = 0;
	let to: number | null = null;
	$effect.pre(() => {
		const l = length();
		untrack(() => {
			from = to ?? l;
			to = l;
		});
	});
	return {
		/** Delay for the item at index i arriving. */
		in: (i: number) => Math.max(0, i - from) * step,
		/** Delay for the item at index i leaving: the last one first. */
		out: (i: number) => Math.max(0, from - 1 - i) * step * 0.5
	};
}
