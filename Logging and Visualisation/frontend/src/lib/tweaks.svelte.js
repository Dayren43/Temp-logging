const env = import.meta.env;
const DEFAULTS = {
	chartStyle: 'area',  // 'line' | 'area'
	density: 'regular',  // 'compact' | 'regular' | 'comfy'
	locationName: env.VITE_LOCATION_NAME || 'Stockholm',
	locationLat: parseFloat(env.VITE_LOCATION_LAT) || 59.3293,
	locationLon: parseFloat(env.VITE_LOCATION_LON) || 18.0686,
};

function load() {
	if (typeof localStorage === 'undefined') return { ...DEFAULTS };
	try { return { ...DEFAULTS, ...JSON.parse(localStorage.getItem('tweaks') ?? '{}') }; }
	catch { return { ...DEFAULTS }; }
}

export const tweaks = $state(load());

export function setTweak(key, val) {
	tweaks[key] = val;
	try { localStorage.setItem('tweaks', JSON.stringify({ ...tweaks })); } catch {}
}
