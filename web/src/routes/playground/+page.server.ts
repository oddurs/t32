import { MAX_LEN } from '$lib/t32';
import { exactUnit } from '$lib/describe';
import { unit } from '$lib/server/math';

// The unit of every length, typeset once at build: the page picks one as the length changes.
export function load() {
	return { units: Array.from({ length: MAX_LEN }, (_, i) => unit(exactUnit(i + 1))) };
}
