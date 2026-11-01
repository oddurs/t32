<script lang="ts">
	import Head from '$lib/components/Head.svelte';
	import { DRAFT } from '$lib/site';
	import Toc from '$lib/components/Toc.svelte';
	import { base } from '$app/paths';
	let { data } = $props();
</script>

<Head title="Guide" description="Using T32 in practice: reading stamps, choosing a length, storing and querying them in SQL, the Rust and TypeScript libraries, and common questions." path="/docs/" />

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft {DRAFT} · Guide (informative)</span>
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
