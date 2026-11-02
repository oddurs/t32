<script lang="ts">
	import { MARK } from '$lib/mark';

	// The name takes the text colour and the full stop the accent, so both follow the theme.
	// Sized by cap height, with its bottom edge on the baseline, so it lines up with text beside it.
	let { cap = '1rem', label = 'T32' }: { cap?: string; label?: string | null } = $props();
</script>

<svg
	class="mark"
	viewBox="0 0 {MARK.width} 32"
	style:height={cap}
	role={label ? 'img' : undefined}
	aria-label={label ?? undefined}
	aria-hidden={label ? undefined : 'true'}
>
	<path d={MARK.name} />
	<rect class="cell" x={MARK.cell.x} y={MARK.cell.y} width={MARK.cell.size} height={MARK.cell.size} />
</svg>

<style>
	.mark {
		display: inline-block;
		width: auto;
		/* Round letters overshoot the cap height and the baseline by a hair; let them. */
		overflow: visible;
		fill: currentColor;
	}
	.cell {
		fill: var(--accent);
	}
</style>
