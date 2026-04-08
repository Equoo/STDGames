<script lang="ts">
	import { currentView } from "$lib/stores/gameStore";
	import lettersLogo from "$lib/assets/icons/stdgames_letters_only.png";
	let mx = $state(0);
	let my = $state(0);

	function handleMousemove(e: MouseEvent) {
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		mx = e.clientX - rect.left;
		my = e.clientY - rect.top;
	}

	function go(view: "home" | "library" | "settings") {
		currentView.set(view);
	}
</script>

<div class="topbar">
	<div class="topbar-content">
		<!-- Logo with mouse-tracking glow -->
		<div
			class="logo-wrapper"
			onmousemove={handleMousemove}
			style="--mx: {mx}px; --my: {my}px; --logo-url: url('{lettersLogo}');"
			role="img"
			aria-label="STDGames"
		>
			<img src={lettersLogo} alt="STDGames" class="logo" />
			<div class="glow-layer"></div>
		</div>
			<nav class="nav">
				<button
					class="nav-btn"
					class:active={$currentView === "home"}
					onclick={() => go("home")}>Home</button
				>
				<button
					class="nav-btn"
					class:active={$currentView === "library"}
					onclick={() => go("library")}>Library</button
				>
				<button
					class="nav-btn"
					class:active={$currentView === "settings"}
					onclick={() => go("settings")}>Settings</button
				>
			</nav>
	</div>
</div>

<style>
	.topbar {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		display: flex;
		height: 7.125rem; /* 3.125rem nav + 4rem header zone */
		padding: 0 1rem;
		background: transparent;
		backdrop-filter: blur(1.5rem) saturate(1.6);
		-webkit-backdrop-filter: blur(1.5rem) saturate(1.6);
		align-items: flex-start;
		z-index: 3;
		pointer-events: none;
	}

	.topbar-content {
		width: 100%;
		display: flex;
		align-items: center;
		height: 3.125rem;
		gap: 1.25rem;
		pointer-events: all;
	}

	/* ── Logo ── */
	.logo-wrapper {
		position: relative;
		flex-shrink: 0;
		height: 5rem;
		width: 5rem;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.logo {
		height: 100%;
		width: 100%;
		object-fit: contain;
		display: block;
	}

	.glow-layer {
		position: absolute;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 0.25s ease;

		background: radial-gradient(
			circle 1rem at var(--mx) var(--my),
			rgba(255, 255, 255, 0.8) 0%,
			rgba(255, 0, 180, 0.85) 35%,
			rgba(0, 180, 255, 0.5) 65%,
			transparent 100%
		);

		/* Clip gradient to letter shapes only */
		mask-image: var(--logo-url);
		mask-size: contain;
		mask-repeat: no-repeat;
		mask-position: center;
		-webkit-mask-image: var(--logo-url);
		-webkit-mask-size: contain;
		-webkit-mask-repeat: no-repeat;
		-webkit-mask-position: center;

		mix-blend-mode: color-dodge;
	}

	:global([data-theme="dark"] .glow-layer) {
		background: radial-gradient(
			circle 1.2rem at var(--mx) var(--my),
			rgba(255, 255, 255, 0.8) 0%,
			rgba(47, 255, 0, 0.5) 35%,
			rgba(255, 255, 255, 1) 65%,
			transparent 100%
		);
	}
	:global([data-theme="light"] .glow-layer) {
		background: radial-gradient(
			circle 1.2rem at var(--mx) var(--my),
			rgba(255, 255, 255, 0.8) 0%,
			rgba(255, 0, 180, 0.5) 35%,
			rgba(0, 180, 255, 0.5) 65%,
			transparent 100%
		);
	}
	.logo-wrapper:hover .glow-layer {
		opacity: 1;
	}

	/* ── Nav ── */
	.nav {
		display: flex;
		align-items: center;
		height: 100%;
		gap: 0.25rem;
	}

	.nav-btn {
		position: relative;
		height: 100%;
		padding: 0 0.9rem;
		background: transparent;
		border: none;
		border-radius: 0.3rem;
		font-family: "Brunson", sans-serif;
		font-size: 1rem;
		letter-spacing: 0.125rem;
		color: var(--text-secondary);
		cursor: pointer;
		transition:
			color 0.2s ease,
			background 0.2s ease;
	}

	.nav-btn:hover {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.06);
	}

	.nav-btn.active {
		color: var(--text-primary);
	}

	/* Animated underline indicator */
	.nav-btn::after {
		content: "";
		position: absolute;
		bottom: 0.3rem;
		left: 50%;
		transform: translateX(-50%) scaleX(0);
		width: calc(100% - 1rem);
		height: 0.15rem;
		border-radius: 0.1rem;
		background: linear-gradient(
			to right,
			rgba(0, 102, 255, 0.9),
			rgba(255, 0, 204, 0.9)
		);
		opacity: 0;
		transition:
			transform 0.25s cubic-bezier(0.4, 0, 0.2, 1),
			opacity 0.25s ease;
	}

	.nav-btn:hover::after {
		transform: translateX(-50%) scaleX(0.5);
		opacity: 0.45;
	}

	.nav-btn.active::after {
		transform: translateX(-50%) scaleX(1);
		opacity: 1;
	}
</style>
