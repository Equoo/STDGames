
<script>
	import mainback from '$lib/assets/icons/stdgames.png';
	import { onMount } from "svelte";

	let loading_state = $state(0);
	let bar;

	onMount(async () => {
		const { listen } = await import("@tauri-apps/api/event");
		listen("progressbar_update", (event) => {
			bar.style.width = `${event.payload}%`;
		});
	});
</script>

<style>
	:global(body) {
		background-color: rgba(0, 0, 0, 0);
	}
	
	#background {
		background-image: url("/assets/icons/stdgames.png");
		background-size: cover;
		background-repeat: no-repeat;
		background-position: center;
		position: fixed;
		top: 0;
		left: 15px;
		width: calc(100% - 30px);
		height: calc(100% - 30px);
		z-index: -1;
	}

	/* progressbar */
	.progress {
		position: fixed;
		bottom: 0;
		left: 0;
		width: 100%;
		height: 20px;
		background-color: #444;
		border-radius: 10px;
		margin-top: 10px;
		z-index: 100000;
	}

	.progress-bar {
		height: 100%;
		background-color: #76c7c0;
		border-radius: 10px;
		transition: width 0.5s ease-in-out;
	}
</style>

<div id="background"></div>
<div class="progress">
	<div bind:this={bar} class="progress-bar"></div>
	<p id="progress-text"></p>
</div>

