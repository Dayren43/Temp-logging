<script>
	import { onMount, onDestroy } from 'svelte';
	import { API_BASE } from '$lib/config.js';

	// null = loading, false = error/unreachable, 'unconfigured' = HA not set up,
	// otherwise the shaped state object from the backend.
	let ac = $state(null);
	let busy = $state(false);          // a write is in flight
	let tempTimer = null;              // debounce for set-temperature
	let pollTimer = null;
	let dirtyUntil = 0;                // ignore polls right after a local change

	const on = $derived(ac && typeof ac === 'object' ? ac.on : false);
	const statusLine = $derived(describe(ac));

	function describe(s) {
		if (s === null) return 'Connecting…';
		if (s === 'unconfigured') return 'Home Assistant not configured';
		if (s === false) return 'Unreachable';
		if (!s.available) return 'Unavailable';
		if (!s.on) return 'Off';
		const action = s.hvac_action && s.hvac_action !== 'idle' && s.hvac_action !== 'off'
			? s.hvac_action.charAt(0).toUpperCase() + s.hvac_action.slice(1)
			: (s.mode.charAt(0).toUpperCase() + s.mode.slice(1));
		return s.target_temp != null ? `${action} · ${s.target_temp}° target` : action;
	}

	async function load() {
		// Don't overwrite an optimistic value the user just set.
		if (Date.now() < dirtyUntil) return;
		try {
			const res = await fetch(`${API_BASE}/ac`);
			if (res.status === 503) { ac = 'unconfigured'; return; }
			if (!res.ok) { ac = false; return; }
			ac = await res.json();
		} catch {
			ac = false;
		}
	}

	async function togglePower() {
		if (!ac || typeof ac !== 'object' || busy) return;
		const next = !ac.on;
		ac = { ...ac, on: next, mode: next ? (ac.mode === 'off' ? 'cool' : ac.mode) : 'off' };
		dirtyUntil = Date.now() + 4000;
		busy = true;
		try {
			await fetch(`${API_BASE}/ac/power`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ on: next })
			});
		} catch { /* poll will resync */ }
		busy = false;
		// Give HA a beat to settle, then refresh for the real hvac_action.
		setTimeout(() => { dirtyUntil = 0; load(); }, 1200);
	}

	function nudge(delta) {
		if (!ac || typeof ac !== 'object') return;
		const step = ac.step || 0.5;
		const min = ac.min_temp, max = ac.max_temp;
		const next = Math.min(max, Math.max(min, Math.round((ac.target_temp + delta * step) / step) * step));
		if (next === ac.target_temp) return;
		ac = { ...ac, target_temp: next };
		dirtyUntil = Date.now() + 4000;
		clearTimeout(tempTimer);
		tempTimer = setTimeout(() => pushTemp(next), 600);
	}

	async function pushTemp(temperature) {
		busy = true;
		try {
			await fetch(`${API_BASE}/ac/temp`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ temperature })
			});
		} catch { /* poll will resync */ }
		busy = false;
		setTimeout(() => { dirtyUntil = 0; load(); }, 1000);
	}

	onMount(() => {
		load();
		pollTimer = setInterval(load, 30_000);
	});
	onDestroy(() => {
		clearInterval(pollTimer);
		clearTimeout(tempTimer);
	});
</script>

