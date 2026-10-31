<script lang="ts">
	import { base } from '$app/paths';
	let { data } = $props();

	const sections = [
		['standing', 'Where the specification stands'],
		['ways', 'Ways to take part'],
		['questions', 'Open questions'],
		['implementations', 'Implementations'],
		['governance', 'How decisions are made']
	];

	const track = [
		{ name: 'Draft 0.1', when: '2026-10-30', what: 'First public draft.', state: 'done' },
		{ name: 'Draft 0.2', when: '2026-10-31', what: 'Complete enough to implement. Public review is open.', state: 'current' },
		{ name: 'Review', when: 'Open', what: 'Open questions decided, errata closed, three independent implementations.', state: '' },
		{ name: 'Version 1.0', when: 'After 60 days', what: 'The core is frozen: a stamp means the same thing forever.', state: '' },
		{ name: 'Working group', when: 'With 1.0', what: 'Chartered by implementers and reviewers, in a neutral home.', state: '' }
	];

	const discussions = 'https://github.com/oddurs/t32/discussions';
</script>

<svelte:head>
	<title>Participate in T32</title>
	<meta name="description" content="How T32 becomes a standard: where the draft stands, the open questions, implementations, and the path to version 1.0 and a working group." />
</svelte:head>

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft 0.2 · Participation</span>
		<h1>Participate</h1>
		<p class="lede">
			A specification becomes a standard by being read, implemented and questioned by people other than
			its author. This page says where that happens, and what it leads to.
		</p>
	</header>

	<aside class="rail">
		<nav class="toc" aria-label="On this page">
			<p class="label">On this page</p>
			<ol>
				{#each sections as [id, title], i (id)}
					<li><span class="no">{i + 1}</span><a href="#{id}">{title}</a></li>
				{/each}
			</ol>
		</nav>
		<p class="label elsewhere">Elsewhere</p>
		<ul class="links">
			<li><a href={discussions}>Discussions</a></li>
			<li><a href="https://github.com/oddurs/t32/issues/new/choose">Report an erratum</a></li>
			<li><a href="https://github.com/oddurs/t32/blob/main/CONTRIBUTING.md">Contributing</a></li>
			<li><a href="https://github.com/oddurs/t32/blob/main/CODE_OF_CONDUCT.md">Code of conduct</a></li>
		</ul>
	</aside>

	<div class="main">
		<h2 id="standing"><span class="secno">1</span>Where the specification stands</h2>

		<ol class="track">
			{#each track as step (step.name)}
				<li class={step.state}>
					<span class="dot" aria-hidden="true"></span>
					<span class="label">{step.name}</span>
					<span class="when data">{step.when}{step.state === 'current' ? ' · now' : ''}</span>
					<span class="what">{step.what}</span>
				</li>
			{/each}
		</ol>

		<p>
			The draft is complete enough to implement and to test against, and it is not yet frozen. Any part
			may change before version 1.0; each change is made in the open, with its reason, and recorded in
			Appendix D of the <a href="{base}/spec/">specification</a>.
		</p>

		<h2 id="ways"><span class="secno">2</span>Ways to take part</h2>

		<dl class="ways">
			<dt>Question it</dt>
			<dd>
				Read the <a href="{base}/spec/">specification</a> as if you had to implement it from the text alone.
				Anything ambiguous, missing or wrong is worth an <a href="https://github.com/oddurs/t32/issues/new/choose">erratum</a>,
				including a sentence that two careful readers could take differently.
			</dd>
			<dt>Weigh in</dt>
			<dd>
				The open questions below are undecided. Experience from your own systems is the most useful
				evidence; bring it to <a href={discussions}>Discussions</a>.
			</dd>
			<dt>Implement it</dt>
			<dd>
				Load the <a href="https://github.com/oddurs/t32/blob/main/vectors/t32-vectors.json">conformance vectors</a>,
				make them pass in your language, and add your implementation to the list. Three independent
				implementations are a condition of version 1.0.
			</dd>
			<dt>Help govern it</dt>
			<dd>
				A working group will be chartered from active implementers and reviewers. If you would serve on it,
				say so in <a href={discussions}>Discussions</a>; interest expressed now shapes the charter later.
			</dd>
		</dl>

		<h2 id="questions"><span class="secno">3</span>Open questions</h2>

		<p>
			These are the questions the editor considers open before version 1.0, as listed in Appendix C of
			the specification. Each states the current position and the kind of evidence that would change it.
		</p>

		<div class="doc questions">{@html data.questions}</div>

		<h2 id="implementations"><span class="secno">4</span>Implementations</h2>

		<div class="doc">{@html data.implementations}</div>

		<h2 id="governance"><span class="secno">5</span>How decisions are made</h2>

		<div class="doc">{@html data.governance}</div>
	</div>
</div>

<style>
	.toc .label,
	.elsewhere {
		margin: 0 0 var(--s3);
	}
	.elsewhere {
		margin-top: var(--s5);
	}
	.toc ol,
	.links {
		list-style: none;
		padding: 0;
		margin: 0;
		font-size: 0.875rem;
		line-height: 1.35;
	}
	.toc li {
		display: grid;
		grid-template-columns: 1.75rem 1fr;
		margin: 0 0 var(--s2);
		padding: 0;
	}
	.links li {
		margin: 0 0 var(--s2);
		padding: 0;
	}
	.no {
		font-size: 0.8125rem;
		font-weight: var(--w-medium);
		font-variant-numeric: tabular-nums;
		color: var(--ink-3);
	}
	.toc a {
		color: var(--ink-2);
		text-decoration: none;
	}
	.toc a:hover {
		color: var(--accent);
	}

	/* The path from draft to standard. */
	.track {
		list-style: none;
		padding: 0;
		margin: var(--s5) 0 var(--s5);
		display: grid;
		grid-template-columns: repeat(5, minmax(0, 1fr));
		gap: 0 var(--s4);
		position: relative;
	}
	.track::before {
		content: '';
		position: absolute;
		left: 0.3rem;
		right: 0;
		top: 0.3rem;
		border-top: 1px solid var(--rule-strong);
	}
	.track li {
		display: flex;
		flex-direction: column;
		gap: var(--s1);
		margin: 0;
		padding: 0;
		position: relative;
	}
	.dot {
		width: 0.6rem;
		height: 0.6rem;
		border-radius: 50%;
		border: 1px solid var(--ink-3);
		background: var(--paper);
		margin-bottom: var(--s3);
	}
	.done .dot {
		background: var(--ink-3);
	}
	.current .dot {
		background: var(--accent);
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--paper), 0 0 0 4px var(--accent);
	}
	.track .label {
		color: var(--ink);
	}
	.current .label {
		color: var(--accent);
	}
	.when {
		font-size: 0.75rem;
		color: var(--ink-3);
	}
	.what {
		font-size: 0.875rem;
		line-height: 1.45;
		color: var(--ink-2);
	}

	.ways {
		display: grid;
		grid-template-columns: 10rem minmax(0, 1fr);
		margin: 0 0 var(--s4);
		max-width: var(--measure);
	}
	.ways dt,
	.ways dd {
		margin: 0;
		padding: var(--s3) 0;
		border-bottom: 1px solid var(--rule);
	}
	.ways dt {
		font-weight: var(--w-strong);
	}
	.ways dd {
		color: var(--ink-2);
	}

	.questions :global(p) {
		max-width: var(--measure);
		padding-left: var(--s4);
		border-left: 1px solid var(--rule-strong);
	}
	.questions :global(p strong:first-child) {
		display: block;
		color: var(--ink);
	}

	@media (max-width: 960px) {
		.toc ol {
			columns: 2 14rem;
			column-gap: var(--s6);
		}
		.toc li {
			break-inside: avoid;
		}
	}
	@media (max-width: 700px) {
		.track {
			grid-template-columns: 1fr;
			gap: var(--s4);
			padding-left: var(--s5);
		}
		.track::before {
			left: 0.3rem;
			top: 0.3rem;
			bottom: 0;
			right: auto;
			border-top: 0;
			border-left: 1px solid var(--rule-strong);
		}
		.dot {
			position: absolute;
			left: calc(-1 * var(--s5));
			top: 0.3rem;
		}
		.ways {
			grid-template-columns: 1fr;
		}
		.ways dt {
			border-bottom: 0;
			padding-bottom: 0;
		}
	}
</style>
