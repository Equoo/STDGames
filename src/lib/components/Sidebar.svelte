<script lang="ts">
	import { filteredGames, searchQuery, selectedGame, currentView, runningGame } from '$lib/stores/gameStore';
	import type { GameDisplay } from '$lib/types/game';

	let expanded = $state(true);

	function handleGameClick(game: GameDisplay) {
		selectedGame.set(game);
		currentView.set('preview');
	}

	function highlightMatch(name: string, query: string): string {
		if (!query) return name;
		const regex = new RegExp(`(${query})`, 'gi');
		return name.replace(regex, '<span class="highlight">$1</span>');
	}

	function toggleSidebar() {
		expanded = !expanded;
	}
</script>

<div class="sidebar" class:collapsed={!expanded}>
	<div class="sidebar-top">
		<div class="search-wrap">
			<div class="search-bar">
				<svg class="search-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<circle cx="11" cy="11" r="8"/>
					<line x1="21" y1="21" x2="16.65" y2="16.65"/>
				</svg>
				<input
					type="text"
					placeholder="Search games..."
					bind:value={$searchQuery}
				/>
			</div>
		</div>
		<button class="toggle-btn" onclick={toggleSidebar} title={expanded ? 'Collapse' : 'Expand'}>
			<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
				<polyline points="15 18 9 12 15 6"/>
			</svg>
		</button>
	</div>

	<div class="sidebar-content">
		<div class="game-list" role="list">
			{#each $filteredGames as game (game.slug)}
				<button
					type="button"
					class="game-list-item"
					class:running={$runningGame === game.slug}
					onclick={() => handleGameClick(game)}
					title={game.name || game.slug}
				>
					<div class="icon-container">
						<img src={game.icon} alt="{game.slug} icon" class="game-list-icon" />
					</div>
					<span class="item-name">{@html highlightMatch(game.name || game.slug, $searchQuery)}</span>
				</button>
			{/each}
		</div>
	</div>
</div>

<style>
	.sidebar {
		display: flex;
		flex: 0 0 auto;
		width: 20%;
		flex-direction: column;
		padding: 0 0.625rem;
		color: var(--text-primary);
		transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
		            padding 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		overflow: hidden;
	}

	.sidebar.collapsed {
		width: 3.5rem;
		padding: 0 0.35rem;
	}

	/* ── Top row (search + toggle) ── */
	.sidebar-top {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.5rem 0;
	}

	.search-wrap {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		opacity: 1;
		transition: flex 0.3s cubic-bezier(0.4, 0, 0.2, 1),
		            opacity 0.2s ease;
	}

	.sidebar.collapsed .search-wrap {
		flex: 0 0 0;
		opacity: 0;
		pointer-events: none;
	}

	.search-bar {
		display: flex;
		width: 100%;
		align-items: center;
		gap: 0.5rem;
		background: transparent;
		border: none;
		border-bottom: 0.0625rem solid var(--border-color);
		border-radius: 0;
		padding: 0.35rem 0.1rem;
		transition: border-color 0.2s ease;
	}

	.search-bar:hover {
		border-bottom-color: var(--border-subtle);
	}

	.search-bar:focus-within {
		border-bottom-color: rgba(121, 250, 0, 0.55);
	}
	:global([data-theme="light"] .search-bar:focus-within) {
		border-bottom-color: rgba(221, 0, 250, 0.55);
	}
	.search-icon {
		flex-shrink: 0;
		width: 0.9rem;
		height: 0.9rem;
		color: var(--text-secondary);
		transition: color 0.2s ease;
	}

	.search-bar:focus-within .search-icon {
		color: rgba(121, 250, 0, 0.7);
	}

	:global([data-theme="light"] .search-bar:focus-within .search-icon) {
		color: rgba(221, 0, 250, 0.55);
	}

	.search-bar input {
		flex: 1;
		min-width: 0;
		padding: 0;
		background: transparent;
		color: var(--text-primary);
		font-size: 0.85rem;
		border: none;
		outline: none;
	}

	.search-bar input::placeholder {
		color: var(--text-secondary);
		font-size: 0.85rem;
	}

	/* ── Toggle button ── */
	.toggle-btn {
		flex-shrink: 0;
		width: 2rem;
		height: 2rem;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-card);
		border: 0.0625rem solid var(--border-color);
		border-radius: 0.4rem;
		color: var(--text-secondary);
		cursor: pointer;
		transition: background 0.2s, border-color 0.2s, color 0.2s;
	}

	.toggle-btn:hover {
		color: var(--text-primary);
		border-color: var(--border-subtle);
		background: rgba(0, 102, 255, 0.15);
	}

	.toggle-btn svg {
		width: 1rem;
		height: 1rem;
		transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.sidebar.collapsed .toggle-btn svg {
		transform: rotate(180deg);
	}

	/* ── Scrollable list ── */
	.sidebar-content {
		display: flex;
		flex: 1;
		min-height: 0;
		width: 100%;
		overflow-y: scroll;
		overflow-x: hidden;
		direction: rtl;
		scrollbar-width: thin;
		scrollbar-color: rgba(120, 50, 200, 0.25) transparent;
	}

	.sidebar-content:hover {
		scrollbar-color: rgba(160, 80, 220, 0.65) transparent;
	}

	.game-list {
		padding: 0;
		margin: 0;
		width: 100%;
		display: flex;
		flex-direction: column;
		direction: ltr;
	}


	/* ── List items ── */
	.game-list-item {
		position: relative;
		display: flex;
		width: 100%;
		min-height: 2.1875rem;
		align-items: center;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		background-color: var(--bg-item);
		border-radius: 0.3125rem;
		margin: 0.125rem 0;
		padding: 0.3125rem;
		font-size: 0.8rem;
		cursor: pointer;
		border: none;
		border-left: 0.2rem solid transparent;
		color: var(--text-primary);
		text-align: left;
		transition: background-color 0.2s ease,
		            border-color 0.2s ease,
		            transform 0.15s ease;
	}

	.game-list-item:hover:not(.running) {
		background-color: var(--bg-card);
		border-left-color: rgba(0, 102, 255, 0.7);
		transform: translateX(0.15rem);
	}

	.game-list-item:hover .game-list-icon {
		transform: scale(1.1);
	}

	.game-list-item.running {
		background-color: rgba(2, 122, 128, 0.49);
		font-weight: bold;
		color: rgba(105, 205, 4, 0.9);
		border-left-color: rgba(105, 205, 4, 0.6);
	}

	.game-list-item.running::after {
		content: ' is Running';
		font-size: 0.8em;
		margin-left: 0.5rem;
	}

	/* Collapsed item layout */
	.sidebar.collapsed .game-list-item {
		justify-content: flex-start;
		padding: 0.4rem 0.3rem;
		border-left-color: transparent !important;
		transform: none !important;
		background: none;
	}

	.sidebar.collapsed .item-name {
		display: none;
	}

	.sidebar.collapsed .game-list-item.running::after {
		display: none;
	}

	.sidebar.collapsed .icon-container {
		margin-right: 0;
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
		transition: transform 0.2s ease;
	}

	:global(.game-list-item .highlight) {
		color: rgba(121, 250, 0, 0.92);
		background-color: rgba(0, 102, 255, 0.3);
	}
</style>
