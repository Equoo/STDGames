<!-- FadeInOut.svelte -->
<script>
  import { fade } from 'svelte/transition';
  import { cubicOut, linear } from 'svelte/easing';
  export let delay = 25;
  export let easing = cubicOut;
  export let duration = 400;
</script>

<div in:fade={{ delay, easing, duration }}>
  <slot />
</div>

