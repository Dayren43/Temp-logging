<script>
	import Footer from './Footer.svelte';
	import '../app.css';

	let { children } = $props();
</script>

<div class="app">
	<main>
		{@render children()}
	</main>
	<Footer />
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
		background-color: #303030; /* dark gray */
		color: #e5e5e5;           /* light gray text */
	}

	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 1rem;
		width: 100%;
		max-width: 64rem;
		margin: 0 auto;
		box-sizing: border-box;
	}

	a {
		color: #9cdcfe; /* light blue accent */
	}
	a:hover {
		color: #7cb8f7;
	}

	:global(footer) {
		background-color: #3b3b3b;
		color: #b0b0b0;
		padding: 1rem;
	}
</style>