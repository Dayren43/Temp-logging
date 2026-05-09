<script>
	import { onDestroy } from 'svelte';

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

	<nav class="rooms" aria-label="Sensor location">
		<button class="room is-active" type="button">
			<span class="room-dot"></span>
			Living room
		</button>
		<button class="room is-disabled" type="button" title="Multi-sensor coming soon" disabled>
			<span class="room-dot is-off"></span>
			Bedroom
		</button>
		<button class="room is-disabled" type="button" disabled>
			<span class="room-dot is-off"></span>
			Office
		</button>
		<button class="room-add" type="button" aria-label="Add sensor">+</button>
	</nav>

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
		.rooms {
			grid-column: 1 / -1;
			order: -1;
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

	.rooms {
		display: flex;
		align-items: center;
		gap: 4px;
		justify-content: center;
	}

	.room {
		appearance: none;
		background: transparent;
		border: 1px solid transparent;
		color: var(--ink-3);
		padding: 7px 14px;
		border-radius: 999px;
		font: inherit;
		font-size: 13px;
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		transition: color 0.15s, background 0.15s, border-color 0.15s;
	}

	.room:hover:not(:disabled) { color: var(--ink-2); }

	.room.is-active {
		background: var(--surface);
		border-color: var(--hairline-strong);
		color: var(--ink);
	}

	.room.is-disabled {
		color: var(--ink-4);
		cursor: default;
	}

	.room-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: #c8d8b0;
		box-shadow: 0 0 0 3px rgba(200, 216, 176, 0.18);
		flex-shrink: 0;
	}

	.room-dot.is-off {
		background: var(--ink-4);
		box-shadow: none;
	}

	.room-add {
		appearance: none;
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background: transparent;
		border: 1px dashed var(--hairline-strong);
		color: var(--ink-3);
		font-size: 16px;
		line-height: 1;
		cursor: pointer;
		margin-left: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: color 0.15s, border-color 0.15s;
	}

	.room-add:hover {
		color: var(--ink);
		border-color: var(--ink-3);
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
		background: #c8d8b0;
		box-shadow: 0 0 0 3px rgba(200, 216, 176, 0.16);
		flex-shrink: 0;
		animation: pulse 2.4s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { box-shadow: 0 0 0 3px rgba(200, 216, 176, 0.16); }
		50%       { box-shadow: 0 0 0 5px rgba(200, 216, 176, 0.06); }
	}
</style>
