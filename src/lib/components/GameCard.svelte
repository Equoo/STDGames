<script lang="ts">
	import type { GameDisplay } from '$lib/types/game';
	import { selectedGame, currentView, runningGame } from '$lib/stores/gameStore';

	interface Props {
		game: GameDisplay;
	}

	let { game }: Props = $props();

	let bgImage = $derived(game.hero || game.cover || game.screenshots?.[0] || '');

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
	title={game.name || game.slug}
>
	<div class="card-bg" style="background-image: url('{bgImage}');"></div>
	<!-- <img class="card-bg" src="{bgImage}"/> -->
	<div class="card-overlay"></div>
	<div class="card-info">
		{#if game.tags && game.tags.length > 0}
			<div class="game-tags">
				{#each game.tags.slice(0, 3) as tag}
					<span class="game-tag">{tag}</span>
				{/each}
			</div>
		{/if}
		<p class="card-name">{game.name ?? game.slug}</p>
	</div>
	{#if $runningGame === game.slug}
		<div class="running-badge">Running</div>
	{/if}
</button>

<style>
	:global(.game-card:hover) .card-bg {
		transform: scale(1.06);
	}

	.card-bg {
		position: absolute;
		inset: 0;
		background-size: cover;
		background-position: center;
		background-color: var(--bg-input);
		transition: transform 0.3s ease;
		transform: translateZ(0);
		will-change: auto transform;
		backface-visibility: hidden;
	}

	.game-card:hover .card-bg {
		transform: scale(1.06);
	}

	.card-overlay {
		position: absolute;
		inset: 0;
		background: linear-gradient(to top, rgba(0,0,0,0.88) 0%, rgba(0,0,0,0.2) 55%, transparent 100%);
		transition: transform 0.3s ease;
		will-change: auto transform;
		backface-visibility: hidden;
	}

	.game-card:hover .card-overlay {
		transform: scale(1.06);
	}

	.card-info {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		padding: 0.8rem 0.9rem;
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
		text-align: left;
	}

	.card-name {
		margin: 0;
		font-family: 'Brunson', sans-serif;
		font-size: 1rem;
		letter-spacing: 0.05rem;
		color: #fff;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}


	.running-badge {
		position: absolute;
		top: 0.6rem;
		right: 0.6rem;
		padding: 0.2rem 0.6rem;
		background: rgba(2, 122, 128, 0.85);
		border: 0.0625rem solid rgba(105, 205, 4, 0.6);
		border-radius: 0.3rem;
		font-size: 0.65rem;
		font-weight: bold;
		color: rgba(105, 205, 4, 0.95);
		text-transform: uppercase;
		letter-spacing: 0.05rem;
	}
</style>
