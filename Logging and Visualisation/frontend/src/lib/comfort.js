// Steadman apparent temperature (BoM, no-wind form).
// Reasonable across cold/temperate/warm; valid for indoor logging.
export function apparentTemp(t, h) {
	if (t == null || h == null) return null;
	const e = (h / 100) * 6.105 * Math.exp((17.27 * t) / (237.7 + t));
	return t + 0.33 * e - 4.0;
}
