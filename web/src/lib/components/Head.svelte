<script lang="ts">
	import { SITE_URL } from '$lib/site';

	// One place for each page's title, description and link-preview tags.
	let { title, description, path }: { title?: string; description: string; path: string } = $props();

	const full = $derived(title ? `${title} · T32` : 'T32 — a compact universal timestamp');
	const url = $derived(SITE_URL + path);
</script>

<svelte:head>
	<title>{full}</title>
	<meta name="description" content={description} />
	<link rel="canonical" href={url} />
	<meta property="og:type" content="website" />
	<meta property="og:site_name" content="T32" />
	<meta property="og:title" content={full} />
	<meta property="og:description" content={description} />
	<meta property="og:url" content={url} />
	<meta property="og:image" content="{SITE_URL}/og.png" />
	<meta property="og:image:width" content="1200" />
	<meta property="og:image:height" content="630" />
	<meta property="og:image:alt" content="G0S7G6B1: a T32 stamp for 31 October 2026, 03:13:00 UTC" />
	<meta name="twitter:card" content="summary_large_image" />
</svelte:head>
