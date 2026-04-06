<script lang="ts">
	import { selectedGame, currentView, runningGame } from '$lib/stores/gameStore';
	import { launchGame, killRunningGame } from '$lib/api/games';
	import Carousel from './Carousel.svelte';

	let isLaunching = $state(false);
	let scrollY = $state(0);
	let containerEl: HTMLElement | undefined = $state(undefined);

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

	function handleScroll() {
		scrollY = containerEl?.scrollTop ?? 0;
	}
</script>

{#if $selectedGame}
	<div class="preview-container" bind:this={containerEl} onscroll={handleScroll}>

		<!-- Hero -->
		<div class="hero">
			<div
				class="hero-bg"
				style="background-image: url('{artworkUrl}'); transform: translateY({scrollY * 0.35}px) scale(1.15);"
			></div>
			<div class="hero-fade"></div>

			<div class="hero-content">
				{#if $selectedGame.tags && $selectedGame.tags.length > 0}
					<div class="tags">
						{#each $selectedGame.tags as tag}
							<span class="tag">{tag}</span>
						{/each}
					</div>
				{/if}
				<h1 class="game-title">{$selectedGame.name || $selectedGame.slug}</h1>

				<div class="hero-actions">
					<button
						class="play-button"
						class:kill={isRunning}
						onclick={handlePlayClick}
						disabled={isLaunching}
					>
						{isRunning ? 'Kill' : isLaunching ? 'Launching...' : 'Play'}
					</button>
					<button class="back-button" onclick={handleBackClick}>← Library</button>
				</div>
			</div>
		</div>

		<!-- Content: progressive frosted glass from transparent → solid -->
		<div class="content">
			{#if $selectedGame.description || $selectedGame.short_description}
				<div class="description-section">
					{#if $selectedGame.short_description}
						<p class="short-desc">{$selectedGame.short_description}</p>
					{/if}
					{#if $selectedGame.description}
						<div class="long-desc">{@html $selectedGame.description}</div>
					{/if}
				</div>
			{/if}

			<div class="carousel-section">
				<Carousel
					screenshots={$selectedGame.screenshots}
					videos={$selectedGame.movies}
					thumbnails={$selectedGame.movies_thumbnails}
				/>
			</div>
		</div>

	</div>
{/if}

<style>
	.preview-container {
		position: relative;
		width: 100%;
		height: 100%;
		overflow-y: auto;
		overflow-x: hidden;
		color: var(--text-primary);
	}

	/* ── Hero ── */
	.hero {
		position: relative;
		width: 100%;
		height: 48vh;
		overflow: hidden;
		flex-shrink: 0;
	}

	.hero-bg {
		position: absolute;
		inset: -15% 0 -15%;
		background-size: cover;
		background-position: center top;
		background-repeat: no-repeat;
		will-change: transform;
	}

	.hero-fade {
		position: absolute;
		inset: 0;
		background: linear-gradient(
			to bottom,
			rgba(0, 0, 0, 0) 0%,
			rgba(0, 0, 0, 0.15) 45%,
			rgba(0, 0, 0, 0.72) 80%,
			rgba(0, 0, 0, 0.88) 100%
		);
		pointer-events: none;
	}

	.hero-content {
		position: absolute;
		bottom: 1.25rem;
		left: 1.75rem;
		z-index: 5;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.tags {
		display: flex;
		gap: 0.4rem;
		flex-wrap: wrap;
	}

	.tag {
		background: rgba(255, 255, 255, 0.15);
		border: 0.0625rem solid rgba(255, 255, 255, 0.3);
		backdrop-filter: blur(0.25rem);
		color: rgba(255, 255, 255, 0.9);
		font-size: 0.7rem;
		padding: 0.2rem 0.6rem;
		border-radius: 0.3rem;
		text-transform: uppercase;
		letter-spacing: 0.05rem;
	}

	.game-title {
		margin: 0;
		font-family: 'Brunson', sans-serif;
		font-size: 2.8rem;
		color: #fff;
		text-shadow: 0 0.125rem 1.5rem rgba(0, 0, 0, 0.8);
		letter-spacing: 0.1rem;
		line-height: 1;
	}

	.hero-actions {
		display: flex;
		gap: 0.6rem;
		align-items: center;
		margin-top: 0.25rem;
	}

	.play-button {
		padding: 0.55rem 1.8rem;
		border: none;
		border-radius: 0.4rem;
		background: linear-gradient(135deg, #0066ff, #00a2ff);
		color: #fff;
		font-size: 0.9rem;
		font-weight: 600;
		cursor: pointer;
		box-shadow: 0 0.25rem 0.9rem rgba(0, 102, 255, 0.5);
		transition: all 0.2s ease;
	}

	.play-button:hover { transform: translateY(-0.1rem); box-shadow: 0 0.4rem 1.2rem rgba(0, 102, 255, 0.7); }
	.play-button:disabled { opacity: 0.6; cursor: not-allowed; transform: none; }
	.play-button.kill { background: linear-gradient(135deg, #e00, #ff4500); box-shadow: 0 0.25rem 0.9rem rgba(220, 0, 0, 0.45); }

	.back-button {
		padding: 0.55rem 1.1rem;
		border-radius: 0.4rem;
		border: 0.0625rem solid rgba(255, 255, 255, 0.3);
		background: rgba(255, 255, 255, 0.12);
		backdrop-filter: blur(0.5rem);
		color: rgba(255, 255, 255, 0.9);
		font-size: 0.85rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.back-button:hover { background: rgba(255, 255, 255, 0.22); }

	/* ── Content: transparent at top → solid at bottom ── */
	.content {
		position: relative;
		z-index: 3;
		background: linear-gradient(to bottom,
			var(--content-fade-start) 0%,
			var(--content-fade-mid) 20%,
			var(--content-fade-end) 42%
		);
	}

	/* ── Description ── */
	.description-section {
		padding: 1.5rem 1.75rem 1.25rem;
		max-width: 75%;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		text-align: left;
		position: relative;
		z-index: 1;
	}

	.short-desc {
		margin: 0;
		font-size: 0.95rem;
		line-height: 1.65;
		color: var(--text-secondary);
	}

	.long-desc {
		font-size: 0.875rem;
		line-height: 1.6;
		color: var(--text-primary);
	}

	.long-desc :global(img) { max-width: 100%; height: auto; }
	.long-desc :global(video) { max-width: 100%; }

	/* ── Carousel ── */
	.carousel-section {
		padding: 1rem 0 2rem;
		position: relative;
		z-index: 1;
	}
</style>
