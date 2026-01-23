<script lang="ts">
	import GameCard from './GameCard.svelte';
	import { filteredGames, sortOrder, activeTag, runningGame } from '$lib/stores/gameStore';
	import type { SortOrder } from '$lib/types/game';

	let dropdownOpen = $state(false);

	function toggleTag(tag: string) {
		if ($activeTag === tag) {
			activeTag.set(null);
		} else {
			activeTag.set(tag);
		}
	}

	function setSortOrder(order: SortOrder) {
		sortOrder.set(order);
		dropdownOpen = false;
	}

	function toggleDropdown() {
		dropdownOpen = !dropdownOpen;
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.custom-dropdown')) {
			dropdownOpen = false;
		}
	}
</script>

<svelte:window onclick={handleClickOutside} />

<div class="library page">
	<div class="library-header">
		<h1 class="title">Library</h1>
		<div class="right">
			<button
				type="button"
				class="tag-button"
				class:active={$activeTag === 'multiplayer'}
				onclick={() => toggleTag('multiplayer')}
			>
				Multiplayer
			</button>
			<button
				type="button"
				class="tag-button"
				class:active={$activeTag === 'solo'}
				onclick={() => toggleTag('solo')}
			>
				Solo
			</button>

			<div class="custom-dropdown">
				<button
					type="button"
					class="dropdown-button"
					class:active={dropdownOpen}
					onclick={toggleDropdown}
				>
					Sort by ▼
				</button>
				{#if dropdownOpen}
					<div class="dropdown-menu" role="menu">
						<button type="button" role="menuitem" onclick={() => setSortOrder('descending')}>
							Name A → Z
						</button>
						<button type="button" role="menuitem" onclick={() => setSortOrder('ascending')}>
							Name Z → A
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>
	<div class="games-container" class:has-running={$runningGame !== ''}>
		{#each $filteredGames as game (game.slug)}
			<GameCard {game} />
		{/each}
	</div>
</div>

<style>
	.page {
		flex: 1;
		height: auto;
		overflow-y: auto;
	}

	.library-header {
		position: sticky;
		display: flex;
		background: rgba(0, 0, 0, 0.6);
		top: 0;
		height: 3.75rem;
		align-items: center;
		justify-content: space-between;
		backdrop-filter: blur(0.625rem);
		z-index: 100;
		padding: 0 0.625rem;
	}

	.title {
		margin: 0;
		padding-left: 0.3125rem;
		margin-left: 1%;
		color: rgba(245, 245, 245, 0.9);
		font-family: 'Brunson', sans-serif;
		font-size: 1.5rem;
		letter-spacing: 0.3125rem;
		white-space: nowrap;
	}

	.right {
		display: flex;
		height: 100%;
		align-items: center;
		gap: 0.625rem;
	}

	.tag-button {
		padding: 0.5rem 0.625rem;
		border-radius: 0.625rem;
		background: rgba(20, 20, 30, 0.7);
		border: 0.0625rem solid rgba(0, 102, 255, 0.3);
		color: white;
		font-size: 0.8rem;
		cursor: pointer;
		transition: all 0.2s;
		white-space: nowrap;
	}

	.tag-button:hover,
	.tag-button.active {
		background: rgba(0, 102, 255, 0.3);
		border-color: rgba(121, 250, 0, 0.92);
	}

	.custom-dropdown {
		position: relative;
		display: flex;
		align-items: center;
	}

	.dropdown-button {
		background: rgba(20, 20, 30, 0.7);
		padding: 0.5rem 0.625rem;
		color: white;
		font-size: 0.75rem;
		border: 0.0625rem solid rgba(0, 102, 255, 0.3);
		border-radius: 0.5rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.dropdown-button:hover,
	.dropdown-button.active {
		background: rgba(0, 102, 255, 0.3);
		border-color: rgba(121, 250, 0, 0.92);
	}

	.dropdown-menu {
		position: absolute;
		top: 100%;
		right: 0;
		background: rgba(10, 10, 20, 0.95);
		min-width: 11.25rem;
		border-radius: 0.5rem;
		box-shadow: 0 0.25rem 0.9375rem rgba(0, 0, 0, 0.3);
		padding: 0;
		margin: 0.3125rem 0 0 0;
		z-index: 200;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.dropdown-menu button {
		padding: 0.625rem 0.9375rem;
		color: white;
		cursor: pointer;
		transition: background 0.2s;
		background: transparent;
		border: none;
		text-align: left;
		font-size: 0.85rem;
		width: 100%;
	}

	.dropdown-menu button:hover {
		background: rgba(255, 0, 204, 0.1);
	}

	.games-container {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		padding: 0.625rem;
	}

	.games-container.has-running :global(.game-card:not(.running)) {
		opacity: 0.4;
	}
</style>
