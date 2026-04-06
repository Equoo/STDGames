<script lang="ts">
	import { theme, currentView } from '$lib/stores/gameStore';
	import type { Theme } from '$lib/stores/gameStore';

	function setTheme(value: Theme) {
		theme.set(value);
	}

	function handleBack() {
		currentView.set('library');
	}
</script>

<div class="settings page">
	<div class="settings-header">
		<button class="back-btn" onclick={handleBack}>← Back</button>
		<h1 class="title">Settings</h1>
	</div>

	<div class="settings-body">
		<section class="settings-section">
			<h2 class="section-title">Appearance</h2>
			<div class="setting-row">
				<span class="setting-label">Theme</span>
				<div class="theme-toggle">
					<button
						class="theme-option"
						class:active={$theme === 'dark'}
						onclick={() => setTheme('dark')}
					>
						Dark
					</button>
					<button
						class="theme-option"
						class:active={$theme === 'light'}
						onclick={() => setTheme('light')}
					>
						Light
					</button>
				</div>
			</div>
		</section>
	</div>
</div>

<style>
	.page {
		flex: 1;
		height: auto;
		overflow-y: auto;
		color: var(--text-primary);
	}

	.settings-header {
		position: sticky;
		top: 0;
		display: flex;
		align-items: center;
		gap: 1rem;
		height: 3.75rem;
		padding: 0 1.25rem;
		background: var(--bg-panel);
		backdrop-filter: blur(0.625rem);
		z-index: 100;
	}

	.title {
		margin: 0;
		font-family: 'Brunson', sans-serif;
		font-size: 1.5rem;
		letter-spacing: 0.3125rem;
		color: var(--text-primary);
	}

	.back-btn {
		background: var(--bg-card);
		border: 0.0625rem solid var(--border-subtle);
		border-radius: 0.5rem;
		color: var(--text-primary);
		padding: 0.4rem 0.8rem;
		font-size: 0.85rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.back-btn:hover {
		background: rgba(0, 102, 255, 0.2);
		border-color: rgba(255, 0, 204, 0.5);
	}

	.settings-body {
		padding: 2rem;
		display: flex;
		flex-direction: column;
		gap: 2rem;
		max-width: 40rem;
	}

	.settings-section {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.section-title {
		margin: 0;
		font-family: 'Brunson', sans-serif;
		font-size: 1rem;
		letter-spacing: 0.2rem;
		color: var(--text-secondary);
		text-transform: uppercase;
		border-bottom: 0.0625rem solid var(--border-subtle);
		padding-bottom: 0.5rem;
	}

	.setting-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1rem;
		background: var(--bg-card);
		border: 0.0625rem solid var(--border-color);
		border-radius: 0.625rem;
	}

	.setting-label {
		font-size: 0.95rem;
	}

	.theme-toggle {
		display: flex;
		gap: 0.25rem;
		background: var(--bg-input);
		border-radius: 0.5rem;
		padding: 0.2rem;
	}

	.theme-option {
		padding: 0.35rem 1rem;
		border-radius: 0.35rem;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 0.85rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.theme-option.active {
		background: rgba(0, 102, 255, 0.4);
		color: var(--text-primary);
		border: 0.0625rem solid var(--border-subtle);
	}

	.theme-option:not(.active):hover {
		color: var(--text-primary);
	}
</style>
