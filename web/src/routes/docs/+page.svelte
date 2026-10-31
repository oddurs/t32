<script lang="ts">
	import Toc from '$lib/components/Toc.svelte';
	import { base } from '$app/paths';
	let { data } = $props();
</script>

<svelte:head>
	<title>T32 guide</title>
	<meta name="description" content="Using T32 in practice: reading stamps, choosing a length, storing and querying in SQL, TypeScript and Rust, and the questions people ask." />
</svelte:head>

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft 0.2 · Guide (informative)</span>
		<h1>Guide</h1>
		<p class="lede">
			Using T32 in practice. Where this guide and the <a href="{base}/spec/">specification</a>
			differ, the specification is right and the guide has a defect.
		</p>
	</header>

	<aside class="rail"><Toc toc={data.toc} /></aside>

	<div class="main">
		<article class="doc">
			{@html data.html}
		</article>
	</div>
</div>
