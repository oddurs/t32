<script lang="ts">
	import Toc from '$lib/components/Toc.svelte';
	let { data } = $props();
</script>

<svelte:head>
	<title>T32 specification (draft 0.2)</title>
	<meta name="description" content="The T32 specification: definitions, encoding, interval semantics, conversion from UTC, test vectors and the TAI − UTC table." />
</svelte:head>

<div class="titleblock">
	<div class="docno">T32 · DRAFT 0.2 · SPECIFICATION</div>
	<h1>T32: a compact universal timestamp</h1>
	<dl>
		<dt>Status</dt><dd>Draft proposal, open for comment</dd>
		<dt>Date</dt><dd>2026-10-31</dd>
		<dt>Source</dt><dd><a href="https://github.com/oddurs/t32/blob/main/SPEC.md">SPEC.md</a></dd>
	</dl>
</div>

<Toc toc={data.toc} />

<article class="doc">
	{@html data.html}
</article>
