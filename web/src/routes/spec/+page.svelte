<script lang="ts">
	import Toc from '$lib/components/Toc.svelte';
	import { base } from '$app/paths';
	let { data } = $props();
</script>

<svelte:head>
	<title>T32 specification, draft 0.2</title>
	<meta name="description" content="The T32 specification: definitions, encoding, interval semantics, conversion from UTC, test vectors, open questions and the TAI − UTC table." />
</svelte:head>

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft 0.2 · Specification</span>
		<h1>T32: a compact universal timestamp</h1>
		<dl class="meta">
			<dt>Status</dt><dd>Draft for public review, not yet frozen</dd>
			<dt>Date</dt><dd>2026-10-31</dd>
			<dt>Editor</dt><dd>Oddur Sigurdsson (<a href="https://github.com/oddurs">@oddurs</a>)</dd>
			<dt>Source</dt><dd><a href="https://github.com/oddurs/t32/blob/main/SPEC.md">SPEC.md</a> · <a href="https://github.com/oddurs/t32/commits/main/SPEC.md">history</a></dd>
			<dt>Comments</dt><dd><a href="https://github.com/oddurs/t32/issues/new/choose">Report an erratum</a> · <a href="{base}/participate/">Participate</a></dd>
		</dl>
	</header>

	<aside class="rail"><Toc toc={data.toc} /></aside>

	<div class="main">
		<article class="doc">
			{@html data.html}
		</article>
	</div>
</div>
