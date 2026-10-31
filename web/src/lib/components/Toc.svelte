<script lang="ts">
	import type { TocEntry } from '$lib/markdown';
	let { toc, title = 'Contents' }: { toc: TocEntry[]; title?: string } = $props();
</script>

<nav class="toc" aria-label={title}>
	<h2 class="toc-title">{title}</h2>
	<ol>
		{#each toc.filter((t) => t.depth === 2) as entry (entry.id)}
			<li>
				<span class="no">{entry.label ?? ''}</span>
				<a href="#{entry.id}">{entry.text}</a>
			</li>
		{/each}
	</ol>
</nav>

<style>
	.toc {
		margin: 0 0 24px;
	}
	.toc-title {
		font-size: 1em;
		border: 0;
		margin: 0 0 8px;
		padding: 0;
	}
	ol {
		list-style: none;
		padding: 0;
		margin: 0;
		columns: 2 18em;
		column-gap: 32px;
		font-size: 14px;
	}
	li {
		margin: 0 0 4px;
		break-inside: avoid;
		display: flex;
		gap: 8px;
	}
	.no {
		min-width: 1.6em;
		text-align: right;
		color: var(--muted);
		font-variant-numeric: tabular-nums;
	}
</style>
