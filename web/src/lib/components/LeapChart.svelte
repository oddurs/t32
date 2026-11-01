<script lang="ts">
	import { PERIODS, STEPS, LEAP_TABLE_EXPIRES_UNIX, deltaAt, daysFromCivil, civilFromDays } from '$lib/t32';

	const NS = 1_000_000_000n;
	const yearStart = (y: number) => Number(daysFromCivil(BigInt(y), 1, 1)) * 86_400;
	const X0 = yearStart(1958);
	const X1 = Number(LEAP_TABLE_EXPIRES_UNIX);
	const Y1 = 40;
	const WHOLE_FROM = yearStart(1972);

	const at = (ns: bigint) => {
		const d = deltaAt(ns);
		return Number(d.num) / Number(d.den);
	};

	// Every period runs from its start to just before the next, so the line is drawn from the
	// table itself: ramps through the 1960s, then flat runs joined by one-second jumps.
	const bounds = [X0, ...PERIODS.map((p) => Number(p.start / NS)).filter((u) => u > X0 && u < X1), X1];
	const points = bounds.slice(0, -1).flatMap((a, i) => {
		const b = bounds[i + 1];
		return [[a, at(BigInt(a) * NS)], [b, at(BigInt(b) * NS - 1n)]];
	});
	const stops = STEPS.map((s) => Number(s.unixNs / NS)).filter((u) => u > X0 && u <= X1);

	// Drawn at the measured width once mounted; until then the viewBox scales this default to fit.
	let w = $state(640);
	const h = 232;
	const m = { top: 12, right: 44, bottom: 28, left: 36 };
	const x = (u: number) => m.left + ((u - X0) / (X1 - X0)) * (w - m.left - m.right);
	const y = (v: number) => m.top + (1 - v / Y1) * (h - m.top - m.bottom);

	const line = $derived(points.map(([u, v], i) => `${i ? 'L' : 'M'}${x(u).toFixed(1)},${y(v).toFixed(1)}`).join(''));
	const area = $derived(`${line}L${x(X1).toFixed(1)},${y(0)}L${x(X0).toFixed(1)},${y(0)}Z`);
	// A label per decade crowds a phone screen; every second decade keeps them apart.
	const xTicks = $derived(
		[1960, 1970, 1980, 1990, 2000, 2010, 2020]
			.filter((yr) => w >= 480 || yr % 20 === 0)
			.map((yr) => ({ yr, u: yearStart(yr) }))
	);
	const yTicks = [0, 10, 20, 30, 40];

	let hover: number | null = $state(null);
	// The slider's value is a decimal year, a number a screen reader can say; the text carries the date.
	const year = (u: number) => Math.round((1970 + u / (365.2425 * 86_400)) * 100) / 100;
	const date = (u: number) => {
		const [yy, mm, dd] = civilFromDays(BigInt(Math.floor(u / 86_400)));
		return `${yy}-${String(mm).padStart(2, '0')}-${String(dd).padStart(2, '0')}`;
	};
	const reading = $derived.by(() => {
		if (hover === null) return null;
		const v = at(BigInt(hover) * NS);
		const text = v === 0 ? '0 s, by definition' : hover >= WHOLE_FROM ? `${v} s` : `${v.toFixed(3)} s`;
		return { u: hover, v, text, date: date(hover) };
	});

	function onMove(e: PointerEvent) {
		const r = (e.currentTarget as SVGElement).getBoundingClientRect();
		const u = X0 + ((e.clientX - r.left - m.left) / (w - m.left - m.right)) * (X1 - X0);
		hover = Math.round(Math.min(X1 - 1, Math.max(X0, u)));
	}
	// The keyboard visits each date where TAI - UTC changes, which is where the chart says anything.
	// Up and right step forward as on any slider; Page keys jump five changes.
	function onKey(e: KeyboardEvent) {
		const cur = hover ?? X0;
		const after = stops.findIndex((u) => u > cur);
		const at = after < 0 ? stops.length : after; // index of the first stop after cur
		const notBefore = stops.findIndex((u) => u >= cur); // stops are sorted
		const before = (notBefore < 0 ? stops.length : notBefore) - 1; // last stop before cur
		const pick = (i: number) => stops[Math.max(0, Math.min(stops.length - 1, i))];
		const next =
			e.key === 'ArrowRight' || e.key === 'ArrowUp' ? (after < 0 ? undefined : stops[after]) :
			e.key === 'ArrowLeft' || e.key === 'ArrowDown' ? (before < 0 ? undefined : stops[before]) :
			e.key === 'PageUp' ? pick(at + 4) :
			e.key === 'PageDown' ? pick(before - 4) :
			e.key === 'Home' ? stops[0] :
			e.key === 'End' ? stops[stops.length - 1] : undefined;
		if (e.key === 'Escape') hover = null;
		if (next === undefined) return;
		e.preventDefault();
		hover = next;
	}
