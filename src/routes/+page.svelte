<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Topbar from '$lib/components/Topbar.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Library from '$lib/components/Library.svelte';
	import GamePreview from '$lib/components/GamePreview.svelte';
	import Settings from '$lib/components/Settings.svelte';
	import Recommendations from '$lib/components/Recommendations.svelte';
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
		}, 1000);
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

		{#if $currentView === 'home'}
			<Recommendations />
		{:else if $currentView === 'library'}
			<Library />
		{:else if $currentView === 'settings'}
			<Settings />
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
		background: var(--bg-frosted);
		backdrop-filter: blur(var(--frosted-blur));
	}

	.big-container {
		display: flex;
		width: 100%;
		height: 100%;
		max-height: 100%;
	}
</style>
