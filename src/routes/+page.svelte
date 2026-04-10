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

	let page = $state<HTMLElement | undefined>();
	let attachedPage = $derived(
		page?.isConnected ? page : undefined
	);

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

		page = document.body;
	});

	onDestroy(() => {
		if (pollingInterval) {
			clearInterval(pollingInterval);
		}
	});
</script>

<div class="frosted-glass">
	<Topbar section={attachedPage}/>

	<div class="big-container">
		<Sidebar />

		{#if $currentView === 'home'}
			<Recommendations section={attachedPage}/>
		{:else if $currentView === 'library'}
			<Library section={attachedPage}/>
		{:else if $currentView === 'settings'}
			<Settings section={attachedPage}/>
		{:else}
			<GamePreview section={attachedPage}/>
		{/if}
	</div>
</div>

<style>
	.frosted-glass {
		position: absolute;
		inset: 0;
		background: var(--bg-frosted);
		backdrop-filter: blur(var(--frosted-blur));
	}

	.big-container {
		display: flex;
		width: 100%;
		height: 100%;
	}
</style>
