<script lang="ts">
	import type { TocEntry } from '$lib/markdown';
	let { toc, title = 'Contents' }: { toc: TocEntry[]; title?: string } = $props();
</script>

<nav class="toc" aria-label={title}>
	<p class="label">{title}</p>
	<ol>
		{#each toc.filter((t) => t.depth === 2) as entry (entry.id)}
			<li class:appendix={!entry.label}>
				<span class="no">{entry.label ?? ''}</span>
				<a href="#{entry.id}">{entry.text.replace(/^Appendix [A-Z]: /, '')}</a>
				{#if !entry.label}<span class="app">{entry.text.match(/^Appendix ([A-Z])/)?.[1] ?? ''}</span>{/if}
			</li>
		{/each}
	</ol>
</nav>

<style>
	/* Appendices carry a letter in the number column. The rest is the shared .toc style. */
	.app {
		font-size: var(--label);
		font-weight: var(--w-medium);
		font-variant-numeric: tabular-nums;
		color: var(--ink-3);
	}
	li.appendix .app {
		grid-column: 1;
		grid-row: 1;
	}
	li.appendix a {
		grid-column: 2;
		grid-row: 1;
	}
	li.appendix .no {
		display: none;
	}
</style>
