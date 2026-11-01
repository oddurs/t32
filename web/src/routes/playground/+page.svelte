<script lang="ts">
	import Head from '$lib/components/Head.svelte';
	import { DRAFT } from '$lib/site';
	import { exponents } from '$lib/exponents';
	import { onMount } from 'svelte';
	import { replaceState } from '$app/navigation';
	import {
		MAX_LEN, fromUnixMs, truncate, contains, format, formatDuration, durationBetween, durationToNs,
		unitSeconds, maxLengthFor, type Stamp
	} from '$lib/t32';
	import { readInput, errorText, describe, encode, humanSpan, secondsString, type Input } from '$lib/describe';

	const MS_LEN = maxLengthFor(1e-3); // Date.now() resolves 1 ms: at most 10 characters

	// Examples marked expert are shown in the advanced view only.
	const examples = [
		{ label: 'Now', value: 'now' },
		{ label: 'Apollo 11 landing', value: '1969-07-20T20:17:40' },
		{ label: 'Last leap second', value: '2016-12-31T23:59:60' },
		{ label: 'Unix 32-bit overflow', value: '2038-01-19T03:14:08' },
		{ label: 'T32 epoch', value: '2000-01-01T00:00:00' },
		{ label: 'Unix epoch', value: '1970-01-01T00:00:00' },
		{ label: 'A day', value: '2026-10-31' },
		{ label: 'Local time', value: '2026-10-30T23:13:00-04:00' },
		{ label: 'Unix time', value: '@1793416380.250', expert: true },
		{ label: 'A coarse stamp', value: 'G0S7' },
		{ label: 'Earliest', value: '00000000', expert: true }
	];

	// ---- converter state ----
	let text = $state('2026-10-31T03:13:00');
	let len = $state(8);
	let live = $state(false);
	let nowMs = $state(0);
	let mounted = $state(false);
	let copied: string | null = $state(null);

	// The input at the longest length it supports. The chosen length only shortens
	// it, so the digit-by-digit table can always show every row the input has.
	type Source =
		| { ok: true; full: Stamp; maxLen: number; kind: 'stamp' | 'date' | 'now'; precision: string; unix: boolean }
		| { ok: false; error: string };
	type Shown = Extract<Source, { ok: true }> & { stamp: Stamp };

	const source: Source = $derived.by(() => {
		if (live) {
			return { ok: true, full: fromUnixMs(nowMs || 0, MS_LEN), maxLen: MS_LEN, kind: 'now', precision: 'the millisecond', unix: false };
		}
		try {
			const input = readInput(text);
			if (input.kind === 'now') return { ok: false, error: '' };
			if (input.kind === 'stamp') {
				return { ok: true, full: input.stamp, maxLen: input.stamp.len, kind: 'stamp', precision: '', unix: false };
			}
			return { ok: true, full: encode(input.fx, input.maxLen), maxLen: input.maxLen, kind: 'date', precision: input.precision, unix: input.unix };
		} catch (e) {
			return { ok: false, error: errorText(e) };
		}
	});

	const current: Shown | null = $derived(
		source.ok ? { ...source, stamp: truncate(source.full, Math.max(1, Math.min(len, source.full.len))) } : null
	);

	// While the input is mid-edit and does not parse, keep showing the last good
	// result, dimmed, rather than collapsing the page under the cursor.
	let lastGood: Shown | null = $state(null);
	$effect(() => {
		if (current) lastGood = current;
	});
	const shown = $derived(current ?? lastGood);
	const stale = $derived(!current);
	const error = $derived(source.ok ? '' : source.error);

	const d = $derived(shown ? describe(shown.stamp) : null);

	// Digits alone are a valid stamp, but they are often meant as Unix time.
	const looksUnix = $derived(current?.kind === 'stamp' && /^\d{9,13}$/.test(text.trim()));
	// Text that reads as a date or the clock can also be a valid stamp, and text that
	// fails as a date can still be one: say which reading is shown, and offer the other.
	const alternative = $derived.by(() => {
		try {
			const input = readInput(text);
			if (input.kind === 'stamp') return input.notADate ? { notADate: true, stamp: null } : null;
			return input.alsoStamp ? { notADate: false, stamp: input.alsoStamp } : null;
		} catch {
			return null;
		}
	});

	const ladder = $derived.by(() => {
		if (!shown) return [];
		const full = shown.full;
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

	// Up and down step through time, as in a number field.
	function onKey(e: KeyboardEvent) {
		if (e.key !== 'ArrowUp' && e.key !== 'ArrowDown') return;
		e.preventDefault();
		step(e.key === 'ArrowUp' ? 1 : -1);
	}

	/** The stamp `delta` units from the current one, at the same length, if it exists. */
	function neighbour(delta: number): Stamp | null {
		if (!current) return null;
		const { value, len: n } = current.stamp;
		const v = value + BigInt(delta);
		return v >= 0n && v < 32n ** BigInt(n) ? { value: v, len: n } : null;
	}

	function step(delta: number) {
		const s = neighbour(delta);
		if (!s) return;
		// A date stays a date when the next stamp starts on a whole second, which
		// is every stamp of up to 8 characters since 1972. Otherwise, the stamp.
		if ((current?.kind === 'date' && !current.unix) || current?.kind === 'now') {
			const start = describe(s).start;
			if (s.len <= 8 && !start.includes('.')) {
				setInput(start);
				len = s.len;
				return;
			}
		}
		setInput(format(s));
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

	// ---- simple and advanced views ----
	type Mode = 'simple' | 'advanced';
	let mode: Mode = $state('simple');

	function setMode(m: Mode) {
		mode = m;
		try {
			localStorage.setItem('t32-playground-mode', m);
		} catch {
			/* storage unavailable: the choice lasts for this visit */
		}
	}

	const MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];

	/**
	 * "2026-10-31T03:13:00.5Z" → { date: "31 October 2026", time: "03:13:00.5" }, years
	 * before 1 CE as BCE. Fractions are cut to `maxFrac` digits for reading.
	 */
	function parts(iso: string, short = false, maxFrac = 9) {
		const m = /^([+-]?\d+)-(\d\d)-(\d\d)T(\d\d:\d\d:\d\d)(?:\.(\d+))?Z$/.exec(iso);
		if (!m) return { date: iso, time: '' };
		const y = Number(m[1]);
		const month = MONTHS[+m[2] - 1];
		const frac = (m[5] ?? '').slice(0, maxFrac);
		return {
			date: `${+m[3]} ${short ? month.slice(0, 3) : month} ${y > 0 ? y : `${1 - y} BCE`}`,
			time: m[4] + (frac ? '.' + frac : '')
		};
	}
	const friendly = (iso: string, short = false, maxFrac = 9) => {
		const p = parts(iso, short, maxFrac);
		return p.time ? `${p.date}, ${p.time}` : p.date;
	};

	/** The stamp's meaning in one plain sentence, naming the date once when it does not change. */
	const meaning = $derived.by(() => {
		if (!d) return '';
		const span = d.unitHuman.replace('≈ ', 'about ');
		const what = span.startsWith('1 ') ? `one ${span.slice(2)}` : `a span of ${span}`;
		// Enough fraction digits to tell the ends apart, and no more.
		const frac = Math.max(2, Math.min(9, Math.ceil((d.stamp.len - 8) * 1.5052) + 1));
		const a = parts(d.start, false, frac);
		const b = parts(d.end, false, frac);
		const range = a.date === b.date ? `on ${a.date}, from ${a.time} to ${b.time}` : `from ${a.date}, ${a.time} to ${b.date}, ${b.time}`;
		return `${what}, ${range} UTC`;
	});

	// Precision in everyday words. Each step is 32 times finer than the one before.
	const PRECISIONS = [
		{ len: 3, label: 'A year' },
		{ len: 4, label: '12 days' },
		{ len: 5, label: '9 hours' },
		{ len: 6, label: '17 minutes' },
		{ len: 7, label: '32 seconds' },
		{ len: 8, label: 'A second' },
		{ len: 10, label: 'A millisecond' },
		{ len: 12, label: 'A microsecond' },
		{ len: 14, label: 'A nanosecond' }
	];
	const precisions = $derived(PRECISIONS.filter((p) => !shown || p.len <= shown.maxLen));

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
		let saved: string | null = null;
		try {
			saved = localStorage.getItem('t32-playground-mode');
		} catch {
			/* storage unavailable */
		}
		const m = q.get('mode') ?? saved;
		if (m === 'simple' || m === 'advanced') mode = m;
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
		if (mode === 'advanced') q.set('mode', 'advanced');
		try {
			replaceState('?' + q.toString(), {});
		} catch {
			/* router not ready yet; the next change will update the URL */
		}
	});
