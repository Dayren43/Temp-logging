<script>
  import { onMount } from 'svelte';
  import { tweaks } from '$lib/tweaks.svelte.js';
  import { COMFORT } from '$lib/config.js';
  import { apparentTemp } from '$lib/comfort.js';

  // ── Layout ─────────────────────────────────────────────────────────────────
  const PAD   = { l: 44, r: 44, t: 20, b: 32 };
  const TT_W  = 168;

  const RANGES = [
    { key: '6h',  label: '6h',  dateTick: false },
    { key: '24h', label: '24h', dateTick: false },
    { key: '3d',  label: '3d',  dateTick: true  },
    { key: '7d',  label: '7d',  dateTick: true  },
    { key: 'all', label: 'all', dateTick: true  },
  ];

  // ── Series / chart options ──────────────────────────────────────────────────
  let showTemp    = true;
  let showFeels   = true;
  let showHumid   = true;
  let showComfort = true;
  let showNight   = true;
  let timeRange   = '24h';
  let dateTick    = false;

  // ── Data ────────────────────────────────────────────────────────────────────
  let fullData  = [];
  let chartData = [];
  let sampled   = [];
  let isLoading = false;
  let error     = null;

  // ── Fluid size (ResizeObserver) ─────────────────────────────────────────────
  let wrapEl = null;
  let W = 800, H = 320;

  // ── Hover crosshair ─────────────────────────────────────────────────────────
  let hover = null; // { idx, cx }

  // ── Derived geometry ────────────────────────────────────────────────────────
  $: iW = W - PAD.l - PAD.r;
  $: iH = H - PAD.t - PAD.b;

  // Dynamic Y-domain: snap to multiples of `step`, always include the must-have values.
  function computeRange(values, step, mustInclude = []) {
    const all = [...values.filter(v => Number.isFinite(v)), ...mustInclude];
    if (!all.length) return { min: 0, max: step * 5 };
    let lo = Math.min(...all);
    let hi = Math.max(...all);
    lo = Math.floor((lo - 1) / step) * step;
    hi = Math.ceil((hi + 1) / step) * step;
    if (hi - lo < step * 2) hi = lo + step * 2;
    return { min: lo, max: hi };
  }

  function makeTicks(min, max, step) {
    const out = [];
    for (let v = min; v <= max + 1e-6; v += step) out.push(v);
    return out;
  }

  // Temp range covers both temp + feels series and always shows the comfort band.
  $: tempRange  = computeRange(sampled.flatMap(d => [d.temp, d.feels]), 2, [COMFORT.lo, COMFORT.hi]);
  $: humidRange = computeRange(sampled.map(d => d.humid), 10);
  $: TEMP_TICKS = makeTicks(tempRange.min, tempRange.max, 2);
  $: HUM_TICKS  = makeTicks(humidRange.min, humidRange.max, 10);

  $: yT  = (v) => PAD.t + iH - ((v - tempRange.min)  / (tempRange.max  - tempRange.min))  * iH;
  $: yH  = (v) => PAD.t + iH - ((v - humidRange.min) / (humidRange.max - humidRange.min)) * iH;
  $: xAt = (i) => PAD.l + (i / Math.max(1, sampled.length - 1)) * iW;

  // ── Sampled data (max 2000 pts) with feels-like ─────────────────────────────
  $: {
    if (chartData.length) {
      const step = Math.max(1, Math.floor(chartData.length / 2000));
      sampled = chartData
        .filter((_, i) => i % step === 0)
        .map(d => ({ ...d, feels: apparentTemp(d.temp, d.humid) }));
    } else {
      sampled = [];
    }
  }

  // ── Path builders (explicit args so $: declarations track all dependencies) ─
  function buildLine(key, data, xFn, scaleFn) {
    if (!data.length) return '';
    return data.map((p, i) =>
      `${i === 0 ? 'M' : 'L'} ${xFn(i).toFixed(1)} ${scaleFn(p[key]).toFixed(1)}`
    ).join(' ');
  }

  function buildArea(key, data, xFn, scaleFn, baseY) {
    if (!data.length) return '';
    const top = buildLine(key, data, xFn, scaleFn);
    const base = baseY.toFixed(1);
    return `${top} L ${xFn(data.length - 1).toFixed(1)} ${base} L ${PAD.l.toFixed(1)} ${base} Z`;
  }

  // Precomputed paths — sampled + xAt are explicit deps so Svelte re-runs on data change
  $: baseY        = PAD.t + iH;
  $: tempLine     = buildLine('temp',  sampled, xAt, yT);
  $: feelsLine    = buildLine('feels', sampled, xAt, yT);
  $: humidLine    = buildLine('humid', sampled, xAt, yH);
  $: tempArea     = buildArea('temp',  sampled, xAt, yT, baseY);
  $: feelsArea    = buildArea('feels', sampled, xAt, yT, baseY);
  $: humidArea    = buildArea('humid', sampled, xAt, yH, baseY);

  // ── Night bands ──────────────────────────────────────────────────────────────
  $: nightBands = computeNightBands(sampled);
  function computeNightBands(data) {
    if (!data.length) return [];
    const out = [];
    let curr = null;
    for (let i = 0; i < data.length; i++) {
      const hr = data[i].time.getHours();
      const night = hr < 6 || hr >= 20;
      if (!curr || curr.night !== night) {
        if (curr) out.push(curr);
        curr = { night, start: i, end: i };
      } else {
        curr.end = i;
      }
    }
    if (curr) out.push(curr);
    return out.filter(b => b.night);
  }

  // ── X-axis ticks ─────────────────────────────────────────────────────────────
  $: xTicks = sampled.length
    ? Array.from({ length: 6 }, (_, i) => {
        const idx = Math.round((i / 5) * (sampled.length - 1));
        return { idx, time: sampled[idx].time };
      })
    : [];

  function fmtTick(t) {
    if (!t) return '';
    if (dateTick) return t.toLocaleDateString(undefined, { weekday: 'short', day: 'numeric' });
    return t.toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit' });
  }

  // ── Stats (foot row) ────────────────────────────────────────────────────────
  $: stats = chartData.length ? {
    tMin: Math.min(...chartData.map(d => d.temp)),
    tMax: Math.max(...chartData.map(d => d.temp)),
    hAvg: chartData.reduce((s, d) => s + d.humid, 0) / chartData.length,
  } : null;

  // ── Latest values for series pills ──────────────────────────────────────────
  $: latestTemp  = sampled.length ? sampled[sampled.length - 1].temp  : null;
  $: latestFeels = sampled.length ? sampled[sampled.length - 1].feels : null;
  $: latestHumid = sampled.length ? sampled[sampled.length - 1].humid : null;

  // ── Tooltip position + dynamic height ──────────────────────────────────────
  $: ttH = 28 + [showTemp, showFeels, showHumid].filter(Boolean).length * 20 + 8;
  $: ttX = hover
    ? (xAt(hover.idx) + 14 + TT_W > W - 8 ? xAt(hover.idx) - TT_W - 14 : xAt(hover.idx) + 14)
    : 0;
  $: ttY = PAD.t + 10;
  $: hp  = hover ? sampled[hover.idx] : null;

  // ── Mouse interaction ───────────────────────────────────────────────────────
  function onMouseMove(e) {
    if (!wrapEl || !sampled.length) return;
    const rect = wrapEl.getBoundingClientRect();
    const x = e.clientX - rect.left;
    if (x < PAD.l || x > PAD.l + iW) { hover = null; return; }
    const idx = Math.max(0, Math.min(sampled.length - 1,
      Math.round(((x - PAD.l) / iW) * (sampled.length - 1))));
    hover = { idx };
  }

  // ── Data fetching ───────────────────────────────────────────────────────────
  async function fetchData() {
    isLoading = true;
    error = null;
    try {
      const q = new URLSearchParams({ range: timeRange });
      if (['3d', '7d', 'all'].includes(timeRange)) q.set('aggregate', '15m');
      else if (timeRange === '24h') q.set('aggregate', '5m');
      const res = await fetch(`http://epsilon.local:3000/data?${q}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const json = await res.json();
      fullData = (json.data || [])
        .map(d => ({
          time: new Date(d.time_bucket || d.timestamp),
          temp:  parseFloat(d.temp),
          humid: parseFloat(d.humid),
        }))
        .sort((a, b) => a.time - b.time);
    } catch (e) {
      error = e.message;
    } finally {
      isLoading = false;
    }
  }

  function parseDuration(str) {
    const re = /(\d+)([dhm])/g;
    let ms = 0, m;
    while ((m = re.exec(str))) {
      const v = parseInt(m[1]);
      if (m[2] === 'd') ms += v * 86400000;
      if (m[2] === 'h') ms += v * 3600000;
      if (m[2] === 'm') ms += v * 60000;
    }
    return ms;
  }

  $: if (timeRange) fetchData();

  $: {
    if (fullData.length) {
      chartData = timeRange === 'all'
        ? fullData
        : fullData.filter(d => Date.now() - d.time.getTime() <= parseDuration(timeRange));
    }
  }

  function setRange(r) { timeRange = r.key; dateTick = r.dateTick; }

  // ── Lifecycle ───────────────────────────────────────────────────────────────
  onMount(() => {
    fetchData();
    const ro = new ResizeObserver(entries => {
      W = Math.max(320, entries[0].contentRect.width);
    });
    ro.observe(wrapEl);
    return () => ro.disconnect();
  });
</script>

<div class="chart-card">

  <!-- ── Head ─────────────────────────────────────────────────────────────── -->
  <div class="chart-head">
    <div class="ranges" role="tablist" aria-label="Time range">
      {#each RANGES as r}
        <button type="button" role="tab" aria-selected={timeRange === r.key}
          class="range" class:is-on={timeRange === r.key}
          on:click={() => setRange(r)}>{r.label}</button>
      {/each}
    </div>

    <div class="series">
      <button type="button" class="series-pill" class:is-on={showTemp}
        aria-pressed={showTemp} on:click={() => showTemp = !showTemp}>
        <span class="pill-swatch" style:background={showTemp ? 'var(--accent-temp)' : 'transparent'} style:border-color="var(--accent-temp)"></span>
        <span class="pill-label">Temp</span>
        <span class="pill-value" style:color={showTemp ? 'var(--accent-temp)' : 'var(--ink-4)'}>
          {latestTemp != null ? `${latestTemp.toFixed(1)}°` : '—'}
        </span>
      </button>

      <button type="button" class="series-pill" class:is-on={showFeels}
        aria-pressed={showFeels} on:click={() => showFeels = !showFeels}>
        <span class="pill-swatch is-dashed" style:border-color="var(--accent-feels)"
          style:background={showFeels ? 'var(--accent-feels)' : 'transparent'}></span>
        <span class="pill-label">Feels</span>
        <span class="pill-value" style:color={showFeels ? 'var(--accent-feels)' : 'var(--ink-4)'}>
          {latestFeels != null ? `${latestFeels.toFixed(1)}°` : '—'}
        </span>
      </button>

      <button type="button" class="series-pill" class:is-on={showHumid}
        aria-pressed={showHumid} on:click={() => showHumid = !showHumid}>
        <span class="pill-swatch" style:background={showHumid ? 'var(--accent-humid)' : 'transparent'} style:border-color="var(--accent-humid)"></span>
        <span class="pill-label">Humidity</span>
        <span class="pill-value" style:color={showHumid ? 'var(--accent-humid)' : 'var(--ink-4)'}>
          {latestHumid != null ? `${latestHumid.toFixed(0)}%` : '—'}
        </span>
      </button>
    </div>
  </div>

  <!-- ── Chart body ────────────────────────────────────────────────────────── -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="chart-wrap" bind:this={wrapEl}
    on:mousemove={onMouseMove}
    on:mouseleave={() => hover = null}>

    {#if isLoading && !sampled.length}
      <div class="chart-status">Loading…</div>
    {:else if error}
      <div class="chart-status error">Error: {error}</div>
    {:else if sampled.length}
      <svg width={W} height={H} aria-label="Temperature and humidity over time">
        <defs>
          <linearGradient id="grad-temp" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%"   stop-color="var(--accent-temp)"  stop-opacity="0.38"/>
            <stop offset="100%" stop-color="var(--accent-temp)"  stop-opacity="0"/>
          </linearGradient>
          <linearGradient id="grad-feels" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%"   stop-color="var(--accent-feels)" stop-opacity="0.28"/>
            <stop offset="100%" stop-color="var(--accent-feels)" stop-opacity="0"/>
          </linearGradient>
          <linearGradient id="grad-humid" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%"   stop-color="var(--accent-humid)" stop-opacity="0.28"/>
            <stop offset="100%" stop-color="var(--accent-humid)" stop-opacity="0"/>
          </linearGradient>
          <clipPath id="clip-plot">
            <rect x={PAD.l} y={PAD.t} width={iW} height={iH}/>
          </clipPath>
        </defs>

        <!-- Night shading -->
        {#if showNight}
          <g clip-path="url(#clip-plot)">
            {#each nightBands as b}
              <rect
                x={xAt(b.start)} y={PAD.t}
                width={Math.max(1, xAt(b.end) - xAt(b.start))}
                height={iH}
                fill="rgba(60,76,110,0.18)"
              />
            {/each}
          </g>
        {/if}

        <!-- Comfort band -->
        {#if showComfort && showTemp}
          <g clip-path="url(#clip-plot)">
            <rect x={PAD.l} y={yT(COMFORT.hi)} width={iW} height={yT(COMFORT.lo) - yT(COMFORT.hi)}
              fill="var(--accent-comfort)" fill-opacity="0.08"/>
            <line x1={PAD.l} x2={PAD.l + iW} y1={yT(COMFORT.hi)} y2={yT(COMFORT.hi)}
              stroke="var(--accent-comfort)" stroke-opacity="0.35" stroke-dasharray="2 4"/>
            <line x1={PAD.l} x2={PAD.l + iW} y1={yT(COMFORT.lo)} y2={yT(COMFORT.lo)}
              stroke="var(--accent-comfort)" stroke-opacity="0.35" stroke-dasharray="2 4"/>
          </g>
        {/if}

        <!-- Y grid + labels (temp left) -->
        {#each TEMP_TICKS as v}
          <line x1={PAD.l} x2={PAD.l + iW} y1={yT(v)} y2={yT(v)}
            stroke="rgba(255,255,255,0.04)" stroke-width="1"/>
          <text x={PAD.l - 8} y={yT(v) + 4} text-anchor="end"
            font-size="10.5" fill="rgba(245,184,164,0.5)"
            font-family="'Geist Mono',ui-monospace,monospace">{v}°</text>
        {/each}

        <!-- Y labels (humid right) -->
        {#if showHumid}
          {#each HUM_TICKS as v}
            <text x={PAD.l + iW + 8} y={yH(v) + 4} text-anchor="start"
              font-size="10.5" fill="rgba(168,200,232,0.5)"
              font-family="'Geist Mono',ui-monospace,monospace">{v}%</text>
          {/each}
        {/if}

        <!-- X axis ticks -->
        {#each xTicks as tk}
          <text x={xAt(tk.idx)} y={PAD.t + iH + 20} text-anchor="middle"
            font-size="10.5" fill="rgba(245,235,224,0.35)"
            font-family="'Geist Mono',ui-monospace,monospace">{fmtTick(tk.time)}</text>
        {/each}

        <!-- Series: areas then lines (order: humid, feels, temp — temp on top) -->
        <g clip-path="url(#clip-plot)">
          {#if tweaks.chartStyle === 'area'}
            {#if showHumid}
              <path d={humidArea} fill="url(#grad-humid)"/>
            {/if}
            {#if showFeels}
              <path d={feelsArea} fill="url(#grad-feels)"/>
            {/if}
            {#if showTemp}
              <path d={tempArea} fill="url(#grad-temp)"/>
            {/if}
          {/if}

          {#if showHumid}
            <path d={humidLine} fill="none"
              stroke="var(--accent-humid)" stroke-width="1.8"
              stroke-linecap="round" stroke-linejoin="round"/>
          {/if}
          {#if showFeels}
            <path d={feelsLine} fill="none"
              stroke="var(--accent-feels)" stroke-width="1.6"
              stroke-dasharray="2 5"
              stroke-linecap="round" stroke-linejoin="round"/>
          {/if}
          {#if showTemp}
            <path d={tempLine} fill="none"
              stroke="var(--accent-temp)" stroke-width="2"
              stroke-linecap="round" stroke-linejoin="round"/>
          {/if}
        </g>

        <!-- Crosshair -->
        {#if hover && hp}
          <line x1={xAt(hover.idx)} x2={xAt(hover.idx)} y1={PAD.t} y2={PAD.t + iH}
            stroke="rgba(245,235,224,0.18)" stroke-width="1"/>

          {#if showTemp}
            <circle cx={xAt(hover.idx)} cy={yT(hp.temp)} r="4.5"
              fill="var(--accent-temp)" stroke="var(--bg)" stroke-width="2"/>
          {/if}
          {#if showFeels}
            <circle cx={xAt(hover.idx)} cy={yT(hp.feels)} r="3.5"
              fill="var(--accent-feels)" stroke="var(--bg)" stroke-width="2"/>
          {/if}
          {#if showHumid}
            <circle cx={xAt(hover.idx)} cy={yH(hp.humid)} r="4"
              fill="var(--accent-humid)" stroke="var(--bg)" stroke-width="2"/>
          {/if}

          <!-- Tooltip (inline SVG, clamped to edges) -->
          <g transform="translate({ttX},{ttY})">
            <rect width={TT_W} height={ttH} rx="10"
              fill="rgba(28,24,20,0.94)" stroke="rgba(245,235,224,0.12)" stroke-width="1"/>
            <text x="12" y="18" font-size="10.5" letter-spacing="0.04em"
              fill="rgba(245,235,224,0.45)"
              font-family="'Geist Mono',ui-monospace,monospace">
              {hp.time.toLocaleString(undefined, { weekday:'short', hour:'2-digit', minute:'2-digit', hour12:false })}
            </text>

            {#if showTemp}
              <g transform="translate(12,34)">
                <circle cx="4" cy="-3" r="3.5" fill="var(--accent-temp)"/>
                <text x="14" y="0" font-size="11" fill="rgba(245,235,224,0.6)"
                  font-family="'Geist',ui-sans-serif,sans-serif">Temp</text>
                <text x={TT_W - 22} y="0" text-anchor="end" font-size="13"
                  fill="var(--ink)" font-family="'Instrument Serif',serif"
                  font-variant-numeric="tabular-nums">{hp.temp.toFixed(1)}°</text>
              </g>
            {/if}
            {#if showFeels}
              <g transform="translate(12,{showTemp ? 54 : 34})">
                <circle cx="4" cy="-3" r="3.5" fill="var(--accent-feels)"/>
                <text x="14" y="0" font-size="11" fill="rgba(245,235,224,0.6)"
                  font-family="'Geist',ui-sans-serif,sans-serif">Feels</text>
                <text x={TT_W - 22} y="0" text-anchor="end" font-size="13"
                  fill="var(--ink)" font-family="'Instrument Serif',serif"
                  font-variant-numeric="tabular-nums">{hp.feels.toFixed(1)}°</text>
              </g>
            {/if}
            {#if showHumid}
              <g transform="translate(12,{(showTemp && showFeels) ? 74 : (showTemp || showFeels) ? 54 : 34})">
                <circle cx="4" cy="-3" r="3.5" fill="var(--accent-humid)"/>
                <text x="14" y="0" font-size="11" fill="rgba(245,235,224,0.6)"
                  font-family="'Geist',ui-sans-serif,sans-serif">Humidity</text>
                <text x={TT_W - 22} y="0" text-anchor="end" font-size="13"
                  fill="var(--ink)" font-family="'Instrument Serif',serif"
                  font-variant-numeric="tabular-nums">{hp.humid.toFixed(0)}%</text>
              </g>
            {/if}
          </g>
        {/if}
      </svg>
    {/if}
  </div>

  <!-- ── Foot ──────────────────────────────────────────────────────────────── -->
  <div class="chart-foot">
    <div class="legend">
      {#if showComfort}
        <button type="button" class="legend-item" on:click={() => showComfort = !showComfort}>
          <span class="legend-band" style:border-color="var(--accent-comfort)"></span>
          Comfort {COMFORT.lo}–{COMFORT.hi}°
        </button>
      {:else}
        <button type="button" class="legend-item off" on:click={() => showComfort = !showComfort}>
          Comfort band
        </button>
      {/if}
      {#if showNight}
        <button type="button" class="legend-item" on:click={() => showNight = !showNight}>
          <span class="legend-night"></span>
          Overnight
        </button>
      {:else}
        <button type="button" class="legend-item off" on:click={() => showNight = !showNight}>
          Overnight
        </button>
      {/if}
    </div>

    {#if stats}
      <div class="rangestats">
        <span>min <b style:color="var(--accent-temp)">{stats.tMin.toFixed(1)}°</b></span>
        <span>max <b style:color="var(--accent-temp)">{stats.tMax.toFixed(1)}°</b></span>
        <span>avg humidity <b style:color="var(--accent-humid)">{Math.round(stats.hAvg)}%</b></span>
      </div>
    {/if}
  </div>

</div>

<style>
  /* ── Card ───────────────────────────────────────────────────────────────── */
  .chart-card {
    background: linear-gradient(180deg, var(--surface) 0%, var(--surface-2) 100%);
    border: 1px solid var(--hairline);
    border-radius: 18px;
    padding: 20px 18px 14px;
    box-shadow:
      0 1px 0 rgba(255,255,255,0.025) inset,
      0 24px 40px -24px rgba(0,0,0,0.5);
  }

  /* ── Head ───────────────────────────────────────────────────────────────── */
  .chart-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
    padding: 0 6px 14px;
  }

  .ranges {
    display: inline-flex;
    gap: 2px;
    padding: 3px;
    border-radius: 10px;
    background: rgba(0,0,0,0.28);
    border: 1px solid var(--hairline);
  }

  .range {
    appearance: none;
    background: transparent;
    border: 0;
    padding: 6px 12px;
    border-radius: 7px;
    color: var(--ink-3);
    font-family: 'Geist Mono', ui-monospace, monospace;
    font-size: 12px;
    letter-spacing: 0.02em;
    cursor: pointer;
    transition: color 0.15s, background 0.15s;
  }
  .range:hover { color: var(--ink); }
  .range.is-on {
    background: var(--surface);
    color: var(--ink);
    box-shadow: 0 1px 0 rgba(255,255,255,0.04) inset, 0 1px 4px rgba(0,0,0,0.25);
  }

  .series { display: flex; gap: 6px; flex-wrap: wrap; }

  .series-pill {
    appearance: none;
    background: transparent;
    border: 1px solid var(--hairline);
    border-radius: 999px;
    padding: 6px 12px 6px 10px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--ink-3);
    cursor: pointer;
    font: inherit;
    font-size: 12px;
    transition: opacity 0.15s, border-color 0.15s, background 0.15s;
    opacity: 0.5;
  }
  .series-pill.is-on {
    opacity: 1;
    background: rgba(0,0,0,0.18);
    border-color: var(--hairline-strong);
    color: var(--ink);
  }

  .pill-swatch {
    width: 10px; height: 10px;
    border-radius: 999px;
    border: 1.5px solid;
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .pill-label { letter-spacing: 0.01em; }
  .pill-value {
    font-family: 'Geist Mono', ui-monospace, monospace;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
    font-size: 11.5px;
    transition: color 0.15s;
  }

  /* ── Chart body ─────────────────────────────────────────────────────────── */
  .chart-wrap {
    width: 100%;
    position: relative;
    cursor: crosshair;
    user-select: none;
  }
  .chart-wrap svg { display: block; }

  .chart-status {
    padding: 60px 40px;
    text-align: center;
    font-size: 13px;
    color: var(--ink-3);
    font-family: 'Geist Mono', ui-monospace, monospace;
  }
  .chart-status.error { color: var(--accent-temp); }

  /* ── Foot ───────────────────────────────────────────────────────────────── */
  .chart-foot {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 10px 12px 4px;
    flex-wrap: wrap;
  }

  .legend {
    display: flex;
    gap: 14px;
    font-size: 11.5px;
    color: var(--ink-3);
    align-items: center;
  }

  .legend-item {
    appearance: none;
    background: none;
    border: none;
    padding: 0;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font: inherit;
    font-size: 11.5px;
    color: var(--ink-3);
    cursor: pointer;
    transition: color 0.15s;
  }
  .legend-item:hover { color: var(--ink); }
  .legend-item.off { color: var(--ink-4); }

  .legend-band {
    width: 22px; height: 10px;
    border-radius: 3px;
    border: 1px dashed;
    background: rgba(155,201,168,0.06);
    flex-shrink: 0;
  }
  .legend-night {
    width: 22px; height: 10px;
    border-radius: 3px;
    background: linear-gradient(90deg, rgba(60,76,110,0.42) 0%, rgba(60,76,110,0.25) 100%);
    flex-shrink: 0;
  }

  .rangestats {
    display: flex;
    gap: 18px;
    font-size: 12px;
    color: var(--ink-3);
    font-family: 'Geist Mono', ui-monospace, monospace;
  }
  .rangestats b {
    font-family: 'Geist', ui-sans-serif, system-ui, sans-serif;
    font-weight: 600;
    margin-left: 4px;
  }
</style>
