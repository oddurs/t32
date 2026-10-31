<script lang="ts">
	import { onMount } from 'svelte';
	import { fromUnixMs, format, truncate, startFx, endFx, fxToUtcString } from '$lib/t32';

	let { ladder = true }: { ladder?: boolean } = $props();

	let ms: number | null = $state(null);

	onMount(() => {
		ms = Date.now();
		const id = setInterval(() => (ms = Date.now()), 100);
		return () => clearInterval(id);
	});

	const full = $derived(ms === null ? null : fromUnixMs(ms, 10));
	const canonical = $derived(full ? format(truncate(full, 8)) : '········');
	const sub = $derived(full ? format(full).slice(8) : '··');
	const iso = $derived(ms === null ? '' : new Date(ms).toISOString().replace(/\.\d+Z$/, 'Z'));

	const spans = ['≈ 1,089 years', '≈ 34 years', '≈ 388 days', '≈ 12.1 days', '≈ 9.1 hours', '≈ 17.1 minutes', '32 seconds', '1 second'];
</script>

<div class="clock">
	<div class="now" aria-live="off">
		<span class="mono big">{canonical}</span><span class="mono big frac">{sub}</span>
	</div>
	<p class="small muted caption">
		Now, in T32 (8 characters, plus 2 more for ≈ 1 ms). {#if iso}ISO 8601: <span class="mono">{iso}</span>{/if}
	</p>

	{#if ladder}
		<div class="table-wrap">
			<table>
				<thead>
					<tr><th>Prefix</th><th>Span</th><th>Interval (UTC)</th></tr>
				</thead>
				<tbody>
					{#each spans as span, i (i)}
						{@const p = full ? truncate(full, i + 1) : null}
						<tr>
							<td class="mono">{p ? format(p) : canonical.slice(0, i + 1)}</td>
							<td class="nw">{span}</td>
							<td class="mono small">
								{#if p}<span class="nw">{fxToUtcString(startFx(p))}</span> – <span class="nw">{fxToUtcString(endFx(p))}</span>{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.clock {
		margin: 0 0 16px;
	}
	.now {
		margin: 8px 0 4px;
	}
	.big {
		font-size: clamp(2.2rem, 9vw, 3.5rem);
		font-weight: 600;
		letter-spacing: 0.04em;
	}
	.frac {
		color: var(--muted);
		font-weight: 400;
	}
	.nw {
		white-space: nowrap;
	}
	.caption {
		margin-bottom: 16px;
	}
</style>
