<script>
	import LineChart from './LineChart.svelte';
	import { onMount } from 'svelte';

	let data = [];

	let ifFetch = false
	onMount(async () => {
		await fetchData();
		
	});

	async function fetchData() {
		ifFetch = true
		try {
			const res = await fetch('http://epsilon.local:3000/get');
			const json = await res.json();
			console.log(json);
			data = {
				temp: json.Temp,
				humid: json.Humid
			}
			console.log('Fetched data:', data);
		} catch (err) {
			console.error('Error fetching data:', err);
		}
		ifFetch = false
	}


	let perceivedTemperature = 0;

	// Compute perceived temperature whenever data changes
	// Uses the US National Weather Service heat index formula (Fahrenheit)
	$: if (data) {
		const T = data.temp * 9/5 + 32; // Celsius -> Fahrenheit
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

		// Convert back to Celsius
		perceivedTemperature = (HI_F - 32) * 5/9;
	}
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<h1 on:click={	!ifFetch ? fetchData : ""}>
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
		justify-content: center;
		align-items: center;
		flex: 0.6;
		text-align: center;
	}
</style>