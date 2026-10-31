<script lang="ts">
	import Toc from '$lib/components/Toc.svelte';
	let { data } = $props();
</script>

<svelte:head>
	<title>T32 docs</title>
	<meta name="description" content="Using T32 in practice: reading stamps, choosing a length, storing and querying in SQL, TypeScript and Rust, and the FAQ." />
</svelte:head>

<div class="titleblock">
	<div class="docno">T32 · DRAFT 0.2 · GUIDE (INFORMATIVE)</div>
	<h1>Docs</h1>
	<p class="muted small">
		Informative guidance. Where this page and the <a href="../spec/">specification</a> differ, the specification wins.
	</p>
</div>

<Toc toc={data.toc} />

<article class="doc">
	{@html data.html}
</article>
