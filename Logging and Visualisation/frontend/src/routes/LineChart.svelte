<script>
  import { line, curveLinear, Delaunay, scaleLinear, scaleUtc } from 'd3';
  import { onMount } from 'svelte';

  const marginTop = 40;
  const marginRight = 50;
  const marginBottom = 30;
  const marginLeft = 50;
  const width = 1200;
  const height = 600;

  const defaultTempDomain = [20, 30]; // °C
  const defaultHumidDomain = [30, 70];

  const colors = ['#ff6b81', '#4db6ff']; // bright pinkish red & light cyan-blue for contrast
  const strokeWidth = 3;
  const strokeOpacity = 0.9;
  const tooltipBackground = '#2f2f2f'; // dark tooltip
  const tooltipTextColor = '#ffffff';

  let fullData = [];
  let chartData = [];
  let isLoading = false;
  let error = null;

  $: showTemp = true;
  $: showHumid = true;
  let DateTick = false;

  let timeRange = '24h'; // default
  let aggregation = '5m'; // default aggregation

  let xVals = [], yValsTemp = [], yValsHumid = [];
  let lines = [], points = [], voronoiGrid;
  let xScale, yScaleTemp, yScaleHumid, xTicks, yTicksTemp, yTicksHumid, dotInfo;

async function fetchData() {
  isLoading = true;
  error = null;
  
  try {
    let queryParams = new URLSearchParams({
      range: timeRange,
    });
    
    // Auto-set aggregation based on range to keep the chart snappy
    if (['3d', '7d', 'all'].includes(timeRange)) {
      queryParams.set('aggregate', '15m'); // 15m is plenty for a week view
    } else if (timeRange === '24h') {
      queryParams.set('aggregate', '5m');
    }

    const url = `http://epsilon.local:3000/data?${queryParams.toString()}`;
    const res = await fetch(url);
    
    if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
    
    const response = await res.json();
    
    if (response.data && response.data.length > 0) {
      fullData = response.data.map(d => {
        // If aggregated, use time_bucket. If raw, use timestamp.
        const dateStr = d.time_bucket || d.timestamp;
        return {
          time: new Date(dateStr),
          // Backend now returns lowercase temp/humid from pg
          temp: parseFloat(d.temp), 
          humid: parseFloat(d.humid)
        };
      });
      
      // Sort data by time (Postgres 'DESC' is good for tables, 
      // but D3 line charts usually want 'ASC')
      fullData.sort((a, b) => a.time - b.time);
    } else {
      fullData = [];
    }
  } catch (err) {
    error = err.message;
  } finally {
    isLoading = false;
  }
}

onMount(async () => {
  await fetchData();
});

  function parseDuration(str) {
    const regex = /(\d+)([dhm])/g;
    let ms = 0;
    let match;
    while ((match = regex.exec(str)) !== null) {
      const val = parseInt(match[1]);
      const unit = match[2];
      if (unit === 'd') ms += val * 24 * 60 * 60 * 1000;
      if (unit === 'h') ms += val * 60 * 60 * 1000;
      if (unit === 'm') ms += val * 60 * 1000;
    }
    return ms;
  }

// Reactive data fetching when time range changes
$: if (timeRange) {
  fetchData();
}

