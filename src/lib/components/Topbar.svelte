<script lang="ts">
	import { currentView } from '$lib/stores/gameStore';
	import { addDesktopIcon, openUrl } from '$lib/api/system';
	import logo from '$lib/assets/icons/stdgames.png';
	import discordLogo from '$lib/assets/icons/discord_100x100.png';
	import settingsIcon from '$lib/assets/icons/settings_white.png';

	function handleLibraryClick() {
		currentView.set('library');
	}

	function handleDiscordClick() {
		openUrl('https://discord.gg/YR7fwGy5D7');
	}

	function handleSettingsClick() {
		console.log('Settings functionality to be implemented');
	}
</script>

<div class="topbar">
	<div class="topbar-content">
		<div class="left-group">
			<img src={logo} alt="Logo" class="logo" />
			<button class="topbar-btn" onclick={addDesktopIcon}>Add to desktop</button>
			<button
				class="topbar-btn"
				class:active={$currentView === 'library'}
				onclick={handleLibraryClick}
			>
				Library
			</button>
		</div>

		<div class="right-group">
			<button class="topbar-btn icon-btn" onclick={handleDiscordClick}>
				<img src={discordLogo} alt="Discord" />
			</button>
			<button class="topbar-btn icon-btn" onclick={handleSettingsClick}>
				<img src={settingsIcon} alt="Settings" />
			</button>
		</div>
	</div>
</div>

<style>
	.topbar {
		position: sticky;
		top: 0;
		display: flex;
		padding: 0.3125rem 0.625rem;
		background: rgba(0, 0, 0, 0.8);
		align-items: center;
		justify-content: space-between;
		z-index: 100;
	}

	.topbar .logo {
		width: 2.5rem;
		height: 2.5rem;
		object-fit: contain;
	}

	.topbar-content {
		width: 100%;
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.left-group,
	.right-group {
		display: flex;
		align-items: center;
		gap: 0.9375rem;
	}

	.right-group {
		margin-left: auto;
	}

	.topbar-btn {
		height: 2.5rem;
		padding: 0.5rem 1rem;
		border-radius: 0.625rem;
		background: rgba(24, 24, 56, 0.5);
		border: 0.125rem solid rgba(0, 102, 255, 0.212);
		font-family: 'Brunson', sans-serif;
		font-size: 1rem;
		letter-spacing: 0.125rem;
		color: white;
		cursor: pointer;
		transition: all 0.2s;
	}

	.topbar-btn:hover {
		background: rgba(0, 102, 255, 0.26);
		border-color: rgba(255, 0, 204, 0.5);
	}

	.topbar-btn.active {
		border-color: rgba(255, 0, 204, 0.65);
	}

	.icon-btn {
		padding: 0.25rem;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.icon-btn img {
		width: 1.75rem;
		height: 1.75rem;
		object-fit: contain;
		transition: transform 0.2s ease-out;
	}

	.icon-btn:hover img {
		transform: scale(1.2);
	}
</style>
