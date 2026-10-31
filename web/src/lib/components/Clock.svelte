<script lang="ts">
	import { onMount } from 'svelte';
	import { fromUnixMs, format, truncate, startFx, endFx, fxToUtcString } from '$lib/t32';

	let ms: number | null = $state(null);

	onMount(() => {
		ms = Date.now();
		const id = setInterval(() => (ms = Date.now()), 100);
		return () => clearInterval(id);
	});

	// Date.now() resolves 1 ms: 10 characters is the most it may claim.
	const full = $derived(ms === null ? null : fromUnixMs(ms, 10));
	const text = $derived(full ? format(full) : '··········');
	const iso = $derived(ms === null ? '' : new Date(ms).toISOString().replace(/\.\d+Z$/, 'Z'));

	const spans = ['1,089 y', '34 y', '388 d', '12.1 d', '9.1 h', '17.1 min', '32 s', '1 s', '31 ms', '0.98 ms'];
	const ladder = $derived(
		full
			? Array.from({ length: 8 }, (_, i) => {
					const p = truncate(full, i + 1);
					return { text: format(p), from: fxToUtcString(startFx(p)), to: fxToUtcString(endFx(p)) };
				})
			: []
	);
</script>

<figure class="specimen" aria-label="The current time in T32">
	<ol class="cells">
		{#each [...text] as ch, i (i)}
			<li class:frac={i >= 8}>
				<span class="ch">{ch}</span>
				<span class="pos">{i + 1}</span>
				<span class="span">{spans[i]}</span>
			</li>
		{/each}
	</ol>
	<figcaption>
		<span class="label">Now</span>
		<span class="data">{text.slice(0, 8)}<span class="muted">{text.slice(8)}</span></span>
		{#if iso}<span class="muted">· {iso}</span>{/if}
		<span class="muted">· each position divides the one before it by 32</span>
	</figcaption>
</figure>

<div class="table-wrap">
	<table class="ladder">
		<thead>
			<tr><th>Prefix</th><th>Span</th><th>Interval, UTC</th></tr>
		</thead>
		<tbody>
			{#each ladder as row, i (i)}
				<tr>
					<td class="data">{row.text}</td>
					<td class="nw">{spans[i]}</td>
					<td class="data small"><span class="nw">{row.from}</span> – <span class="nw">{row.to}</span></td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	.specimen {
		margin: 0 0 var(--s5);
	}
	.cells {
		display: grid;
		grid-template-columns: repeat(10, minmax(0, 1fr));
		list-style: none;
		padding: 0;
		margin: 0 0 var(--s3);
		border-top: 1px solid var(--rule-strong);
		border-bottom: 1px solid var(--rule-strong);
	}
	.cells li {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--s3) 0 var(--s2);
		margin: 0;
		border-left: 1px solid var(--rule);
	}
	.cells li:first-child {
		border-left: 0;
	}
	.cells li.frac {
		background: var(--wash);
	}
	.cells li:nth-child(9) {
		border-left: 1px solid var(--rule-strong);
	}
	.ch {
		font-size: clamp(1.6rem, 5.2vw, 3rem);
		font-weight: var(--w-head);
		font-variant-numeric: tabular-nums lining-nums;
		letter-spacing: -0.01em;
		line-height: 1.1;
	}
	.frac .ch {
		color: var(--ink-3);
		font-weight: var(--w-text);
	}
	.pos,
	.span {
		font-size: 0.6875rem;
		color: var(--ink-3);
		font-variant-numeric: tabular-nums;
	}
	.pos {
		font-weight: var(--w-strong);
	}
	.pos {
		margin-top: var(--s2);
	}
	.span {
		white-space: nowrap;
	}
	figcaption {
		font-size: var(--small);
		display: flex;
		flex-wrap: wrap;
		gap: var(--s1) var(--s2);
		align-items: baseline;
	}
	figcaption .label {
		margin-right: var(--s1);
	}
	.nw {
		white-space: nowrap;
	}
	@media (max-width: 600px) {
		.span {
			display: none;
		}
	}
</style>
