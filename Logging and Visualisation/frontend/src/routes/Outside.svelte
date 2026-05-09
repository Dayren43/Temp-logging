<script>
	let { inside = null, outside = null } = $props();

	const SVG_W = 220;
	const SVG_H = 36;

	function recommendation(tI, hI, tO, hO) {
		if (tI == null || tO == null) return { state: 'neutral', verb: 'Up to you', why: 'Loading outdoor data…' };
		if (tO > tI + 1 && tI > 22) return { state: 'closed', verb: 'Keep windows closed', why: `Outside is ${(tO - tI).toFixed(1)}° warmer — opening up would heat the room.` };
		if (hO > 70) return { state: 'closed', verb: 'Keep windows closed', why: `Outside humidity is ${hO}% — too damp to let in.` };
		if (tO < 5) return { state: 'closed', verb: 'Keep windows closed', why: `It's ${tO.toFixed(1)}° outside — you'd lose all the warmth.` };
		if (tI > 23.5 && tO < tI - 1.5) return { state: 'open', verb: 'Open a window', why: `Outside is ${(tI - tO).toFixed(1)}° cooler — fresh air will bring the temperature down.` };
		if (hI > 60 && hO < hI - 8) return { state: 'open', verb: 'Crack a window', why: `Drier outside (${hO}% vs ${hI}%) — a few minutes will help.` };
		if (tO < tI - 6) return { state: 'closed', verb: 'Keep windows closed', why: `${(tI - tO).toFixed(1)}° colder outside — not worth the heat loss.` };
		return { state: 'neutral', verb: 'Up to you', why: 'Conditions are similar — no real benefit to opening up.' };
	}

	const rec = $derived(recommendation(
		inside?.temp ?? null,
		inside?.humid ?? null,
		outside?.temp ?? null,
		outside?.humid ?? null
	));

	const tDelta = $derived(
		inside?.temp != null && outside?.temp != null ? inside.temp - outside.temp : null
	);
	const hDelta = $derived(
		inside?.humid != null && outside?.humid != null ? inside.humid - outside.humid : null
	);

	const sparkPath = $derived(() => {
		const pts = outside?.hourly;
		if (!pts || pts.length < 2) return { line: '', area: '' };
		const temps = pts.map(p => p.temp);
		const lo = Math.min(...temps);
		const hi = Math.max(...temps);
		const span = (hi - lo) || 1;
		const cmds = pts.map((p, i) => {
			const x = (i / (pts.length - 1)) * SVG_W;
			const y = SVG_H - ((p.temp - lo) / span) * (SVG_H - 6) - 3;
			return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
		}).join(' ');
		return { line: cmds, area: `${cmds} L ${SVG_W} ${SVG_H} L 0 ${SVG_H} Z` };
	});

	const axisLabels = $derived(() => {
		const pts = outside?.hourly;
		if (!pts || pts.length < 2) return ['', '', ''];
		const fmt = h => h.toString().padStart(2, '0') + ':00';
		return [fmt(pts[0].hour), fmt(pts[Math.floor(pts.length / 2)].hour), fmt(pts[pts.length - 1].hour)];
	});

	const updatedMin = $derived(
		outside?.updatedAt ? Math.max(1, Math.round((Date.now() - outside.updatedAt) / 60_000)) : null
	);
</script>

