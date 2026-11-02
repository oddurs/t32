<script lang="ts">
	import { resolve } from '$app/paths';
	import Head from '$lib/components/Head.svelte';
	import Mark from '$lib/components/Mark.svelte';
	import { MARK } from '$lib/mark';
	import { DRAFT } from '$lib/site';

	let { data } = $props();

	const sections = [
		['principles', 'Principles'],
		['mark', 'The mark'],
		['colour', 'Colour'],
		['type', 'Type'],
		['space', 'Space'],
		['frame', 'Frame'],
		['parts', 'Parts']
	] as const;

	const use: Record<string, string> = {
		'--paper': 'The ground of every page',
		'--ink': 'Text and headings',
		'--ink-2': 'Ledes, captions, secondary text',
		'--ink-3': 'Labels, section numbers, axis ticks',
		'--rule': 'Hairlines between rows',
		'--rule-strong': 'Rules that open or close a table or figure; input borders',
		'--wash': 'Code, notes, fractional cells',
		'--accent': 'Links, the primary action, the marked cell',
		'--accent-rule': 'Link underlines',
		'--mark': 'Highlighted rows, selection',
		'--err': 'Errors'
	};

	const roles = [
		{ role: 'Display', where: 'h1', size: '--h1', step: '5 → 7', lh: '--lh-display', tr: '--tr-display', w: '--w-display', cls: 'r-display', sample: 'A compact universal timestamp' },
		{ role: 'Section', where: 'h2', size: '--h2', step: '3', lh: '--lh-head', tr: '--tr-h2', w: '--w-head', cls: 'r-h2', sample: 'Conversion from UTC' },
		{ role: 'Subsection', where: 'h3', size: '--h3', step: '1', lh: '--lh-head', tr: '--tr-h3', w: '--w-head', cls: 'r-h3', sample: 'Leap seconds' },
		{ role: 'Lede', where: '.lede', size: '--lede', step: '1', lh: '--lh-lede', tr: '--tr-lede', w: '--w-text', cls: 'lede', sample: 'T32 writes any moment as eight characters.' },
		{ role: 'Text', where: 'body', size: '--text', step: '0', lh: '--lh-text', tr: '', w: '--w-text', cls: '', sample: 'A stamp names an interval, never a false instant. Drop characters for a coarser span; add them for finer time.' },
		{ role: 'Small', where: '.small, table, .caption', size: '--small', step: '−1', lh: '--lh-small', tr: '', w: '--w-text', cls: 'small', sample: 'Each position divides the one before it by 32.' },
		{ role: 'Label', where: '.label, th, .meta dt', size: '--label', step: '−2', lh: '--lh-tight', tr: '--tr-label', w: '--w-strong', cls: 'label', sample: `Draft ${DRAFT} · Specification` }
	];

	// Where each step is used. Steps with no entry are there for figures and for what comes next.
	const stepRole: Record<string, string> = {
		'−2': 'Label',
		'−1': 'Small',
		'0': 'Text',
		'1': 'Lede, h3',
		'2': 'h2 on phones',
		'3': 'h2',
		'5': 'h1 on phones',
		'7': 'h1',
		'8': 'Clock'
	};
	const holds: Record<string, string> = {
		'--measure': 'Running text: about seventy characters',
		'--page': 'Header, page and footer',
		'--rail': 'The contents and metadata column',
		'--gutter': 'The side margin; 16px on phones',
		'--section': 'The space above a section’s rule; 48px on phones',
		'--radius': 'Corners of controls and code'
	};
	const weights = [
		['--w-text', 'Text'],
		['--w-medium', 'Medium'],
		['--w-strong', 'Strong'],
		['--w-head', 'Heading'],
		['--w-display', 'Display']
	] as const;

	const ratio = (r: number) => `${r.toFixed(1)}:1`;
	const tok = (list: { name: string; value: string }[], name: string) => list.find((t) => t.name === name)?.value ?? '';
</script>

<Head
	title="Design system"
	description="The tokens, type, mark and parts the T32 site is built from, read from its stylesheet when the site is built."
	path="/system/"
