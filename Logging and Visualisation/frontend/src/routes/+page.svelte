<script>
	import LineChart from './LineChart.svelte';
	import TopBar from './TopBar.svelte';
	import Hero from './Hero.svelte';
	import { onMount } from 'svelte';

	const cached = typeof localStorage !== 'undefined'
		? JSON.parse(localStorage.getItem('last_reading') || 'null')
		: null;

	let data = $state(cached ?? { temp: null, humid: null });
	let lastFetchTime = $state(cached?.fetchTime ?? null);
	let ifFetch = false;

	onMount(async () => {
		// Show the latest DB row if no cache, then replace with live reading
		if (!cached) fetchLatestFromDB();
		fetchLive();
	});

	async function fetchLatestFromDB() {
		try {
			const res = await fetch('http://epsilon.local:3000/data?limit=1');
			const json = await res.json();
			const row = json.data?.[0];
			if (row && data.temp == null) {
				data = { temp: parseFloat(row.temp), humid: parseFloat(row.humid) };
				lastFetchTime = new Date(row.timestamp).getTime();
			}
		} catch (err) {
			console.error('Error fetching latest from DB:', err);
		}
	}

	async function fetchLive() {
		ifFetch = true;
		try {
			const res = await fetch('http://epsilon.local:3000/get');
			const json = await res.json();
			const fetchTime = Date.now();
			data = { temp: json.temp ?? json.Temp, humid: json.humid ?? json.Humid };
			lastFetchTime = fetchTime;
			localStorage.setItem('last_reading', JSON.stringify({ ...data, fetchTime }));
		} catch (err) {
			console.error('Error fetching live data:', err);
		}
		ifFetch = false;
	}
</script>

<svelte:head>
	<title>Temperature Monitor</title>
</svelte:head>

<TopBar lastTimestamp={lastFetchTime} />
<Hero temp={data.temp} humid={data.humid} />
<LineChart />