</script>

<Head title="Playground" description="Type a date or a T32 stamp and see the span of time it stands for, character by character. Converts UTC, local time and Unix time, and compares stamps." path="/playground/" />

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Draft {DRAFT} · Playground</span>
		<h1>Playground</h1>
		<p class="lede">Type a date or a T32 stamp, and see the span of time it stands for.</p>
	</header>

	<aside class="rail">
		<p class="label">Examples</p>
		<ul class="examples">
			{#each examples.filter((ex) => mode === 'advanced' || !('expert' in ex)) as ex (ex.value)}
				<li><button class="link" onclick={() => setInput(ex.value)}>{ex.label}</button></li>
			{/each}
		</ul>
		<div class="about">
			<p class="label">{mode === 'simple' ? 'You can type' : 'Accepted input'}</p>
			{#if mode === 'simple'}
			<dl class="forms">
				<dt>A date</dt>
				<dd><span class="data">2026-10-31</span></dd>
				<dt>A date and time, in UTC</dt>
				<dd><span class="data">2026-10-31 15:30</span></dd>
				<dt>Your local time</dt>
				<dd>Add your offset: <span class="data">2026-10-31 11:30-04:00</span></dd>
				<dt>A T32 stamp</dt>
				<dd><span class="data">G0S7G6B1</span></dd>
				<dt>The time right now</dt>
				<dd><span class="data">now</span></dd>
			</dl>
			{:else}
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
			{/if}
			<p class="hint">
				In the box, ↑ and ↓ step to the next and previous stamp. Everything runs in your browser. The address keeps your input, so a link shows the same thing.
				<button class="link" onclick={() => copy('link', location.href)}>{copied === 'link' ? 'Link copied' : 'Copy link'}</button>
			</p>
		</div>
	</aside>

	<div class="main">
		<div class="modes" role="group" aria-label="View">
			<button class="mode" aria-pressed={mode === 'simple'} onclick={() => setMode('simple')}>Simple</button>
			<button class="mode" aria-pressed={mode === 'advanced'} onclick={() => setMode('advanced')}>Advanced</button>
		</div>

		{#if mode === 'advanced'}<h2><span class="secno">1</span>Convert</h2>{/if}

		<div class="controls" class:simple={mode === 'simple'}>
			<label class="field">
				<span class="label">{mode === 'simple' ? 'A date, a time or a stamp' : 'Stamp, date or Unix time'}</span>
				<input
					type="text"
					value={text}
					oninput={onType}
					onkeydown={onKey}
					spellcheck="false"
					autocomplete="off"
					autocapitalize="characters"
					class="data biginput"
					aria-describedby="conv-note"
					placeholder={mode === 'simple' ? 'e.g. 2026-10-31 15:30, or G0S7G6B1' : 'Stamp, date, @unix or now'}
				/>
			</label>

			{#if mode === 'advanced'}
			<label class="field len">
				<span class="label lenlabel">Length <span class="unitnow">{humanSpan(unitSeconds(len))}</span></span>
				<span class="lenrow">
					<input type="range" min="1" max={shown ? shown.maxLen : MAX_LEN} bind:value={len} aria-label="Length" />
					<input type="number" min="1" max={shown ? shown.maxLen : MAX_LEN} bind:value={len} class="data num" aria-label="Length (number)" />
				</span>
			</label>
			{/if}
		</div>

		<div id="conv-note">
			{#if !current}
				{#if error}<p class="error" role="alert">{error}</p>{/if}
			{:else if looksUnix}
				<p class="muted small">Read as a stamp, since digits are valid characters. For Unix time, write <span class="data">@{text.trim()}</span>.</p>
			{:else if mode === 'simple'}
				{#if current.maxLen < 8}
					<p class="muted small">
						{#if current.kind === 'stamp'}
							A short stamp names a long span. It can be made less precise, but not more: detail that was never recorded cannot be added.
						{:else}
							This is given to {current.precision}, so the stamp is no more precise than that. {current.precision === 'the day' ? 'Add a time, such as 15:30,' : 'Add seconds'} for more.
						{/if}
					</p>
				{/if}
			{:else if current.maxLen < MAX_LEN}
				<p class="muted small">
					{#if current.kind === 'stamp'}
						A stamp can be shortened but not lengthened: resolution that was not recorded cannot be added.
					{:else if current.kind === 'now'}
						Your clock resolves 1 ms, so at most {current.maxLen} characters. More would overstate it.
					{:else}
						This {current.unix ? 'Unix time' : 'date'} is written to {current.precision}, so at most {current.maxLen} characters. Write it more finely to go further.
					{/if}
				</p>
			{/if}
			{#if current && alternative?.stamp}
				<p class="muted small">
					Read as {current.kind === 'now' ? 'the clock' : 'a date'}. It is also valid stamp text:
					<button class="link" onclick={() => setInput(format(alternative.stamp!))}>read it as the stamp {format(alternative.stamp)}</button>.
				</p>
			{:else if current && alternative?.notADate}
				<p class="muted small">Not a valid date, so read as stamp text.</p>
			{/if}
		</div>

		{#if shown && d}
			<div class="output" class:stale aria-busy={stale}>
			<div class="result">
				<div class="stamprow">
					<div class="data stamp">{d.canonical}</div>
					<div class="actions">
						<button class="btn" onclick={() => step(-1)} disabled={!neighbour(-1)} title="The stamp before (↓ in the box)">← Earlier</button>
						<button class="btn" onclick={() => step(1)} disabled={!neighbour(1)} title="The stamp after (↑ in the box)">Later →</button>
						{#if live}<button class="btn" onclick={pause}>Pause</button>{/if}
						<button class="btn copy" onclick={() => copy('stamp', d.canonical)} disabled={stale}>{copied === 'stamp' ? 'Copied' : 'Copy'}</button>
					</div>
				</div>
				{#if mode === 'simple'}
					<p class="meaning"><span class="data">{d.canonical}</span> stands for {meaning}.{#if live}<span class="live"> Live.</span>{/if}</p>
					{#if local}<p class="small muted local">In your time zone, it starts {local}.</p>{/if}
				{:else}
					<div class="small muted">
						{d.unitHuman} wide{#if live}<span class="live"> · live</span>{/if}{d.provisional ? ' · provisional: beyond the published leap-second table' : ''}
					</div>
				{/if}
			</div>

			{#if mode === 'simple'}
				<div class="precision">
					<span class="label" id="precision-label">How precise</span>
					<div class="chips" role="group" aria-labelledby="precision-label">
						{#each precisions as p (p.len)}
							<button class="chip" aria-pressed={len === p.len} onclick={() => (len = p.len)} title="{p.len} characters">{p.label}</button>
						{/each}
					</div>
					<p class="small muted">Each character is 32 times finer than the one before it, so a shorter stamp names a longer span.</p>
				</div>

				<h3>How to read it</h3>
				<p class="small muted">Read a stamp from the left. Each character picks one of 32 equal parts of the span before it; the bar shows which.</p>
				<ol class="reading">
					{#each ladder.slice(0, shown.stamp.len) as row (row.stamp.len)}
						<li>
							<span class="data prefix"><span class="head">{row.head}</span>{row.last}</span>
							<span class="strip" role="img" aria-label="Part {row.part + 1} of 32">
								{#each { length: 32 } as _, k (k)}<i class:on={k === row.part}></i>{/each}
							</span>
							<span class="span-text"><span class="nowrap">{friendly(row.start, true)}</span> to <span class="nowrap">{friendly(row.end, true)}</span></span>
						</li>
					{/each}
				</ol>
			{:else}

			<div class="table-wrap">
				<table class="facts">
					<tbody>
						<tr><th>Starts (UTC)</th><td><span class="data">{d.start}</span>{#if d.inserted}<span class="small muted"> · inserted time: UTC has no ordinary label for this instant</span>{/if}</td></tr>
						<tr><th>Ends, exclusive (UTC)</th><td class="data">{d.end}</td></tr>
						<tr><th>Midpoint (UTC)</th><td class="data">{d.mid}</td></tr>
						<tr><th>Starts (your time zone)</th><td>{local || '—'}</td></tr>
						<tr><th>Unix time at start</th><td class="data">{d.unixStart}</td></tr>
						<tr><th>TAI − UTC at start</th><td class="data">{d.taiMinusUtc}</td></tr>
						<tr class="group"><th>Display form</th><td class="data">{d.display}</td></tr>
						<tr><th>Length · unit</th><td>{d.stamp.len} characters · {@html exponents(d.unitExact)} ({d.unitHuman})</td></tr>
						<tr><th>Value V</th><td class="data">{d.valueDec} = {d.valueHex}</td></tr>
					</tbody>
				</table>
			</div>

			<h3>Digit by digit</h3>
			<p class="small muted">Each character picks one of 32 equal parts of the span before it; the bar shows which. Choose a row to use that length.</p>
			<div class="table-wrap">
				<table class="ladder">
					<thead>
						<tr><th class="num">Len</th><th>Stamp and part</th><th class="unit">Unit</th><th>Interval (UTC)</th></tr>
					</thead>
					<tbody>
						{#each ladder as row (row.stamp.len)}
							<!-- The whole row is a target for the pointer; the button in it serves the keyboard. -->
							<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
							<tr class="pick" class:hl={row.stamp.len === d.stamp.len} class:finer={row.stamp.len > d.stamp.len} onclick={() => (len = row.stamp.len)}>
								<td class="num"><button class="link" aria-pressed={row.stamp.len === d.stamp.len} aria-label="Use length {row.stamp.len}">{row.stamp.len}</button></td>
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
			</div>
		{/if}

		{#if mode === 'advanced'}
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
	/* View switch: a two-part segmented control. */
	.modes {
		display: inline-flex;
		margin: 0 0 var(--s5);
		border: 1px solid var(--rule-strong);
		border-radius: 3px;
		padding: 2px;
		gap: 2px;
	}
	.mode {
		font-size: var(--small);
		font-weight: var(--w-medium);
		padding: 0.3rem 0.9rem;
		border: 0;
		border-radius: 2px;
		background: none;
		color: var(--ink-2);
	}
	.mode:hover {
		color: var(--ink);
	}
	.mode[aria-pressed='true'] {
		background: var(--ink);
		color: var(--paper);
	}
	.controls.simple {
		grid-template-columns: minmax(0, 1fr);
		max-width: 34rem;
	}

	/* Simple view */
	.meaning {
		font-size: 1.125rem;
		line-height: 1.5;
		margin: var(--s3) 0 var(--s1);
		max-width: var(--measure);
	}
	.local {
		margin: 0;
	}
	.precision {
		margin: 0 0 var(--s5);
	}
	.precision .label {
		display: block;
		margin-bottom: var(--s2);
	}
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: var(--s2);
		margin-bottom: var(--s2);
	}
	.chip {
		font-size: var(--small);
		padding: 0.3rem 0.75rem;
		border: 1px solid var(--rule-strong);
		border-radius: 999px;
		background: var(--paper);
		color: var(--ink-2);
	}
	.chip:hover {
		color: var(--ink);
		border-color: var(--ink-3);
	}
	.chip[aria-pressed='true'] {
		background: var(--accent);
		border-color: var(--accent);
		color: var(--paper);
	}
	.reading {
		list-style: none;
		padding: 0;
		margin: 0 0 var(--s5);
		border-top: 1px solid var(--rule-strong);
	}
	.reading li {
		display: grid;
		grid-template-columns: 9rem 10rem minmax(0, 1fr);
		gap: var(--s4);
		align-items: center;
		margin: 0;
		padding: var(--s2) 0;
		border-bottom: 1px solid var(--rule);
		font-size: var(--small);
	}
	.reading .strip {
		margin: 0;
	}
	.span-text {
		color: var(--ink-2);
		font-variant-numeric: tabular-nums;
	}

	.output {
		transition: opacity 120ms ease;
	}
	.output.stale {
		opacity: 0.38;
	}
	.lenlabel {
		display: flex;
		justify-content: space-between;
		gap: var(--s3);
	}
	.unitnow {
		text-transform: none;
		letter-spacing: 0;
		font-weight: var(--w-text);
		color: var(--ink-2);
		font-size: var(--small);
		line-height: 1;
	}
	.facts tr.group th,
	.facts tr.group td {
		border-top: 1px solid var(--rule-strong);
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
	tr.pick {
		cursor: pointer;
	}
	tr.pick:hover:not(.hl) td {
		background: var(--wash);
	}
	tr.pick button.link {
		text-decoration-color: transparent;
	}
	tr.pick:hover button.link,
	tr.hl button.link {
		text-decoration-color: currentColor;
	}
	/* Rows finer than the chosen length: available, but not in use. */
	tr.finer td {
		color: var(--ink-3);
	}
	tr.finer .prefix .head {
		color: var(--rule-strong);
	}
	tr.finer .strip i.on {
		background: var(--ink-3);
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
		.reading li {
			grid-template-columns: minmax(0, 1fr) 8rem;
			gap: var(--s1) var(--s3);
		}
		.reading .span-text {
			grid-column: 1 / -1;
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
