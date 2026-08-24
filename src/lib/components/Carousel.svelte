<script lang="ts">
	import { onMount } from 'svelte';
	import { resizedUrl } from '$lib/api/images';

	const ACTIVE_WIDTH = 0.66;
	const CONTAINER_HEIGHT_REM = 31.25;
	const ASPECT_RATIO = 16 / 9;
	const INACTIVE_WIDTH = 0.26;
	const GAP_REM = 0.9375;
	const VISIBLE_RANGE = 3;

	interface Props {
		screenshots?: string[];
		videos?: string[];
		thumbnails?: string[];
	}

	let { screenshots = [], videos = [], thumbnails = [] }: Props = $props();

	let containerEl: HTMLDivElement | undefined = $state(undefined);
	let trackEl: HTMLDivElement | undefined = $state(undefined);
	let activeIndex: int = 0;
	let items: HTMLElement[] = $state([]);
	let scrollDir = $state<1 | -1>(1);
	let lastScrollTime = 0;
	const SCROLL_DELAY_MS = 50;

	function layoutItems() {
        if (!containerEl) return;
        const rootFontSize = parseFloat(
            getComputedStyle(document.documentElement).fontSize
        );
        const H = CONTAINER_HEIGHT_REM * rootFontSize;
        const activeW = H * ASPECT_RATIO;
        const inactiveW = activeW * 0.4;
        const gap = GAP_REM * rootFontSize;
        const W = containerEl.clientWidth;
        const activeNinW = activeW + inactiveW + gap
        const previewOffset = (W - activeNinW) / 2;
        const pivotX =
            scrollDir > 0
                ? W - activeW - previewOffset
                : previewOffset;
        // Track which indices should be visible
        const visibleIndices = new Set<number>();
        visibleIndices.add(activeIndex);

        for (let i = 1; i <= VISIBLE_RANGE; i++) {
            visibleIndices.add((activeIndex + i) % items.length);
            visibleIndices.add((activeIndex - i + items.length) % items.length);
        }

        // Hide all and disable transitions for non-visible items
        items.forEach((item, idx) => {
            if (!visibleIndices.has(idx)) {
                item.style.transition = 'none';
                item.style.opacity = '0';
                item.style.pointerEvents = 'none';
            } else {
                item.style.transition = '';
            }
        });

        // Force reflow to apply the transition: none immediately
        void containerEl.offsetHeight;

        // Now apply visible item styles with transitions enabled
        items.forEach((item, idx) => {
            if (visibleIndices.has(idx)) {
                item.style.opacity = '1';
                item.style.pointerEvents = 'auto';
            }
        });

        // --- active ---
        const active = items[activeIndex];
        active.style.width = `${activeW}px`;
        active.style.height = `${H}px`;
        active.style.transform = `translateX(${pivotX}px)`;
        active.style.zIndex = '2';

        // --- right side ---
        let x = pivotX + activeW + gap;
        for (let i = 1; i <= items.length / 2; i++) {
            const idx = (activeIndex + i) % items.length;
            const item = items[idx];
            item.style.width = `${inactiveW}px`;
            item.style.height = `${H}px`;
            item.style.transform = `translateX(${x}px)`;
            item.style.zIndex = '1';
            x += inactiveW + gap;
        }

        // --- left side ---
        x = pivotX - gap;
        for (let i = 1; i <= items.length / 2; i++) {
            const idx = (activeIndex - i + items.length) % items.length;
            const item = items[idx];
            x -= inactiveW;
            item.style.width = `${inactiveW}px`;
            item.style.height = `${H}px`;
            item.style.transform = `translateX(${x}px)`;
            item.style.zIndex = '1';
            x -= gap;
        }
    }

	function handleWheel(e: WheelEvent) {
		const now = Date.now();
		
		// Check if enough time has passed since last scroll
		if (now - lastScrollTime < SCROLL_DELAY_MS) {
			return; // Ignore this scroll event
		}
		
		e.preventDefault();
		if (items.length === 0) return;

		const oldActive = activeIndex;
		scrollDir = e.deltaY > 0 ? 1 : -1;

		activeIndex =
			(activeIndex + scrollDir + items.length) % items.length;

		layoutItems();

		// video handling
		items[oldActive]?.querySelector('video')?.pause();
		items[activeIndex]?.querySelector('video')?.play();
		
		// Update last scroll time
		lastScrollTime = now;
	}

	// Build media items array
	let mediaItems = $derived.by(() => {
		const result: Array<{ type: 'video' | 'image'; src: string; thumbnail?: string }> = [];
		const totalScreenshots = screenshots?.length || 0;
		const totalVideos = videos?.length || 0;
		const totalMedia = totalScreenshots + totalVideos;

		if (totalMedia === 0) return result;

		const videosInterval = totalVideos > 0 ? Math.floor(totalScreenshots / totalVideos) : Infinity;
		let videoIndex = 0;
		let screenshotIndex = 0;

		for (let i = 0; i < totalMedia; i++) {
			const isVideo = totalVideos > 0 && videosInterval > 0 && i % (videosInterval + 1) === 0 && videoIndex < totalVideos;

			if (isVideo && videos) {
				result.push({
					type: 'video',
					src: videos[videoIndex],
					thumbnail: resizedUrl(thumbnails?.[videoIndex], { w: 480 })
				});
				videoIndex++;
			} else if (screenshotIndex < totalScreenshots && screenshots) {
				// Every item is mounted in the DOM at once (only visibility is
				// toggled), so all screenshots load up front - resize them
				// down from full resolution to what the carousel actually
				// displays them at.
				result.push({
					type: 'image',
					src: resizedUrl(screenshots[screenshotIndex], { w: 960 }) ?? screenshots[screenshotIndex]
				});
				screenshotIndex++;
			}
		}

		return result;
	});

	function toggleVideo(button: HTMLButtonElement) {
		const videoItem = button.closest('.carousel-item') as HTMLElement;
		const video = videoItem?.querySelector('video');
		if (!video) return;

		if (video.paused) {
			// Pause all other videos
			document.querySelectorAll('.carousel-item video').forEach((v) => {
				if (v !== video) {
					(v as HTMLVideoElement).pause();
					const otherButton = v.parentElement?.querySelector('.play-btn') as HTMLButtonElement;
					if (otherButton) updatePlayButton(otherButton, false);
				}
			});
			video.play();
			updatePlayButton(button, true);
		} else {
			video.pause();
			updatePlayButton(button, false);
		}
	}

	function updatePlayButton(button: HTMLButtonElement, isPlaying: boolean) {
		const icon = button.querySelector('.play-icon');
		if (!icon) return;

		if (isPlaying) {
			icon.innerHTML = '<rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect>';
		} else {
			icon.innerHTML = '<polygon points="5,3 19,12 5,21"></polygon>';
		}
	}

	onMount(() => {
		if (!trackEl || !containerEl) return;

		items = Array.from(trackEl.children) as HTMLElement[];
		layoutItems();

		window.addEventListener('resize', layoutItems);
		return () => window.removeEventListener('resize', layoutItems);
	})