/>

<div class="with-rail">
	<header class="titleblock">
		<span class="label">T32 · Design system</span>
		<h1>How the site is set</h1>
		<p class="lede">
			The tokens, type and parts this site is built from. Values are read from the stylesheet when the
			site is built, and every specimen is drawn by the rules it describes, so the page cannot disagree
			with the site.
		</p>
	</header>

	<aside class="rail">
		<nav class="toc" aria-label="Design system">
			<p class="label">Contents</p>
			<ol>
				{#each sections as [id, title], i (id)}
					<li><span class="no">{i + 1}</span><a href="#{id}">{title}</a></li>
				{/each}
			</ol>
		</nav>
	</aside>

	<div class="main">
		<h2 id="principles"><span class="secno">1</span>Principles</h2>

		<dl class="pairs">
			<dt>The text decides</dt>
			<dd>The site serves a specification. Nothing on it competes with the words, and nothing is there only to decorate.</dd>
			<dt>Rules, not boxes</dt>
			<dd>Hairlines separate things. A tinted ground is kept for code, notes and the digits after the eighth.</dd>
			<dt>One accent</dt>
			<dd>Blue marks what can be followed or chosen: links, the primary action, the marked cell of a figure.</dd>
			<dt>Figures line up</dt>
			<dd>Numbers in a column use tabular figures. Stamps are data, not code, and are set in the text face.</dd>
			<dt>Nothing moves</dt>
			<dd>Whatever arrives late has its space held for it, by placeholders that cannot be mistaken for values.</dd>
		</dl>

		<h2 id="mark"><span class="secno">2</span>The mark</h2>

		<p>
			The name, drawn from Public Sans at weight 720 and spaced by hand, with a square full stop. The full
			stop is the marked cell of the zoom figure: one part in 32, picked out.
		</p>

		<figure class="mark-sheet">
			<div class="mark-hero"><Mark cap="clamp(3rem, 9vw, 5.5rem)" /></div>
			<div class="mark-sizes" aria-label="The mark at four sizes">
				{#each ['0.75rem', '1rem', '1.5rem', '2.25rem'] as cap (cap)}
					<div><Mark {cap} label={null} /><span class="label">Cap {parseFloat(cap) * 16}px</span></div>
				{/each}
				<div class="tiles">
					{#each [16, 32, 64] as size (size)}
						<img src="{resolve('/')}favicon.svg" width={size} height={size} alt="" />
					{/each}
					<span class="label">Tile, 16 to 64px</span>
				</div>
			</div>
		</figure>

		<figure>
			<svg class="construction" viewBox="-6 -6 {MARK.width + 12} 44" role="img" aria-labelledby="construction-caption">
				<line x1="-6" x2={MARK.width + 6} y1="0" y2="0" />
				<line x1="-6" x2={MARK.width + 6} y1="32" y2="32" />
				<line x1={MARK.cell.x} x2={MARK.cell.x} y1="-6" y2="38" />
				<line x1={MARK.cell.x + MARK.cell.size} x2={MARK.cell.x + MARK.cell.size} y1="-6" y2="38" />
				<path class="glyphs" d={MARK.name} />
				<rect class="cell" x={MARK.cell.x} y={MARK.cell.y} width={MARK.cell.size} height={MARK.cell.size} />
			</svg>
			<figcaption class="caption" id="construction-caption">
				Construction. Cap height and baseline are 32 units apart. The full stop is a square
				{MARK.cell.size.toFixed(2)} units wide, the thickness of the T's crossbar, and stands one 3-to-2 gap
				after the 2.
			</figcaption>
		</figure>

		<ul class="rules">
			<li>Draw it from the paths in <a href="https://github.com/oddurs/t32/blob/main/web/static/mark.svg"><code>mark.svg</code></a>; never retype it in a font. <a href="https://github.com/oddurs/t32/blob/main/tools/mark.py"><code>tools/mark.py</code></a> makes it.</li>
			<li>Keep half its cap height clear on every side. Below a 12px cap height, use the tile.</li>
			<li>The name takes the text colour, the full stop the accent. Nothing else changes.</li>
			<li>In running text the name is just T32, in the text face.</li>
		</ul>

		<h2 id="colour"><span class="secno">3</span>Colour</h2>

		<p>
			Warm paper, near-black ink, two greys and one blue. The dark scheme is its own set of values, chosen
			for the dark ground, not the light one inverted. Contrast is measured on paper; the build fails if
			any text colour falls under 4.5:1 on paper or wash, in either scheme.
		</p>

		<div class="table-wrap">
			<table class="colours">
				<thead>
					<tr><th>Token</th><th>Light</th><th>Dark</th><th>Use</th><th class="num">On paper</th></tr>
				</thead>
				<tbody>
					{#each data.colours as c (c.name)}
						<tr>
							<td><code>{c.name}</code></td>
							<td class="data"><span class="chip" style:background={c.light}></span>{c.light}</td>
							<td class="data"><span class="chip" style:background={c.dark}></span>{c.dark}</td>
							<td>{use[c.name] ?? ''}</td>
							<td class="num data">{c.ratio ? `${ratio(c.ratio[0])} · ${ratio(c.ratio[1])}` : ''}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="type"><span class="secno">4</span>Type</h2>

		<h3>Families</h3>

		<div class="families">
			<div>
				<p class="family-name">Public Sans</p>
				<p class="glyphs-sample">Aa Bb Gg Rr 0123456789</p>
				<p class="small muted">Everything but code. A descendant of Franklin Gothic, drawn for the United States government.</p>
			</div>
			<div>
				<p class="family-name mono">Roboto Mono</p>
				<p class="glyphs-sample mono">Aa Bb Gg Rr 0123456789</p>
				<p class="small muted">Code, and only code. Its x-height matches Public Sans one step down the scale, so inline code sits level with the text.</p>
			</div>
		</div>

		<div class="table-wrap">
			<table class="weights">
				<thead>
					<tr><th>Weight</th>{#each weights as [name, role] (name)}<th>{role}</th>{/each}</tr>
				</thead>
				<tbody>
					<tr>
						<th scope="row">Specimen</th>
						{#each weights as [name] (name)}<td class="weight-sample" style:font-weight="var({name})">Rg</td>{/each}
					</tr>
					<tr><th scope="row">Light</th>{#each weights as [name] (name)}<td class="data">{tok(data.weights.light, name)}</td>{/each}</tr>
					<tr><th scope="row">Dark</th>{#each weights as [name] (name)}<td class="data">{tok(data.weights.dark, name)}</td>{/each}</tr>
				</tbody>
			</table>
		</div>
		<p class="small muted measure-text">Light type on a dark ground reads heavier, so the dark scheme takes a little weight off. The variable font makes the in-between weights exact.</p>

		<h3>Scale</h3>

		<p>
			Each step is 2<sup>1/5</sup> times the one below it, so five steps double the size, as five bits
			make one character. Step 0 is 16px; step 5 is 32.
		</p>

		<div class="table-wrap">
			<table class="scale">
				<thead>
					<tr><th class="num">Step</th><th>Token</th><th class="num">px</th><th class="num">rem</th><th>Role</th><th>Specimen</th></tr>
				</thead>
				<tbody>
					{#each data.scale as s (s.name)}
						<tr>
							<td class="num data">{s.step}</td>
							<td><code>{s.name}</code></td>
							<td class="num data">{s.px.toFixed(1)}</td>
							<td class="num data">{parseFloat(s.value).toFixed(3)}</td>
							<td class="muted">{stepRole[s.step] ?? ''}</td>
							<td class="specimen" style:font-size="var({s.name})">Atomic seconds</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h3>Roles</h3>

		<p>
			Seven roles, each on a step of the scale. Leading tightens as the size grows and tracking closes
			up; labels, set in capitals, open out instead.
		</p>

		<div class="roles">
			{#each roles as r (r.role)}
				<div class="role">
					<div class="spec">
						<p class="role-name">{r.role}</p>
						<p class="small muted">
							<code>{r.where}</code><br />
							Step {r.step} · <code>{r.size}</code><br />
							Leading {tok(data.leading, r.lh)}{#if r.tr}<br />Tracking {tok(data.tracking, r.tr).replace('-', '−')}{/if}
						</p>
					</div>
					<p class="sample {r.cls}">{r.sample}</p>
				</div>
			{/each}
		</div>

		<h3>Figures</h3>

		<p>
			Numbers that run in text are proportional. Numbers in a column, and every stamp, are tabular and
			lining, with a little added tracking so that a stamp's characters can be counted.
		</p>

		<div class="figures">
			<div>
				<p class="label">Proportional</p>
				<p class="prop">1,089 y<br />34 y<br />388 d<br />11.1 h</p>
			</div>
			<div>
				<p class="label">Tabular, <code>.data</code></p>
				<p class="data">1,089 y<br />&nbsp;&nbsp;&nbsp;&nbsp;34 y<br />&nbsp;&nbsp;&nbsp;388 d<br />&nbsp;&nbsp;11.1 h</p>
			</div>
			<div>
				<p class="label">A stamp</p>
				<p><span class="data">G0S7G6B1</span> in text<br /><code>G0S7G6B1</code> in code</p>
			</div>
		</div>

		<h2 id="space"><span class="secno">5</span>Space</h2>

		<p>
			A 4px grid, in nine steps that roughly double. Inside a part, the small steps; between parts, the
			middle ones; the largest open a section.
		</p>

		<div class="table-wrap">
			<table class="space">
				<thead>
					<tr><th>Token</th><th class="num">px</th><th class="num">rem</th><th class="bar-col"><span class="visually-hidden">Size</span></th></tr>
				</thead>
				<tbody>
					{#each data.space as s (s.name)}
						<tr>
							<td><code>{s.name}</code></td>
							<td class="num data">{s.px}</td>
							<td class="num data">{parseFloat(s.value)}</td>
							<td><span class="bar" style:width="var({s.name})"></span></td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<h2 id="frame"><span class="secno">6</span>Frame</h2>

		<p>
			A page is a rail and a main column, inside a fixed width. Running text is held to a measure of
			about seventy characters, wherever the column is wider.
		</p>

		<div class="table-wrap">
			<table>
				<thead>
					<tr><th>Token</th><th>Value</th><th>Holds</th></tr>
				</thead>
				<tbody>
					{#each data.frame as f (f.name)}
						<tr><td><code>{f.name}</code></td><td class="data">{f.value}</td><td>{holds[f.name]}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<figure class="measure">
			<p>
				This paragraph is set to the measure. A line of about seventy characters is long enough to keep
				the eye moving and short enough for it to find the start of the next line without hunting.
			</p>
			<span class="measure-rule" aria-hidden="true"></span>
			<figcaption class="caption">The measure, <code>--measure</code>, about seventy characters of text.</figcaption>
		</figure>

		<h2 id="parts"><span class="secno">7</span>Parts</h2>

		<p>The pieces pages are made of. Each is a class or an element in the stylesheet, with no props to learn.</p>

		<div class="parts">
			<section class="part" aria-labelledby="p-actions">
				<h3 id="p-actions">Actions <code>.btn</code> <code>.btn.primary</code> <code>button.link</code></h3>
				<p class="row">
					<button class="btn primary" type="button">Convert</button>
					<button class="btn" type="button">Copy</button>
					<button class="btn" type="button" disabled>Unavailable</button>
					<button class="link" type="button">Use the current time</button>
				</p>
				<p class="small muted">One primary action to a view, at most. A text action is a button that looks like a link because it does not lead anywhere.</p>
			</section>

			<section class="part" aria-labelledby="p-links">
				<h3 id="p-links">Links <code>a</code></h3>
				<p>Read the <a href={resolve('spec/')}>specification</a>, or the <a href="https://github.com/oddurs/t32">source</a>. The underline is a tint of the link colour until it is pointed at.</p>
			</section>

			<section class="part" aria-labelledby="p-fields">
				<h3 id="p-fields">Fields <code>input</code> <code>select</code></h3>
				<p class="row">
					<input type="text" value="2026-10-31T03:13:00Z" aria-label="UTC time" size="22" />
					<select aria-label="Length"><option>8 characters</option><option>10 characters</option></select>
				</p>
			</section>

			<section class="part" aria-labelledby="p-meta">
				<h3 id="p-meta">Metadata <code>dl.meta</code></h3>
				<dl class="meta">
					<dt>Status</dt><dd>Draft {DRAFT}, for public review</dd>
					<dt>Time scale</dt><dd>TAI, International Atomic Time</dd>
				</dl>
			</section>

			<section class="part" aria-labelledby="p-table">
				<h3 id="p-table">Tables <code>table</code> <code>.num</code> <code>.data</code> <code>tr.hl</code></h3>
				<div class="table-wrap">
					<table>
						<thead><tr><th>Prefix</th><th class="num">Characters</th><th>Span</th></tr></thead>
						<tbody>
							<tr><td class="data">G0S</td><td class="num">3</td><td>388 days</td></tr>
							<tr class="hl"><td class="data">G0S7G6B1</td><td class="num">8</td><td>1 second</td></tr>
						</tbody>
					</table>
				</div>
			</section>

			<section class="part" aria-labelledby="p-note">
				<h3 id="p-note">Note <code>.note</code></h3>
				<div class="note"><p>A note says something the reader would otherwise get wrong. If it only adds emphasis, it is not a note.</p></div>
			</section>

			<section class="part" aria-labelledby="p-code">
				<h3 id="p-code">Code <code>code</code> <code>pre</code></h3>
				<p>Inline, as in <code>Stamp::parse("G0S7")</code>, and as a block:</p>
				<pre><code>let utc: UtcDateTime = "2026-10-31T03:13:00Z".parse()?;
let stamp = Stamp::new(Instant::from_utc(utc)?, 8)?;
assert_eq!(stamp.to_string(), "G0S7G6B1");</code></pre>
			</section>

			<section class="part" aria-labelledby="p-quote">
				<h3 id="p-quote">Quotation <code>blockquote</code></h3>
				<blockquote><p>A stamp, once written, keeps its meaning.</p></blockquote>
			</section>
		</div>
	</div>
</div>

<style>
	.pairs {
		display: grid;
		grid-template-columns: 11rem minmax(0, 1fr);
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

	/* The mark */
	.mark-sheet {
		margin: var(--s5) 0 var(--s6);
		border-top: 1px solid var(--rule-strong);
		border-bottom: 1px solid var(--rule-strong);
	}
	.mark-hero {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--s8) var(--s5);
		border-bottom: 1px solid var(--rule);
	}
	.mark-sizes {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		gap: var(--s5) var(--s7);
		padding: var(--s5) 0;
	}
	.mark-sizes > div {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--s3);
	}
	.tiles {
		flex-direction: row !important;
		flex-wrap: wrap;
		align-items: flex-end !important;
		margin-left: auto;
	}
	.tiles .label {
		flex-basis: 100%;
	}
	.construction {
		display: block;
		width: 100%;
		max-width: 32rem;
		height: auto;
		overflow: visible;
	}
	.construction line {
		stroke: var(--accent-rule);
		stroke-width: 1;
		vector-effect: non-scaling-stroke;
	}
	.construction .glyphs {
		fill: var(--ink);
	}
	.construction .cell {
		fill: var(--accent);
	}
	.rules {
		max-width: var(--measure);
	}

	/* Colour */
	.chip {
		display: inline-block;
		width: 0.875rem;
		height: 0.875rem;
		margin-right: var(--s2);
		vertical-align: -0.125rem;
		border-radius: var(--radius);
		box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--ink) 18%, transparent);
	}
	.colours td:first-child,
	.scale td:nth-child(-n + 5),
	.scale code,
	.colours code {
		white-space: nowrap;
	}
	.colours td:nth-child(4) {
		color: var(--ink-2);
		min-width: 12rem;
	}

	/* Type */
	.families {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: var(--s6);
		margin-bottom: var(--s5);
	}
	.family-name {
		margin: 0 0 var(--s1);
		font-size: var(--fs-3);
		font-weight: var(--w-head);
		letter-spacing: var(--tr-h2);
		line-height: var(--lh-head);
	}
	.glyphs-sample {
		margin: 0 0 var(--s3);
		font-size: var(--fs-2);
		color: var(--ink-2);
		font-variant-numeric: lining-nums;
	}
	.mono {
		font-family: var(--code);
		letter-spacing: -0.01em;
	}
	.scale .specimen {
		width: 100%;
		max-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: clip;
		line-height: 1.1;
		font-weight: var(--w-head);
		letter-spacing: -0.01em;
		color: var(--ink);
	}
	.weights th[scope='row'] {
		font-size: var(--label);
		font-weight: var(--w-strong);
		letter-spacing: var(--tr-label);
		text-transform: uppercase;
		color: var(--ink-3);
	}
	.weight-sample {
		font-size: var(--fs-4);
		line-height: 1.2;
	}
	.measure-text {
		max-width: var(--measure);
	}
	.roles {
		border-top: 1px solid var(--rule-strong);
		margin-bottom: var(--s5);
	}
	.role {
		display: grid;
		grid-template-columns: 12rem minmax(0, 1fr);
		gap: var(--s5);
		align-items: baseline;
		padding: var(--s5) 0;
		border-bottom: 1px solid var(--rule);
	}
	.role-name {
		margin: 0 0 var(--s1);
		font-weight: var(--w-strong);
	}
	.spec p {
		margin: 0;
	}
	.spec code {
		font-size: 0.93em;
		padding: 0;
		background: none;
	}
	.sample {
		margin: 0;
		max-width: var(--measure);
	}
	.r-display {
		font-size: var(--h1);
		font-weight: var(--w-display);
		line-height: var(--lh-display);
		letter-spacing: var(--tr-display);
	}
	.r-h2 {
		font-size: var(--h2);
		font-weight: var(--w-head);
		line-height: var(--lh-head);
		letter-spacing: var(--tr-h2);
	}
	.r-h3 {
		font-size: var(--h3);
		font-weight: var(--w-head);
		line-height: var(--lh-head);
		letter-spacing: var(--tr-h3);
	}
	.figures {
		display: grid;
		grid-template-columns: repeat(3, max-content);
		gap: var(--s7);
		margin-bottom: var(--s4);
	}
	.figures .label {
		margin-bottom: var(--s2);
	}
	.prop {
		font-variant-numeric: proportional-nums lining-nums;
	}

	/* Space */
	.space td:last-child {
		width: 100%;
	}
	.bar {
		display: block;
		height: 0.75rem;
		background: var(--rule-strong);
	}
	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
		white-space: nowrap;
	}

	/* Frame */
	.measure {
		max-width: var(--measure);
		margin-top: var(--s5);
	}
	.measure p {
		margin: 0 0 var(--s2);
	}
	.measure-rule {
		display: block;
		height: var(--s2);
		border: 1px solid var(--accent-rule);
		border-top: 0;
	}

	/* Parts */
	.parts {
		border-top: 1px solid var(--rule-strong);
	}
	.part {
		padding: var(--s5) 0;
		border-bottom: 1px solid var(--rule);
	}
	.part h3 {
		margin: 0 0 var(--s4);
		font-size: var(--text);
	}
	.part h3 code {
		margin-left: var(--s2);
		font-size: var(--label);
		font-weight: var(--w-text);
		color: var(--ink-2);
	}
	.part > :last-child,
	.part .table-wrap:last-child {
		margin-bottom: 0;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--s3);
	}

	@media (max-width: 600px) {
		.pairs,
		.role,
		.families {
			grid-template-columns: 1fr;
		}
		.role {
			gap: var(--s3);
		}
		.pairs dt {
			border-bottom: 0;
			padding-bottom: 0;
		}
		.pairs dd {
			padding-top: var(--s1);
		}
		.figures {
			grid-template-columns: repeat(2, max-content);
			gap: var(--s5);
		}
		.tiles {
			margin-left: 0;
		}
	}
</style>
