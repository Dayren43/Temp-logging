<script>
  import { onMount } from 'svelte';

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

  // ── Bucket by calendar day ──────────────────────────────────────────────────
  $: days = bucketByDay(raw);

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

  // ── Hourly averages (0–23) across all days ──────────────────────────────────
  $: hourly = computeHourly(raw);

  function computeHourly(data) {
    const buckets = Array.from({ length: 24 }, () => ({ ts: [], hs: [] }));
    for (const p of data) buckets[p.time.getHours()].ts.push(p.temp);
    return buckets.map((b, hr) => ({
      hr,
      t: b.ts.length ? b.ts.reduce((a, x) => a + x, 0) / b.ts.length : null,
    }));
  }

  $: warmest = hourly.reduce((acc, b) => b.t != null && (acc == null || b.t > acc.t) ? b : acc, null);
  $: coolest = hourly.reduce((acc, b) => b.t != null && (acc == null || b.t < acc.t) ? b : acc, null);

  // ── Comfort share ────────────────────────────────────────────────────────────
  $: comfortShare = raw.length
    ? raw.filter(p => p.temp >= 20 && p.temp <= 24).length / raw.length
    : 0;
  $: comfortPct = Math.round(comfortShare * 100);
  const RING_R  = 28;
  const RING_C  = 2 * Math.PI * RING_R;
  $: ringOffset = RING_C * (1 - comfortShare);

  // ── Today vs yesterday ──────────────────────────────────────────────────────
  $: today     = days[days.length - 1]     ?? null;
  $: yesterday = days[days.length - 2]     ?? null;
  $: dayDelta  = today && yesterday ? today.tAvg - yesterday.tAvg : 0;
  $: deltaWord = Math.abs(dayDelta) < 0.2  ? 'about the same'
               : dayDelta > 0              ? 'warmer'
               :                             'cooler';

  // ── Daily rhythm curve ───────────────────────────────────────────────────────
  const RH_W = 240, RH_H = 60;
  $: rhythmPath = buildRhythmPath(hourly);

  function buildRhythmPath(hrs) {
    const ts    = hrs.map(h => h.t).filter(v => v != null);
    if (!ts.length) return '';
    const lo    = Math.min(...ts), hi = Math.max(...ts), span = hi - lo || 1;
    const pts   = hrs.map((h, i) => {
      const x = (i / 23) * RH_W;
      const y = h.t != null ? RH_H - ((h.t - lo) / span) * (RH_H - 8) - 4 : RH_H;
      return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
    });
    return pts.join(' ');
  }

  // ── Driest stretch ───────────────────────────────────────────────────────────
  $: driest = findDriest(raw);

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
  $: weekMaxSwing = days.length ? Math.max(...days.map(d => d.tMax - d.tMin)) : null;

  // ── Daily highs/lows bar chart ───────────────────────────────────────────────
  $: barDays  = days.slice(-7);
  $: barTemps = barDays.flatMap(d => [d.tMin, d.tMax]);
  $: barLo    = barTemps.length ? Math.min(...barTemps) : 0;
  $: barHi    = barTemps.length ? Math.max(...barTemps) : 1;
  $: barSpan  = barHi - barLo || 1;

  function fmtDay(d) {
    return d.label.toLocaleDateString(undefined, { weekday: 'short' });
  }
  function fmtHour(hr) {
    return hr == null ? '—' : `${String(hr).padStart(2, '0')}:00`;
  }
</script>

