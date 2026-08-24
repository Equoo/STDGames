<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Topbar from '$lib/components/Topbar.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Library from '$lib/components/Library.svelte';
	import GamePreview from '$lib/components/GamePreview.svelte';
	import { gameLibrary, runningGame, currentView } from '$lib/stores/gameStore';
	import { fetchGameLibrary, getRunningGame } from '$lib/api/games';

	let pollingInterval: ReturnType<typeof setInterval>;

	onMount(async () => {
		// Load game library
		const library = await fetchGameLibrary();
		console.log(library);
		if (library) {
			gameLibrary.set(library);
		}

		// Poll for running game status
		pollingInterval = setInterval(async () => {
			const running = await getRunningGame();
			runningGame.set(running);
		}, 100);
	});

	onDestroy(() => {
		if (pollingInterval) {
			clearInterval(pollingInterval);
		}
	});
</script>

<Topbar />

<div class="frosted-glass">
	<div class="big-container">
		<Sidebar />

		{#if $currentView === 'library'}
			<Library />
		{:else}
			<GamePreview />
		{/if}
	</div>
</div>

<style>
	.frosted-glass {
		position: absolute;
		top: 3.125rem;
		left: 0;
		height: calc(100% - 3.125rem);
		width: 100%;
		background: linear-gradient(to bottom, rgba(0, 0, 0, 0.8), rgba(46, 46, 46, 0.34));
		backdrop-filter: blur(6.25rem);
	}

	.big-container {
		display: flex;
		width: 100%;
		height: 100%;
		max-height: 100%;
	}
</style>
