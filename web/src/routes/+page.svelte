<script lang="ts">
	import { base } from '$app/paths';
	import Clock from '$lib/components/Clock.svelte';
</script>

<svelte:head>
	<title>T32 — a compact universal timestamp</title>
	<meta name="description" content="T32 writes any moment as 8 sortable base32 characters: atomic seconds since 2000, where a stamp's length is its resolution. A draft specification, open for review." />
</svelte:head>

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft 0.2</span>
		<h1>A compact universal timestamp</h1>
		<p class="lede">
			T32 writes any moment as eight base32 characters: a count of atomic seconds since 2000,
			offset so the past encodes too. A stamp's length is its resolution, and its text sorts as time.
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

		<h2>Principles</h2>

		<dl class="principles">
			<dt>Length is resolution</dt>
			<dd>Drop characters for a coarser span, append them for finer time. Every stamp names an interval, never a false instant.</dd>
			<dt>Text order is time order</dt>
			<dd>Sort the strings byte by byte and they are in chronological order. A prefix is a range query.</dd>
			<dt>Uniform seconds</dt>
			<dd>T32 counts TAI, which has no leap seconds. From 1961 on, subtracting two stamps gives elapsed time.</dd>
			<dt>Exact and stable</dt>
			<dd>Conversion from UTC is exact at every step of the leap-second table, and a stamp, once written, always means the same interval.</dd>
		</dl>

		<h2>Compared</h2>

		<div class="table-wrap">
			<table>
				<thead>
					<tr><th>Format</th><th>One second</th><th class="num">Chars</th><th>Sorts as text</th><th>States resolution</th><th>Leap seconds</th></tr>
				</thead>
				<tbody>
					<tr><td>ISO 8601, UTC</td><td class="data">2026-10-31T03:13:00Z</td><td class="num">20</td><td>Within one format and zone</td><td>Partly</td><td>Has 23:59:60</td></tr>
					<tr><td>Unix time</td><td class="data">1793416380</td><td class="num">10</td><td>When zero-padded</td><td>No</td><td>Repeats a second</td></tr>
					<tr><td>TAI64 label</td><td class="data">@400000006ae55ce1</td><td class="num">17</td><td>Yes</td><td>Fixed per variant</td><td>None</td></tr>
					<tr><td>ULID, time part</td><td class="data">01M67Z4FK0</td><td class="num">10</td><td>Yes</td><td>No, always ms</td><td>Follows Unix time</td></tr>
					<tr class="hl"><td><strong>T32</strong></td><td class="data">G0S7G6B1</td><td class="num">8</td><td>Yes</td><td>Yes, by length</td><td>None</td></tr>
				</tbody>
			</table>
		</div>

		<h2>Status</h2>

		<p>
			T32 is a draft specification, published for public review. It is complete enough to implement and
			to test against, and it will not be frozen until it has been read widely, implemented independently
			and questioned properly. Comments, implementations and objections are welcome, and the
			<a href="{base}/participate/">participation page</a> sets out where they go and what they lead to.
		</p>

		<h2>Where to begin</h2>

		<dl class="principles">
			<dt><a href="{base}/playground/">Playground</a></dt>
			<dd>Paste a stamp or a date and see the interval it names.</dd>
			<dt><a href="{base}/spec/">Specification</a></dt>
			<dd>The normative text: definitions, conversion from UTC, grammar, test vectors.</dd>
			<dt><a href="{base}/docs/">Guide</a></dt>
			<dd>Using T32 in practice: storage, range queries, Rust and TypeScript, and the questions people ask.</dd>
			<dt><a href="{base}/tables/">Tables</a></dt>
			<dd>Reference tables: alphabet, eras, prefixes by year, leap seconds.</dd>
			<dt><a href="{base}/participate/">Participate</a></dt>
			<dd>Open questions, implementations, and the path to version 1.0.</dd>
		</dl>
	</div>
</div>

<style>
	.principles {
		display: grid;
		grid-template-columns: 13rem minmax(0, 1fr);
		gap: 0;
		margin: 0 0 var(--s4);
		max-width: var(--measure);
	}
	.principles dt,
	.principles dd {
		margin: 0;
		padding: var(--s3) 0;
		border-bottom: 1px solid var(--rule);
	}
	.principles dt {
		font-weight: var(--w-strong);
		padding-right: var(--s4);
	}
	.principles dd {
		color: var(--ink-2);
	}
	@media (max-width: 600px) {
		.principles {
			grid-template-columns: 1fr;
		}
		.principles dt {
			border-bottom: 0;
			padding-bottom: 0;
		}
		.principles dd {
			padding-top: var(--s1);
		}
	}
</style>
