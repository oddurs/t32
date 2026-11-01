<script lang="ts">
	import { ALPHABET, unitSeconds } from '$lib/t32';
	import { humanSpan } from '$lib/describe';

	let { stamp }: { stamp: string } = $props();

	const rows = $derived(
		[...stamp].map((ch, i) => ({
			head: stamp.slice(0, i),
			last: ch,
			part: ALPHABET.indexOf(ch),
			span: humanSpan(unitSeconds(i + 1)).replace('≈ ', '')
		}))
	);
</script>

<figure class="zoom">
	<div class="grid">
		<span class="label">Prefix</span><span></span><span class="label right">Span</span>
		{#each rows as row, i (i)}
			<span class="data prefix"><span class="muted">{row.head}</span><strong>{row.last}</strong></span>
			<span class="bar" role="img" aria-label="{row.last}: part {row.part + 1} of 32">
				{#each { length: 32 } as _, k (k)}<i class:on={k === row.part}></i>{/each}
			</span>
			<span class="span">{row.span}</span>
			{#if i < rows.length - 1}
				<!-- The chosen part, opened out to the full width of the next row. -->
				<svg class="link" viewBox="0 0 32 1" preserveAspectRatio="none" aria-hidden="true">
					<polygon points="{row.part},0 {row.part + 1},0 32,1 0,1" />
				</svg>
			{/if}
		{/each}
	</div>
	<figcaption class="caption">
		Reading <span class="data">{stamp}</span> from the left. Each character picks one of 32 equal parts
		of the span before it, so every row is the marked part of the row above, enlarged 32 times.
	</figcaption>
</figure>

<style>
	.grid {
		display: grid;
		grid-template-columns: 6.5rem minmax(0, 1fr) 7.5rem;
		align-items: center;
		column-gap: var(--s4);
	}
	.grid > .label {
		margin-bottom: var(--s2);
	}
	.right,
	.span {
		text-align: right;
	}
	/* Pinned to the first column: a connector row before it would otherwise pull it right. */
	.prefix {
		grid-column: 1;
		font-size: var(--small);
		letter-spacing: 0.04em;
	}
	.prefix strong {
		color: var(--accent);
	}
	.span {
		font-size: var(--small);
		color: var(--ink-2);
		white-space: nowrap;
		font-variant-numeric: tabular-nums;
	}
	.bar {
		display: flex;
		height: 0.875rem;
	}
	/* Each part fills exactly 1/32 of the bar, so the connector above can point at it by index. */
	.bar i {
		flex: 1;
		padding: 0 1px;
		background: var(--rule);
		background-clip: content-box;
	}
	.bar i.on {
		background-color: var(--accent);
	}
	.link {
		grid-column: 2;
		display: block;
		width: 100%;
		height: 1.125rem;
	}
	.link polygon {
		fill: var(--accent);
		fill-opacity: 0.1;
		stroke: var(--accent-rule);
		stroke-width: 1;
		vector-effect: non-scaling-stroke;
	}
	@media (max-width: 600px) {
		.grid {
			grid-template-columns: 5.25rem minmax(0, 1fr) 5rem;
			column-gap: var(--s2);
		}
		.bar i {
			padding: 0 0.5px;
		}
	}
</style>
