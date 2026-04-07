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
		min-height: 0;
		overflow-y: scroll;
		scrollbar-width: thin;
		scrollbar-color: rgba(120, 50, 200, 0.25) transparent;
	}

	.page:hover {
		scrollbar-color: rgba(160, 80, 220, 0.65) transparent;
	}

	.library-header {
		position: sticky;
		display: flex;
		top: 0;
		height: 4rem;
		align-items: center;
		justify-content: space-between;
		background: var(--bg-panel);
		backdrop-filter: blur(1.5rem) saturate(1.6);
		-webkit-backdrop-filter: blur(1.5rem) saturate(1.6);
		border-bottom: 0.0625rem solid var(--border-color);
		z-index: 2;
		padding: 0 1rem;
	}

	.title {
		margin: 0;
		padding-left: 0.3125rem;
		color: var(--text-primary);
		font-family: 'Brunson', sans-serif;
		font-size: 1.5rem;
		letter-spacing: 0.3125rem;
		white-space: nowrap;
	}

	.right {
		display: flex;
		height: 100%;
		align-items: center;
		gap: 0.5rem;
	}

	.tag-button {
		padding: 0.4rem 0.75rem;
		border-radius: 0.5rem;
		background: var(--bg-card);
		border: 0.0625rem solid var(--border-color);
		color: var(--text-secondary);
		font-size: 0.8rem;
		cursor: pointer;
		transition: color 0.2s, background-color 0.2s;
		white-space: nowrap;
	}

	.tag-button:hover {
		color: var(--text-primary);
		background-color: var(--bg-input);
	}

	.tag-button.active {
		background: rgba(0, 102, 255, 0.2);
		border-color: rgba(121, 250, 0, 0.7);
		color: var(--text-primary);
	}

	.custom-dropdown {
		position: relative;
		display: flex;
		align-items: center;
	}

	.dropdown-button {
		background: var(--bg-card);
		padding: 0.4rem 0.75rem;
		color: var(--text-secondary);
		font-size: 0.8rem;
		border: 0.0625rem solid var(--border-color);
		border-radius: 0.5rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.dropdown-button:hover,
	.dropdown-button.active {
		color: var(--text-primary);
		border-color: var(--border-subtle);
		background: rgba(0, 102, 255, 0.15);
	}

	.dropdown-menu {
		position: absolute;
		top: 100%;
		right: 0;
		background: var(--bg-dropdown);
		min-width: 11.25rem;
		border-radius: 0.5rem;
		border: 0.0625rem solid var(--border-subtle);
		box-shadow: 0 0.5rem 1.5rem rgba(0, 0, 0, 0.2);
		padding: 0.25rem;
		margin: 0.25rem 0 0 0;
		z-index: 200;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		gap: 0.125rem;
	}

	.dropdown-menu button {
		padding: 0.5rem 0.75rem;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all 0.15s;
		background: transparent;
		border: none;
		border-radius: 0.35rem;
		text-align: left;
		font-size: 0.85rem;
		width: 100%;
	}

	.dropdown-menu button:hover {
		background: var(--bg-card);
		color: var(--text-primary);
	}

	.games-container {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
		gap: 1rem;
		padding: 1.5rem;
	}

	.games-container.has-running :global(.game-card:not(.running)) {
		opacity: 0.4;
	}
</style>
