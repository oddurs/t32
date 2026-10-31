<script lang="ts">
	import { base } from '$app/paths';
	import Clock from '$lib/components/Clock.svelte';
</script>

<svelte:head>
	<title>T32 — a compact universal timestamp</title>
	<meta name="description" content="T32 writes any moment as 8 sortable base32 characters: atomic seconds since 2000, where a stamp's length is its precision." />
</svelte:head>

<div class="titleblock">
	<div class="docno">T32 · DRAFT 0.2</div>
	<h1>T32: a compact universal timestamp</h1>
	<dl>
		<dt>Status</dt><dd>Draft proposal, open for comment</dd>
		<dt>Date</dt><dd>2026-10-31</dd>
		<dt>Timescale</dt><dd>TAI (International Atomic Time)</dd>
		<dt>Reference</dt><dd><a href="{base}/spec/">Specification</a> · <a href="https://crates.io/crates/t32">Rust (crates.io)</a> · <a href="https://github.com/oddurs/t32/blob/main/web/src/lib/t32.ts">TypeScript</a></dd>
	</dl>
</div>

<Clock />

<h2>Summary</h2>

<p>
	T32 writes any moment as 8 base32 characters: a count of atomic seconds since 2000, offset so
	the past encodes too. The canonical form covers roughly 15,420 BCE to 19,420 CE at one-second
	resolution.
</p>

<ul>
	<li><strong>Length is resolution.</strong> Drop characters for a coarser span, append them for finer time. Every stamp names an interval, never a false instant.</li>
	<li><strong>Text order is time order.</strong> Sort the strings byte-wise and they are in chronological order; a prefix is a range query.</li>
	<li><strong>Uniform seconds.</strong> T32 counts TAI, so there are no leap seconds inside it. From 1961 on, subtracting two stamps gives true elapsed time.</li>
	<li><strong>Human-safe.</strong> Crockford base32: no I, L, O or U, case-insensitive on input, ASCII only.</li>
</ul>

<h2>Compared</h2>

<div class="table-wrap">
	<table>
		<thead>
			<tr><th>Format</th><th>Example (1 s)</th><th class="num">Length</th><th>Sortable as text</th><th>Resolution stated</th><th>Leap seconds</th></tr>
		</thead>
		<tbody>
			<tr><td>ISO 8601 (UTC)</td><td class="mono">2026-10-31T03:13:00Z</td><td class="num">20</td><td>If same format and zone</td><td>Partly (by truncation)</td><td>Has 23:59:60</td></tr>
			<tr><td>Unix time</td><td class="mono">1793416380</td><td class="num">10</td><td>Only if zero-padded</td><td>No</td><td>Repeats a second</td></tr>
			<tr><td>TAI64 (external form)</td><td class="mono">@400000006ae55ce1</td><td class="num">17</td><td>Yes</td><td>No (fixed per variant)</td><td>None (TAI)</td></tr>
			<tr><td>ULID (time part)</td><td class="mono">01M67Z4FK0</td><td class="num">10</td><td>Yes</td><td>No (ms)</td><td>Follows Unix time</td></tr>
			<tr class="hl"><td><strong>T32</strong></td><td class="mono">G0S7G6B1</td><td class="num">8</td><td>Yes</td><td>Yes, by length</td><td>None (TAI)</td></tr>
		</tbody>
	</table>
</div>

<h2>Start here</h2>

<ul>
	<li><a href="{base}/playground/">Playground</a>: paste a stamp or a date and see what it means.</li>
	<li><a href="{base}/spec/">Specification</a>: the normative text, with test vectors.</li>
	<li><a href="{base}/docs/">Docs</a>: storing, querying and converting T32 in practice, and the FAQ.</li>
	<li><a href="{base}/tables/">Tables</a>: alphabet, eras, precision, durations, leap seconds.</li>
</ul>
