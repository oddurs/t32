<script lang="ts">
	import Head from '$lib/components/Head.svelte';
	import { DRAFT, DRAFT_DATE, DRAFT_PDF, SITE_URL } from '$lib/site';
	import Toc from '$lib/components/Toc.svelte';
	import { resolve } from '$app/paths';
	let { data } = $props();
</script>

<svelte:head>
	<link rel="alternate" type="application/pdf" href="{SITE_URL}/{DRAFT_PDF}" title="T32 as an Internet-Draft (PDF)" />
</svelte:head>

<Head title="Specification" description="The T32 specification, draft {DRAFT}: definitions, grammar, encoding, interval semantics, conversion from UTC, test vectors and the TAI − UTC table." path="/spec/" />

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft {DRAFT} · Specification</span>
		<h1>T32: a compact universal timestamp</h1>
		<dl class="meta">
			<dt>Status</dt><dd>Draft for public review, not yet frozen</dd>
			<dt>Date</dt><dd>{DRAFT_DATE}</dd>
			<dt>Editor</dt><dd>Oddur Sigurdsson (<a href="https://github.com/oddurs">@oddurs</a>)</dd>
			<dt>Source</dt><dd><a href="https://github.com/oddurs/t32/blob/main/SPEC.md">SPEC.md</a> · <a href="https://github.com/oddurs/t32/commits/main/SPEC.md">history</a></dd>
			<dt>PDF</dt><dd><a href="{resolve('/')}{DRAFT_PDF}" type="application/pdf">As an Internet-Draft</a> (<span class="data">{DRAFT_PDF.replace('.pdf', '')}</span>)</dd>
			<dt>Comments</dt><dd><a href="https://github.com/oddurs/t32/issues/new/choose">Report a problem</a> · <a href={resolve('participate/')}>Participate</a></dd>
		</dl>
	</header>

	<aside class="rail"><Toc toc={data.toc} /></aside>

	<div class="main">
		<article class="doc">
			{@html data.html}
		</article>
	</div>
</div>
