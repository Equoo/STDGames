<script lang="ts">
	import { selectedGame, currentView, runningGame } from '$lib/stores/gameStore';
	import { launchGame, killRunningGame } from '$lib/api/games';
	import Carousel from './Carousel.svelte';

	let isLaunching = $state(false);

	let artworkUrl = $derived(
		$selectedGame?.hero || $selectedGame?.cover || $selectedGame?.screenshots?.[0] || ''
	);

	let isRunning = $derived($runningGame === $selectedGame?.slug);

	function handleBackClick() {
		currentView.set('library');
	}

	async function handlePlayClick() {
		if (!$selectedGame) return;

		if (isRunning) {
			await killRunningGame();
		} else {
			isLaunching = true;
			await launchGame($selectedGame.slug);
			isLaunching = false;
		}
	}
</script>

{#if $selectedGame}
	<div class="game-preview-container">
		<button class="back-button" onclick={handleBackClick}>← Back to library</button>

		<div class="game-preview">
			<div class="image-crop-container">
				<div class="game-preview-artwork" style="background-image: url('{artworkUrl}');"></div>
				<h1 class="title-overlay">{$selectedGame.name || $selectedGame.slug}</h1>

				{#if $selectedGame.tags && $selectedGame.tags.length > 0}
					<div class="game-genres">
						{#each $selectedGame.tags as tag}
							<div class="game-genres-item">{tag}</div>
						{/each}
					</div>
				{/if}

				<div class="game-meta">
					{#if $selectedGame.short_description}
						<div class="game-description">{$selectedGame.short_description}</div>
					{/if}
				</div>

				<div class="button-overlay">
					<button
						class="play-button"
						class:kill-button={isRunning}
						onclick={handlePlayClick}
						disabled={isLaunching}
					>
						{isRunning ? 'Kill the process' : 'Play'}
					</button>
					<button class="game-settings-button">⚙️ Settings</button>
				</div>

				<div class="scroll-indicator">
					<div class="scroll-text">Scroll</div>
					<div class="scroll-arrow">
						<svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
							<path d="M12 5v14M19 12l-7 7-7-7" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
						</svg>
					</div>
				</div>
			</div>
		</div>

		<div class="game-details">
			<Carousel
				screenshots={$selectedGame.screenshots}
				videos={$selectedGame.movies}
				thumbnails={$selectedGame.movies_thumbnails}
			/>
		</div>

		{#if $selectedGame.description}
			<div class="description-section">
				{@html $selectedGame.description}
			</div>
		{/if}
	</div>
{/if}

<style>
	.game-preview-container {
		position: relative;
		width: 100%;
		padding: 0;
		margin: 0;
		margin-right: 0.7rem;
		color: white;
		background: none;
		overflow-y: auto;
		overflow-x: hidden;
	}

	.back-button {
		position: absolute;
		top: 2rem;
		right: 0.625rem;
		z-index: 10;
		background: rgba(0, 0, 0, 0.5);
		border: none;
		color: white;
		width: auto;
		height: 2.5rem;
		padding: 0 1rem;
		border-radius: 0.625rem;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all 0.3s ease;
		font-size: 0.9rem;
	}

	.back-button:hover {
		background: rgba(0, 102, 255, 0.7);
		transform: scale(1.05);
	}

	.game-preview {
		position: relative;
		width: 100%;
		border-radius: 0.75rem;
		overflow: hidden;
		box-shadow: 0 0.625rem 1.875rem rgba(0, 0, 0, 0.5);
	}

	.image-crop-container {
		position: relative;
		width: 100%;
		height: calc(100vh - 3.9rem);
		min-height: calc(100vh - 3.9rem);
		overflow: hidden;
	}

	.image-crop-container::after {
		content: '';
		position: absolute;
		background: linear-gradient(to top, rgba(0, 0, 0, 0.94), transparent);
		bottom: 0;
		left: 0;
		width: 100%;
		height: 100%;
		z-index: 2;
		pointer-events: none;
	}

	.game-preview-artwork {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		background-size: cover;
		background-position: center;
		background-repeat: no-repeat;
		transition: transform 0.5s ease;
		z-index: 1;
	}

	.title-overlay {
		position: absolute;
		background: rgba(0, 0, 0, 0.5);
		left: 1.25rem;
		top: 2rem;
		height: auto;
		max-width: 70%;
		padding: 0.25rem 0.625rem;
		font-family: 'Brunson', sans-serif;
		color: rgba(235, 234, 234, 0.9);
		letter-spacing: 0.1875rem;
		text-shadow: 0 0.125rem 0.625rem rgba(0, 0, 0, 0.7);
		border-radius: 0.3125rem;
		z-index: 3;
		font-size: 1.8rem;
		margin: 0;
	}

	.game-genres {
		display: flex;
		flex-wrap: wrap;
		position: absolute;
		gap: 0.3125rem;
		top: 7rem;
		left: 1.25rem;
		z-index: 1000;
	}

	.game-genres-item {
		background: rgba(0, 0, 0, 0.5);
		padding: 0.3125rem 0.75rem;
		border-radius: 0.625rem;
		color: rgba(255, 255, 255, 0.9);
		font-size: 0.8rem;
		border: 0.0625rem solid rgba(255, 255, 255, 0.65);
	}

	.game-meta {
		display: flex;
		gap: 1.875rem;
		margin-bottom: 1.875rem;
	}

	.game-description {
		position: absolute;
		bottom: 5rem;
		left: 1.875rem;
		max-width: 50rem;
		font-size: 1.1rem;
		line-height: 1.6;
		margin-bottom: 1.875rem;
		color: rgba(255, 255, 255, 0.9);
		z-index: 10000;
	}

	.button-overlay {
		position: absolute;
		bottom: 0.625rem;
		left: 0.625rem;
		right: 0.625rem;
		display: flex;
		justify-content: space-between;
		z-index: 3;
	}

	.play-button {
		display: flex;
		padding: 0.75rem 1.5rem;
		gap: 0.5rem;
		color: white;
		font-size: 1rem;
		font-weight: 600;
		align-items: center;
		cursor: pointer;
		border: none;
		border-radius: 0.5rem;
		background: linear-gradient(135deg, #0066ff, #00a2ff);
		box-shadow: 0 0.25rem 0.9375rem rgba(0, 102, 255, 0.4);
		transition: all 0.3s ease;
	}

	.play-button:hover {
		transform: translateY(-0.125rem);
		box-shadow: 0 0.375rem 1.25rem rgba(0, 102, 255, 0.6);
	}

	.play-button:disabled {
		opacity: 0.7;
		cursor: not-allowed;
	}

	.play-button.kill-button {
		background: linear-gradient(135deg, #ff0000, #ff4500);
	}

	.game-settings-button {
		padding: 0.375rem 0.75rem;
		background: rgba(255, 255, 255, 0.1);
		backdrop-filter: blur(0.625rem);
		color: white;
		border: 0.0625rem solid rgba(255, 255, 255, 0.2);
		border-radius: 0.5rem;
		cursor: pointer;
		font-size: 0.9rem;
		transition: all 0.3s ease;
	}

	.game-settings-button:hover {
		background: rgba(255, 255, 255, 0.2);
	}

	.scroll-indicator {
		position: absolute;
		bottom: 0;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		z-index: 4;
		padding-bottom: 1rem;
		animation: bounce 2s infinite;
	}

	.scroll-text {
		font-size: 0.875rem;
		color: rgba(255, 255, 255, 0.8);
		text-transform: uppercase;
		letter-spacing: 0.1rem;
		font-weight: 500;
	}

	.scroll-arrow {
		width: 2rem;
		height: 2rem;
		color: rgba(255, 255, 255, 0.8);
	}

	.scroll-arrow svg {
		width: 100%;
		height: 100%;
	}

	@keyframes bounce {
		0%, 100% {
			transform: translateX(-50%) translateY(0);
		}
		50% {
			transform: translateX(-50%) translateY(-0.2rem);
		}
	}

	.game-details {
		padding-top: 1.875rem;
		margin: 0 auto;
	}

	.description-section {
		margin-left: 50%;
		transform: translateX(-50%);
		color: #ffffff;
		text-align: left;
		line-height: 1.5em;
		font-size: 0.875rem;
		margin-top: 1.875rem;
		overflow: hidden;
		width: 38.5rem;
		max-width: 100%;
		font-weight: normal;
		padding-bottom: 3.125rem;
	}

	.description-section :global(img) {
		max-width: 100%;
		height: auto;
	}

	.description-section :global(video) {
		max-width: 100%;
	}
</style>
