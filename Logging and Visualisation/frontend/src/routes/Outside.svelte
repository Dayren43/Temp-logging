<script>
	let { outside = null } = $props();

	const SVG_W = 220;
	const SVG_H = 36;

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
		return {
			line: cmds,
			area: `${cmds} L ${SVG_W} ${SVG_H} L 0 ${SVG_H} Z`
		};
	});

	const axisLabels = $derived(() => {
		const pts = outside?.hourly;
		if (!pts || pts.length < 2) return ['', '', ''];
		const fmt = h => h.toString().padStart(2, '0') + ':00';
		return [fmt(pts[0].hour), fmt(pts[Math.floor(pts.length / 2)].hour), fmt(pts[pts.length - 1].hour)];
	});

	const updatedMin = $derived(
		outside?.updatedAt
			? Math.max(1, Math.round((Date.now() - outside.updatedAt) / 60_000))
			: null
	);
</script>

{#if outside}
<section class="outside">
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
			{#each axisLabels() as label}
				<span>{label}</span>
			{/each}
		</div>
	</div>
	{/if}
</section>
{/if}

<style>
	.outside {
		display: grid;
		grid-template-columns: auto auto 1fr;
		align-items: center;
		gap: 32px;
		padding: 20px 24px;
		border-radius: 16px;
		border: 1px solid var(--hairline);
		background: var(--surface);
	}

	@media (max-width: 720px) {
		.outside {
			grid-template-columns: 1fr 1fr;
			grid-template-rows: auto auto;
		}
	}

	@media (max-width: 480px) {
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
		align-items: baseline;
		gap: 16px;
	}

	.outside-temp {
		display: flex;
		align-items: baseline;
		font-family: 'Instrument Serif', serif;
		line-height: 1;
	}

	.outside-temp-num {
		font-size: 52px;
		color: var(--accent-feels);
		letter-spacing: -0.02em;
	}

	.outside-temp-unit {
		font-size: 24px;
		color: var(--ink-3);
		font-style: italic;
	}

	.outside-meta {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding-bottom: 4px;
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
</style>
