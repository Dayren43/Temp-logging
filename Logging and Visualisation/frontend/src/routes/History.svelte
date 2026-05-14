<script>
  import { onMount } from 'svelte';
  import { COMFORT } from '$lib/config.js';

  // ── Data ────────────────────────────────────────────────────────────────────
  let raw       = [];   // hourly points [{time, temp, humid}]
  let loading   = true;

  onMount(async () => {
    try {
      const res  = await fetch('http://epsilon.local:3000/data?range=7d&aggregate=1h');
      const json = await res.json();
      raw = (json.data || [])
        .map(d => ({
          time:  new Date(d.time_bucket || d.timestamp),
          temp:  parseFloat(d.temp),
          humid: parseFloat(d.humid),
        }))
        .sort((a, b) => a.time - b.time);
    } catch (e) {
      console.error('History fetch failed', e);
    } finally {
      loading = false;
    }
  });

  // ── Bucket by calendar day, keep only the last 7 full(er) days ──────────────
  $: days = bucketByDay(raw).slice(-7);
  $: cutoff = days.length ? days[0].label : null;
  $: recentRaw = cutoff ? raw.filter(p => p.time >= cutoff) : raw;

  function bucketByDay(data) {
    const map = new Map();
    for (const p of data) {
      const d   = p.time;
      const key = `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
      if (!map.has(key)) map.set(key, { key, label: d, temps: [], hums: [] });
      map.get(key).temps.push(p.temp);
      map.get(key).hums.push(p.humid);
    }
    return [...map.values()].map(d => ({
      key:   d.key,
      label: d.label,
      tMin:  Math.min(...d.temps),
      tMax:  Math.max(...d.temps),
      tAvg:  d.temps.reduce((a, b) => a + b, 0) / d.temps.length,
      hAvg:  d.hums.reduce((a, b)  => a + b, 0) / d.hums.length,
    }));
  }

  // ── Comfort share (for header totals) ────────────────────────────────────────
  $: comfortPct = recentRaw.length
    ? Math.round(recentRaw.filter(p => p.temp >= COMFORT.lo && p.temp <= COMFORT.hi).length / recentRaw.length * 100)
    : 0;

  // ── Day vs night averages (day 08–22, night 22–08) ──────────────────────────
  $: dayNight = computeDayNight(recentRaw);

  function computeDayNight(data) {
    if (!data.length) return null;
    const day = [], night = [];
    for (const p of data) {
      const hr = p.time.getHours();
      (hr >= 8 && hr < 22 ? day : night).push(p.temp);
    }
    if (!day.length || !night.length) return null;
    const avg = arr => arr.reduce((a, b) => a + b, 0) / arr.length;
    const dayAvg = avg(day), nightAvg = avg(night);
    return { dayAvg, nightAvg, delta: dayAvg - nightAvg };
  }

  // ── Driest stretch ───────────────────────────────────────────────────────────
  $: driest = findDriest(recentRaw);

  function findDriest(data) {
    if (!data.length) return { len: 0, start: null, end: null, threshold: 35 };
    const sortedH = data.map(p => p.humid).sort((a, b) => a - b);
    const p25     = sortedH[Math.floor(sortedH.length * 0.25)];
    for (const thr of [35, 40, p25 + 1]) {
      let best = { len: 0, start: null, end: null, threshold: thr };
      let curr = null;
      for (const p of data) {
        if (p.humid < thr) {
          if (!curr) curr = { len: 1, start: p.time, end: p.time };
          else { curr.len += 1; curr.end = p.time; }
          if (curr.len > best.len) best = { ...curr, threshold: thr };
        } else {
          curr = null;
        }
      }
      if (best.len > 0) return best;
    }
    return { len: 0, start: null, end: null, threshold: 35 };
  }

  $: driestHours = driest.len
    ? Math.max(1, Math.round((driest.end - driest.start) / 3600000) + 1)
    : 0;
  $: driestWhen  = driest.start
    ? driest.start.toLocaleDateString(undefined, { weekday: 'short', day: 'numeric' })
    : null;

  // ── Week summary ─────────────────────────────────────────────────────────────
  $: weekAvgTemp  = days.length ? days.reduce((a, d) => a + d.tAvg, 0) / days.length : null;
  $: weekAvgHumid = days.length ? days.reduce((a, d) => a + d.hAvg, 0) / days.length : null;

  // ── Daily highs/lows bar chart ───────────────────────────────────────────────
  $: barTemps = days.flatMap(d => [d.tMin, d.tMax]);
  $: barLo    = barTemps.length ? Math.min(...barTemps) : 0;
  $: barHi    = barTemps.length ? Math.max(...barTemps) : 1;
  $: barSpan  = barHi - barLo || 1;

  function fmtDay(d) {
    return d.label.toLocaleDateString(undefined, { weekday: 'short' });
  }
</script>

{#if !loading && raw.length}
<section class="history">
  <div class="history-head">
    <h2 class="history-title">Last 7 days</h2>
    {#if weekAvgTemp != null}
    <div class="history-totals">
      <div class="history-total">
        <span class="history-total-label">Week avg</span>
        <span class="history-total-value temp">{weekAvgTemp.toFixed(1)}°</span>
      </div>
      <div class="history-total">
        <span class="history-total-label">Avg humidity</span>
        <span class="history-total-value humid">{Math.round(weekAvgHumid)}%</span>
      </div>
      <div class="history-total">
        <span class="history-total-label">In comfort</span>
        <span class="history-total-value">{comfortPct}%</span>
      </div>
    </div>
    {/if}
  </div>

  <div class="history-grid">

    <!-- 1. Daily highs & lows -->
    <div class="hbox">
      <div class="hbox-label">Daily highs &amp; lows</div>
      <div class="hbox-bars">
        {#each days as d}
          {@const top = ((barHi - d.tMax) / barSpan) * 100}
          {@const bot = ((barHi - d.tMin) / barSpan) * 100}
          <div class="hbox-bar-col">
            <div class="hbox-bar-cap">{d.tMax.toFixed(0)}°</div>
            <div class="hbox-bar-track">
              <div class="hbox-bar" style:top="{top}%" style:height="calc({bot - top}% + 4px)"></div>
            </div>
            <div class="hbox-bar-floor">{d.tMin.toFixed(0)}°</div>
            <div class="hbox-bar-day">{fmtDay(d)}</div>
          </div>
        {/each}
      </div>
    </div>

    <!-- 2. Day vs night -->
    <div class="hbox">
      <div class="hbox-label">Day vs night</div>
      {#if dayNight}
        <div class="hbox-dn">
          <div class="dn-pair">
            <div class="dn-pair-label">Day</div>
            <div class="dn-pair-value" style:color="var(--accent-temp)">{dayNight.dayAvg.toFixed(1)}°</div>
            <div class="dn-pair-sub">08–22</div>
          </div>
          <div class="dn-arrow" aria-hidden="true">{dayNight.delta >= 0 ? '↓' : '↑'}</div>
          <div class="dn-pair">
            <div class="dn-pair-label">Night</div>
            <div class="dn-pair-value" style:color="var(--accent-feels)">{dayNight.nightAvg.toFixed(1)}°</div>
            <div class="dn-pair-sub">22–08</div>
          </div>
        </div>
        <div class="hbox-foot">
          <span class="hbox-line">
            {Math.abs(dayNight.delta) < 0.3
              ? 'Steady overnight'
              : dayNight.delta > 0
                ? `Cools ${dayNight.delta.toFixed(1)}° overnight`
                : `Warms ${Math.abs(dayNight.delta).toFixed(1)}° overnight`}
          </span>
        </div>
      {:else}
        <div class="hbox-empty">
          <div class="hbox-empty-icon" aria-hidden="true">~</div>
          <div class="hbox-empty-line">Not enough data</div>
        </div>
      {/if}
    </div>

    <!-- 3. Driest stretch -->
    <div class="hbox">
      <div class="hbox-label">Driest stretch</div>
      {#if driestHours > 0}
        <div class="hbox-hero">
          <span class="hbox-num" style:color="var(--accent-humid)">{driestHours}</span>
          <span class="hbox-unit">hr</span>
        </div>
        <div class="hbox-foot">
          <span class="hbox-line">below {driest.threshold}% humidity</span>
          {#if driestWhen}
            <span class="hbox-sub">starting {driestWhen}</span>
          {/if}
        </div>
      {:else}
        <div class="hbox-empty">
          <div class="hbox-empty-icon" aria-hidden="true">~</div>
          <div class="hbox-empty-line">Steady humidity</div>
          <div class="hbox-empty-sub">No notably dry stretch this week</div>
        </div>
      {/if}
    </div>

  </div>
</section>
{/if}

<style>
  .history {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 8px 4px 0;
  }

  .history-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 24px;
    flex-wrap: wrap;
    padding: 0 2px;
  }

  .history-title {
    margin: 0;
    font-family: 'Instrument Serif', serif;
    font-style: italic;
    font-size: 28px;
    font-weight: 400;
    color: var(--ink);
    letter-spacing: -0.005em;
    line-height: 1.1;
  }

  .history-totals {
    display: flex;
    gap: 28px;
    align-items: flex-end;
    flex-shrink: 0;
  }

  .history-total {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }

  .history-total-label {
    font-family: 'Geist Mono', ui-monospace, monospace;
    font-size: 10px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-4);
  }

  .history-total-value {
    font-family: 'Instrument Serif', serif;
    font-size: 22px;
    line-height: 1;
    color: var(--ink-2);
    font-variant-numeric: tabular-nums;
  }

  .history-total-value.temp  { color: var(--accent-temp); }
  .history-total-value.humid { color: var(--accent-humid); }

  .history-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 14px;
  }

  @media (max-width: 1080px) { .history-grid { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 680px)  { .history-grid { grid-template-columns: 1fr; } }


  /* ── Box base ─────────────────────────────────────────────────────────────── */
  .hbox {
    background: var(--surface);
    border: 1px solid var(--hairline);
    border-radius: 16px;
    padding: 18px 18px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 1px 0 rgba(255,255,255,0.02) inset;
  }

  .hbox-label {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-4);
    font-weight: 500;
  }

  /* ── Hero number ─────────────────────────────────────────────────────────── */
  .hbox-hero {
    display: flex;
    align-items: baseline;
    gap: 4px;
    font-family: 'Instrument Serif', serif;
    line-height: 0.9;
    font-variant-numeric: tabular-nums;
  }

  .hbox-num  { font-size: 56px; letter-spacing: -0.02em; }
  .hbox-unit { font-size: 22px; color: var(--ink-3); font-style: italic; }

  /* ── Footer area ─────────────────────────────────────────────────────────── */
  .hbox-foot {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .hbox-line { font-size: 13px; color: var(--ink-2); font-family: 'Instrument Serif', serif; font-style: italic; }
  .hbox-sub  { font-size: 11.5px; color: var(--ink-4); font-family: 'Geist Mono', ui-monospace, monospace; }

  /* ── Bar chart ───────────────────────────────────────────────────────────── */
  .hbox-bars {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 6px;
    flex: 1;
    min-height: 110px;
    padding-top: 4px;
  }

  .hbox-bar-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
  }

  .hbox-bar-track {
    position: relative;
    width: 100%;
    flex: 1;
    min-height: 60px;
  }

  .hbox-bar {
    position: absolute;
    left: 25%; right: 25%;
    border-radius: 999px;
    background: linear-gradient(180deg, var(--accent-temp) 0%, var(--accent-feels) 100%);
    opacity: 0.85;
  }

  .hbox-bar-cap {
    font-size: 10px;
    font-family: 'Geist Mono', ui-monospace, monospace;
    font-variant-numeric: tabular-nums;
    color: var(--accent-temp);
    line-height: 1;
  }

  .hbox-bar-floor {
    font-size: 10px;
    font-family: 'Geist Mono', ui-monospace, monospace;
    font-variant-numeric: tabular-nums;
    color: var(--ink-3);
    padding-top: 2px;
  }

  .hbox-bar-day {
    font-size: 9.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ink-4);
    font-family: 'Geist Mono', ui-monospace, monospace;
    padding-top: 3px;
    border-top: 1px solid var(--hairline);
    width: 100%;
    text-align: center;
  }

  /* ── Day vs night ────────────────────────────────────────────────────────── */
  .hbox-dn {
    display: flex;
    align-items: center;
    justify-content: space-around;
    gap: 12px;
    padding-top: 4px;
  }

  .dn-pair {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .dn-pair-label {
    font-family: 'Geist Mono', ui-monospace, monospace;
    font-size: 10px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-4);
  }

  .dn-pair-value {
    font-family: 'Instrument Serif', serif;
    font-size: 32px;
    line-height: 1;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.01em;
  }

  .dn-pair-sub {
    font-family: 'Geist Mono', ui-monospace, monospace;
    font-size: 9.5px;
    color: var(--ink-4);
    font-variant-numeric: tabular-nums;
  }

  .dn-arrow {
    font-family: 'Instrument Serif', serif;
    font-size: 24px;
    color: var(--ink-3);
    line-height: 1;
  }

  /* ── Empty state ─────────────────────────────────────────────────────────── */
  .hbox-empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: center;
    flex: 1;
    gap: 6px;
    padding: 4px 0;
  }

  .hbox-empty-icon {
    font-family: 'Instrument Serif', serif;
    font-style: italic;
    font-size: 56px;
    line-height: 0.8;
    color: var(--ink-4);
    letter-spacing: -0.04em;
  }

  .hbox-empty-line { font-family: 'Instrument Serif', serif; font-style: italic; font-size: 18px; color: var(--ink-2); }
  .hbox-empty-sub  { font-size: 12px; color: var(--ink-4); }
</style>
