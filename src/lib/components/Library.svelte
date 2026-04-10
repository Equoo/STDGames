<script lang="ts">
	import GameCard from "./GameCard.svelte";
	import {
		filteredGames,
		sortOrder,
		activeTag,
		runningGame,
	} from "$lib/stores/gameStore";
	import type { SortOrder } from "$lib/types/game";
	import BlurIn from "./anim/BlurIn.svelte";
	import SlideIn from "./anim/SlideIn.svelte";
	import FlyIn from "./anim/FlyIn.svelte";
	import FadeIn from "./anim/FadeIn.svelte";

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
		if (!target.closest(".custom-dropdown")) {
			dropdownOpen = false;
		}
	}

	import ContrastText from '$lib/components/ContrastText.svelte';
	let { section } = $props();
</script>

<svelte:window onclick={handleClickOutside} />

<div class="library page">
	<div class="page-body scrollable">
		<div class="library-header page-headers">
			<FlyIn>
				<h1 class="page-title no-capture"><ContrastText container={section}>Library</ContrastText></h1>
			</FlyIn>
			<FadeIn>
				<div class="right">
					<button
						type="button"
						class="tag-button"
						class:active={$activeTag === "multiplayer"}
						onclick={() => toggleTag("multiplayer")}
					>
						Multiplayer
					</button>
					<button
						type="button"
						class="tag-button"
						class:active={$activeTag === "solo"}
						onclick={() => toggleTag("solo")}
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
								<button
									type="button"
									role="menuitem"
									onclick={() => setSortOrder("descending")}
								>
									Name A → Z
								</button>
								<button
									type="button"
									role="menuitem"
									onclick={() => setSortOrder("ascending")}
								>
									Name Z → A
								</button>
							</div>
						{/if}
					</div>
				</div>
			</FadeIn>
		</div>
		<FadeIn>
			<div
				class="games-container"
				class:has-running={$runningGame !== ""}
			>
				{#each $filteredGames as game (game.slug)}
					<GameCard {game} />
				{/each}
			</div>
		</FadeIn>
	</div>
</div>

<style>
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
		transition:
			color 0.2s,
			background-color 0.2s;
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
		transform: translateZ(0);
		will-change: transform;
	}

	.games-container.has-running :global(.game-card:not(.running)) {
		opacity: 0.4;
	}
</style>