</script>

<figure class="chart">
	<div class="plot" bind:clientWidth={w}>
		<svg
			width="100%"
			viewBox="0 0 {w} {h}"
			role="slider"
			tabindex="0"
			aria-label="TAI − UTC from 1958 to 2027: zero until 1961, rising gradually to 9.9 s by the end of 1971, then exactly 10 s, and one second at a time to 37 s from 2017. Arrow keys step through the changes."
			aria-valuemin={year(X0)}
			aria-valuemax={year(X1)}
			aria-valuenow={year(reading?.u ?? X0)}
			aria-valuetext={reading ? `${reading.date}: ${reading.text}` : 'No date chosen'}
			onpointermove={onMove}
			onpointerleave={() => (hover = null)}
			onkeydown={onKey}
			onblur={() => (hover = null)}
		>
			{#each yTicks as v (v)}
				<line class="grid" x1={m.left} x2={w - m.right} y1={y(v)} y2={y(v)} />
				<text class="tick" x={m.left - 8} y={y(v)} text-anchor="end" dominant-baseline="middle">{v}</text>
			{/each}
			{#each xTicks as t (t.yr)}
				<text class="tick" x={x(t.u)} y={h - m.bottom + 18} text-anchor="middle">{t.yr}</text>
			{/each}
			<path class="area" d={area} />
			<path class="line" d={line} />
			<text class="note" x={x(WHOLE_FROM) + 6} y={y(10) + 16}>10 s</text>
			<circle class="end" cx={x(X1)} cy={y(37)} r="4" />
			<text class="note" x={x(X1) + 9} y={y(37)} dominant-baseline="middle">37 s</text>
			{#if reading}
				<line class="cross" x1={x(reading.u)} x2={x(reading.u)} y1={m.top} y2={h - m.bottom} />
				<circle class="end" cx={x(reading.u)} cy={y(reading.v)} r="4" />
			{/if}
		</svg>
		{#if reading}
			<div class="tip" class:flip={x(reading.u) > w * 0.6} style="left: {x(reading.u)}px" aria-live="polite">
				<span class="data">{reading.date}</span>
				<strong class="data">{reading.text}</strong>
			</div>
		{/if}
	</div>
	<figcaption class="caption">
		TAI − UTC in seconds, from Appendix A. Until 1972 UTC ran at an adjusted rate, so the offset grew
		continuously; since then it has changed only by whole leap seconds, the last at the end of 2016.
	</figcaption>
</figure>

<style>
	.plot {
		position: relative;
	}
	svg {
		display: block;
		overflow: visible;
		touch-action: pan-y;
	}
	svg:focus-visible {
		outline-offset: 4px;
	}
	.grid {
		stroke: var(--rule);
		stroke-width: 1;
		shape-rendering: crispEdges;
	}
	.tick,
	.note {
		font-size: 0.6875rem;
		font-variant-numeric: tabular-nums;
		fill: var(--ink-3);
	}
	.note {
		fill: var(--ink-2);
		font-weight: var(--w-strong);
	}
	.area {
		fill: var(--accent);
		fill-opacity: 0.1;
	}
	.line {
		fill: none;
		stroke: var(--accent);
		stroke-width: 2;
		stroke-linejoin: round;
		stroke-linecap: round;
	}
	.end {
		fill: var(--accent);
		stroke: var(--paper);
		stroke-width: 2;
	}
	.cross {
		stroke: var(--rule-strong);
		stroke-width: 1;
	}
	.tip {
		position: absolute;
		top: 0;
		margin-left: var(--s3);
		display: flex;
		flex-direction: column;
		padding: var(--s1) var(--s2);
		background: var(--paper);
		border: 1px solid var(--rule-strong);
		font-size: var(--small);
		line-height: 1.35;
		white-space: nowrap;
		pointer-events: none;
	}
	.tip.flip {
		transform: translateX(-100%);
		margin-left: calc(-1 * var(--s3));
	}
</style>
