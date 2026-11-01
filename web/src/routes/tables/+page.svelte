<script lang="ts">
	import Head from '$lib/components/Head.svelte';
	import EraStrip from '$lib/components/EraStrip.svelte';
	import LeapChart from '$lib/components/LeapChart.svelte';
	import { exponents } from '$lib/exponents';
	import {
		ALPHABET, MAX_LEN, CANONICAL_LEN, STEPS, parse, format, fromFx, startFx, endFx, deltaAt,
		fxToUtcString, fxToUtcStringExact, parseUtc, formatDuration, durationFromNs, unitSeconds,
		civilFromDays, maxLengthFor, PERIODS
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
	const binary: Record<number, string> = { 8: 'T32-8', 10: 'T32-10', 12: 'T32-12', 14: 'T32-14', 16: 'T32-16' };
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
		let s = fromFx(parseUtc(from), len);
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
		['1/32 second', NS / 32n],
		['1/1024 second', NS / 1024n],
		['1 millisecond (truncated)', 1_000_000n]
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
		const s = /^\d|^[+-]/.test(v) && v.includes('-') && v.length > 9 ? fromFx(parseUtc(v), CANONICAL_LEN) : parse(v);
		return { name, stamp: format(s), utc: fxToUtcStringExact(startFx(s)) };
	});

	// 8. Leap seconds: every whole-second insertion; 9. every other step.
	const dateOf = (unixNs: bigint, dayOffset = 0n) => {
		const [y, m, d] = civilFromDays(unixNs / (86_400n * NS) + dayOffset);
		return `${y}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
	};
	const leaps = STEPS.filter((x) => x.wholeSecond).map((x) => {
		const label = `${dateOf(x.unixNs, -1n)}T23:59:60`;
		const dat = deltaAt(x.unixNs);
		return {
			label: label.replace('T', ' '),
			stamp: format(fromFx(parseUtc(label), CANONICAL_LEN)),
			after: Number(dat.num / dat.den)
		};
	});
	const fractional = STEPS.filter((x) => !x.wholeSecond).map((x) => ({
		date: dateOf(x.unixNs),
		step: (x.seconds > 0 ? '+' : '−') + Math.abs(x.seconds).toFixed(7).replace(/0+$/, ''),
		kind: x.seconds > 0 ? 'Inserted' : 'Removed: labels in the last ' + Math.abs(x.seconds) + ' s of the day before never occurred'
	}));
	// 9. The rate-adjusted periods, read from the implementation's copy of Appendix A.
	const seventh = (v: bigint) => (Number(v) / 1e7).toFixed(7);
	const rubber = PERIODS.filter((p) => p.r !== 0n).map((p) => ({
		from: dateOf(p.start),
		a: seventh(p.a),
		m: Number(p.m / (86_400n * NS)) + 40_587,
		r: seventh(p.r)
	}));
	const resolutions: [string, number][] = [['1 second', 1], ['100 ms', 0.1], ['10 ms', 0.01], ['1 ms', 1e-3], ['1 µs', 1e-6], ['1 ns', 1e-9], ['1 ps', 1e-12]];

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
		['Spec date + 0.5 s (ms clock)', '2026-10-31T03:13:00.500', 10],
		['Spec date + 0.123 s (µs clock)', '2026-10-31T03:13:00.123000', 12],
		['Rubber era, exact (ns clock)', '1966-06-15T12:00:00.123456789', 14],
		['Before 1961, taken as UT', '1675-08-20T00:00:00', 8]
	];
	const vectorRows = vectors.map(([name, utc, n]) => ({ name, utc, n, stamp: format(fromFx(parseUtc(utc), n)) }));

	const sections = [
		['alphabet', 'Alphabet'],
		['precision', 'Length and resolution'],
		['eras', 'Eras (first character)'],
		['generations', 'Two-character prefixes, 1900–2100'],
		['years', 'Three-character prefixes, 2020–2035'],
		['durations', 'Common durations'],
		['moments', 'Notable moments'],
		['leap-seconds', 'Leap seconds'],
		['rubber', 'TAI − UTC, 1961–1971, and its steps'],
		['vectors', 'Test vectors']
	];
</script>

<Head title="Tables" description="Reference tables for T32: the alphabet, length and resolution, eras, prefixes by year, durations, notable moments, leap seconds and test vectors." path="/tables/" />

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft 0.2 · Reference tables</span>
		<h1>Tables</h1>
		<p class="lede">Reference values for T32, each computed by the TypeScript implementation when the site is built.</p>
	</header>

	<aside class="rail">
		<nav class="toc" aria-label="Tables">
			<p class="label">Tables</p>
			<ol>
				{#each sections as [id, title], i (id)}
					<li><span class="no">{i + 1}</span><a href="#{id}">{title}</a></li>
				{/each}
			</ol>
		</nav>
	</aside>

	<div class="main">
		<h2 id="alphabet"><span class="secno">1</span>Alphabet</h2>
		<p>Crockford base32. The letters I, L, O and U are never written; on input, I and L read as 1 and O as 0. U is rejected.</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th>Char</th><th class="num">Value</th><th>Bits</th><th>Also accepted</th><th>Char</th><th class="num">Value</th><th>Bits</th><th>Also accepted</th></tr></thead>
				<tbody>
					{#each alphabet.slice(0, 16) as a, i (a.c)}
						{@const b = alphabet[i + 16]}
						<tr>
							<td class="data">{a.c}</td><td class="num">{a.v}</td><td class="data">{a.bits}</td><td class="data">{a.alias}</td>
							<td class="data">{b.c}</td><td class="num">{b.v}</td><td class="data">{b.bits}</td><td class="data">{b.alias}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="precision"><span class="secno">2</span>Length and resolution</h2>
		<p>A stamp's length is the resolution of the record, not the accuracy of the clock.</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th class="num">Length</th><th>Unit (exact)</th><th>Unit (approx.)</th><th class="num">Bits</th><th>Binary form</th></tr></thead>
				<tbody>
					{#each precision as p (p.n)}
						<tr class:hl={p.n === CANONICAL_LEN}>
							<td class="num">{p.n}</td><td class="data">{@html exponents(p.exact)}</td><td>{p.human}</td><td class="num">{p.bits}</td><td class="data">{p.binary}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>An encoder must not write more characters than its source resolves: at most the length whose unit is nearest the source's resolution.</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th>Source resolution</th><th class="num">Maximum length</th><th>Unit at that length</th></tr></thead>
				<tbody>
					{#each resolutions as [name, r] (name)}
						{@const n = maxLengthFor(r)}
						<tr><td>{name}</td><td class="num">{n}</td><td>{humanSpan(unitSeconds(n))}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="eras"><span class="secno">3</span>Eras (first character)</h2>
		<p>
			Each era is 2<sup>35</sup> seconds, about 1,089 years. Years are UTC, in the proleptic Gregorian
			calendar, so era G begins in 1999: the epoch, midnight TAI, is 23:59:28 UTC on 31 December.
		</p>
		<EraStrip {eras} current="G" />
		<div class="table-wrap">
			<table>
				<thead><tr><th>Char</th><th>From</th><th>Until</th><th>Char</th><th>From</th><th>Until</th></tr></thead>
				<tbody>
					{#each eras.slice(0, 16) as e, i (e.c)}
						{@const f = eras[i + 16]}
						<tr class:hl={e.c === 'G' || f.c === 'G'}>
							<td class="data">{e.c}</td><td>{e.from}</td><td>{e.to}</td>
							<td class="data">{f.c}</td><td>{f.from}</td><td>{f.to}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="generations"><span class="secno">4</span>Two-character prefixes, 1900–2100</h2>
		<p>Each is 2<sup>30</sup> seconds, about 34 years. Boundaries are given to the day.</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th>Prefix</th><th>From (UTC)</th><th>Until (UTC)</th></tr></thead>
				<tbody>
					{#each generations as g (g.p)}
						<tr><td class="data">{g.p}</td><td class="data">{day(g.from)}</td><td class="data">{day(g.to)}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="years"><span class="secno">5</span>Three-character prefixes, 2020–2035</h2>
		<p>Each is 2<sup>25</sup> seconds, about 388 days. Useful as partition keys and for range queries.</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th>Prefix</th><th>From (UTC)</th><th>Until (UTC)</th></tr></thead>
				<tbody>
					{#each years as y (y.p)}
						<tr><td class="data">{y.p}</td><td class="data">{minute(y.from)}</td><td class="data">{minute(y.to)}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="durations"><span class="secno">6</span>Common durations</h2>
		<p>A duration is a signed whole number of 2<sup>−40</sup> s, written exactly. A decimal value that is not a whole number of these units is truncated toward zero.</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th>Duration</th><th class="num">Seconds</th><th>T32 duration</th></tr></thead>
				<tbody>
					{#each durations as [name, ns] (name)}
						<tr>
							<td>{name}</td>
							<td class="num data">{(Number(ns) / 1e9).toLocaleString('en-US', { maximumFractionDigits: 6 })}</td>
							<td class="data">{formatDuration(durationFromNs(ns))}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="moments"><span class="secno">7</span>Notable moments</h2>
		<p>Each moment is encoded at length 8. Before 1972, TAI − UTC was fractional, so a one-second stamp starts part-way through a UTC second.</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th>Moment</th><th>Stamp starts (UTC)</th><th>T32</th></tr></thead>
				<tbody>
					{#each momentRows as m (m.name)}
						<tr><td>{m.name}</td><td class="data small">{m.utc}</td><td class="data">{m.stamp}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="leap-seconds"><span class="secno">8</span>Leap seconds</h2>
		<p>
			All 27 inserted leap seconds, each with its own T32 stamp: in T32 they are ordinary seconds. IERS Bulletin C 72
			announced no leap second for the end of 2026, and the CGPM has made UTC continuous from 2027-05-20, so this list is
			expected to be complete.
		</p>
		<LeapChart />
		<div class="table-wrap">
			<table>
				<thead><tr><th class="num">#</th><th>Inserted second (UTC)</th><th>T32</th><th class="num">TAI − UTC after</th></tr></thead>
				<tbody>
					{#each leaps as l, i (l.label)}
						<tr><td class="num">{i + 1}</td><td class="data">{l.label}</td><td class="data">{l.stamp}</td><td class="num">{l.after} s</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="rubber"><span class="secno">9</span>TAI − UTC, 1961–1971, and its steps</h2>
		<p>Before 1972, UTC ran at an adjusted rate. TAI − UTC = a + (MJD − m) × r seconds, where MJD is the Modified Julian Date of the UTC instant. Before 1961 it is defined as 0.</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th>From</th><th class="num">a (s)</th><th class="num">m (MJD)</th><th class="num">r (s/day)</th></tr></thead>
				<tbody>
					{#each rubber as row (row.from)}
						<tr><td class="data">{row.from}</td><td class="num data">{row.a}</td><td class="num">{row.m}</td><td class="num data">{row.r}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p>Where the formula changes, TAI − UTC can jump. A rise inserts time that UTC has no ordinary label for; a fall removes labels, which converters must reject. These are every step other than the 27 leap seconds:</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th>Boundary (UTC)</th><th class="num">Step (s)</th><th>Effect</th></tr></thead>
				<tbody>
					{#each fractional as f (f.date)}
						<tr><td class="data">{f.date}</td><td class="num data">{f.step}</td><td>{f.kind}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="vectors"><span class="secno">10</span>Test vectors</h2>
		<p>Encoding vectors, recomputed here by the TypeScript implementation. The <a href="../spec/#test-vectors">specification</a> also has decoder, parser, duration and error vectors.</p>
		<div class="table-wrap">
			<table>
				<thead><tr><th>Case</th><th>UTC</th><th class="num">Length</th><th>Stamp</th></tr></thead>
				<tbody>
					{#each vectorRows as v (v.name)}
						<tr><td>{v.name}</td><td class="data small">{v.utc}</td><td class="num">{v.n}</td><td class="data">{v.stamp}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
</div>

<style>
	.toc .label {
		margin: 0 0 var(--s3);
	}
	.toc ol {
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
	@media (max-width: 960px) {
		.toc ol {
			columns: 2 14rem;
			column-gap: var(--s6);
		}
		.toc li {
			break-inside: avoid;
		}
	}
</style>
