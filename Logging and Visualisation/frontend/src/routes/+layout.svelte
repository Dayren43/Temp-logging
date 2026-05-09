<script>
	import '../app.css';
	import { tweaks } from '$lib/tweaks.svelte.js';
	let { children } = $props();
</script>

<div class="app" data-density={tweaks.density}>
	{@render children()}
</div>

<style>
	.app {
		max-width: 1240px;
		margin: 0 auto;
		padding: 32px 40px 64px;
		display: flex;
		flex-direction: column;
		gap: 32px;
	}

	.app[data-density="compact"] {
		gap: 20px;
		padding: 20px 40px 48px;
	}

	.app[data-density="comfy"] {
		gap: 48px;
		padding: 48px 40px 80px;
	}

	@media (max-width: 680px) {
		.app { padding: 20px 18px 48px; }
	}
</style>