$: {
  if (fullData.length) {
    if (timeRange === 'all') {
      chartData = fullData
    } else{
      const durationMs = parseDuration(timeRange.trim());
      const now = Date.now();
      if (durationMs)
        chartData = fullData.filter(d => now - d.time.getTime() <= durationMs);
    }
  }
}

  // Performance optimization: throttle chart updates and limit data points
  let chartUpdateTimeout = null;
  
  function updateChart() {
    if (chartData.length === 0) return;
    
    // Limit data points to prevent stack overflow (max 2000 points for chart)
    const maxPoints = 2000;
    const step = Math.max(1, Math.floor(chartData.length / maxPoints));
    const sampledData = chartData.filter((_, index) => index % step === 0);
    
    xVals = sampledData.map(d => d.time);
    yValsTemp = sampledData.map(d => d.temp);
    yValsHumid = sampledData.map(d => d.humid);

    xScale = scaleUtc()
      .domain([xVals[0], xVals[xVals.length - 1]])
      .range([marginLeft, width - marginRight]);

    yScaleTemp = scaleLinear()
      .domain(sampledData.length > 0
        ? [Math.min(defaultTempDomain[0], Math.min(...yValsTemp)),
          Math.max(defaultTempDomain[1], Math.max(...yValsTemp))]
        : defaultTempDomain
      )
      .nice()
      .range([height - marginBottom, marginTop]);

    yScaleHumid = scaleLinear()
      .domain(sampledData.length > 0
        ? [Math.min(defaultHumidDomain[0], Math.min(...yValsHumid)),
          Math.max(defaultHumidDomain[1], Math.max(...yValsHumid))]
        : defaultHumidDomain
      )
      .nice()
      .range([height - marginBottom, marginTop]);

    const tempLine = line()
      .curve(curveLinear)
      .x((_, i) => xScale(xVals[i]))
      .y((_, i) => yScaleTemp(yValsTemp[i]));

    const humidLine = line()
      .curve(curveLinear)
      .x((_, i) => xScale(xVals[i]))
      .y((_, i) => yScaleHumid(yValsHumid[i]));

    // Filter selected lines based on toggles
    lines = [];
    lines.push(tempLine(showTemp ? sampledData : []))
    lines.push(humidLine(showHumid ? sampledData : []));

    points = [
      ...sampledData.map((d, i) => ({ x: xVals[i], y: yValsTemp[i], series: 0 })),
      ...sampledData.map((d, i) => ({ x: xVals[i], y: yValsHumid[i], series: 1 }))
    ];

    const pointsScaled = points.map(p => [
      xScale(p.x),
      p.series === 0 ? yScaleTemp(p.y) : yScaleHumid(p.y),
      p.series
    ]);
    
    // Only create Voronoi diagram for reasonable number of points
    if (pointsScaled.length <= 5000) {
      const delaunayGrid = Delaunay.from(pointsScaled);
      voronoiGrid = delaunayGrid.voronoi([0, 0, width, height]);
    } else {
      voronoiGrid = null; // Skip for very large datasets
    }

    xTicks = xScale.ticks(8);
    yTicksTemp = yScaleTemp.ticks(6);
    yTicksHumid = yScaleHumid.ticks(6);
  }

  // Throttled reactive update
  $: if (chartData.length > 0) {
    if (chartUpdateTimeout) {
      clearTimeout(chartUpdateTimeout);
    }
    chartUpdateTimeout = setTimeout(updateChart, 100); // 100ms throttle
  }
</script>

<!-- Range selection UI -->
<div class="controls">
  <button on:click={() => { timeRange = '6h'; DateTick = false;console.log("btn 6h");}}>Last 6h</button>
  <button on:click={() => { timeRange = '24h';DateTick = false;console.log("btn 24h");}}>Last 24h</button>
  <button on:click={() => { timeRange = '3d'; DateTick = true; console.log("btn 3d");}}>Last 3d</button>
  <button on:click={() => { timeRange = '7d'; DateTick = true; console.log("btn 7d");}}>Last 7d</button>
  <button on:click={() => { timeRange = 'all';DateTick = true; console.log("all");}}>All</button>
  <input
    type="text"
    placeholder="e.g. 3d7h"
    bind:value={timeRange}
    title="Enter custom time range using 'd', 'h', 'm'. Example: '2d5h30m'"
  />

  <div  style="width: 50px;"></div>
  <!-- Toggle buttons -->
<button
  class:active={showTemp}
  class="temp"
  on:click={() => showTemp = !showTemp}
>
  Temp
</button>

<button
  class:active={showHumid}
  class="humid"
  on:click={() => showHumid = !showHumid}
>
  Humid
</button>
</div>

