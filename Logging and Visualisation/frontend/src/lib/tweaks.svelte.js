const DEFAULTS = {
	chartStyle: 'area',  // 'line' | 'area'
	density: 'regular',  // 'compact' | 'regular' | 'comfy'
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