</script>

{#if mediaItems.length > 0}
	<div class="carousel-container" bind:this={containerEl} onwheel={handleWheel}>
		<div class="carousel-track" bind:this={trackEl}>
			{#each mediaItems as item, i}
				<div class="carousel-item" class:video={item.type === 'video'}>
					{#if item.type === 'video'}
						<video
							class="media-content"
							muted
							loop
							autoplay={i === 0}
							poster={item.thumbnail}
						>
							<source src={item.src} type="video/mp4" />
							<track kind="captions" />
						</video>
						<div class="video-overlay">
							<button
								type="button"
								class="play-btn"
								aria-label="Play or pause video"
								onclick={(e) => toggleVideo(e.currentTarget as HTMLButtonElement)}
							>
								<svg class="play-icon" viewBox="0 0 24 24" aria-hidden="true">
									<polygon points="5,3 19,12 5,21"></polygon>
								</svg>
							</button>
						</div>
					{:else}
						<img class="media-content" src={item.src} alt="Screenshot {i + 1}" />
					{/if}
				</div>
			{/each}
		</div>
	</div>
{/if}

<style>
/*	.carousel-track {
		gap: 1.875rem;
*/

	.carousel-container {
		position: relative;
		overflow: hidden;
		height: 31.25rem;
		width: 100%;
		border-radius: 1.25rem;
		box-shadow: 0 1.25rem 2.5rem rgba(0, 0, 0, 0.2);
	}

	.carousel-track {
		position: relative;
		height: 100%;
	}

	.carousel-item {
		position: absolute;
		top: 0;
		height: 100%;
		/*transition:
			transform 0.3s ease,
			width 0.3s ease;*/
		transition:
			transform 0.8s cubic-bezier(0.22, 1, 0.36, 1),
			width 0.8s cubic-bezier(0.22, 1, 0.36, 1),
			box-shadow 0.8s ease;
		will-change: transform, width;
		/*width: calc(26% - 0.9375rem);*/
	}



	.carousel-item:hover {
		box-shadow: 0 0.9375rem 1.875rem rgba(0, 0, 0, 0.3);
	}

	.media-content {
		width: 100%;
		height: 100%;
		object-fit: cover;
		border-radius: 0.9375rem;
		transition: transform 0.5s ease;
	}

	/*.carousel-item:hover .media-content {
		transform: scale(1.02);
	}*/

	.video-overlay {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.3);
		opacity: 0;
		transition: opacity 0.3s ease;
	}

	.carousel-item.video:hover .video-overlay {
		opacity: 1;
	}

	.play-btn {
		width: 3.75rem;
		height: 3.75rem;
		border-radius: 50%;
		background: rgba(255, 255, 255, 0.9);
		border: none;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.3s ease;
		backdrop-filter: blur(0.625rem);
		padding: 0;
		margin: 0;
	}

	.play-btn:hover {
		transform: scale(1.1);
		background: rgba(255, 255, 255, 1);
	}

	.play-icon {
		width: 1.25rem;
		height: 1.25rem;
		fill: #333;
		margin-left: 0.125rem;
	}
</style>
