<script>
	import { tweaks, setTweak } from '$lib/tweaks.svelte.js';

	let open = $state(false);

	let coordsText = $state(`${tweaks.locationLat}, ${tweaks.locationLon}`);
	let coordsErr = $state('');

	function commitCoords(s) {
		const m = /^\s*(-?\d+(?:\.\d+)?)\s*[,\s]\s*(-?\d+(?:\.\d+)?)\s*$/.exec(s);
		if (!m) { coordsErr = 'expected: lat, lon'; return; }
		const lat = parseFloat(m[1]), lon = parseFloat(m[2]);
		if (lat < -90 || lat > 90 || lon < -180 || lon > 180) {
			coordsErr = 'out of range';
			return;
		}
		setTweak('locationLat', lat);
		setTweak('locationLon', lon);
		coordsText = `${lat}, ${lon}`;
		coordsErr = '';
	}
</script>

<div class="tweaks-wrap">
	{#if open}
	<div class="tweaks-panel" role="dialog" aria-label="Display settings">
		<div class="tweaks-head">
			<span class="tweaks-title">Display</span>
			<button class="tweaks-close" onclick={() => open = false} aria-label="Close">
				<svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
					<path d="M1 1 L9 9 M9 1 L1 9" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
				</svg>
			</button>
		</div>

		<div class="tweaks-body">
			<div class="tweaks-sect">Chart</div>

			<div class="tweaks-row">
				<span class="tweaks-label">Style</span>
				<div class="tweaks-seg" role="radiogroup">
					{#each ['line', 'area'] as style}
					<button
						class="tweaks-seg-btn"
						class:is-active={tweaks.chartStyle === style}
						role="radio"
						aria-checked={tweaks.chartStyle === style}
						onclick={() => setTweak('chartStyle', style)}
					>{style}</button>
					{/each}
				</div>
			</div>

			<div class="tweaks-sect">Layout</div>

			<div class="tweaks-row">
				<span class="tweaks-label">Density</span>
				<div class="tweaks-seg" role="radiogroup">
					{#each ['compact', 'regular', 'comfy'] as d}
					<button
						class="tweaks-seg-btn"
						class:is-active={tweaks.density === d}
						role="radio"
						aria-checked={tweaks.density === d}
						onclick={() => setTweak('density', d)}
					>{d}</button>
					{/each}
				</div>
			</div>

			<div class="tweaks-sect">Location</div>

			<div class="tweaks-row tweaks-row-stack">
				<span class="tweaks-label">Name</span>
				<input
					class="tweaks-input"
					type="text"
					value={tweaks.locationName}
					onchange={(e) => setTweak('locationName', e.currentTarget.value)}
				/>
			</div>

			<div class="tweaks-row tweaks-row-stack">
				<span class="tweaks-label">Coords</span>
				<input
					class="tweaks-input"
					class:is-err={coordsErr}
					type="text"
					placeholder="lat, lon"
					bind:value={coordsText}
					onchange={() => commitCoords(coordsText)}
				/>
				{#if coordsErr}<span class="tweaks-err">{coordsErr}</span>{/if}
			</div>
		</div>
	</div>
	{/if}

	<button
		class="tweaks-fab"
		class:is-open={open}
		onclick={() => open = !open}
		aria-label="Display settings"
		title="Display settings"
	>
		<svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
			<line x1="2" y1="4" x2="14" y2="4"/>
			<line x1="2" y1="8" x2="14" y2="8"/>
			<line x1="2" y1="12" x2="14" y2="12"/>
			<circle cx="5" cy="4" r="1.5" fill="currentColor" stroke="none"/>
			<circle cx="10" cy="8" r="1.5" fill="currentColor" stroke="none"/>
			<circle cx="6" cy="12" r="1.5" fill="currentColor" stroke="none"/>
		</svg>
	</button>
</div>

<style>
	.tweaks-wrap {
		position: fixed;
		bottom: 24px;
		right: 24px;
		z-index: 200;
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 10px;
	}

	.tweaks-panel {
		width: 240px;
		background: var(--surface-2, #281f18);
		border: 1px solid var(--hairline-strong);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: 0 16px 48px rgba(0,0,0,0.5), 0 2px 8px rgba(0,0,0,0.3);
	}

	.tweaks-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px 10px;
		border-bottom: 1px solid var(--hairline);
	}

	.tweaks-title {
		font-family: 'Geist', ui-sans-serif, sans-serif;
		font-size: 12px;
		font-weight: 600;
		letter-spacing: 0.02em;
		color: var(--ink-2);
	}

	.tweaks-close {
		appearance: none;
		border: none;
		background: transparent;
		color: var(--ink-3);
		width: 22px;
		height: 22px;
		border-radius: 6px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background 0.12s, color 0.12s;
	}

	.tweaks-close:hover {
		background: var(--hairline-strong);
		color: var(--ink);
	}

	.tweaks-body {
		padding: 10px 14px 14px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.tweaks-sect {
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 9.5px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-4);
		padding-top: 4px;
	}

	.tweaks-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
	}

	.tweaks-label {
		font-family: 'Geist', ui-sans-serif, sans-serif;
		font-size: 12px;
		color: var(--ink-2);
		font-weight: 500;
		flex-shrink: 0;
	}

	.tweaks-seg {
		display: flex;
		background: rgba(0,0,0,0.28);
		border-radius: 7px;
		padding: 2px;
		gap: 1px;
	}

	.tweaks-seg-btn {
		appearance: none;
		border: none;
		background: transparent;
		color: var(--ink-3);
		font-family: 'Geist', ui-sans-serif, sans-serif;
		font-size: 11px;
		font-weight: 500;
		padding: 4px 8px;
		border-radius: 5px;
		cursor: pointer;
		transition: background 0.12s, color 0.12s;
		white-space: nowrap;
	}

	.tweaks-seg-btn:hover:not(.is-active) {
		color: var(--ink-2);
	}

	.tweaks-seg-btn.is-active {
		background: var(--surface);
		color: var(--ink);
		box-shadow: 0 1px 3px rgba(0,0,0,0.3);
	}

	.tweaks-row-stack {
		flex-direction: column;
		align-items: stretch;
		gap: 4px;
	}

	.tweaks-input {
		appearance: none;
		background: rgba(0,0,0,0.28);
		border: 1px solid var(--hairline);
		border-radius: 7px;
		color: var(--ink);
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 11.5px;
		padding: 5px 8px;
		width: 100%;
		box-sizing: border-box;
		transition: border-color 0.12s;
	}

	.tweaks-input:focus {
		outline: none;
		border-color: var(--hairline-strong);
	}

	.tweaks-input.is-err {
		border-color: var(--accent-temp);
	}

	.tweaks-err {
		color: var(--accent-temp);
		font-family: 'Geist Mono', ui-monospace, monospace;
		font-size: 10px;
	}

	.tweaks-fab {
		appearance: none;
		border: 1px solid var(--hairline-strong);
		background: var(--surface);
		color: var(--ink-3);
		width: 36px;
		height: 36px;
		border-radius: 50%;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 4px 12px rgba(0,0,0,0.3);
		transition: color 0.15s, background 0.15s, border-color 0.15s;
	}

	.tweaks-fab:hover,
	.tweaks-fab.is-open {
		color: var(--ink);
		background: var(--surface-2);
		border-color: var(--hairline-strong);
	}
</style>
