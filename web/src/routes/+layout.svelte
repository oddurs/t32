<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { base } from '$app/paths';

	let { children } = $props();

	const nav = [
		{ href: '/', label: 'Overview' },
		{ href: '/playground/', label: 'Playground' },
		{ href: '/docs/', label: 'Guide' },
		{ href: '/spec/', label: 'Specification' },
		{ href: '/tables/', label: 'Tables' },
		{ href: '/participate/', label: 'Participate' }
	];
	const current = (href: string) => page.url.pathname === base + href;
</script>

<a class="skip" href="#content">Skip to content</a>

<header class="site">
	<div class="bar">
		<a class="brand" href="{base}/" aria-label="T32, overview">
			<span class="mark">T32</span>
			<span class="edition">Draft 0.2</span>
		</a>
		<nav aria-label="Main">
			{#each nav as item (item.href)}
				<a href="{base}{item.href}" aria-current={current(item.href) ? 'page' : undefined}>{item.label}</a>
			{/each}
			<a class="ext" href="https://github.com/oddurs/t32">Source</a>
		</nav>
	</div>
</header>

<main class="page" id="content">
	{@render children()}
</main>

<footer class="site">
	<div class="cols">
		<div>
			<p class="label">T32</p>
			<p>A compact universal timestamp. Draft 0.2, published for public review.</p>
		</div>
		<div>
			<p class="label">Editor</p>
			<p>Oddur Sigurdsson<br /><a href="https://github.com/oddurs">@oddurs</a></p>
		</div>
		<div>
			<p class="label">Take part</p>
			<p>
				<a href="{base}/participate/">Participate</a><br />
				<a href="https://github.com/oddurs/t32/discussions">Discussions</a> ·
				<a href="https://github.com/oddurs/t32/issues">Issues</a>
			</p>
		</div>
		<div>
			<p class="label">Licence</p>
			<p>
				MIT OR Apache-2.0<br />
				<a href="https://crates.io/crates/t32">crates.io</a> ·
				<a href="https://github.com/oddurs/t32">Source</a>
			</p>
		</div>
	</div>
</footer>

<style>
	.skip {
		position: absolute;
		left: -999px;
	}
	.skip:focus {
		left: var(--s4);
		top: var(--s4);
		background: var(--paper);
		padding: var(--s2) var(--s3);
		z-index: 10;
	}
	header.site {
		border-bottom: 1px solid var(--rule);
	}
	.bar {
		max-width: 72rem;
		margin: 0 auto;
		padding: var(--s4) var(--s5);
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		justify-content: space-between;
		gap: var(--s2) var(--s6);
	}
	.brand {
		display: flex;
		align-items: baseline;
		gap: var(--s3);
		text-decoration: none;
		color: var(--ink);
	}
	.mark {
		font-weight: 760;
		font-size: 1.1875rem;
		letter-spacing: -0.02em;
		line-height: 1;
	}
	.edition {
		font-size: var(--label);
		font-weight: var(--w-strong);
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--ink-3);
	}
	nav a {
		font-size: var(--small);
		font-weight: var(--w-medium);
		letter-spacing: 0.005em;
	}
	nav {
		display: flex;
		flex-wrap: wrap;
		gap: var(--s2) var(--s5);
	}
	nav a {
		color: var(--ink-2);
		text-decoration: none;
		padding-bottom: 2px;
		border-bottom: 1px solid transparent;
	}
	nav a:hover {
		color: var(--ink);
	}
	nav a[aria-current='page'] {
		color: var(--ink);
		border-bottom-color: var(--ink);
	}
	nav a {
		padding-bottom: 3px;
	}
	.ext::after {
		content: ' ↗';
		color: var(--ink-3);
	}
	footer.site {
		border-top: 1px solid var(--rule);
		font-size: var(--small);
		color: var(--ink-2);
	}
	.cols {
		max-width: 72rem;
		margin: 0 auto;
		padding: var(--s6) var(--s5) var(--s7);
		display: grid;
		grid-template-columns: 2fr 1fr 1fr 1fr;
		gap: var(--s5) var(--s6);
	}
	.cols p {
		margin: 0;
		line-height: 1.5;
	}
	.cols .label {
		margin-bottom: var(--s2);
	}
	@media (max-width: 760px) {
		.cols {
			grid-template-columns: 1fr 1fr;
		}
		.bar {
			padding: var(--s3) var(--s4);
		}
		nav {
			gap: var(--s2) var(--s4);
		}
	}
</style>
