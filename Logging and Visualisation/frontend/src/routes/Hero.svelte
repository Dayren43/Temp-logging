<script>
	import { onDestroy } from 'svelte';
	import { COMFORT } from '$lib/config.js';
	import { tweaks } from '$lib/tweaks.svelte.js';

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

	const cond = $derived(getCondition(temp, humid));

	function getCondition(t, h) {
		if (t == null) return { word: '—', hint: '' };
		if (t < 18) return { word: 'Chilly', hint: 'Grab a sweater' };
		if (t > 26) {
			if (h > 60) return { word: 'Muggy', hint: 'Crack a window' };
			return { word: 'Toasty', hint: 'Could use some air' };
		}
		if (h > 65) return { word: 'A bit damp', hint: 'Maybe ventilate' };
		if (h < 30) return { word: 'Dry', hint: 'A plant might disagree' };
		if (t >= COMFORT.lo && t <= COMFORT.hi && h >= 35 && h <= 55) return { word: 'Cozy', hint: 'Right in the pocket' };
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
			{tweaks.locationName}{outsideCondition ? ` · ${outsideCondition}` : ''}
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
</style>
