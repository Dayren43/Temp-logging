<script>
	import { onMount } from 'svelte';

	function emptyGrid() {
		const now = new Date();
		const grid = [];
		for (let i = 11; i >= 0; i--) {
			const d = new Date(now.getFullYear(), now.getMonth() - i, 1);
			grid.push({
				key: `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`,
				date: d,
				label: d.toLocaleDateString(undefined, { month: 'short' }),
				year: d.getFullYear(),
				month: d.getMonth(),
				empty: true
			});
		}
		return grid;
	}

	let months = $state(emptyGrid());

	onMount(async () => {
		try {
			const res = await fetch('http://epsilon.local:3000/monthly?months=12');
			const json = await res.json();
			const filled = emptyGrid();
			for (const row of (json.data || [])) {
				const d = new Date(row.month);
				const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
				const cell = filled.find(m => m.key === key);
				if (cell) Object.assign(cell, {
					empty: false,
					tAvg: parseFloat(row.avg_temp),
					tMin: parseFloat(row.min_temp),
					tMax: parseFloat(row.max_temp),
					hAvg: Math.round(parseFloat(row.avg_humid)),
					comfortPct: parseFloat(row.comfort_pct) || 0
				});
			}
			months = filled;
		} catch (e) {
			console.error('Year fetch failed', e);
		}
	});

	const realMonths = $derived(months.filter(m => !m.empty));
	const tAvgs = $derived(realMonths.map(m => m.tAvg));
	const lo = $derived(tAvgs.length ? Math.min(...tAvgs) : 19);
	const hi = $derived(tAvgs.length ? Math.max(...tAvgs) : 25);
	const span = $derived((hi - lo) || 1);

	const yearAvgT = $derived(tAvgs.length ? tAvgs.reduce((a, b) => a + b, 0) / tAvgs.length : null);
	const yearAvgH = $derived(realMonths.length ? realMonths.reduce((a, m) => a + m.hAvg, 0) / realMonths.length : null);
	const yearComfort = $derived(realMonths.length ? realMonths.reduce((a, m) => a + m.comfortPct, 0) / realMonths.length : null);
	const peakMonth = $derived(realMonths.length ? realMonths.reduce((a, m) => m.tAvg > a.tAvg ? m : a, realMonths[0]) : null);
	const coldestMonth = $derived(realMonths.length ? realMonths.reduce((a, m) => m.tAvg < a.tAvg ? m : a, realMonths[0]) : null);

	// Interpolate between --accent-feels (#c8d8b0) and --accent-temp (#f5b8a4)
	function stripStyle(m) {
		const t = m.empty ? 0 : (m.tAvg - lo) / span;
		const r = Math.round(200 + 45 * t);
		const g = Math.round(216 - 32 * t);
		const b = Math.round(176 - 12 * t);
		const a = (0.35 + 0.5 * t).toFixed(2);
		return `background:rgba(${r},${g},${b},${a})`;
	}
</script>

