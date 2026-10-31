<script lang="ts">
	import { exponents } from '$lib/exponents';
	import { onMount } from 'svelte';
	import { replaceState } from '$app/navigation';
	import {
		MAX_LEN, fromUnixMs, truncate, contains, format, formatDuration, durationBetween, durationToNs,
		unitSeconds, maxLengthFor, type Stamp
	} from '$lib/t32';
	import { readInput, errorText, describe, encode, humanSpan, secondsString, type Input } from '$lib/describe';

	const MS_LEN = maxLengthFor(1e-3); // Date.now() resolves 1 ms: at most 10 characters

	const examples = [
		{ label: 'Now', value: 'now' },
		{ label: 'Apollo 11 landing', value: '1969-07-20T20:17:40' },
		{ label: 'Last leap second', value: '2016-12-31T23:59:60' },
		{ label: 'Unix 32-bit overflow', value: '2038-01-19T03:14:08' },
		{ label: 'T32 epoch', value: '2000-01-01T00:00:00' },
		{ label: 'Unix epoch', value: '1970-01-01T00:00:00' },
		{ label: 'A day', value: '2026-10-31' },
		{ label: 'Local time', value: '2026-10-30T23:13:00-04:00' },
		{ label: 'Unix time', value: '@1793416380.250' },
		{ label: 'A coarse stamp', value: 'G0S7' },
		{ label: 'Earliest', value: '00000000' }
	];

	// ---- converter state ----
	let text = $state('2026-10-31T03:13:00');
	let len = $state(8);
	let live = $state(false);
	let nowMs = $state(0);
	let mounted = $state(false);
	let copied: string | null = $state(null);

	type Parsed =
		| { ok: true; stamp: Stamp; maxLen: number; kind: 'stamp' | 'date' | 'now'; precision: string; unix: boolean }
		| { ok: false; error: string };

	const parsed: Parsed = $derived.by(() => {
		if (live) {
			const s = fromUnixMs(nowMs || 0, MS_LEN);
			return { ok: true, stamp: truncate(s, Math.min(len, MS_LEN)), maxLen: MS_LEN, kind: 'now', precision: 'the millisecond', unix: false };
		}
		try {
			const input = readInput(text);
			if (input.kind === 'now') return { ok: false, error: '' };
			if (input.kind === 'stamp') {
				const n = Math.min(len, input.stamp.len);
				return { ok: true, stamp: truncate(input.stamp, n), maxLen: input.stamp.len, kind: 'stamp', precision: '', unix: false };
			}
			return {
				ok: true,
				stamp: encode(input.fx, Math.min(len, input.maxLen)),
				maxLen: input.maxLen,
				kind: 'date',
				precision: input.precision,
				unix: input.unix
			};
		} catch (e) {
			return { ok: false, error: errorText(e) };
		}
	});

	const d = $derived(parsed.ok ? describe(parsed.stamp) : null);

	// Digits alone are a valid stamp, but they are often meant as Unix time.
	const looksUnix = $derived(parsed.ok && parsed.kind === 'stamp' && /^\d{9,13}$/.test(text.trim()));

	const ladder = $derived.by(() => {
		if (!parsed.ok) return [];
		const full = parsed.stamp;
		return Array.from({ length: full.len }, (_, i) => {
			const row = describe(truncate(full, i + 1));
			return { ...row, head: row.canonical.slice(0, -1), last: row.canonical.slice(-1), part: Number(row.stamp.value % 32n) };
		});
	});

	const local = $derived.by(() => {
		if (!mounted || !d || d.startUnixMs === null) return '';
		try {
			return new Intl.DateTimeFormat(undefined, { dateStyle: 'full', timeStyle: 'long' }).format(new Date(d.startUnixMs));
		} catch {
			return '';
		}
	});

	/** The longest length the input supports, or null if it does not parse. */
	function lengthOf(input: Input): number {
		return input.kind === 'stamp' ? input.stamp.len : input.kind === 'date' ? input.maxLen : 8;
	}

	function setInput(value: string) {
		copied = null;
		let input: Input | null = null;
		try {
			input = readInput(value);
		} catch {
			/* the error shows below; keep the length */
		}
		if (input?.kind === 'now') {
			live = true;
			nowMs = Date.now();
			text = value;
			len = 8;
			return;
		}
		live = false;
		text = value;
		if (input) len = lengthOf(input);
	}

	const onType = (e: Event) => setInput((e.currentTarget as HTMLInputElement).value);

	/** The stamp `delta` units from the current one, at the same length, if it exists. */
	function neighbour(delta: number): Stamp | null {
		if (!parsed.ok) return null;
		const { value, len: n } = parsed.stamp;
		const v = value + BigInt(delta);
		return v >= 0n && v < 32n ** BigInt(n) ? { value: v, len: n } : null;
	}

	function step(delta: number) {
		const s = neighbour(delta);
		if (s) setInput(format(s));
	}

	/** Stops the live clock, keeping the instant it showed, to the millisecond. */
	function pause() {
		const keep = len;
		setInput(new Date(nowMs).toISOString());
		len = keep;
	}

	async function copy(what: string, value: string) {
		try {
			await navigator.clipboard.writeText(value);
			copied = what;
			setTimeout(() => {
				if (copied === what) copied = null;
			}, 1600);
		} catch {
			copied = null;
		}
	}

	// ---- compare state ----
	let aText = $state('G0S7');
	let bText = $state('2026-10-31T03:13:00');

	function toStamp(t: string): Stamp {
		const input = readInput(t);
		if (input.kind === 'now') return fromUnixMs(Date.now(), MS_LEN);
		return input.kind === 'stamp' ? input.stamp : encode(input.fx, input.maxLen);
	}

	function swap() {
		[aText, bText] = [bText, aText];
	}

	const cmp = $derived.by(() => {
		try {
			const a = toStamp(aText);
			const b = toStamp(bText);
			const as = format(a);
			const bs = format(b);
			const diff = durationBetween(a, b);
			let relation: string;
			if (as === bs) relation = 'A and B are the same stamp.';
			else if (contains(a, b)) relation = 'A contains B (A is a prefix of B).';
			else if (contains(b, a)) relation = 'B contains A (B is a prefix of A).';
			else relation = as < bs ? 'A is entirely before B.' : 'A is entirely after B.';
			return {
				ok: true as const,
				a: describe(a),
				b: describe(b),
				relation,
				order: as < bs ? 'A sorts before B' : as > bs ? 'A sorts after B' : 'Equal',
				diff: secondsString(durationToNs(diff)) + ' s',
				diffT32: formatDuration(diff),
				uA: humanSpan(unitSeconds(a.len)),
				uB: humanSpan(unitSeconds(b.len))
			};
		} catch (e) {
			return { ok: false as const, error: errorText(e) };
		}
	});

	// ---- lifecycle: live clock and shareable URL ----
	onMount(() => {
		mounted = true;
		const q = new URLSearchParams(location.search);
		if (q.get('q')) setInput(q.get('q')!);
		const n = Number(q.get('n'));
		if (n >= 1 && n <= MAX_LEN) len = n;
		if (q.get('a')) aText = q.get('a')!;
		if (q.get('b')) bText = q.get('b')!;
		const id = setInterval(() => {
			if (live) nowMs = Date.now();
		}, 100);
		return () => clearInterval(id);
	});

	$effect(() => {
		if (!mounted) return;
		const q = new URLSearchParams();
		q.set('q', live ? 'now' : text);
		q.set('n', String(len));
		q.set('a', aText);
		q.set('b', bText);
		try {
			replaceState('?' + q.toString(), {});
		} catch {
			/* router not ready yet; the next change will update the URL */
		}
	});
