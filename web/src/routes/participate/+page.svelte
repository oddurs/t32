<script lang="ts">
	import Head from '$lib/components/Head.svelte';
	import Toc from '$lib/components/Toc.svelte';
	import { DRAFT } from '$lib/site';
	import { resolve } from '$app/paths';
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
		{ name: 'Draft 0.2', when: '2026-10-31', what: 'Complete enough to implement. Public review opened.', state: 'done' },
		{ name: 'Draft 0.3', when: '2026-11-01', what: 'Fixes from the first review.', state: 'current' },
		{ name: 'Review', when: 'Open', what: 'Open questions settled, and three implementations passing.', state: '' },
		{ name: 'Version 1.0', when: 'After 60 days', what: 'Core rules fixed. Stamps keep their meaning.', state: '' },
		{ name: 'Working group', when: 'With 1.0', what: 'A group of implementers and reviewers takes over.', state: '' }
	];

	const discussions = 'https://github.com/serre-ai/t32/discussions';
</script>

<Head title="Participate" description="How T32 becomes a standard: where the draft stands, ways to take part, the open questions, implementations, and how decisions are made." path="/participate/" />

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft {DRAFT} · Participation</span>
		<h1>Participate</h1>
		<p class="lede">
			T32 is a draft. Before it can become a standard, people other than its author need to read it,
			build it and report what's wrong with it. This page explains how to take part and what happens next.
		</p>
	</header>

	<aside class="rail">
		<Toc toc={sections.map(([id, text], i) => ({ depth: 2, id, text, label: String(i + 1) }))} title="On this page" />
		<p class="label elsewhere">Elsewhere</p>
		<ul class="links">
			<li><a href={discussions}>Discussions</a></li>
			<li><a href="https://github.com/serre-ai/t32/issues/new/choose">Report a problem</a></li>
			<li><a href="https://github.com/serre-ai/t32/blob/main/CONTRIBUTING.md">Contributing</a></li>
			<li><a href="https://github.com/serre-ai/t32/blob/main/CODE_OF_CONDUCT.md">Code of conduct</a></li>
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
			The draft is complete enough to implement and test against, but it isn't final. Any part may change
			before version 1.0. Every change is made in public, with the reason for it, and listed in Appendix D
			of the <a href={resolve('spec/')}>specification</a>.
		</p>

		<h2 id="ways"><span class="secno">2</span>Ways to take part</h2>

		<dl class="ways">
			<dt>Question it</dt>
			<dd>
				Read the <a href={resolve('spec/')}>specification</a> as if you had to implement it from the text alone.
				If anything is unclear, missing or wrong, <a href="https://github.com/serre-ai/t32/issues/new/choose">report it</a>.
				That includes any sentence two careful readers could take to mean different things.
			</dd>
			<dt>Weigh in</dt>
			<dd>
				The questions below haven't been decided yet. What you've seen in your own systems is the most
				useful input. Post it in <a href={discussions}>Discussions</a>.
			</dd>
			<dt>Implement it</dt>
			<dd>
				Load the <a href="https://github.com/serre-ai/t32/blob/main/vectors/t32-vectors.json">conformance vectors</a>,
				make them pass in your language, and add your implementation to the list. Version 1.0 needs at
				least three implementations written independently.
			</dd>
			<dt>Help govern it</dt>
			<dd>
				A working group made up of active implementers and reviewers will take over the project. If you'd
				like to be part of it, say so in <a href={discussions}>Discussions</a>. Knowing who is interested
				helps decide how the group is set up.
			</dd>
		</dl>

		<h2 id="questions"><span class="secno">3</span>Open questions</h2>

		<div class="doc questions">{@html data.questions}</div>

		<h2 id="implementations"><span class="secno">4</span>Implementations</h2>

		<div class="doc">{@html data.implementations}</div>

		<h2 id="governance"><span class="secno">5</span>How decisions are made</h2>

		<div class="doc">{@html data.governance}</div>
	</div>
</div>

<style>
	.elsewhere {
		margin: var(--s5) 0 var(--s3);
	}
	.links {
		list-style: none;
		padding: 0;
		margin: 0;
		font-size: var(--small);
		line-height: var(--lh-tight);
	}
	.links li {
		margin: 0 0 var(--s2);
		padding: 0;
	}

	/* The path from draft to standard. */
	.track {
		list-style: none;
		padding: 0;
		margin: var(--s5) 0 var(--s5);
		display: grid;
		/* One column per step, however many there are. */
		grid-auto-flow: column;
		grid-auto-columns: minmax(0, 1fr);
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
		font-size: var(--label);
		color: var(--ink-3);
	}
	.what {
		font-size: var(--small);
		line-height: var(--lh-small);
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

	@media (max-width: 700px) {
		.track {
			grid-auto-flow: row;
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
