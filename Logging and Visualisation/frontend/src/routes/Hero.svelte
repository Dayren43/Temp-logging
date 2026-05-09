<script>
	import { onDestroy } from 'svelte';

	let { temp = null, humid = null, outside = null } = $props();

	const outsideCondition = $derived(outside?.condition ?? '');

	let now = $state(new Date());
	const tick = setInterval(() => { now = new Date(); }, 60_000);
	onDestroy(() => clearInterval(tick));

	const currentTime = $derived(
		now.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', hour12: false })
	);

	const intPart = $derived(temp != null ? Math.trunc(temp) : '—');
	const decPart = $derived(temp != null ? Math.abs(Math.round((temp - Math.trunc(temp)) * 10)) : '');

	const feelsLike = $derived(computeFeelsLike(temp, humid));
	const cond = $derived(getCondition(temp, humid));

	function computeFeelsLike(t, h) {
		if (t == null || h == null) return null;
		const T = t * 9 / 5 + 32;
		const R = h;
		const HI_F = -42.379 +
			2.04901523 * T +
			10.14333127 * R -
			0.22475541 * T * R -
			0.00683783 * T * T -
			0.05481717 * R * R +
			0.00122874 * T * T * R +
			0.00085282 * T * R * R -
			0.00000199 * T * T * R * R;
		return (HI_F - 32) * 5 / 9;
	}

	function getCondition(t, h) {
		if (t == null) return { word: '—', hint: '' };
		if (t < 18) return { word: 'Chilly', hint: 'Grab a sweater' };
		if (t > 26) {
			if (h > 60) return { word: 'Muggy', hint: 'Crack a window' };
			return { word: 'Toasty', hint: 'Could use some air' };
		}
		if (h > 65) return { word: 'A bit damp', hint: 'Maybe ventilate' };
		if (h < 30) return { word: 'Dry', hint: 'A plant might disagree' };
		if (t >= 20 && t <= 24 && h >= 35 && h <= 55) return { word: 'Cozy', hint: 'Right in the pocket' };
		return { word: 'Comfortable', hint: 'All quiet' };
	}

</script>

