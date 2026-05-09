<script>
	import LineChart from './LineChart.svelte';
	import TopBar from './TopBar.svelte';
	import { onMount } from 'svelte';

	let data = [];
	let lastFetchTime = null;
	let ifFetch = false;

	onMount(async () => {
		await fetchData();
	});

	async function fetchData() {
		ifFetch = true;
		try {
			const res = await fetch('http://epsilon.local:3000/get');
			const json = await res.json();
			data = {
				temp: json.temp || json.Temp,
				humid: json.humid || json.Humid
			};
			lastFetchTime = Date.now();
		} catch (err) {
			console.error('Error fetching data:', err);
		}
		ifFetch = false;
	}

	let perceivedTemperature = 0;

	$: if (data) {
		const T = data.temp * 9/5 + 32;
		const R = data.humid;
		const HI_F = -42.379 +
			2.04901523 * T +
			10.14333127 * R -
			0.22475541 * T * R -
			0.00683783 * T * T -
			0.05481717 * R * R +
			0.00122874 * T * T * R +
			0.00085282 * T * R * R -
			0.00000199 * T * T * R * R;
		perceivedTemperature = (HI_F - 32) * 5/9;
	}
</script>

<svelte:head>
	<title>Temperature Monitor</title>
</svelte:head>

<TopBar lastTimestamp={lastFetchTime} />

<section>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<h1 on:click={!ifFetch ? fetchData : ""}>
		Current Conditions <br />
		Perceived: {perceivedTemperature.toFixed(1)}°C <br />
		Temp: {data?.temp}°C, Humid: {data?.humid}%
	</h1>
	<LineChart />
</section>

<style>
	section {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: 16px;
	}

	h1 {
		font-family: 'Geist', ui-sans-serif, system-ui, sans-serif;
		font-size: 1.4rem;
		font-weight: 500;
		color: var(--ink-2);
		cursor: pointer;
		margin: 0;
		line-height: 1.6;
	}
</style>
