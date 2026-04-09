<script lang="ts">
	import {
		selectedGame,
		currentView,
		runningGame,
		favorites,
	} from "$lib/stores/gameStore";
	import { launchGame, killRunningGame } from "$lib/api/games";
	import Carousel from "./Carousel.svelte";
	import FadeIn from "./anim/FadeIn.svelte";
	import SlideIn from "./anim/SlideIn.svelte";
	import ProgBlur from "./effect/ProgBlur.svelte";
	import { cubicOut } from "svelte/easing";
	import { slide, fade } from "svelte/transition";

	let heroContentHeight = $state();
	let isLaunching = $state(false);
	let scrollY = $state(0);
	let containerEl: HTMLElement | undefined = $state(undefined);

	let artworkUrl = $derived(
		$selectedGame?.hero ||
			$selectedGame?.cover ||
			$selectedGame?.screenshots?.[0] ||
			"",
	);

	let isRunning = $derived($runningGame === $selectedGame?.slug);
	let isFavorite = $derived(
		$selectedGame ? $favorites.includes($selectedGame.slug) : false,
	);

	function handleBackClick() {
		currentView.set("library");
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
	{#key $selectedGame}
		<h1
			in:slide={{ easing: cubicOut, duration: 600, axis: "y" }}
			out:fade={{ easing: cubicOut, duration: 500 }}
			class="game-title page-headers"
		>
			{#if $selectedGame.logo}
				<img src={$selectedGame.logo} />
			{:else}
				{$selectedGame.name || $selectedGame.slug}
			{/if}
		</h1>
	{/key}
	<div
		class="preview-container scrollable"
		bind:this={containerEl}
		onscroll={handleScroll}
	>
		<!-- Hero -->
		<div class="hero">
			<img
	   			crossOrigin='anonymous'
				class="hero-bg"
				src={artworkUrl}
				style="transform: translateY({scrollY * 0.4}px);"
			/>
			{#if $selectedGame.tags && $selectedGame.tags.length > 0}
				<div class="game-tags">
					{#each $selectedGame.tags as tag}
						<span class="game-tag">{tag}</span>
					{/each}
				</div>
			{/if}
			<div class="hero-fade"></div>
			<div class="glass-transition"></div>
			<!-- Scroll hint outside .hero so it stacks above hero-content (z:10) -->
			<div class="scroll-hint" class:hidden={scrollY > 80}>
				<span class="scroll-label">scroll</span>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="4"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<polyline points="6 9 12 15 18 9" />
				</svg>
			</div>
		</div>

		<div
			class="hero-content-wrapper"
			style="margin-top: calc(100vh - 3rem - {heroContentHeight}px)"
		>
			<button
				class="fav-btn"
				class:active={isFavorite}
				onclick={() =>
					$selectedGame && favorites.toggle($selectedGame.slug)}
				title={isFavorite
					? "Remove from favorites"
					: "Add to favorites"}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path
						d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"
					/>
				</svg>
			</button>
			<div class="hero-content" bind:clientHeight={heroContentHeight}>
				<div class="hero-actions">
					<button
						class="play-button"
						class:kill={isRunning}
						onclick={handlePlayClick}
						disabled={isLaunching}
					>
						{isRunning
							? "Kill"
							: isLaunching
								? "Launching..."
								: "Play"}
					</button>
					<button class="back-button" onclick={handleBackClick}
						>← Library</button
					>
				</div>
			</div>
		</div>

		<!-- Content: progressive frosted glass from transparent → solid -->
		<div class="content">
			{#if $selectedGame.description || $selectedGame.short_description}
				<div class="description-section">
					{#if $selectedGame.short_description}
						<p class="short-desc">
							{$selectedGame.short_description}
						</p>
					{/if}
					{#if $selectedGame.description}
						<div class="long-desc">
							{@html $selectedGame.description}
						</div>
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
		overflow-x: hidden;
		color: var(--text-primary);
	}

	/* ── Hero ── */
	.hero {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100vh;
		z-index: 0;
		overflow: hidden;
	}

	.hero-bg {
		will-change: transform;
		position: absolute;
		top: 8%;
		left: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
		object-position: center center;

		transform: translateZ(0);
	}

	.hero-fade {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}

	:global(.hero .game-tags) {
		margin: 1rem;
		gap: 1rem;
	}

	.hero-content-wrapper {
		position: sticky;
		margin-left: 2rem;
		top: 4rem;
		left: 1.75rem;
		z-index: 5;
	}
	.hero-content {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		height: fit-content;
		width: fit-content;
	}
	.hero-actions {
		display: flex;
		gap: 0.6rem;
		align-items: center;
	}

	.game-title {
		display: flex;
		flex-direction: column;
		justify-content: center;
		margin: 0;
		position: absolute;
		left: 20%;
		width: 80%;
		height: 7.125rem;
		z-index: 6;
		font-family: "Brunson", sans-serif;
		font-size: 2.8rem;
		text-align: center;
		text-shadow: 0 0.125rem 1.5rem rgba(0, 0, 0, 0.8);
		letter-spacing: 0.1rem;
		line-height: 1;
		transition:
			left 0.3s ease,
			width 0.3s ease;
		pointer-events: none;
	}

	:global(.sidebar.collapsed ~ .game-title) {
		left: 3.5rem;
		width: calc(100% - 3.5rem);
	}
	.game-title h1 {
		color: var(--text-primary);
		text-align: center;
	}
	.game-title img {
		height: 7.125rem;
		max-width: 25%;
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

	.play-button:hover {
		transform: translateY(-0.1rem);
		box-shadow: 0 0.4rem 1.2rem rgba(0, 102, 255, 0.7);
	}
	.play-button:disabled {
		opacity: 0.6;
		cursor: not-allowed;
		transform: none;
	}
	.play-button.kill {
		background: linear-gradient(135deg, #e00, #ff4500);
		box-shadow: 0 0.25rem 0.9rem rgba(220, 0, 0, 0.45);
	}

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

	:global([data-theme="dark"] .back-button) {
		color: rgba(255, 255, 255, 0.9);
	}

	:global([data-theme="light"] .back-button) {
		color: rgba(0, 0, 0, 0.9);
	}
	.back-button:hover {
		background: rgba(255, 255, 255, 0.22);
	}

	.content {
		position: relative;
		z-index: 1;
		margin-top: 100vh;
		width: 100%;
		height: auto;
		background: linear-gradient(
			to bottom,
			var(--content-fade-start) 100%,
			var(--content-fade-mid) 100%,
			var(--content-fade-end) 100%
		);
	}

	.glass-transition {
		position: absolute;
		bottom: 0;
		top: auto;
		left: 0;
		right: 0;
		height: 30vh;
		pointer-events: none;
		z-index: 1;
		backdrop-filter: blur(1.8rem) saturate(1.5);
		-webkit-backdrop-filter: blur(1.8rem) saturate(1.5);
		background: linear-gradient(
			to bottom,
			transparent 0%,
			var(--content-fade-mid) 55%,
			var(--content-fade-end) 100%
		);
		mask-image: linear-gradient(
			to bottom,
			transparent 0%,
			rgba(0, 0, 0, 0.5) 25%,
			rgba(0, 0, 0, 0.85) 60%,
			black 100%
		);
		-webkit-mask-image: linear-gradient(
			to bottom,
			transparent 0%,
			rgba(0, 0, 0, 0.5) 25%,
			rgba(0, 0, 0, 0.85) 60%,
			black 100%
		);
	}

	/* ── Favorite button ── */
	.fav-btn {
		position: absolute;
		right: 1rem;
		width: 2.4rem;
		height: 2.4rem;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		border: 0.0625rem solid rgba(255, 255, 255, 0.25);
		background: rgba(104, 104, 104, 0.35);
		backdrop-filter: blur(0.5rem);
		cursor: pointer;
		transition:
			background 0.2s ease,
			border-color 0.2s ease,
			transform 0.15s ease;
	}

	.fav-btn:hover {
		background: rgba(0, 0, 0, 0.55);
		border-color: rgba(255, 80, 120, 0.6);
		transform: scale(1.1);
	}

	.fav-btn svg {
		width: 1.1rem;
		height: 1.1rem;
		fill: none;
		stroke: rgba(255, 255, 255, 0.7);
		transition:
			fill 0.2s ease,
			stroke 0.2s ease;
	}

	.fav-btn.active svg {
		fill: rgba(255, 60, 100, 0.9);
		stroke: rgba(255, 60, 100, 0.9);
	}

	.fav-btn:hover svg {
		stroke: rgba(255, 100, 130, 0.9);
	}

	/* ── Scroll hint ── */
	.scroll-hint {
		position: absolute;
		bottom: 0;
		left: 50%;
		transform: translateX(-50%);
		z-index: 5;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
		color: rgba(255, 255, 255, 0.7);
		opacity: 1;
		transition: opacity 1s ease;
		pointer-events: none;
	}
	:global([data-theme="dark"] .scroll-hint) {
		color: rgba(255, 255, 255, 0.7);
	}
	:global([data-theme="light"] .scroll-hint) {
		color: rgba(0, 0, 0, 0.7);
	}
	.scroll-hint.hidden {
		transition: opacity 2s ease-in-out;
		opacity: 0;
	}

	.scroll-label {
		font-size: 1rem;
		letter-spacing: 0.15rem;
		opacity: 1;
		transition: opacity 1s ease;
		text-transform: uppercase;
	}

	.scroll-hint svg {
		width: 1.3rem;
		height: 1.3rem;
		animation: bounce-down 1.8s ease-in-out infinite;
	}

	@keyframes bounce-down {
		0%,
		100% {
			transform: translateY(0);
		}
		50% {
			transform: translateY(0.35rem);
		}
	}

	/* ── Description ── */
	.description-section {
		padding: 0.3rem 1.75rem 1.25rem;
		max-width: 75%;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		text-align: left;
		position: relative;
		z-index: 0;
	}

	.short-desc {
		margin: 0;
		font-size: 1rem;
		line-height: 1.65;
		color: var(--text-secondary);
	}

	.long-desc {
		font-size: 0.875rem;
		line-height: 1.6;
		color: var(--text-primary);
	}

	.long-desc :global(img) {
		max-width: 100%;
		height: auto;
	}
	.long-desc :global(video) {
		max-width: 100%;
	}

	/* ── Carousel ── */
	.carousel-section {
		padding: 1rem 0 2rem;
		position: relative;
		z-index: 1;
	}
</style>
