<script lang="ts">
	import type { GameDisplay } from '$lib/types/game';
	import { selectedGame, currentView, runningGame } from '$lib/stores/gameStore';
	import { resizedUrl } from '$lib/api/images';

	interface Props {
		game: GameDisplay;
	}

	let { game }: Props = $props();

	// The library grid can render dozens of cards at once, each only
	// ~12rem wide, so request a thumbnail instead of the full-resolution
	// cover art (2x for hi-dpi displays).
	let coverUrl = $derived(resizedUrl(game.cover, { w: 400 }));

	function handleClick() {
		selectedGame.set(game);
		currentView.set('preview');
	}
</script>

<button
	type="button"
	class="game-card"
	class:running={$runningGame === game.slug}
	onclick={handleClick}
	aria-label="Open {game.name || game.slug}"
>
	<div class="game-cover" style="background-image: url('{coverUrl}');"></div>
</button>

<style>
	.game-card {
		display: flex;
		flex-direction: column;
		width: 12rem;
		height: 17rem;
		margin: 0.3125rem;
		padding: 0.5rem;
		border-radius: 0.625rem;
		background: rgba(30, 30, 45, 0.6);
		justify-content: center;
		align-items: center;
		text-align: center;
		color: #fff;
		backdrop-filter: blur(0.625rem);
		border: 0.0625rem solid rgba(255, 255, 255, 0.1);
		transition: all 0.3s ease;
		cursor: pointer;
	}

	.game-card:hover {
		transform: translateY(-0.3125rem);
		box-shadow: 0 0.625rem 1.25rem rgba(0, 102, 255, 0.3);
	}

	.game-cover {
		width: 100%;
		height: 100%;
		background: linear-gradient(145deg, #1e2a3a, #2a3a4a);
		border-radius: 0.625rem;
		background-size: cover;
		background-position: center;
		transition: transform 0.3s ease;
	}

	.game-card:hover .game-cover {
		transform: scale(1.02);
	}

	.game-card.running .game-cover {
		padding-top: 50%;
	}

	.game-card.running .game-cover::after {
		content: 'Running';
		width: 100%;
		height: 100%;
		background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
		display: flex;
		justify-content: center;
		align-items: center;
		color: #fff;
		border-radius: 0.625rem;
		font-weight: bold;
	}
</style>
