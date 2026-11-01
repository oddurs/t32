<script lang="ts">
	import { base } from '$app/paths';
	import Clock from '$lib/components/Clock.svelte';
	import Head from '$lib/components/Head.svelte';
	import ZoomFigure from '$lib/components/ZoomFigure.svelte';
	import { parse, parseUtc, fromFx, format } from '$lib/t32';

	// The worked example, computed by the implementation so the numbers cannot drift.
	const BIAS = 2n ** 39n;
	const example = fromFx(parseUtc('2026-10-31T03:13:00Z'), 8);
	const n = (v: bigint) => v.toLocaleString('en-US');
	const worked = {
		tai: n(example.value - BIAS),
		biased: n(example.value),
		stamp: format(example)
	};
	const coarse = parse('G0S7');
</script>

<Head
	description="T32 writes any moment as eight sortable characters: atomic seconds since 2000, in base32, where a stamp's length is its resolution. A draft specification, open for review."
	path="/"
/>

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft 0.2</span>
		<h1>A compact universal timestamp</h1>
		<p class="lede">
			T32 writes any moment as eight characters: a count of atomic seconds since 2000, in base32.
			Fewer characters name a longer span, more name finer time, and sorting the text sorts the times.
		</p>
		<p class="cta">
			<a class="btn primary" href="{base}/playground/">Try it in the playground</a>
			<a class="btn" href="{base}/spec/">Read the specification</a>
		</p>
	</header>

	<aside class="rail after" aria-label="Document">
		<p class="label">Document</p>
		<dl class="meta">
			<dt>Status</dt><dd>Draft 0.2, for public review</dd>
			<dt>Date</dt><dd>2026-10-31</dd>
			<dt>Editor</dt><dd>Oddur Sigurdsson</dd>
			<dt>Time scale</dt><dd>TAI, International Atomic Time</dd>
			<dt>Text</dt><dd><a href="{base}/spec/">Specification</a></dd>
			<dt>Conformance</dt><dd><a href="https://github.com/oddurs/t32/blob/main/vectors/t32-vectors.json">594 vectors</a></dd>
			<dt>Code</dt><dd><a href="https://crates.io/crates/t32">Rust</a> · <a href="https://github.com/oddurs/t32/blob/main/web/src/lib/t32.ts">TypeScript</a></dd>
		</dl>
	</aside>

	<div class="main">
		<Clock />

		<h2>How it works</h2>

		<ol class="steps">
			<li>
				<strong>Count.</strong> Take the instant as seconds of International Atomic Time (TAI) since the
				start of 2000. TAI is the atomic time scale that UTC is derived from, and it has no leap seconds,
				so every second is the same length.
			</li>
			<li>
				<strong>Offset.</strong> Add 2<sup>39</sup> seconds, about 17,400 years, so that earlier dates are
				positive too and still sort in order.
			</li>
			<li>
				<strong>Write.</strong> Write the result in Crockford base32, five bits to a character. Eight
				characters count whole seconds; each character added is 32 times finer, each one removed 32 times
				coarser.
			</li>
		</ol>

		<div class="table-wrap">
			<table class="worked">
				<tbody>
					<tr><th>UTC</th><td class="data">2026-10-31 03:13:00</td></tr>
					<tr><th>TAI seconds since 2000</th><td class="data">{worked.tai}</td></tr>
					<tr><th>Plus 2<sup>39</sup></th><td class="data">{worked.biased}</td></tr>
					<tr><th>In base32</th><td class="data"><strong>{worked.stamp}</strong></td></tr>
				</tbody>
			</table>
		</div>

		<p>
			Shorten it to <span class="data">{format(coarse)}</span> and it names the 12 days that contain that
			second. Each character narrows the span before it by a factor of 32; the
			<a href="{base}/playground/">playground</a> shows the same steps for any date.
		</p>

		<ZoomFigure stamp={worked.stamp} />

		<h2>Properties</h2>

		<dl class="pairs">
			<dt>Length is resolution</dt>
			<dd>Drop characters for a coarser span, add them for finer time. A stamp always names an interval, never a false instant.</dd>
			<dt>Text order is time order</dt>
			<dd>Sorted byte by byte, stamps are in chronological order, so a prefix is a range query.</dd>
			<dt>Uniform seconds</dt>
			<dd>T32 counts TAI, which has no leap seconds. From 1961 on, the difference of two stamps is the time elapsed between them.</dd>
			<dt>Exact and stable</dt>
			<dd>Conversion from UTC is exact at every step of the leap-second table, and a stamp, once written, keeps its meaning.</dd>
		</dl>

		<h2>Compared</h2>

		<div class="table-wrap">
			<table>
				<thead>
					<tr><th>Format</th><th>The same second</th><th class="num">Chars</th><th>Sorts as text</th><th>States resolution</th><th>Leap seconds</th></tr>
				</thead>
				<tbody>
					<tr><td>ISO 8601, UTC</td><td class="data">2026-10-31T03:13:00Z</td><td class="num">20</td><td>Within one format and zone</td><td>Partly</td><td>Has 23:59:60</td></tr>
					<tr><td>Unix time</td><td class="data">1793416380</td><td class="num">10</td><td>When zero-padded</td><td>No</td><td>Repeats a second</td></tr>
					<tr><td>TAI64 label</td><td class="data">@400000006ae55ce1</td><td class="num">17</td><td>Yes</td><td>Fixed per variant</td><td>None</td></tr>
					<tr><td>ULID, time part</td><td class="data">01M67Z4FK0</td><td class="num">10</td><td>Yes</td><td>No, always ms</td><td>Follows Unix time</td></tr>
					<tr class="hl"><td><strong>T32</strong></td><td class="data">{worked.stamp}</td><td class="num">8</td><td>Yes</td><td>Yes, by length</td><td>None</td></tr>
				</tbody>
			</table>
		</div>

		<h2>Status</h2>

		<p>
			T32 is a draft, published for public review. It is complete enough to implement and test against,
			and it will not be frozen until it has been read widely, implemented independently and questioned
			properly. Comments, implementations and objections are all welcome; the
			<a href="{base}/participate/">participation page</a> says where they go and what they lead to.
		</p>

		<h2>Where to begin</h2>

		<dl class="pairs">
			<dt><a href="{base}/playground/">Playground</a></dt>
			<dd>Type a date or a stamp and see what it means.</dd>
			<dt><a href="{base}/docs/">Guide</a></dt>
			<dd>Reading stamps, choosing a length, storing them in SQL, the Rust and TypeScript libraries, and common questions.</dd>
			<dt><a href="{base}/spec/">Specification</a></dt>
			<dd>The normative text: definitions, grammar, conversion from UTC and test vectors.</dd>
			<dt><a href="{base}/tables/">Tables</a></dt>
			<dd>The alphabet, eras, prefixes by year, durations and leap seconds.</dd>
			<dt><a href="{base}/participate/">Participate</a></dt>
			<dd>Open questions, implementations, and the path to version 1.0.</dd>
		</dl>
	</div>
</div>

<style>
	.cta {
		display: flex;
		flex-wrap: wrap;
		gap: var(--s2) var(--s3);
		margin: var(--s5) 0 0;
	}
	.steps {
		max-width: var(--measure);
		padding-left: 1.5em;
	}
	.steps li {
		margin-bottom: var(--s2);
		padding-left: 0.3em;
	}
	.worked {
		margin-top: var(--s2);
	}
	.worked th {
		padding-right: var(--s6);
	}
	.pairs {
		display: grid;
		grid-template-columns: 13rem minmax(0, 1fr);
		gap: 0;
		margin: 0 0 var(--s4);
		max-width: var(--measure);
	}
	.pairs dt,
	.pairs dd {
		margin: 0;
		padding: var(--s3) 0;
		border-bottom: 1px solid var(--rule);
	}
	.pairs dt {
		font-weight: var(--w-strong);
		padding-right: var(--s4);
	}
	.pairs dd {
		color: var(--ink-2);
	}
	@media (max-width: 600px) {
		.pairs {
			grid-template-columns: 1fr;
		}
		.pairs dt {
			border-bottom: 0;
			padding-bottom: 0;
		}
		.pairs dd {
			padding-top: var(--s1);
		}
	}
</style>
