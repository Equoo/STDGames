<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { theme } from '$lib/stores/gameStore';
	import { onMount } from 'svelte';

	let { children } = $props();

	onMount(() => {
		const stored = localStorage.getItem('theme') ?? 'dark';
		document.documentElement.setAttribute('data-theme', stored);

		// Show scrollbar while scrolling, hide after idle
		const timers = new WeakMap<HTMLElement, ReturnType<typeof setTimeout>>();

		function onScroll(e: Event) {
			const el = e.target as HTMLElement;
			if (!(el instanceof HTMLElement)) return;
			el.classList.add('is-scrolling');
			const prev = timers.get(el);
			if (prev) clearTimeout(prev);
			timers.set(el, setTimeout(() => {
				el.classList.remove('is-scrolling');
				timers.delete(el);
			}, 1000));
		}

		document.addEventListener('scroll', onScroll, true);
		return () => document.removeEventListener('scroll', onScroll, true);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children?.()}
