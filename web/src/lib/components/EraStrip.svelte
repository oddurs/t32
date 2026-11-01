<script lang="ts">
	let { eras, current }: { eras: { c: string; from: string; to: string }[]; current: string } = $props();
	const mid = $derived(eras.length / 2);
</script>

<figure class="eras">
	<div class="strip" role="img" aria-label="The 32 eras, {eras[0].from} to {eras[eras.length - 1].to}. Era {current} begins at the epoch, at the centre of the range.">
		{#each eras as e (e.c)}
			<span class="era" class:on={e.c === current} title="{e.c}: {e.from} to {e.to}">{e.c}</span>
		{/each}
	</div>
	<div class="axis" aria-hidden="true">
		<span class="tick" style="left: 0">{eras[0].from}</span>
		<span class="tick centre" style="left: {(100 * mid) / eras.length}%">The epoch, 2000 TAI</span>
		<span class="tick end" style="left: 100%">{eras[eras.length - 1].to}</span>
	</div>
	<figcaption class="caption">
		The eight-character range, split by its first character. The bias puts the epoch at the exact centre:
		instants before 2000 start with 0 to F, and instants after it with G to Z.
	</figcaption>
</figure>

<style>
	.strip {
		display: flex;
		height: 1.75rem;
	}
	.era {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-right: 2px;
		background: var(--wash);
		box-shadow: inset 0 0 0 1px var(--rule);
		font-size: min(var(--label), 2.6vw);
		font-weight: var(--w-medium);
		font-variant-numeric: tabular-nums;
		color: var(--ink-3);
		cursor: default;
	}
	.era:last-child {
		margin-right: 0;
	}
	/* The text inside a filled mark is the one place it may contrast with the fill rather than the page. */
	.era.on {
		background: var(--accent);
		box-shadow: none;
		color: var(--paper);
		font-weight: var(--w-strong);
	}
	.axis {
		position: relative;
		height: 2.25rem;
		border-top: 1px solid var(--rule-strong);
		margin-top: var(--s2);
		font-size: var(--label);
		color: var(--ink-2);
		font-variant-numeric: tabular-nums;
	}
	.tick {
		position: absolute;
		top: 0;
		padding-top: var(--s2);
		white-space: nowrap;
	}
	.tick::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		height: 0.375rem;
		border-left: 1px solid var(--rule-strong);
	}
	.centre {
		transform: translateX(-50%);
	}
	.centre::before {
		left: 50%;
	}
	.end {
		transform: translateX(-100%);
	}
	.end::before {
		left: auto;
		right: 0;
	}
</style>