<section class="hero">
	<div class="hero-eyebrow">
		<span class="eyebrow-label">Right now</span>
		<span class="eyebrow-sep" aria-hidden="true"></span>
		<span class="eyebrow-time">{currentTime}</span>
		<span class="eyebrow-sep" aria-hidden="true"></span>
		<span class="eyebrow-loc">
			Järfälla{outsideCondition ? ` · ${outsideCondition}` : ''}
		</span>
	</div>

	<div class="hero-row">
		<div class="hero-temp">
			<span class="hero-int">{intPart}</span>
			<span class="hero-dec">{temp != null ? `.${decPart}` : ''}</span>
			<span class="hero-unit">°C</span>
		</div>

		<div class="hero-meta">
			<div class="hero-word">{cond.word}</div>
			{#if cond.hint}
				<div class="hero-hint">{cond.hint}</div>
			{/if}
		</div>
	</div>

	<div class="hero-stats">
		<div class="stat">
			<div class="stat-label">Feels like</div>
			<div class="stat-value feels">
				{feelsLike != null ? feelsLike.toFixed(1) : '—'}<span class="stat-unit">°</span>
			</div>
		</div>
		<div class="stat-sep" aria-hidden="true"></div>
		<div class="stat">
			<div class="stat-label">Humidity</div>
			<div class="stat-value humid">
				{humid != null ? humid : '—'}<span class="stat-unit">%</span>
			</div>
		</div>
	</div>
</section>

<style>
	.hero {
		display: flex;
		flex-direction: column;
		gap: 22px;
		padding: 8px 4px 4px;
	}

	.hero-eyebrow {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 11.5px;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: var(--ink-3);
	}

	.eyebrow-label { font-weight: 500; }

	.eyebrow-sep {
		width: 28px;
		height: 1px;
		background: var(--hairline-strong);
		flex-shrink: 0;
	}

	.eyebrow-time {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-variant-numeric: tabular-nums;
		letter-spacing: 0.04em;
		color: var(--ink-2);
	}

	.eyebrow-loc {
		font-family: 'Geist Mono', ui-monospace, monospace;
		letter-spacing: 0.04em;
		color: var(--ink-3);
		text-transform: none;
	}

	.hero-row {
		display: flex;
		align-items: flex-end;
		gap: 32px;
		flex-wrap: wrap;
	}

	.hero-temp {
		display: flex;
		align-items: baseline;
		font-family: 'Instrument Serif', serif;
		line-height: 0.85;
		font-variant-numeric: tabular-nums;
	}

	.hero-int {
		font-size: clamp(140px, 18vw, 220px);
		letter-spacing: -0.04em;
		font-weight: 400;
		color: var(--accent-temp);
	}

	.hero-dec {
		font-size: clamp(56px, 7vw, 90px);
		color: var(--ink-2);
		letter-spacing: -0.02em;
	}

	.hero-unit {
		font-size: clamp(40px, 5vw, 60px);
		color: var(--ink-3);
		margin-left: 6px;
		font-style: italic;
	}

	.hero-meta {
		padding-bottom: 16px;
		max-width: 280px;
	}

	.hero-word {
		font-family: 'Instrument Serif', serif;
		font-size: 32px;
		font-style: italic;
		color: var(--ink);
		line-height: 1.1;
	}

	.hero-hint {
		margin-top: 8px;
		font-size: 13px;
		color: var(--ink-3);
		letter-spacing: 0.01em;
	}

	.hero-rec {
		margin-top: 10px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		align-items: flex-start;
	}

	.hero-rec-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		border-radius: 999px;
		font-size: 12px;
		font-weight: 500;
		border: 1px solid;
		cursor: default;
	}

	.hero-rec-badge[data-state="open"] {
		background: rgba(155, 201, 168, 0.14);
		border-color: rgba(155, 201, 168, 0.45);
		color: #c8d8b0;
	}

	.hero-rec-badge[data-state="closed"] {
		background: rgba(245, 184, 164, 0.12);
		border-color: rgba(245, 184, 164, 0.4);
		color: #f5b8a4;
	}

	.hero-rec-badge[data-state="neutral"] {
		background: rgba(245, 235, 224, 0.06);
		border-color: var(--hairline-strong);
		color: var(--ink-2);
	}

	.hero-rec-why {
		font-family: 'Instrument Serif', serif;
		font-style: italic;
		font-size: 13px;
		color: var(--ink-3);
	}

	.hero-stats {
		display: flex;
		align-items: stretch;
		gap: 28px;
		padding: 18px 0 4px;
		border-top: 1px solid var(--hairline);
		flex-wrap: wrap;
	}

	.stat {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 80px;
	}

	.stat-label {
		font-size: 11px;
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--ink-4);
		font-weight: 500;
	}

	.stat-value {
		font-family: 'Instrument Serif', serif;
		font-size: 28px;
		line-height: 1;
		letter-spacing: -0.01em;
		font-variant-numeric: tabular-nums;
		color: var(--ink);
		display: inline-flex;
		align-items: baseline;
		gap: 1px;
	}

	.stat-value.feels { color: var(--accent-feels); }
	.stat-value.humid { color: var(--accent-humid); }

	.stat-unit {
		font-size: 16px;
		color: var(--ink-3);
		margin-left: 2px;
		font-style: italic;
	}

	.stat-delta {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 18px;
		font-variant-numeric: tabular-nums;
		color: var(--ink-2);
	}

	.trend-arrow {
		margin-right: 4px;
		flex-shrink: 0;
		align-self: center;
	}

	.stat-sep {
		width: 1px;
		background: var(--hairline);
		align-self: stretch;
	}
</style>
