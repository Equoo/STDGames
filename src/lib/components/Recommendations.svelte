<script lang="ts">
	import GameCard from './GameCard.svelte';
	import { gameLibrary, favorites } from '$lib/stores/gameStore';
	import type { GameDisplay } from '$lib/types/game';
	import BlurIn from "./Anim.svelte";

	let picks: GameDisplay[] = $state([]);
	let initialized = false;

	$effect(() => {
		const games = $gameLibrary;
		if (games.length === 0 || initialized) return;
		initialized = true;
		shuffle();
	});

	function shuffle() {
		const copy = [...$gameLibrary];
		for (let i = copy.length - 1; i > 0; i--) {
			const j = Math.floor(Math.random() * (i + 1));
			[copy[i], copy[j]] = [copy[j], copy[i]];
		}
		picks = copy.slice(0, 6);
	}

</script>

<div class="page">
	<div class="page-body scrollable">
	<div class="rec-header page-headers">
	<BlurIn>
		<h1 class="page-title">Discover</h1>
	</BlurIn>
	</div>

	<div class="rec-body">
		{#if $gameLibrary.length === 0}
			<div class="empty">Loading your library…</div>
		{:else}
			<!-- Random -->
			<div class="section">
				<div class="section-header">
					<h2 class="section-title">Random picks</h2>
					<button class="reshuffle-btn" onclick={shuffle}>↺ Reshuffle</button>
				</div>
				<div class="rec-grid">
					{#each picks as game (game.slug)}
						<GameCard {game} />
					{/each}
				</div>
			</div>

			<!-- favorites -->
			<div class="section">
				<div class="section-header">
					<h2 class="section-title">For you</h2>
				</div>
				{#if $favorites.length === 0}
					<div class="placeholder">
						<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
						</svg>
						<p>Heart a game in its preview to pin it here.</p>
					</div>
				{:else}
					<div class="rec-grid">
						{#each $gameLibrary.filter(g => $favorites.includes(g.slug)) as game (game.slug)}
							<GameCard {game} />
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</div>
	</div>
</div>

<style>

	/* ── Body ── */
	.rec-body {
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 2.5rem;
	}

	.empty {
		color: var(--text-secondary);
		font-size: 0.9rem;
	}

	/* ── Section ── */
	.section {
		display: flex;
		flex-direction: column;
		gap: 0.9rem;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.section-title {
		margin: 0;
		font-family: 'Brunson', sans-serif;
		font-size: 1rem;
		letter-spacing: 0.2rem;
		color: var(--text-secondary);
		text-transform: uppercase;
	}

	.reshuffle-btn {
		background: transparent;
		border: none;
		color: var(--text-secondary);
		font-size: 0.8rem;
		cursor: pointer;
		padding: 0.2rem 0.5rem;
		border-radius: 0.3rem;
		transition: color 0.2s, background 0.2s;
	}

	.reshuffle-btn:hover {
		color: var(--text-primary);
		background: var(--bg-card);
	}

	/* ── Grid ── */
	.rec-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
		gap: 1rem;
	}

	/* ── Placeholder ── */
	.placeholder {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 0.75rem;
		padding: 3rem 1rem;
		border: 0.0625rem dashed var(--border-subtle);
		border-radius: 0.75rem;
		color: var(--text-secondary);
		font-size: 0.85rem;
	}

	.placeholder svg {
		width: 2rem;
		height: 2rem;
		opacity: 0.4;
	}

	.placeholder p {
		margin: 0;
	}
</style>
