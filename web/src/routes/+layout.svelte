<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { base } from '$app/paths';

	let { children } = $props();

	const nav = [
		{ href: '/', label: 'Overview' },
		{ href: '/spec/', label: 'Specification' },
		{ href: '/playground/', label: 'Playground' },
		{ href: '/docs/', label: 'Docs' },
		{ href: '/tables/', label: 'Tables' }
	];
	const current = (href: string) => page.url.pathname === base + href;
</script>

<header class="site">
	<div class="inner">
		<a class="brand" href="{base}/">T32</a>
		<nav aria-label="Main">
			{#each nav as item (item.href)}
				<a href="{base}{item.href}" aria-current={current(item.href) ? 'page' : undefined}>{item.label}</a>
			{/each}
			<a href="https://github.com/oddurs/t32">GitHub</a>
		</nav>
	</div>
</header>

<main class="page">
	{@render children()}
</main>

<footer class="site">
	<div class="inner small muted">
		T32 is a draft proposal, not an adopted standard. Comments and objections:
		<a href="https://github.com/oddurs/t32/issues">GitHub issues</a>.
	</div>
</footer>

<style>
	header.site {
		border-bottom: 1px solid var(--border-muted);
		background: var(--subtle);
	}
	.inner {
		max-width: 880px;
		margin: 0 auto;
		padding: 10px 16px;
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 4px 20px;
	}
	.brand {
		font-family: var(--mono);
		font-weight: 700;
		color: var(--fg);
	}
	nav {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 16px;
		font-size: 14px;
	}
	nav a {
		color: var(--muted);
	}
	nav a[aria-current='page'] {
		color: var(--fg);
		font-weight: 600;
	}
	footer.site {
		border-top: 1px solid var(--border-muted);
	}
	footer .inner {
		display: block;
		padding: 16px;
	}
</style>
