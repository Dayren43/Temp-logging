<script>
	import LineChart from './LineChart.svelte';
	import TopBar from './TopBar.svelte';
	import Hero from './Hero.svelte';
	import AcControl from './AcControl.svelte';
	import History from './History.svelte';
	import Outside from './Outside.svelte';
	import Year from './Year.svelte';
	import Tweaks from './Tweaks.svelte';
	import { tweaks } from '$lib/tweaks.svelte.js';
	import { onMount } from 'svelte';

	const cached = typeof localStorage !== 'undefined'
		? JSON.parse(localStorage.getItem('last_reading') || 'null')
		: null;

	let data = $state(cached ?? { temp: null, humid: null });
	let lastFetchTime = $state(cached?.fetchTime ?? null);
	let outside = $state(null);

	onMount(() => {
		// Show the latest DB row if no cache, then replace with live reading
		if (!cached) fetchLatestFromDB();
		fetchLive();

		// Keep the live reading fresh while the page stays open.
		const interval = setInterval(() => {
			fetchLive();
			const { locationLat: lat, locationLon: lon } = tweaks;
			if (Number.isFinite(lat) && Number.isFinite(lon)) fetchOutside(lat, lon);
		}, 5 * 60_000);

		// Re-fetch immediately when the tab becomes visible again, so a
		// backgrounded/throttled tab shows current data as soon as it's focused.
		const onVisible = () => {
			if (document.visibilityState === 'visible') fetchLive();
		};
		document.addEventListener('visibilitychange', onVisible);

		return () => {
			clearInterval(interval);
			document.removeEventListener('visibilitychange', onVisible);
		};
	});

	// Re-fetch outside weather whenever the configured coords change.
	$effect(() => {
		const lat = tweaks.locationLat;
		const lon = tweaks.locationLon;
		if (!Number.isFinite(lat) || !Number.isFinite(lon)) return;
		fetchOutside(lat, lon);
	});

	async function fetchLatestFromDB() {
		try {
			const res = await fetch('http://epsilon.local:3000/data?limit=1', { cache: 'no-store' });
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
		try {
			const res = await fetch('http://epsilon.local:3000/get', { cache: 'no-store' });
			const json = await res.json();
			const fetchTime = Date.now();
			data = { temp: json.temp ?? json.Temp, humid: json.humid ?? json.Humid };
			lastFetchTime = fetchTime;
			localStorage.setItem('last_reading', JSON.stringify({ ...data, fetchTime }));
		} catch (err) {
			console.error('Error fetching live data:', err);
		}
	}

	async function fetchOutside(lat, lon) {
		try {
			const res = await fetch(
				`https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}` +
				'&current=temperature_2m,relative_humidity_2m,weather_code,apparent_temperature,windspeed_10m' +
				'&hourly=temperature_2m&forecast_days=2&timezone=Europe%2FStockholm'
			);
			const json = await res.json();
			const nowHour = new Date().toISOString().slice(0, 13);
			const startIdx = json.hourly.time.findIndex(t => t.startsWith(nowHour));
			const from = startIdx >= 0 ? startIdx : 0;
			const hourly = json.hourly.time.slice(from, from + 12).map((t, i) => ({
				t: new Date(t).getTime(),
				hour: new Date(t).getHours(),
				temp: json.hourly.temperature_2m[from + i]
			}));
			outside = {
				temp: json.current.temperature_2m,
				feels: json.current.apparent_temperature,
				humid: json.current.relative_humidity_2m,
				wind: Math.round(json.current.windspeed_10m),
				condition: wmoToCondition(json.current.weather_code),
				hourly,
				updatedAt: Date.now()
			};
		} catch (e) {
			console.error('Outside weather fetch failed', e);
		}
	}

	function wmoToCondition(code) {
		if (code === 0) return 'clear sky';
		if (code === 1) return 'mainly clear';
		if (code === 2) return 'partly cloudy';
		if (code === 3) return 'overcast';
		if (code <= 48) return 'foggy';
		if (code <= 55) return 'drizzle';
		if (code <= 65) return 'rainy';
		if (code <= 77) return 'snowy';
		if (code <= 82) return 'showers';
		return 'thunderstorm';
	}
</script>

<svelte:head>
	<title>Temperature Monitor</title>
</svelte:head>

<TopBar lastTimestamp={lastFetchTime} />
<Hero temp={data.temp} humid={data.humid} {outside} />
<AcControl />
<Outside inside={data} {outside} />
<LineChart />
<History />
<Year />
<Tweaks />
