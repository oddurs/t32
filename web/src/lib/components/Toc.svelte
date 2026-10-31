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
	.label {
		margin: 0 0 var(--s3);
	}
	ol {
		list-style: none;
		padding: 0;
		margin: 0;
		font-size: 0.875rem;
		line-height: 1.35;
	}
	li {
		display: grid;
		grid-template-columns: 1.6rem 1fr;
		margin: 0 0 var(--s2);
		padding: 0;
	}
	li.appendix {
		grid-template-columns: 1.6rem 1fr;
	}
	.no,
	.app {
		font-family: var(--mono);
		font-size: 0.75rem;
		color: var(--ink-3);
		padding-top: 0.1em;
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
	a {
		color: var(--ink-2);
		text-decoration: none;
	}
	a:hover {
		color: var(--accent);
	}
	@media (max-width: 960px) {
		ol {
			columns: 2 14rem;
			column-gap: var(--s6);
		}
		li {
			break-inside: avoid;
		}
	}
</style>
