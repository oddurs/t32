<script lang="ts">
	import { base } from '$app/paths';
	import { page } from '$app/state';
</script>

<svelte:head>
	<title>{page.status === 404 ? 'Not found' : 'Error'} · T32</title>
	<meta name="robots" content="noindex" />
</svelte:head>

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Error {page.status}</span>
		<h1>{page.status === 404 ? 'Not found' : 'Something went wrong'}</h1>
		<p class="lede">
			{page.status === 404 ? 'There is no page at this address.' : page.error?.message}
			Try one of these instead.
		</p>
	</header>

	<div class="main">
		<dl class="pairs">
			<dt><a href="{base}/">Overview</a></dt>
			<dd>What T32 is, in a few minutes.</dd>
			<dt><a href="{base}/playground/">Playground</a></dt>
			<dd>Type a date or a stamp and see what it means.</dd>
			<dt><a href="{base}/spec/">Specification</a></dt>
			<dd>The normative text.</dd>
		</dl>
	</div>
</div>

<style>
	.pairs {
		display: grid;
		grid-template-columns: 10rem minmax(0, 1fr);
		margin: 0;
		max-width: var(--measure);
	}
	.pairs dt,
	.pairs dd {
		margin: 0;
		padding: var(--s3) 0;
		border-bottom: 1px solid var(--rule);
	}
	.pairs dd {
		color: var(--ink-2);
	}
</style>
