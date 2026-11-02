import { tex } from '$lib/server/math';

export function load() {
	return { math: { bias: tex('2^{39}') } };
}
