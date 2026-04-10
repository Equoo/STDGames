<script lang="ts">
	import { theme } from "$lib/stores/gameStore";
	import type { Theme } from "$lib/stores/gameStore";
	import { addDesktopIcon, openUrl } from "$lib/api/system";
	import BlurIn from "./anim/BlurIn.svelte";
	import SlideIn from "./anim/SlideIn.svelte";
	import FlyIn from "./anim/FlyIn.svelte";
	import FadeIn from "./anim/FadeIn.svelte";

	function setTheme(value: Theme) {
		theme.set(value);
	}

	import ContrastText from '$lib/components/ContrastText.svelte';
	let { section } = $props();
</script>

<div class="settings page">
	<div class="page-body scrollable">
		<div class="settings-header page-headers">
			<FlyIn>
				<h1 class="page-title no-capture"><ContrastText container={section}>Settings</ContrastText></h1>
			</FlyIn>
		</div>

		<div class="settings-body">
			<section class="settings-section">
				<h2 class="section-title">Appearance</h2>
				<div class="setting-row">
					<span class="setting-label">Theme</span>
					<div class="theme-toggle">
						<button
							class="theme-option"
							class:active={$theme === "dark"}
							onclick={() => setTheme("dark")}>Dark</button
						>
						<button
							class="theme-option"
							class:active={$theme === "light"}
							onclick={() => setTheme("light")}>Light</button
						>
					</div>
				</div>
			</section>

			<section class="settings-section">
				<h2 class="section-title">System</h2>
				<div class="community-row">
					<span class="setting-label"
						>Add a shortcut to your desktop</span
					>
					<button
						class="action-btn community-btn"
						onclick={addDesktopIcon}>Add to desktop</button
					>
				</div>
			</section>

			<section class="settings-section">
				<h2 class="section-title">Community</h2>
				<div class="community-row">
					<span class="community-label">Join us on Discord</span>
					<button
						class="action-btn community-btn"
						onclick={() => openUrl("https://discord.gg/YR7fwGy5D7")}
					>
						Discord
					</button>
				</div>
				<div class="community-row">
					<span class="community-label">Give us a star on Github</span
					>
					<button
						class="action-btn community-btn"
						onclick={() =>
							openUrl("https://github.com/Equoo/STDGames")}
					>
						Github
					</button>
				</div>
			</section>
		</div>
	</div>
</div>

<style>
	/* ── 3-column body ── */
	.settings-body {
		padding: 2rem;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 1.5rem;
		align-content: start;
	}

	.settings-section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.section-title {
		margin: 0;
		font-family: "Brunson", sans-serif;
		font-size: 0.9rem;
		letter-spacing: 0.2rem;
		color: var(--text-secondary);
		text-transform: uppercase;
		border-bottom: 0.0625rem solid var(--border-subtle);
		padding-bottom: 0.5rem;
	}

	.setting-row {
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
		padding: 0.85rem 1rem;
		background: var(--bg-card);
		border: 0.0625rem solid var(--border-color);
		border-radius: 0.625rem;
	}

	.setting-label {
		font-size: 0.88rem;
		color: var(--text-secondary);
	}

	.community-row {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 0.6rem;
		padding: 0.85rem 1rem;
		background: var(--bg-card);
		border: 0.0625rem solid var(--border-color);
		border-radius: 0.625rem;
	}

	.community-label {
		font-size: 0.88rem;
		color: var(--text-secondary);
	}

	.theme-toggle {
		display: flex;
		gap: 0.25rem;
		background: var(--bg-input);
		border-radius: 0.5rem;
		padding: 0.2rem;
	}

	.theme-option {
		flex: 1;
		padding: 0.35rem 0.5rem;
		border-radius: 0.35rem;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 0.82rem;
		cursor: pointer;
		transition: all 0.18s;
	}

	.theme-option.active {
		background: rgba(0, 102, 255, 0.4);
		color: var(--text-primary);
		border: 0.0625rem solid var(--border-subtle);
	}

	.theme-option:not(.active):hover {
		color: var(--text-primary);
	}

	.community-btn {
		margin-left: auto;
		flex-shrink: 0;
	}

	.community-btn:hover {
		background: rgba(88, 101, 242, 0.35);
		border-color: rgba(88, 101, 242, 0.85);
	}
</style>