<!-- Status indicators -->
<div class="status">
  {#if isLoading}
    <span class="loading">Loading data...</span>
  {:else if error}
    <span class="error">Error: {error}</span>
  {:else if fullData.length > 0}
    <span class="info">
      {fullData.length} total data points | 
      {chartData.length} points in range | 
      {Math.min(2000, chartData.length)} points displayed
    </span>
  {/if}
</div>

<!-- Chart stays the same as your original -->
<div class="chart-container">
  {#if chartData.length > 0}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_mouse_events_have_key_events -->
    <svg {width} {height} cursor="crosshair" on:mouseout={() => dotInfo = null}>
      <path fill="none" stroke={colors[0]} d={lines[0]} stroke-opacity={strokeOpacity} stroke-width={strokeWidth}/>
      <path fill="none" stroke={colors[1]} d={lines[1]} stroke-opacity={strokeOpacity} stroke-width={strokeWidth}/>

      <g transform="translate(0,{height - marginBottom})">
        <path stroke="black" d="M{marginLeft},0.5 H{width - marginRight}"/>
        {#each xTicks as tick}
          <g transform="translate({xScale(tick)},0)">
            <line stroke="black" y2="6" />
            <text font-size="10" y="20" text-anchor="middle">
              {DateTick ? tick.toLocaleDateString(): tick.toLocaleTimeString([], {hour12: false, hour: '2-digit', minute:'2-digit'})}
            </text>
          </g>
        {/each}
      </g>

      <g transform="translate({marginLeft},0)">
        <path stroke="black" d="M0,{marginTop} V{height - marginBottom}"/>
        {#each yTicksTemp as tick}
          <g transform="translate(0,{yScaleTemp(tick)})">
            <line x1={-6} x2={0} stroke="black"/>
            <text x="-10" y="5" font-size="10" text-anchor="end">{tick}°C</text>
          </g>
        {/each}
      </g>

      <g transform="translate({width - marginRight},0)">
        <path stroke="black" d="M0,{marginTop} V{height - marginBottom}"/>
        {#each yTicksHumid as tick}
          <g transform="translate(0,{yScaleHumid(tick)})">
            <line x1={0} x2={6} stroke="black"/>
            <text x="10" y="5" font-size="10" text-anchor="start">{tick}%</text>
          </g>
        {/each}
      </g>

      {#if voronoiGrid}
        {#each points as p, i}
          <path fill-opacity="0" d={voronoiGrid.renderCell(i)}
            on:mouseover={(e) => dotInfo = [p, i, e]}/>
        {/each}
      {/if}
    </svg>
  {/if}
</div>

{#if dotInfo}
  <div class="tooltip"
    style="position:absolute; left:{dotInfo[2].clientX + 12}px; top:{dotInfo[2].clientY + 12}px;
           pointer-events:none; background-color:{tooltipBackground}; color:{tooltipTextColor}">
    {dotInfo[0].x.toLocaleTimeString([], {hour12: false, hour: '2-digit', minute:'2-digit'})}<br/>
    {dotInfo[0].series === 0
      ? `${dotInfo[0].y.toFixed(2)} °C`
      : `${dotInfo[0].y.toFixed(2)} %`}
  </div>
{/if}

<style>
  .controls {
    display: flex;
    gap: 8px;
    margin-bottom: 10px;
    justify-content: center;
  }
  .chart-container {
    background-color: #1e1e1e; /* match layout */
    border-radius: 8px;
    padding: 0.5rem;
  }

  svg {
    background-color: #1e1e1e; /* match layout */
  }

  path[stroke="black"],
  line[stroke="black"] {
    stroke: #888 !important; /* gray gridlines */
  }

  text {
    fill: #ccc; /* axis label text */
  }
  button {
    padding: 4px 8px;
    cursor: pointer;
  }
  input {
    width: 100px;
    padding: 4px;
  }
  .tooltip {
    border-radius: 5px;
    padding: 5px;
    font-size: 12px;
    box-shadow: rgba(0,0,0,0.4) 0px 2px 4px,
                rgba(0,0,0,0.3) 0px 7px 13px -3px;
  }
  
  .status {
    margin: 10px 0;
    text-align: center;
    font-size: 0.9rem;
    color: #ccc;
  }
  
  .loading {
    color: #4db6ff;
    font-weight: bold;
  }
  
  .error {
    color: #ff6b81;
    font-weight: bold;
  }
  
  .info {
    color: #ccc;
    background-color: #2a2a2a;
    padding: 4px 8px;
    border-radius: 4px;
  }

    button {
    padding: 0.4rem 0.8rem;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease-in-out;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  /* Default look */
  button.temp {
    background-color: #ffcccc; /* light red */
    color: #660000;
  }
  button.humid {
    background-color: #cce5ff; /* light blue */
    color: #003366;
  }

  /* Active look */
  button.active.temp {
    background-color: #ff4444; /* strong red */
    color: white;
    transform: translateY(1px);
    box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3);
  }
  button.active.humid {
    background-color: #3399ff; /* strong blue */
    color: white;
    transform: translateY(1px);
    box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3);
  }
</style>