<section class="ac" data-on={on} data-action={ac && typeof ac === 'object' ? ac.hvac_action : null}>
	<div class="ac-head">
		<div class="ac-icon" aria-hidden="true">
			<svg width="34" height="34" viewBox="0 0 34 34">
				<rect x="4" y="7" width="26" height="12" rx="3" fill="none"
					stroke="currentColor" stroke-width="1.4" />
				<line x1="8" y1="12" x2="26" y2="12" stroke="currentColor" stroke-width="1.2" opacity="0.6" />
				<path d="M9 23 q2 3 0 6 M17 23 q2 3 0 6 M25 23 q2 3 0 6" fill="none"
					stroke="currentColor" stroke-width="1.4" stroke-linecap="round" opacity="0.7" />
			</svg>
		</div>
		<div class="ac-meta">
			<div class="ac-name">Air conditioning</div>
			<div class="ac-status">{statusLine}</div>
		</div>
	</div>

	{#if ac && typeof ac === 'object'}
		<div class="ac-temp">
			<span class="ac-temp-label">Target</span>
			<div class="ac-stepper">
				<button class="ac-step" onclick={() => nudge(-1)} aria-label="Lower target temperature">−</button>
				<span class="ac-temp-val">{ac.target_temp != null ? ac.target_temp : '—'}<span class="ac-deg">°</span></span>
				<button class="ac-step" onclick={() => nudge(1)} aria-label="Raise target temperature">+</button>
			</div>
			{#if ac.current_temp != null}
				<span class="ac-current">room {ac.current_temp}°</span>
			{/if}
		</div>

		<button
			class="ac-toggle"
			role="switch"
			aria-checked={on}
			aria-label="Toggle air conditioning"
			data-on={on}
			disabled={busy}
			onclick={togglePower}
		>
			<span class="ac-toggle-track"><span class="ac-toggle-knob"></span></span>
			<span class="ac-toggle-text">{on ? 'On' : 'Off'}</span>
		</button>
	{:else if ac === 'unconfigured'}
		<div class="ac-hint">Set <code>HA_URL</code>, <code>HA_TOKEN</code> &amp; <code>AC_ENTITY_ID</code> in <code>.env</code></div>
	{:else if ac === false}
		<button class="ac-retry" onclick={load}>Retry</button>
	{/if}
</section>

<style>
	.ac {
		display: grid;
		grid-template-columns: 1fr auto auto;
		align-items: center;
		gap: 28px;
		padding: 18px 24px;
		border-radius: 16px;
		border: 1px solid var(--hairline);
		background: var(--surface);
		position: relative;
		overflow: hidden;
		color: var(--ink-3);
	}

	/* A cool glow when the AC is actively running. */
	.ac::before {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: inherit;
		opacity: 0;
		transition: opacity 0.4s;
		pointer-events: none;
		background: radial-gradient(420px 200px at 88% 50%, rgba(168, 200, 232, 0.10), transparent 70%);
	}
	.ac[data-on="true"]::before { opacity: 1; }

	@media (max-width: 560px) {
		.ac {
			grid-template-columns: 1fr auto;
			grid-template-rows: auto auto;
			gap: 16px 20px;
		}
		.ac-head { grid-column: 1 / -1; }
	}

	.ac-head {
		display: flex;
		align-items: center;
		gap: 14px;
	}

	.ac-icon {
		color: var(--ink-4);
		transition: color 0.3s;
		flex-shrink: 0;
	}
	.ac[data-on="true"] .ac-icon { color: var(--accent-humid); }

	.ac-meta {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.ac-name {
		font-family: 'Instrument Serif', serif;
		font-style: italic;
		font-size: 18px;
		color: var(--ink);
		line-height: 1.1;
	}

	.ac-status {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 11px;
		letter-spacing: 0.04em;
		color: var(--ink-3);
	}

	.ac-temp {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
	}

	.ac-temp-label {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 10px;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-4);
	}

	.ac-stepper {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.ac-step {
		width: 28px;
		height: 28px;
		border-radius: 999px;
		border: 1px solid var(--hairline-strong);
		background: var(--surface-2);
		color: var(--ink-2);
		font-size: 18px;
		line-height: 1;
		cursor: pointer;
		display: grid;
		place-items: center;
		transition: border-color 0.15s, color 0.15s;
	}
	.ac-step:hover { border-color: var(--accent-humid); color: var(--accent-humid); }

	.ac-temp-val {
		font-family: 'Instrument Serif', serif;
		font-size: 30px;
		color: var(--ink);
		line-height: 1;
		min-width: 52px;
		text-align: center;
		font-variant-numeric: tabular-nums;
	}

	.ac-deg { color: var(--ink-3); font-style: italic; }

	.ac-current {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 10px;
		color: var(--ink-4);
		letter-spacing: 0.04em;
	}

	.ac-toggle {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
	}
	.ac-toggle:disabled { opacity: 0.6; cursor: default; }

	.ac-toggle-track {
		width: 48px;
		height: 26px;
		border-radius: 999px;
		background: var(--surface-2);
		border: 1px solid var(--hairline-strong);
		position: relative;
		transition: background 0.2s, border-color 0.2s;
	}
	.ac-toggle[data-on="true"] .ac-toggle-track {
		background: rgba(168, 200, 232, 0.30);
		border-color: var(--accent-humid);
	}

	.ac-toggle-knob {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--ink-3);
		transition: transform 0.2s, background 0.2s;
	}
	.ac-toggle[data-on="true"] .ac-toggle-knob {
		transform: translateX(22px);
		background: var(--accent-humid);
	}

	.ac-toggle-text {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 10px;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-3);
	}

	.ac-hint {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 11px;
		color: var(--ink-4);
		grid-column: 2 / -1;
		text-align: right;
		line-height: 1.5;
	}
	.ac-hint code {
		color: var(--ink-2);
		background: var(--surface-2);
		padding: 1px 4px;
		border-radius: 4px;
	}

	.ac-retry {
		justify-self: end;
		padding: 6px 14px;
		border-radius: 999px;
		border: 1px solid var(--hairline-strong);
		background: var(--surface-2);
		color: var(--ink-2);
		font-size: 12px;
		cursor: pointer;
	}
	.ac-retry:hover { border-color: var(--accent-humid); color: var(--accent-humid); }
</style>