<section class="year">
	<div class="year-head">
		<div>
			<div class="year-eyebrow">Last 12 months</div>
			<h3 class="year-title">A whole year, mostly indoors</h3>
		</div>
		{#if yearAvgT != null}
		<div class="year-totals">
			<div class="year-total">
				<span class="year-total-label">Year avg</span>
				<span class="year-total-value temp">{yearAvgT.toFixed(1)}°</span>
			</div>
			<div class="year-total">
				<span class="year-total-label">Avg humidity</span>
				<span class="year-total-value humid">{Math.round(yearAvgH)}%</span>
			</div>
			<div class="year-total">
				<span class="year-total-label">In comfort</span>
				<span class="year-total-value">{Math.round(yearComfort)}%</span>
			</div>
		</div>
		{/if}
	</div>

	<div class="year-grid">
		{#each months as m (m.key)}
		<div class="ycell" class:ycell-empty={m.empty}
			title={m.empty ? `${m.label} ${m.year} · no data` : `${m.label} ${m.year} · ${m.tAvg.toFixed(1)}° · ${m.hAvg}%`}>
			{#if !m.empty}
				<div class="ycell-strip" aria-hidden="true" style={stripStyle(m)}></div>
			{/if}
			<div class="ycell-month">
				{m.label}
				{#if m.month === 0}<span class="ycell-year"> {m.year}</span>{/if}
			</div>
			{#if m.empty}
				<div class="ycell-val empty">—</div>
				<div class="ycell-range empty"><span>—</span></div>
				<div class="ycell-humid empty">—</div>
			{:else}
				<div class="ycell-val">{m.tAvg.toFixed(1)}°</div>
				<div class="ycell-range">
					<span>{m.tMin.toFixed(0)}</span>
					<i></i>
					<span>{m.tMax.toFixed(0)}</span>
				</div>
				<div class="ycell-humid">{m.hAvg}<em>%</em></div>
			{/if}
		</div>
		{/each}
	</div>

	{#if peakMonth && coldestMonth}
	<div class="year-foot">
		<span>Warmest month <b class="foot-temp">{peakMonth.label} · {peakMonth.tAvg.toFixed(1)}°</b></span>
		<span class="year-foot-sep" aria-hidden="true">·</span>
		<span>Coolest <b class="foot-feels">{coldestMonth.label} · {coldestMonth.tAvg.toFixed(1)}°</b></span>
		{#if realMonths.length < 12}
		<span class="year-foot-sep" aria-hidden="true">·</span>
		<span class="year-foot-note">Sensor has been running {realMonths.length} month{realMonths.length === 1 ? '' : 's'} so far</span>
		{/if}
	</div>
	{/if}
</section>

<style>
	.year {
		display: flex;
		flex-direction: column;
		gap: 20px;
		padding-top: 8px;
	}

	.year-head {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: 24px;
		flex-wrap: wrap;
	}

	.year-eyebrow {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 10.5px;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: var(--ink-4);
		margin-bottom: 6px;
	}

	.year-title {
		font-family: 'Instrument Serif', serif;
		font-style: italic;
		font-size: 28px;
		font-weight: 400;
		color: var(--ink);
		margin: 0;
		line-height: 1.1;
	}

	.year-totals {
		display: flex;
		gap: 28px;
		align-items: flex-end;
		flex-shrink: 0;
	}

	.year-total {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 4px;
	}

	.year-total-label {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 10px;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-4);
	}

	.year-total-value {
		font-family: 'Instrument Serif', serif;
		font-size: 22px;
		line-height: 1;
		color: var(--ink-2);
		font-variant-numeric: tabular-nums;
	}

	.year-total-value.temp  { color: var(--accent-temp); }
	.year-total-value.humid { color: var(--accent-humid); }

	.year-grid {
		display: grid;
		grid-template-columns: repeat(12, 1fr);
		gap: 4px;
	}

	@media (max-width: 900px) {
		.year-grid { grid-template-columns: repeat(6, 1fr); }
	}

	@media (max-width: 560px) {
		.year-grid { grid-template-columns: repeat(4, 1fr); }
	}

	.ycell {
		display: flex;
		flex-direction: column;
		gap: 5px;
		padding: 10px 10px 12px 15px;
		border-radius: 10px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		min-width: 0;
		position: relative;
		overflow: hidden;
	}

	.ycell-empty { opacity: 0.35; }

	.ycell-strip {
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 3px;
		border-radius: 10px 0 0 10px;
		pointer-events: none;
	}

	.ycell-month {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 10px;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--ink-3);
		white-space: nowrap;
	}

	.ycell-year {
		color: var(--ink-4);
		font-size: 9px;
	}

	.ycell-val {
		font-family: 'Instrument Serif', serif;
		font-size: 18px;
		line-height: 1;
		color: var(--accent-temp);
		font-variant-numeric: tabular-nums;
	}

	.ycell-val.empty,
	.ycell-range.empty,
	.ycell-humid.empty {
		color: var(--ink-4);
		font-size: 13px;
	}

	.ycell-range {
		display: flex;
		align-items: center;
		gap: 3px;
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 9.5px;
		color: var(--ink-4);
		font-variant-numeric: tabular-nums;
	}

	.ycell-range i {
		flex: 1;
		height: 1px;
		background: var(--hairline-strong);
		min-width: 6px;
	}

	.ycell-humid {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 11px;
		color: var(--accent-humid);
		font-variant-numeric: tabular-nums;
	}

	.ycell-humid em {
		font-style: normal;
		font-size: 9px;
		color: var(--ink-4);
	}

	.year-foot {
		display: flex;
		align-items: center;
		gap: 10px;
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 11.5px;
		color: var(--ink-3);
		flex-wrap: wrap;
	}

	.year-foot-sep { color: var(--ink-4); }
	.foot-temp  { color: var(--accent-temp);  font-style: normal; }
	.foot-feels { color: var(--accent-feels); font-style: normal; }

	.year-foot-note {
		color: var(--ink-4);
		font-style: italic;
		font-family: 'Instrument Serif', serif;
		font-size: 12px;
	}
</style>
