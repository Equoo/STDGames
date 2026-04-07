<script lang="ts">
	import { gameLibrary, selectedGame, currentView, favorites } from '$lib/stores/gameStore';
	import type { GameDisplay } from '$lib/types/game';

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

	function getImage(game: GameDisplay): string {
		return game.hero ?? game.cover ?? game.screenshots?.[0] ?? '';
	}

	function openGame(game: GameDisplay) {
		selectedGame.set(game);
		currentView.set('preview');
	}
</script>

<div class="page">
	<div class="rec-header">
		<h1 class="rec-title">Discover</h1>
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
						<button class="rec-card" onclick={() => openGame(game)}>
							<div class="rec-card-bg" style="background-image: url('{getImage(game)}');"></div>
							<div class="rec-card-overlay"></div>
							<div class="rec-card-info">
								{#if game.tags && game.tags.length > 0}
									<div class="rec-card-tags">
										{#each game.tags.slice(0, 3) as tag}
											<span class="rec-tag">{tag}</span>
										{/each}
									</div>
								{/if}
								<p class="rec-card-name">{game.name ?? game.slug}</p>
							</div>
						</button>
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
							<button class="rec-card" onclick={() => openGame(game)}>
								<div class="rec-card-bg" style="background-image: url('{getImage(game)}');"></div>
								<div class="rec-card-overlay"></div>
								<div class="rec-card-info">
									{#if game.tags && game.tags.length > 0}
										<div class="rec-card-tags">
											{#each game.tags.slice(0, 3) as tag}
												<span class="rec-tag">{tag}</span>
											{/each}
										</div>
									{/if}
									<p class="rec-card-name">{game.name ?? game.slug}</p>
								</div>
							</button>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.page {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		color: var(--text-primary);
	}

	/* ── Header ── */
	.rec-header {
		flex-shrink: 0;
		padding: 0 1.25rem;
		height: 3.75rem;
		display: flex;
		align-items: baseline;
		gap: 1rem;
		background: var(--bg-panel);
		backdrop-filter: blur(0.625rem);
		border-bottom: 0.0625rem solid var(--border-color);
	}

	.rec-title {
		margin: 0;
		font-family: 'Brunson', sans-serif;
		font-size: 1.5rem;
		letter-spacing: 0.3125rem;
	}

	.rec-subtitle {
		margin: 0;
		font-size: 0.8rem;
		color: var(--text-secondary);
	}

	/* ── Body ── */
	.rec-body {
		flex: 1;
		overflow-y: auto;
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

	/* ── Card ── */
	.rec-card {
		position: relative;
		aspect-ratio: 16 / 9;
		border-radius: 0.75rem;
		overflow: hidden;
		cursor: pointer;
		border: 0.0625rem solid var(--border-color);
		background: var(--bg-card);
		padding: 0;
		transition: transform 0.22s ease, box-shadow 0.22s ease, border-color 0.22s ease;
	}

	.rec-card:hover {
		transform: scale(1.025);
		box-shadow: 0 0.6rem 2rem rgba(0, 0, 0, 0.45);
		border-color: var(--border-subtle);
	}

	.rec-card-bg {
		position: absolute;
		inset: 0;
		background-size: cover;
		background-position: center;
		transition: transform 0.3s ease;
	}

	.rec-card:hover .rec-card-bg {
		transform: scale(1.06);
	}

	.rec-card-overlay {
		position: absolute;
		inset: 0;
		background: linear-gradient(to top, rgba(0,0,0,0.88) 0%, rgba(0,0,0,0.2) 55%, transparent 100%);
	}

	.rec-card-info {
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

	.rec-card-name {
		margin: 0;
		font-family: 'Brunson', sans-serif;
		font-size: 1rem;
		letter-spacing: 0.05rem;
		color: #fff;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.rec-card-tags {
		display: flex;
		gap: 0.3rem;
		flex-wrap: wrap;
	}

	.rec-tag {
		font-size: 0.62rem;
		padding: 0.15rem 0.45rem;
		background: rgba(255,255,255,0.15);
		border: 0.0625rem solid rgba(255,255,255,0.25);
		border-radius: 0.25rem;
		color: rgba(255,255,255,0.85);
		text-transform: uppercase;
		letter-spacing: 0.05rem;
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
