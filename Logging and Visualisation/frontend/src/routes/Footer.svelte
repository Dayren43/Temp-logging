<script>
	import { page } from '$app/state';
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
    justify-content: flex-end; /* or space-between if you have multiple sections */
    align-items: center;
    padding: 1rem 2rem;
    background-color: #696969; /* dark gray background so it’s visible */
    color: white;
    position: relative; /* keep it in normal flow */
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
