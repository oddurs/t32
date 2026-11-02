<script lang="ts">
	import { onMount } from 'svelte';
	import { fly } from 'svelte/transition';
	import { backOut, cubicIn } from 'svelte/easing';
	import { format, type Stamp } from '$lib/t32';
	import { shortSpan } from '$lib/describe';
	import { reducedMotion, stagger } from '$lib/motion.svelte';

	// A stamp as a row of character cells. Every position the input can reach is drawn, the unused
	// ones faint, with the span each position names beneath it. Choosing a cell cuts the stamp
	// there, and dragging across the row sweeps the length; added characters drop in from above,
	// one after another, and removed ones fall away. Cells past `stated`, the length the input
	// itself supports, are hatched: they treat the input as exact and add no information.
	let { full, stated = full.len, len = $bindable() }: { full: Stamp; stated?: number; len: number } = $props();

	let row: HTMLDivElement;
	let dragging = false;
	/** The length a pointer at clientX asks for: the cell under it, clamped to the row. */
	function lengthAt(clientX: number): number {
		const r = row.getBoundingClientRect();
		return Math.max(1, Math.min(chars.length, Math.floor(((clientX - r.left) / r.width) * chars.length) + 1));
	}
	function down(e: PointerEvent) {
		if (e.button !== 0) return;
		dragging = true;
		row.setPointerCapture(e.pointerId);
		len = lengthAt(e.clientX);
	}
	function move(e: PointerEvent) {
		if (!dragging) return;
		const n = lengthAt(e.clientX);
		if (n !== len) len = n;
	}
	function up() {
		dragging = false;
	}

	const chars = $derived([...format(full)]);
	const n = $derived(Math.max(1, Math.min(len, chars.length)));
	const delay = stagger(() => n);

	let still = $state(false);
	onMount(() => {
		still = reducedMotion();
	});
	const t = (ms: number) => (still ? 0 : ms);
</script>

<div
	class="digits"
	bind:this={row}
	style:--n={chars.length}
	role="group"
	aria-label="Characters of the stamp. Choose one, or drag across them, to end the stamp there."
	onpointerdown={down}
	onpointermove={move}
	onpointerup={up}
	onpointercancel={up}
>
	{#each chars as c, i (i)}
		<button
			type="button"
			class="cell"
			class:kept={i < n}
			class:last={i === n - 1}
			class:frac={i >= 8}
			class:assumed={i >= stated}
			aria-pressed={i === n - 1}
			aria-label="{i + 1} {i ? 'characters' : 'character'}, {shortSpan(i + 1)}{i >= stated ? ', past what the input states' : ''}"
			onclick={(e) => {
				// A pointer has already set the length on press; this serves the keyboard.
				if (e.detail === 0) len = i + 1;
			}}
		>
			<span class="glyph" aria-hidden="true">
				<span class="ghost">{c}</span>
				{#if i < n}
					<span
						class="ch"
						in:fly={{ y: -36, duration: t(420), delay: t(delay.in(i)), easing: backOut, opacity: 0 }}
						out:fly={{ y: 22, duration: t(220), delay: t(delay.out(i)), easing: cubicIn, opacity: 0 }}>{c}</span
					>
				{/if}
			</span>
			<span class="pos" aria-hidden="true">{i + 1}</span>
			<span class="span" aria-hidden="true">{shortSpan(i + 1)}</span>
		</button>
	{/each}
</div>

<style>
	.digits {
		container-type: inline-size;
		/* Horizontal drags set the length; vertical ones still scroll the page. */
		touch-action: pan-y;
		user-select: none;
		cursor: ew-resize;
		display: grid;
		grid-template-columns: repeat(var(--n), minmax(0, 1fr));
		border-top: 1px solid var(--rule-strong);
		border-bottom: 1px solid var(--rule-strong);
	}
	.cell {
		display: flex;
		flex-direction: column;
		align-items: center;
		min-width: 0;
		padding: var(--s3) 0 var(--s2);
		background: none;
		border: 0;
		border-left: 1px solid var(--rule);
		border-radius: 0;
		color: var(--ink);
		font-variant-numeric: tabular-nums lining-nums;
		cursor: inherit;
	}
	.cell:first-child {
		border-left: 0;
	}
	/* The ninth character starts the fractions of a second. */
	.cell:nth-child(9) {
		border-left-color: var(--rule-strong);
	}
	.cell.frac {
		background: var(--wash);
	}
	/* Past what the input states: the input taken as exact, digits with no information in them. */
	.cell.assumed {
		background: repeating-linear-gradient(135deg, transparent 0 5px, var(--rule) 5px 6px);
	}
	.assumed .ch {
		color: var(--ink-3);
		font-weight: var(--w-text);
	}
	.cell:hover .ghost {
		opacity: 0.6;
	}
	.cell:focus-visible {
		outline-offset: -2px;
	}
	/* Cells shrink with the row, so a 16-character stamp fits a phone. */
	.glyph {
		display: grid;
		font-size: min(var(--fs-7), 100cqw / var(--n) * 1.15);
		font-weight: var(--w-head);
		line-height: 1.1;
	}
	.glyph > * {
		grid-area: 1 / 1;
		text-align: center;
	}
	.ghost {
		color: var(--ink-3);
		opacity: 0.28;
		transition: opacity 200ms ease;
	}
	.kept .ghost {
		opacity: 0;
	}
	.frac .ch {
		color: var(--ink-2);
		font-weight: var(--w-medium);
	}
	/* The last character kept is the one that sets the span: it carries the accent, as the marked cell does. */
	.ch {
		position: relative;
	}
	.last .ch::after {
		content: '';
		position: absolute;
		left: 18%;
		right: 18%;
		bottom: -0.12em;
		height: 3px;
		background: var(--accent);
	}
	.pos,
	.span {
		font-size: var(--label);
		line-height: var(--lh-tight);
		color: var(--ink-3);
		white-space: nowrap;
	}
	/* In a narrow cell the unit drops under its number rather than running into the next cell. */
	.span {
		white-space: normal;
		text-align: center;
		padding: 0 2px;
	}
	.pos {
		margin-top: var(--s2);
		font-weight: var(--w-strong);
	}
	.last .pos {
		color: var(--accent);
	}
	.kept .span {
		color: var(--ink-2);
	}
	/* Too narrow for a span under each character: the sentence below names it instead. */
	@container (max-width: 34rem) {
		.span {
			display: none;
		}
	}
</style>
