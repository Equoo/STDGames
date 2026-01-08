<script lang="ts">
	import logo from '$lib/assets/icons/stdgames.png';
	import discord_logo from '$lib/assets/icons/discord_100x100.png';

	let ffg = "ggfg";
</script>

<div class="topbar">
	<div class="topbar-content">
		<div class="left-group">
			<img src={logo} alt="Logo" class="logo">
			<button id="addicon-button">Add to desktop</button>
			<button id="library-button" class="active">Library</button>
		</div>

		<div class="right-group">
			<button id="aboutus-button">
				<img src={discord_logo} alt="Discord">
			</button>
			<button id="settings-button">
				<img src="/resources/icons/settings_white.png" alt="Settings">
			</button>
		</div>
	</div>
</div>