{#if !loading && raw.length}
<section class="history">
  <div class="history-head">
    <h2 class="history-title">Last 7 days</h2>
    <div class="history-sub">{days.length} days · {raw.length} readings</div>
  </div>

  <div class="history-grid">

    <!-- 1. Today vs yesterday -->
    <div class="hbox">
      <div class="hbox-label">Today vs yesterday</div>
      <div class="hbox-hero">
        <span class="hbox-num" style:color="var(--accent-temp)">
          {dayDelta >= 0 ? '+' : ''}{dayDelta.toFixed(1)}
        </span>
        <span class="hbox-unit">°C</span>
      </div>
      <div class="hbox-foot">
        <span class="hbox-line">{deltaWord} on average</span>
        <div class="hbox-mini">
          <div class="hbox-mini-row">
            <span>Yesterday</span>
            <b>{yesterday ? yesterday.tAvg.toFixed(1) + '°' : '—'}</b>
          </div>
          <div class="hbox-mini-row">
            <span>Today</span>
            <b>{today ? today.tAvg.toFixed(1) + '°' : '—'}</b>
          </div>
        </div>
      </div>
    </div>

    <!-- 2. Daily highs & lows -->
    <div class="hbox">
      <div class="hbox-label">Daily highs &amp; lows</div>
      <div class="hbox-bars">
        {#each barDays as d}
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

    <!-- 3. Time in comfort zone -->
    <div class="hbox">
      <div class="hbox-label">Time in comfort zone</div>
      <div class="hbox-ring">
        <svg width="84" height="84" viewBox="0 0 84 84" aria-hidden="true">
          <circle cx="42" cy="42" r={RING_R} fill="none"
            stroke="rgba(245,235,224,0.08)" stroke-width="6"/>
          <circle cx="42" cy="42" r={RING_R} fill="none"
            stroke="var(--accent-comfort)" stroke-width="6"
            stroke-linecap="round"
            stroke-dasharray={RING_C}
            stroke-dashoffset={ringOffset}
            transform="rotate(-90 42 42)"/>
          <text x="42" y="47" text-anchor="middle"
            font-size="20" font-family="'Instrument Serif', serif"
            font-variant-numeric="tabular-nums"
            fill="var(--ink)">{comfortPct}%</text>
        </svg>
        <div class="hbox-ring-meta">
          <div class="hbox-ring-line">of the week</div>
          <div class="hbox-ring-sub">between 20° and 24°</div>
        </div>
      </div>
    </div>

    <!-- 4. Daily rhythm -->
    <div class="hbox">
      <div class="hbox-label">Daily rhythm</div>
      <div class="hbox-rhythm">
        <svg viewBox="0 0 {RH_W} {RH_H}" preserveAspectRatio="none" class="rhythm-svg">
          <path d="{rhythmPath} L {RH_W} {RH_H} L 0 {RH_H} Z"
            fill="var(--accent-temp)" fill-opacity="0.12"/>
          <path d={rhythmPath} fill="none"
            stroke="var(--accent-temp)" stroke-width="1.5"
            stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <div class="hbox-rhythm-axis">
          <span>00</span><span>06</span><span>12</span><span>18</span><span>24</span>
        </div>
      </div>
      <div class="hbox-foot hbox-foot-row">
        <div class="hbox-pair">
          <span class="hbox-pair-label">Warmest hour</span>
          <span class="hbox-pair-value" style:color="var(--accent-temp)">
            {fmtHour(warmest?.hr)} <em>·</em> {warmest?.t?.toFixed(1)}°
          </span>
        </div>
        <div class="hbox-pair">
          <span class="hbox-pair-label">Coolest hour</span>
          <span class="hbox-pair-value" style:color="var(--accent-feels)">
            {fmtHour(coolest?.hr)} <em>·</em> {coolest?.t?.toFixed(1)}°
          </span>
        </div>
      </div>
    </div>

    <!-- 5. Driest stretch -->
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

    <!-- 6. Week in numbers -->
    <div class="hbox">
      <div class="hbox-label">Week in numbers</div>
      <ul class="hbox-list">
        <li>
          <span>Average</span>
          <b style:color="var(--accent-temp)">{weekAvgTemp?.toFixed(1)}°</b>
        </li>
        <li>
          <span>Average humidity</span>
          <b style:color="var(--accent-humid)">{weekAvgHumid != null ? Math.round(weekAvgHumid) + '%' : '—'}</b>
        </li>
        <li>
          <span>Largest daily swing</span>
          <b>{weekMaxSwing?.toFixed(1)}°</b>
        </li>
        <li>
          <span>Days tracked</span>
          <b>{days.length}</b>
        </li>
      </ul>
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
    align-items: baseline;
    justify-content: space-between;
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
  }

  .history-sub {
    font-size: 11.5px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-4);
    font-family: 'Geist Mono', ui-monospace, monospace;
  }

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
    min-height: 180px;
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

  .hbox-foot-row {
    flex-direction: row;
    justify-content: space-between;
    align-items: flex-end;
  }

  .hbox-line { font-size: 13px; color: var(--ink-2); font-family: 'Instrument Serif', serif; font-style: italic; }
  .hbox-sub  { font-size: 11.5px; color: var(--ink-4); font-family: 'Geist Mono', ui-monospace, monospace; }

  /* ── Mini table ──────────────────────────────────────────────────────────── */
  .hbox-mini {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-top: 6px;
    border-top: 1px solid var(--hairline);
  }

  .hbox-mini-row {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--ink-3);
    font-variant-numeric: tabular-nums;
  }

  .hbox-mini-row b { color: var(--ink); font-weight: 500; }

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

  /* ── Donut ring ──────────────────────────────────────────────────────────── */
  .hbox-ring { display: flex; align-items: center; gap: 14px; flex: 1; }

  .hbox-ring-meta { display: flex; flex-direction: column; gap: 4px; }

  .hbox-ring-line {
    font-family: 'Instrument Serif', serif;
    font-style: italic;
    font-size: 16px;
    color: var(--ink);
  }

  .hbox-ring-sub { font-size: 11.5px; color: var(--ink-3); }

  /* ── Daily rhythm ────────────────────────────────────────────────────────── */
  .hbox-rhythm { display: flex; flex-direction: column; gap: 4px; flex: 1; }

  .rhythm-svg { width: 100%; height: 60px; display: block; }

  .hbox-rhythm-axis {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--ink-4);
    font-family: 'Geist Mono', ui-monospace, monospace;
    letter-spacing: 0.04em;
  }

  .hbox-pair { display: flex; flex-direction: column; gap: 3px; }

  .hbox-pair-label {
    font-size: 10.5px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-4);
  }

  .hbox-pair-value {
    font-family: 'Instrument Serif', serif;
    font-size: 20px;
    letter-spacing: -0.01em;
    font-variant-numeric: tabular-nums;
  }

  .hbox-pair-value em { font-style: normal; color: var(--ink-4); margin: 0 4px; }

  /* ── List (week in numbers) ─────────────────────────────────────────────── */
  .hbox-list {
    list-style: none;
    margin: 0; padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .hbox-list li {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-size: 12.5px;
    color: var(--ink-3);
    padding-bottom: 7px;
    border-bottom: 1px dashed var(--hairline);
    font-variant-numeric: tabular-nums;
  }

  .hbox-list li:last-child { border-bottom: 0; padding-bottom: 0; }

  .hbox-list b {
    font-family: 'Instrument Serif', serif;
    font-weight: 400;
    font-size: 18px;
    color: var(--ink);
    letter-spacing: -0.01em;
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
