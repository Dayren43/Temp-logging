<script>
	import github from '$lib/images/github.svg';
</script>

<footer>
	<div class="corner">
		<a href="https://github.com/Dayren43/Temp-logging">
			<img src={github} alt="GitHub" />
		</a>
	</div>
</footer>

<style>
  footer {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    padding: 1rem 2rem;
    color: var(--ink-3);
    position: relative;
    z-index: 10;
  }

  .corner {
    width: 3em;
    height: 3em;
  }

  .corner a {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }

  .corner img {
    width: 2em;
    height: 2em;
    object-fit: contain;
  }

  a:hover {
    color: var(--color-theme-1);
  }
</style>
