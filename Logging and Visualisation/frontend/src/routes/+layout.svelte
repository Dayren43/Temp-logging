<script>
	import '../app.css';
	let { children } = $props();
</script>

<div class="app">
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

	@media (max-width: 680px) {
		.app {
			padding: 20px 18px 48px;
		}
	}
</style>