{#if outside}
<section class="outside" data-state={rec.state}>
	<div class="outside-loc">
		<svg class="cloud-icon" width="40" height="32" viewBox="0 0 40 32" aria-hidden="true">
			<path d="M11 24 q-7 0 -7 -6 q0 -5 5 -6 q1 -5 7 -5 q5 0 7 4 q2 -1 4 -1 q5 0 6 5 q5 0 5 5 q0 4 -5 4 z"
				fill="rgba(245,235,224,0.08)" stroke="rgba(245,235,224,0.32)" stroke-width="1" />
			<circle cx="29" cy="11" r="4.5" fill="rgba(245,184,164,0.55)" />
		</svg>
		<div class="outside-loc-meta">
			<div class="outside-loc-name">Järfälla</div>
			<div class="outside-loc-cond">
				{outside.condition}{updatedMin != null ? ` · updated ${updatedMin} min ago` : ''}
			</div>
		</div>
	</div>

	<div class="outside-now">
		<div class="outside-temp">
			<span class="outside-temp-num">{Math.round(outside.temp)}</span>
			<span class="outside-temp-unit">°</span>
		</div>
		<div class="outside-meta">
			<div class="outside-meta-row">
				<span class="outside-meta-label">Feels</span>
				<span class="outside-meta-value">{Math.round(outside.feels)}°</span>
			</div>
			<div class="outside-meta-row">
				<span class="outside-meta-label">Humid</span>
				<span class="outside-meta-value">{outside.humid}%</span>
			</div>
			<div class="outside-meta-row">
				<span class="outside-meta-label">Wind</span>
				<span class="outside-meta-value">{outside.wind} km/h</span>
			</div>
		</div>
	</div>

	{#if outside.hourly && outside.hourly.length >= 2}
	<div class="outside-spark">
		<div class="outside-spark-label">Next 12 hours</div>
		<svg viewBox="0 0 {SVG_W} {SVG_H}" preserveAspectRatio="none" class="outside-spark-svg">
			<path d={sparkPath().area} fill="var(--accent-feels)" fill-opacity="0.18" />
			<path d={sparkPath().line} fill="none" stroke="var(--accent-feels)"
				stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
		</svg>
		<div class="outside-spark-axis">
			{#each axisLabels() as label}<span>{label}</span>{/each}
		</div>
	</div>
	{/if}

	<div class="outside-rec">
		<div class="outside-rec-badge" data-state={rec.state}>
			<svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
				<rect x="2" y="2" width="10" height="10" rx="1.4" fill="none" stroke="currentColor" stroke-width="1.4"/>
				<line x1="7" y1="2" x2="7" y2="12" stroke="currentColor" stroke-width="1.4"/>
				<line x1="2" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="1.4"/>
				{#if rec.state === 'open'}
					<path d="M9 4 L11 4 L11 6" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
				{/if}
			</svg>
			<span>{rec.verb}</span>
		</div>
		<div class="outside-rec-why">{rec.why}</div>
		{#if tDelta != null}
		<div class="outside-rec-deltas">
			<span class="delta-pair"><em>Inside</em><b class="delta-temp">{inside.temp.toFixed(1)}°</b></span>
			<span class="delta-arrow">{Math.abs(tDelta) < 0.3 ? '≈' : tDelta > 0 ? '→ cooler' : '→ warmer'}</span>
			<span class="delta-pair"><em>Outside</em><b class="delta-outside">{outside.temp.toFixed(1)}°</b></span>
			{#if hDelta != null}
			<span class="delta-sep">·</span>
			<span class="delta-humid">Humidity <b>{hDelta > 0 ? '+' : ''}{hDelta.toFixed(0)}%</b></span>
			{/if}
		</div>
		{/if}
	</div>
</section>
{/if}

<style>
	.outside {
		display: grid;
		grid-template-columns: auto auto 1fr auto;
		align-items: center;
		gap: 32px;
		padding: 20px 24px;
		border-radius: 16px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		position: relative;
		overflow: hidden;
	}

	.outside::before {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: inherit;
		opacity: 0;
		transition: opacity 0.4s;
		pointer-events: none;
	}

	.outside[data-state="open"]::before {
		background: radial-gradient(400px 200px at 80% 50%, rgba(155,201,168,0.08), transparent 70%);
		opacity: 1;
	}

	.outside[data-state="closed"]::before {
		background: radial-gradient(400px 200px at 80% 50%, rgba(245,184,164,0.07), transparent 70%);
		opacity: 1;
	}

	@media (max-width: 900px) {
		.outside {
			grid-template-columns: 1fr 1fr;
			grid-template-rows: auto auto;
		}
	}

	@media (max-width: 560px) {
		.outside { grid-template-columns: 1fr; }
	}

	.outside-loc {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.cloud-icon { flex-shrink: 0; }

	.outside-loc-meta {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.outside-loc-name {
		font-family: 'Instrument Serif', serif;
		font-style: italic;
		font-size: 18px;
		color: var(--ink);
		line-height: 1.1;
	}

	.outside-loc-cond {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 11px;
		letter-spacing: 0.04em;
		color: var(--ink-3);
		text-transform: lowercase;
	}

	.outside-now {
		display: flex;
		align-items: flex-end;
		gap: 14px;
	}

	.outside-temp {
		display: flex;
		align-items: flex-end;
		font-family: 'Instrument Serif', serif;
		line-height: 1;
	}

	.outside-temp-num {
		font-size: 52px;
		color: var(--accent-feels);
		letter-spacing: -0.02em;
		line-height: 0.88;
	}

	.outside-temp-unit {
		font-size: 24px;
		color: var(--ink-3);
		font-style: italic;
		line-height: 1;
		margin-bottom: 2px;
	}

	.outside-meta {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding-bottom: 2px;
	}

	.outside-meta-row {
		display: flex;
		align-items: baseline;
		gap: 6px;
	}

	.outside-meta-label {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 10px;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-4);
		width: 36px;
	}

	.outside-meta-value {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 13px;
		color: var(--ink-2);
		font-variant-numeric: tabular-nums;
	}

	.outside-spark {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.outside-spark-label {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 10px;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-4);
	}

	.outside-spark-svg {
		width: 100%;
		height: 36px;
		display: block;
	}

	.outside-spark-axis {
		display: flex;
		justify-content: space-between;
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 9.5px;
		color: var(--ink-4);
		font-variant-numeric: tabular-nums;
	}

	.outside-rec {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-width: 240px;
	}

	.outside-rec-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 12px;
		border-radius: 999px;
		font-size: 12px;
		font-weight: 500;
		border: 1px solid;
		width: fit-content;
	}

	.outside-rec-badge[data-state="open"] {
		background: rgba(155,201,168,0.14);
		border-color: rgba(155,201,168,0.45);
		color: #c8d8b0;
	}

	.outside-rec-badge[data-state="closed"] {
		background: rgba(245,184,164,0.12);
		border-color: rgba(245,184,164,0.4);
		color: #f5b8a4;
	}

	.outside-rec-badge[data-state="neutral"] {
		background: rgba(245,235,224,0.06);
		border-color: var(--hairline-strong);
		color: var(--ink-2);
	}

	.outside-rec-why {
		font-family: 'Instrument Serif', serif;
		font-style: italic;
		font-size: 13px;
		color: var(--ink-3);
		line-height: 1.4;
	}

	.outside-rec-deltas {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 11px;
		color: var(--ink-3);
	}

	.delta-pair {
		display: inline-flex;
		align-items: baseline;
		gap: 4px;
	}

	.delta-pair em { font-style: normal; color: var(--ink-4); }
	.delta-temp    { color: var(--accent-temp); }
	.delta-outside { color: var(--accent-feels); }
	.delta-arrow   { color: var(--ink-3); font-size: 10px; }
	.delta-sep     { color: var(--ink-4); }
	.delta-humid b { color: var(--accent-humid); }
</style>
