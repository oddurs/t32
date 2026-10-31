<script lang="ts">
	import {
		ALPHABET, MAX_LEN, CANONICAL_LEN, LEAP_TABLE, parse, format, fromBiasedFx, startFx, endFx,
		fxToUtcString, fxToUtcStringExact, parseUtc, formatDuration, unitSeconds, civilFromDays
	} from '$lib/t32';
	import { humanSpan, exactUnit } from '$lib/describe';

	const NS = 1_000_000_000n;

	// Astronomical year → readable year.
	const yearText = (fx: bigint) => {
		const y = Number(/^([+-]?\d+)-/.exec(fxToUtcString(fx))![1]);
		return y <= 0 ? `${(1 - y).toLocaleString('en-US')} BCE` : y.toLocaleString('en-US', { useGrouping: y >= 10000 });
	};
	const day = (fx: bigint) => fxToUtcString(fx).replace(/T.*/, '');
	const minute = (fx: bigint) => fxToUtcString(fx).replace(/:\d\dZ$/, 'Z').replace('T', ' ');

	// 1. Alphabet
	const aliases: Record<string, string> = { '0': 'O, o', '1': 'I, i, L, l' };
	const alphabet = [...ALPHABET].map((c, v) => ({ c, v, bits: v.toString(2).padStart(5, '0'), alias: aliases[c] ?? '' }));

	// 2. Precision
	const binary: Record<number, string> = { 8: 'T32/40', 12: 'T32/64', 16: 'T32/80' };
	const precision = Array.from({ length: MAX_LEN }, (_, i) => {
		const n = i + 1;
		return { n, exact: exactUnit(n), human: humanSpan(unitSeconds(n)), bits: 5 * n, binary: binary[n] ?? '' };
	});

	// 3. Eras
	const eras = [...ALPHABET].map((c) => {
		const s = parse(c);
		return { c, from: yearText(startFx(s)), to: yearText(endFx(s)) };
	});

	// 4. Generations, 1900–2100
	const prefixesBetween = (from: string, to: string, len: number) => {
		const out: { p: string; from: bigint; to: bigint }[] = [];
		let s = fromBiasedFx(parseUtc(from), len);
		const stop = parseUtc(to);
		while (startFx(s) < stop) {
			out.push({ p: format(s), from: startFx(s), to: endFx(s) });
			s = { value: s.value + 1n, len };
		}
		return out;
	};
	const generations = prefixesBetween('1900-01-01', '2100-01-01', 2);
	const years = prefixesBetween('2020-01-01', '2036-01-01', 3);

	// 6. Durations
	const durations: [string, bigint][] = [
		['1 second', NS],
		['1 minute', 60n * NS],
		['1 hour', 3600n * NS],
		['1 day (86,400 SI s)', 86_400n * NS],
		['1 week', 604_800n * NS],
		['30 days', 2_592_000n * NS],
		['365 days', 31_536_000n * NS],
		['Julian year (365.25 d)', 31_557_600n * NS],
		['1 millisecond', 1_000_000n],
		['1/32 second', NS / 32n],
		['1/1024 second', NS / 1024n]
	];

	// 7. Notable moments
	const moments: [string, string][] = [
		['Start of range', '00000000'],
		['Royal Observatory foundation stone (Gregorian)', '1675-08-20T00:00:00'],
		['Start of the 1961 UTC definition', '1961-01-01T00:00:00'],
		['Apollo 11 lunar landing', '1969-07-20T20:17:40'],
		['Unix epoch', '1970-01-01T00:00:00'],
		['UTC becomes TAI − 10 s', '1972-01-01T00:00:00'],
		['GPS epoch', '1980-01-06T00:00:00'],
		['T32 epoch (2000-01-01 TAI)', '1999-12-31T23:59:28'],
		['Last leap second (so far)', '2016-12-31T23:59:60'],
		['Signed 32-bit Unix time overflows', '2038-01-19T03:14:08'],
		['End of range', 'ZZZZZZZZ']
	];
	const momentRows = moments.map(([name, v]) => {
		const s = /^\d|^[+-]/.test(v) && v.includes('-') && v.length > 9 ? fromBiasedFx(parseUtc(v), CANONICAL_LEN) : parse(v);
		return { name, stamp: format(s), utc: fxToUtcStringExact(startFx(s)) };
	});

	// 8. Leap seconds: the inserted second before each step after 1972-01-01.
	const leaps = LEAP_TABLE.slice(1).map(([t, d]) => {
		const [y, m, dd] = civilFromDays(t / 86400n - 1n);
		const label = `${y}-${String(m).padStart(2, '0')}-${String(dd).padStart(2, '0')}T23:59:60`;
		return { label: label.replace('T', ' '), stamp: format(fromBiasedFx(parseUtc(label), CANONICAL_LEN)), after: Number(d) };
	});

	// 10. Test vectors
	const vectors: [string, string, number][] = [
		['T32 epoch', '1999-12-31T23:59:28', 8],
		['Unix epoch (rubber era)', '1970-01-01T00:00:00', 8],
		['Apollo 11 landing', '1969-07-20T20:17:40', 8],
		['UTC becomes integer', '1972-01-01T00:00:00', 8],
		['Before last leap second', '2016-12-31T23:59:59', 8],
		['Last leap second', '2016-12-31T23:59:60', 8],
		['After last leap second', '2017-01-01T00:00:00', 8],
		['Spec date', '2026-10-31T03:13:00', 8],
		['Spec date + 0.5 s', '2026-10-31T03:13:00.5', 10],
		['Spec date + 0.123 s', '2026-10-31T03:13:00.123', 12],
		['Pre-1961, taken as is', '1675-08-20T00:00:00', 8]
	];
	const vectorRows = vectors.map(([name, utc, n]) => ({ name, utc, n, stamp: format(fromBiasedFx(parseUtc(utc), n)) }));

	const sections = [
		['alphabet', 'Alphabet'],
		['precision', 'Precision by length'],
		['eras', 'Eras (first character)'],
		['generations', 'Two-character prefixes, 1900–2100'],
		['years', 'Three-character prefixes, 2020–2035'],
		['durations', 'Common durations'],
		['moments', 'Notable moments'],
		['leap-seconds', 'Leap seconds'],
		['rubber', 'TAI − UTC, 1961–1971'],
		['vectors', 'Test vectors']
	];
