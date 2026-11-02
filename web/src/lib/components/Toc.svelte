<script lang="ts">
	import type { TocEntry } from '$lib/markdown';

	// A page's contents. Beside the text on a wide screen it is a list; on a narrow one, where it
	// would come before the text, it folds into one line to open. Both are rendered and CSS shows
	// one, so nothing moves when the page loads.
	let { toc, title = 'Contents' }: { toc: TocEntry[]; title?: string } = $props();

	const entries = $derived(
		toc
			.filter((t) => t.depth === 2)
			.map((t) => ({
				id: t.id,
				// Appendices are numbered by letter, taken from their title.
				no: t.label ?? t.text.match(/^Appendix ([A-Z])/)?.[1] ?? '',
				text: t.text.replace(/^Appendix [A-Z]: /, '')
			}))
	);
</script>

{#snippet list()}
	<ol>
		{#each entries as e (e.id)}
			<li><span class="no">{e.no}</span><a href="#{e.id}">{e.text}</a></li>
		{/each}
	</ol>
{/snippet}

<nav class="toc" aria-label={title}>
	<div class="open">
		<p class="label">{title}</p>
		{@render list()}
	</div>
	<details class="fold">
		<summary><span class="label">{title}</span><span class="count">{entries.length} sections</span></summary>
		{@render list()}
	</details>
</nav>

<style>
	.fold {
		display: none;
	}
	@media (max-width: 960px) {
		.open {
			display: none;
		}
		.fold {
			display: block;
			border-top: 1px solid var(--rule-strong);
			border-bottom: 1px solid var(--rule);
		}
		summary {
			display: flex;
			align-items: baseline;
			gap: var(--s3);
			padding: var(--s3) 0;
			cursor: pointer;
			list-style: none;
		}
		summary::-webkit-details-marker {
			display: none;
		}
		/* A plus that turns to a minus: the one mark the control needs. */
		summary::after {
			content: '+';
			margin-left: auto;
			font-size: var(--fs-1);
			line-height: 1;
			color: var(--ink-3);
		}
		.fold[open] summary::after {
			content: '−';
		}
		.fold .label {
			margin: 0;
		}
		.count {
			font-size: var(--small);
			color: var(--ink-3);
		}
		.fold :global(ol) {
			padding-bottom: var(--s3);
		}
	}
</style>