</script>

<svelte:head>
	<title>T32 playground</title>
	<meta name="description" content="Convert between T32 stamps, UTC dates and Unix time, see the interval a stamp names digit by digit, and compare two stamps." />
</svelte:head>

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft 0.2 · Playground</span>
		<h1>Playground</h1>
		<p class="lede">Type a stamp, a date or a Unix time, and see the interval it names.</p>
	</header>

	<aside class="rail">
		<p class="label">Examples</p>
		<ul class="examples">
			{#each examples as ex (ex.value)}
				<li><button class="link" onclick={() => setInput(ex.value)}>{ex.label}</button></li>
			{/each}
		</ul>
		<div class="about">
			<p class="label">Accepted input</p>
			<dl class="forms">
				<dt>A stamp</dt>
				<dd>Either case, hyphens between characters: <span class="data">g0s7-g6b1</span></dd>
				<dt>A UTC date</dt>
				<dd>To the day, minute, second or finer: <span class="data">2016-12-31T23:59:60Z</span></dd>
				<dt>Local time</dt>
				<dd>With an offset: <span class="data">2026-10-31T05:13+02:00</span></dd>
				<dt>Unix time</dt>
				<dd>After an @: <span class="data">@1793416380</span></dd>
				<dt>The clock</dt>
				<dd><span class="data">now</span></dd>
			</dl>
			<p class="hint">
				Everything runs in your browser. The address keeps your input, so a link shows the same thing.
				<button class="link" onclick={() => copy('link', location.href)}>{copied === 'link' ? 'Link copied' : 'Copy link'}</button>
			</p>
		</div>
	</aside>

	<div class="main">
		<h2><span class="secno">1</span>Convert</h2>

		<div class="controls">
			<label class="field">
				<span class="label">Stamp, date or Unix time</span>
				<input
					type="text"
					value={text}
					oninput={onType}
					spellcheck="false"
					autocomplete="off"
					autocapitalize="characters"
					class="data biginput"
					aria-describedby="conv-note"
				/>
			</label>

			<label class="field len">
				<span class="label">Length <span class="muted">({parsed.ok ? humanSpan(unitSeconds(len)) : '–'} per step)</span></span>
				<span class="lenrow">
					<input type="range" min="1" max={parsed.ok ? parsed.maxLen : MAX_LEN} bind:value={len} aria-label="Length" />
					<input type="number" min="1" max={parsed.ok ? parsed.maxLen : MAX_LEN} bind:value={len} class="data num" aria-label="Length (number)" />
				</span>
			</label>
		</div>

		<div id="conv-note">
			{#if !parsed.ok}
				{#if parsed.error}<p class="error" role="alert">{parsed.error}</p>{/if}
			{:else if looksUnix}
				<p class="muted small">Read as a stamp, since digits are valid characters. For Unix time, write <span class="data">@{text.trim()}</span>.</p>
			{:else if parsed.maxLen < MAX_LEN}
				<p class="muted small">
					{#if parsed.kind === 'stamp'}
						A stamp can be shortened but not lengthened: resolution that was not recorded cannot be added.
					{:else if parsed.kind === 'now'}
						Your clock resolves 1 ms, so at most {parsed.maxLen} characters. More would overstate it.
					{:else}
						This {parsed.unix ? 'Unix time' : 'date'} is written to {parsed.precision}, so at most {parsed.maxLen} characters. Write it more finely to go further.
					{/if}
				</p>
			{/if}
		</div>

		{#if parsed.ok && d}
			<div class="result">
				<div class="stamprow">
					<div class="data stamp">{d.canonical}</div>
					<div class="actions">
						<button class="btn" onclick={() => step(-1)} disabled={!neighbour(-1)} title="The {d.unitHuman.replace('≈ ', '')} before">← Earlier</button>
						<button class="btn" onclick={() => step(1)} disabled={!neighbour(1)} title="The {d.unitHuman.replace('≈ ', '')} after">Later →</button>
						{#if live}<button class="btn" onclick={pause}>Pause</button>{/if}
						<button class="btn" onclick={() => copy('stamp', d.canonical)}>{copied === 'stamp' ? 'Copied' : 'Copy'}</button>
					</div>
				</div>
				<div class="small muted">
					{d.unitHuman} wide{#if live}<span class="live"> · live</span>{/if}{d.provisional ? ' · provisional: beyond the published leap-second table' : ''}
				</div>
			</div>

			<div class="table-wrap">
				<table class="facts">
					<tbody>
						<tr><th>Canonical</th><td class="data">{d.canonical}</td></tr>
						<tr><th>Display form</th><td class="data">{d.display}</td></tr>
						<tr><th>Length · unit</th><td>{d.stamp.len} characters · {@html exponents(d.unitExact)} ({d.unitHuman})</td></tr>
						<tr><th>Starts (UTC)</th><td><span class="data">{d.start}</span>{#if d.inserted}<span class="small muted"> · inserted time: UTC has no ordinary label for this instant</span>{/if}</td></tr>
						<tr><th>Ends, exclusive (UTC)</th><td class="data">{d.end}</td></tr>
						<tr><th>Midpoint (UTC)</th><td class="data">{d.mid}</td></tr>
						<tr><th>Starts (your time zone)</th><td>{local || '—'}</td></tr>
						<tr><th>Unix time at start</th><td class="data">{d.unixStart}</td></tr>
						<tr><th>TAI − UTC at start</th><td class="data">{d.taiMinusUtc}</td></tr>
						<tr><th>Value V</th><td class="data">{d.valueDec} = {d.valueHex}</td></tr>
					</tbody>
				</table>
			</div>

			<h3>Digit by digit</h3>
			<p class="small muted">Each character picks one of 32 equal parts of the span before it; the bar shows which. Choose a length to use it.</p>
			<div class="table-wrap">
				<table class="ladder">
					<thead>
						<tr><th class="num">Len</th><th>Stamp and part</th><th class="unit">Unit</th><th>Interval (UTC)</th></tr>
					</thead>
					<tbody>
						{#each ladder as row (row.stamp.len)}
							<tr class:hl={row.stamp.len === d.stamp.len}>
								<td class="num"><button class="link" onclick={() => (len = row.stamp.len)} aria-label="Use length {row.stamp.len}">{row.stamp.len}</button></td>
								<td>
									<span class="data prefix"><span class="head">{row.head}</span>{row.last}</span>
									<span class="strip" role="img" aria-label="Part {row.part + 1} of 32">
										{#each { length: 32 } as _, k (k)}<i class:on={k === row.part}></i>{/each}
									</span>
								</td>
								<td class="nowrap unit">{row.unitHuman}</td>
								<td class="data small"><span class="nowrap">{row.start} –</span> <span class="nowrap">{row.end}</span></td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}

		<h2><span class="secno">2</span>Compare</h2>

		<p class="small muted">Two stamps, dates or Unix times. A date is encoded at the length its written precision allows (seconds → 8, milliseconds → 10).</p>

		<div class="controls two">
			<label class="field">
				<span class="label">A</span>
				<input type="text" bind:value={aText} spellcheck="false" autocomplete="off" class="data" />
			</label>
			<label class="field">
				<span class="label">B</span>
				<input type="text" bind:value={bText} spellcheck="false" autocomplete="off" class="data" />
			</label>
		</div>
		<p class="small"><button class="link" onclick={swap}>Swap A and B</button></p>

		{#if !cmp.ok}
			<p class="error" role="alert">{cmp.error}</p>
		{:else}
			<p><strong>{cmp.relation}</strong></p>
			<div class="table-wrap">
				<table>
					<thead>
						<tr><th></th><th>A</th><th>B</th></tr>
					</thead>
					<tbody>
						<tr><th>Stamp</th><td class="data">{cmp.a.canonical}</td><td class="data">{cmp.b.canonical}</td></tr>
						<tr><th>Unit</th><td>{cmp.a.unitHuman}</td><td>{cmp.b.unitHuman}</td></tr>
						<tr><th>Starts (UTC)</th><td class="data small">{cmp.a.start}</td><td class="data small">{cmp.b.start}</td></tr>
						<tr><th>Ends (UTC)</th><td class="data small">{cmp.a.end}</td><td class="data small">{cmp.b.end}</td></tr>
					</tbody>
				</table>
			</div>
			<div class="table-wrap">
				<table>
					<tbody>
						<tr><th>Text order</th><td>{cmp.order}</td></tr>
						<tr><th>B − A (start to start)</th><td class="data">{cmp.diff}</td></tr>
						<tr><th>As a T32 duration</th><td class="data">{cmp.diffT32}</td></tr>
						<tr><th>Between the instants</th><td>more than (B − A) − {cmp.uA}, less than (B − A) + {cmp.uB}</td></tr>
					</tbody>
				</table>
			</div>
		{/if}
	</div>
</div>

<style>
	.controls {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 15rem;
		gap: var(--s3) var(--s5);
		align-items: end;
		margin-bottom: var(--s3);
	}
	.controls.two {
		grid-template-columns: 1fr 1fr;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: var(--s2);
		min-width: 0;
	}
	.field .label .muted {
		text-transform: none;
		letter-spacing: 0;
	}
	.biginput {
		font-size: 1.125rem;
		padding: var(--s2) var(--s3);
	}
	.lenrow {
		display: flex;
		gap: var(--s2);
		align-items: center;
	}
	.lenrow input[type='range'] {
		flex: 1;
		min-width: 0;
	}
	.num {
		width: 4.5em;
	}
	.examples {
		list-style: none;
		padding: 0;
		margin: 0 0 var(--s5);
	}
	.examples li {
		margin: 0 0 var(--s1);
		padding: 0;
	}
	.hint {
		color: var(--ink-2);
		font-size: 0.875rem;
		line-height: 1.5;
	}
	.result {
		margin: var(--s4) 0 var(--s4);
		padding: var(--s4) 0 var(--s3);
		border-top: 1px solid var(--rule-strong);
		border-bottom: 1px solid var(--rule);
	}
	.stamprow {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: var(--s2) var(--s5);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--s2);
	}
	.live {
		color: var(--accent);
	}
	#conv-note {
		min-height: 1.6rem;
	}
	#conv-note p {
		margin: 0;
	}

	/* The rail's list of accepted forms. */
	.forms {
		margin: 0 0 var(--s4);
		font-size: 0.875rem;
		line-height: 1.45;
	}
	.forms dt {
		font-weight: var(--w-strong);
		color: var(--ink);
	}
	.forms dd {
		margin: 0 0 var(--s2);
		color: var(--ink-2);
	}
	.forms .data {
		white-space: nowrap;
	}

	/* Digit by digit: each prefix, with a bar of its 32 parts. */
	.ladder td {
		vertical-align: top;
	}
	.prefix {
		display: block;
	}
	.prefix .head {
		color: var(--ink-3);
	}
	.strip {
		display: flex;
		gap: 1px;
		width: 10rem;
		margin: 0.35rem 0 0.15rem;
	}
	.strip i {
		flex: 1;
		height: 0.4rem;
		background: var(--rule);
	}
	.strip i.on {
		background: var(--accent);
	}
	tr.hl .strip i:not(.on) {
		background: color-mix(in srgb, var(--rule-strong) 55%, var(--mark));
	}
	.stamp {
		font-size: clamp(2rem, 6vw, 2.875rem);
		font-weight: var(--w-head);
		font-variant-numeric: tabular-nums lining-nums;
		letter-spacing: 0.02em;
		line-height: 1.15;
		word-break: break-all;
	}
	th {
		white-space: nowrap;
	}
	.nowrap {
		white-space: nowrap;
	}
	@media (max-width: 960px) {
		.examples {
			display: flex;
			flex-wrap: wrap;
			gap: var(--s1) var(--s4);
		}
		.about {
			display: none;
		}
	}
	@media (max-width: 600px) {
		.ladder .unit {
			display: none;
		}
		.strip {
			width: 8rem;
		}
		.stamprow {
			align-items: flex-start;
			flex-direction: column;
		}
		.controls,
		.controls.two {
			grid-template-columns: 1fr;
		}
	}
</style>
