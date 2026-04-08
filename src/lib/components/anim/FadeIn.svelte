<!-- FadeIn.svelte -->
<script>
  import { fade } from 'svelte/transition';
  import { cubicIn, cubicInOut, cubicOut, linear } from 'svelte/easing';
  export let delay = 25;
  export let easing = cubicInOut;
  export let duration = 500;
</script>

<div in:fade={{ delay, easing, duration }}>
  <slot />
</div>

