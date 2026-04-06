<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { theme } from '$lib/stores/gameStore';
	import { onMount } from 'svelte';
	export const ssr = false;

	let { children } = $props();

	onMount(() => {
		const stored = localStorage.getItem('theme') ?? 'dark';
		document.documentElement.setAttribute('data-theme', stored);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children?.()}
