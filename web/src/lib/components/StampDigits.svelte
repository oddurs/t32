<script lang="ts">
	import { onMount } from 'svelte';
	import { fly } from 'svelte/transition';
	import { backOut, cubicIn } from 'svelte/easing';
	import { format, type Stamp } from '$lib/t32';
	import { shortSpan } from '$lib/describe';
	import { reducedMotion, stagger } from '$lib/motion.svelte';

	// A stamp as a row of character cells. Every position the input can reach is drawn, the unused
	// ones faint, with the span each position names beneath it. Choosing a cell cuts the stamp
	// there; added characters drop in from above, one after another, and removed ones fall away.
	let { full, len = $bindable() }: { full: Stamp; len: number } = $props();

	const chars = $derived([...format(full)]);
	const n = $derived(Math.max(1, Math.min(len, chars.length)));
	const delay = stagger(() => n);

	let still = $state(false);
	onMount(() => {
		still = reducedMotion();
	});
	const t = (ms: number) => (still ? 0 : ms);
</script>

<div class="digits" style:--n={chars.length} role="group" aria-label="Characters of the stamp. Choose one to end the stamp there.">
	{#each chars as c, i (i)}
		<button
			type="button"
			class="cell"
			class:kept={i < n}
			class:last={i === n - 1}
			class:frac={i >= 8}
			aria-pressed={i === n - 1}
			aria-label="{i + 1} {i ? 'characters' : 'character'}, {shortSpan(i + 1)}"
			onclick={() => (len = i + 1)}
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