</script>

<svelte:head>
	<title>T32 tables</title>
	<meta name="description" content="Reference tables for T32: alphabet, precision, eras, prefixes by year, durations, notable moments, leap seconds and test vectors." />
</svelte:head>

<div class="titleblock">
	<div class="docno">T32 · DRAFT 0.1 · REFERENCE TABLES</div>
	<h1>Tables</h1>
	<p class="muted small">Every value on this page is computed by the reference TypeScript implementation when the site is built.</p>
</div>

<nav class="toc" aria-label="Tables">
	<ol>
		{#each sections as [id, title], i (id)}
			<li><span class="no">{i + 1}</span> <a href="#{id}">{title}</a></li>
		{/each}
	</ol>
</nav>

<h2 id="alphabet"><span class="secno">1</span> Alphabet</h2>
<p>Crockford base32. The letters I, L, O and U are never written; on input, I and L read as 1 and O as 0. U is rejected.</p>
<div class="table-wrap">
	<table>
		<thead><tr><th>Char</th><th class="num">Value</th><th>Bits</th><th>Also accepted</th><th>Char</th><th class="num">Value</th><th>Bits</th><th>Also accepted</th></tr></thead>
		<tbody>
			{#each alphabet.slice(0, 16) as a, i (a.c)}
				{@const b = alphabet[i + 16]}
				<tr>
					<td class="mono">{a.c}</td><td class="num">{a.v}</td><td class="mono">{a.bits}</td><td class="mono">{a.alias}</td>
					<td class="mono">{b.c}</td><td class="num">{b.v}</td><td class="mono">{b.bits}</td><td class="mono">{b.alias}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<h2 id="precision"><span class="secno">2</span> Precision by length</h2>
<div class="table-wrap">
	<table>
		<thead><tr><th class="num">Length</th><th>Unit (exact)</th><th>Unit (approx.)</th><th class="num">Bits</th><th>Binary form</th></tr></thead>
		<tbody>
			{#each precision as p (p.n)}
				<tr class:hl={p.n === CANONICAL_LEN}>
					<td class="num">{p.n}</td><td class="mono">{p.exact}</td><td>{p.human}</td><td class="num">{p.bits}</td><td class="mono">{p.binary}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<h2 id="eras"><span class="secno">3</span> Eras (first character)</h2>
<p>Each era is 2^35 seconds, about 1,089 years. Years are proleptic Gregorian.</p>
<div class="table-wrap">
	<table>
		<thead><tr><th>Char</th><th>From</th><th>Until</th><th>Char</th><th>From</th><th>Until</th></tr></thead>
		<tbody>
			{#each eras.slice(0, 16) as e, i (e.c)}
				{@const f = eras[i + 16]}
				<tr class:hl={e.c === 'G' || f.c === 'G'}>
					<td class="mono">{e.c}</td><td>{e.from}</td><td>{e.to}</td>
					<td class="mono">{f.c}</td><td>{f.from}</td><td>{f.to}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<h2 id="generations"><span class="secno">4</span> Two-character prefixes, 1900–2100</h2>
<p>Each is 2^30 seconds, about 34 years. Boundaries are given to the day.</p>
<div class="table-wrap">
	<table>
		<thead><tr><th>Prefix</th><th>From (UTC)</th><th>Until (UTC)</th></tr></thead>
		<tbody>
			{#each generations as g (g.p)}
				<tr><td class="mono">{g.p}</td><td class="mono">{day(g.from)}</td><td class="mono">{day(g.to)}</td></tr>
			{/each}
		</tbody>
	</table>
</div>

<h2 id="years"><span class="secno">5</span> Three-character prefixes, 2020–2035</h2>
<p>Each is 2^25 seconds, about 388 days. Useful as partition keys and for range queries.</p>
<div class="table-wrap">
	<table>
		<thead><tr><th>Prefix</th><th>From (UTC)</th><th>Until (UTC)</th></tr></thead>
		<tbody>
			{#each years as y (y.p)}
				<tr><td class="mono">{y.p}</td><td class="mono">{minute(y.from)}</td><td class="mono">{minute(y.to)}</td></tr>
			{/each}
		</tbody>
	</table>
</div>

<h2 id="durations"><span class="secno">6</span> Common durations</h2>
<p>Durations are SI seconds with a mandatory sign. Fractions are truncated to four base32 digits.</p>
<div class="table-wrap">
	<table>
		<thead><tr><th>Duration</th><th class="num">Seconds</th><th>T32 duration</th></tr></thead>
		<tbody>
			{#each durations as [name, ns] (name)}
				<tr>
					<td>{name}</td>
					<td class="num mono">{(Number(ns) / 1e9).toLocaleString('en-US', { maximumFractionDigits: 6 })}</td>
					<td class="mono">{formatDuration(ns, 4)}</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<h2 id="moments"><span class="secno">7</span> Notable moments</h2>
<p>Each moment encoded at length 8. Before 1972, TAI − UTC was fractional, so a one-second stamp starts part-way through a UTC second.</p>
<div class="table-wrap">
	<table>
		<thead><tr><th>Moment</th><th>Stamp starts (UTC)</th><th>T32</th></tr></thead>
		<tbody>
			{#each momentRows as m (m.name)}
				<tr><td>{m.name}</td><td class="mono small">{m.utc}</td><td class="mono">{m.stamp}</td></tr>
			{/each}
		</tbody>
	</table>
</div>

<h2 id="leap-seconds"><span class="secno">8</span> Leap seconds</h2>
<p>
	All 27 inserted leap seconds, each with its own T32 stamp: in T32 they are ordinary seconds. The table is
	valid until 2027-06-28; IERS Bulletin C 72 announced no leap second for the end of 2026.
</p>
<div class="table-wrap">
	<table>
		<thead><tr><th class="num">#</th><th>Inserted second (UTC)</th><th>T32</th><th class="num">TAI − UTC after</th></tr></thead>
		<tbody>
			{#each leaps as l, i (l.label)}
				<tr><td class="num">{i + 1}</td><td class="mono">{l.label}</td><td class="mono">{l.stamp}</td><td class="num">{l.after} s</td></tr>
			{/each}
		</tbody>
	</table>
</div>

<h2 id="rubber"><span class="secno">9</span> TAI − UTC, 1961–1971</h2>
<p>Before 1972, UTC ran at an adjusted rate. TAI − UTC = a + (MJD − m) × r seconds, with MJD of the UTC instant. Before 1961 it is defined as 0.</p>
<div class="table-wrap">
	<table>
		<thead><tr><th>From</th><th class="num">a (s)</th><th class="num">m (MJD)</th><th class="num">r (s/day)</th></tr></thead>
		<tbody>
			{#each [
				['1961-01-01', '1.4228180', 37300, '0.0012960'], ['1961-08-01', '1.3728180', 37300, '0.0012960'],
				['1962-01-01', '1.8458580', 37665, '0.0011232'], ['1963-11-01', '1.9458580', 37665, '0.0011232'],
				['1964-01-01', '3.2401300', 38761, '0.0012960'], ['1964-04-01', '3.3401300', 38761, '0.0012960'],
				['1964-09-01', '3.4401300', 38761, '0.0012960'], ['1965-01-01', '3.5401300', 38761, '0.0012960'],
				['1965-03-01', '3.6401300', 38761, '0.0012960'], ['1965-07-01', '3.7401300', 38761, '0.0012960'],
				['1965-09-01', '3.8401300', 38761, '0.0012960'], ['1966-01-01', '4.3131700', 39126, '0.0025920'],
				['1968-02-01', '4.2131700', 39126, '0.0025920']
			] as row (row[0])}
				<tr><td class="mono">{row[0]}</td><td class="num mono">{row[1]}</td><td class="num">{row[2]}</td><td class="num mono">{row[3]}</td></tr>
			{/each}
		</tbody>
	</table>
</div>

<h2 id="vectors"><span class="secno">10</span> Test vectors</h2>
<p>An implementation is conforming when it reproduces these (and the decoding and parsing vectors in the <a href="../spec/#test-vectors">specification</a>).</p>
<div class="table-wrap">
	<table>
		<thead><tr><th>Case</th><th>UTC</th><th class="num">Length</th><th>Stamp</th></tr></thead>
		<tbody>
			{#each vectorRows as v (v.name)}
				<tr><td>{v.name}</td><td class="mono small">{v.utc}</td><td class="num">{v.n}</td><td class="mono">{v.stamp}</td></tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	.toc ol {
		list-style: none;
		padding: 0;
		margin: 0 0 24px;
		columns: 2 16em;
		column-gap: 32px;
		font-size: 14px;
	}
	.toc li {
		margin: 0 0 4px;
		break-inside: avoid;
	}
	.no {
		display: inline-block;
		min-width: 1.6em;
		text-align: right;
		color: var(--muted);
		margin-right: 6px;
	}
	h2 {
		scroll-margin-top: 16px;
	}
</style>
