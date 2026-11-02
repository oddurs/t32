import { MAX_LEN } from '$lib/t32';
import { exactUnit } from '$lib/describe';
import { tex, unit } from '$lib/server/math';

export function load() {
	return {
		units: Array.from({ length: MAX_LEN }, (_, i) => unit(exactUnit(i + 1))),
		math: {
			era: tex('2^{35}'),
			generation: tex('2^{30}'),
			year: tex('2^{25}'),
			tick: tex('2^{-40}'),
			rubber: tex('\\text{TAI} - \\text{UTC} = a + (\\text{MJD} - m) \\times r', true)
		}
	};
}
