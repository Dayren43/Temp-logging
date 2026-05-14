<script>
	import { onDestroy } from 'svelte';
	import { SENSOR } from '$lib/config.js';

	let { lastTimestamp = null } = $props();

	let now = $state(new Date());
	const tick = setInterval(() => { now = new Date(); }, 60_000);
	onDestroy(() => clearInterval(tick));

	const dayName = $derived(now.toLocaleDateString(undefined, { weekday: 'long' }));
	const shortDate = $derived(
		now.getDate() + ' ' +
		now.toLocaleDateString(undefined, { month: 'short' }).toUpperCase()
	);

	const lastUpdated = $derived(formatAgo(lastTimestamp, now));

	function formatAgo(ts, _now) {
		if (!ts) return 'just now';
		const ms = Date.now() - ts;
		if (ms < 60_000) return 'just now';
		const m = Math.round(ms / 60_000);
		if (m < 60) return `${m} min ago`;
		return `${Math.round(m / 60)} hr ago`;
	}
</script>

<header class="topbar">
	<div class="datepill">
		<div class="datepill-day">{dayName}</div>
		<div class="datepill-date">{shortDate}</div>
	</div>

	<div class="sensor" aria-label="Sensor location">
		<span class="sensor-dot"></span>
		<span class="sensor-name">{SENSOR.name}</span>
	</div>

	<div class="meta">
		<span class="status-dot" aria-hidden="true"></span>
		<span>Live · {lastUpdated}</span>
	</div>
</header>

<style>
	.topbar {
		display: grid;
		grid-template-columns: auto 1fr auto;
		align-items: center;
		gap: 32px;
	}

	@media (max-width: 680px) {
		.topbar {
			grid-template-columns: 1fr 1fr;
			grid-template-rows: auto auto;
		}
		.sensor {
			grid-column: 1 / -1;
			order: -1;
			justify-self: center;
		}
	}

	.datepill {
		display: flex;
		flex-direction: column;
		gap: 2px;
		line-height: 1.05;
	}

	.datepill-day {
		font-family: 'Instrument Serif', serif;
		font-style: italic;
		font-size: 20px;
		color: var(--ink);
		letter-spacing: -0.005em;
	}

	.datepill-date {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 11px;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--ink-4);
	}

	.sensor {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		justify-self: center;
		padding: 7px 14px;
		border: 1px solid var(--hairline-strong);
		border-radius: 999px;
		background: var(--surface);
		color: var(--ink);
		font-size: 13px;
	}

	.sensor-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--accent-feels);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-feels) 18%, transparent);
		flex-shrink: 0;
	}

	.meta {
		display: flex;
		align-items: center;
		gap: 8px;
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 12.5px;
		color: var(--ink-3);
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}

	.status-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--accent-feels);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-feels) 16%, transparent);
		flex-shrink: 0;
		animation: pulse 2.4s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-feels) 16%, transparent); }
		50%       { box-shadow: 0 0 0 5px color-mix(in srgb, var(--accent-feels)  6%, transparent); }
	}
</style>
