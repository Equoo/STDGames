<script lang="ts">
	import { filteredGames, searchQuery, selectedGame, currentView, runningGame } from '$lib/stores/gameStore';
	import type { GameDisplay } from '$lib/types/game';

	function handleGameClick(game: GameDisplay) {
		selectedGame.set(game);
		currentView.set('preview');
	}

	function highlightMatch(name: string, query: string): string {
		if (!query) return name;
		const regex = new RegExp(`(${query})`, 'gi');
		return name.replace(regex, '<span class="highlight">$1</span>');
	}
</script>

<div class="sidebar">
	<div class="search-container">
		<div class="search-bar">
			<span class="search-icon">⌕</span>
			<input
				type="text"
				placeholder="Search ..."
				bind:value={$searchQuery}
			/>
		</div>
	</div>
	<div class="sidebar-content">
		<div class="game-list" role="list">
			{#each $filteredGames as game (game.slug)}
				<button
					type="button"
					class="game-list-item"
					class:running={$runningGame === game.slug}
					onclick={() => handleGameClick(game)}
				>
					<div class="icon-container">
						<img src={game.icon} alt="{game.slug} icon" class="game-list-icon" />
					</div>
					<span>{@html highlightMatch(game.name || game.slug, $searchQuery)}</span>
				</button>
			{/each}
		</div>
	</div>
</div>

<style>
	.sidebar {
		display: flex;
		flex: 0 0 20%;
		flex-direction: column;
		padding: 0 0.625rem;
		color: #fff;
	}

	.search-container {
		background: rgba(0, 0, 0, 1);
		display: flex;
		position: relative;
		height: 3.125rem;
		width: 100%;
		z-index: 5;
	}

	.search-bar {
		position: relative;
		display: flex;
		height: 80%;
		width: 100%;
		align-items: center;
		background: rgba(20, 20, 30, 0.7);
		border: 0.0625rem solid rgba(0, 102, 255, 0.3);
		border-radius: 0.625rem;
		backdrop-filter: blur(0.3125rem);
		transition: all 0.2s ease;
	}

	.search-bar:hover {
		border-color: rgba(255, 0, 204, 0.4);
	}

	.search-bar:focus-within {
		border-color: rgba(121, 250, 0, 0.4);
		background: rgba(0, 102, 255, 0.1);
	}

	.search-icon {
		height: 100%;
		width: 2.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
		color: rgba(255, 255, 255, 0.6);
		font-size: 1.5rem;
	}

	.search-bar input {
		flex: 1;
		min-width: 0;
		padding: 0.25rem 0.5rem;
		background: transparent;
		color: white;
		font-size: 0.9rem;
		border: none;
		outline: none;
	}

	.search-bar input::placeholder {
		color: rgba(255, 255, 255, 0.6);
	}

	.sidebar-content {
		display: flex;
		width: 100%;
		height: auto;
		overflow-y: auto;
		overflow-x: hidden;
	}

	.game-list {
		padding: 0;
		margin: 0;
		width: 100%;
		display: flex;
		flex-direction: column;
	}

	.game-list-item {
		display: flex;
		width: 100%;
		min-height: 2.1875rem;
		align-items: center;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		background-color: rgba(37, 66, 94, 0.48);
		border-radius: 0.3125rem;
		margin: 0.125rem 0;
		padding: 0.3125rem;
		font-size: 0.8rem;
		cursor: pointer;
		transition: background-color 0.3s;
		border: none;
		color: white;
		text-align: left;
	}

	.game-list-item:hover {
		background-color: rgba(50, 50, 50, 0.33);
	}

	.game-list-item.running {
		background-color: rgba(2, 122, 128, 0.49);
		font-weight: bold;
		color: rgba(105, 205, 4, 0.9);
	}

	.game-list-item.running::after {
		content: ' is Running';
		font-size: 0.8em;
		margin-left: 0.5rem;
	}

	.icon-container {
		flex-shrink: 0;
		width: 1.875rem;
		height: 1.875rem;
		overflow: hidden;
		border-radius: 0.625rem;
		margin-right: 0.5rem;
	}

	.game-list-icon {
		width: 1.875rem;
		height: 1.875rem;
		object-fit: cover;
	}

	:global(.game-list-item .highlight) {
		color: rgba(121, 250, 0, 0.92);
		background-color: rgba(0, 102, 255, 0.3);
	}
</style>
