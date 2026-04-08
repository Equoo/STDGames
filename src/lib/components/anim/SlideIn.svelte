<!-- SlideIn.svelte -->
<script>
  import { slide } from 'svelte/transition';
  import { cubicOut, linear } from 'svelte/easing';
  export let easing = cubicOut;
  export let delay = 0;
  export let duration = 600;
  export let axis = 'y';
</script>

<div in:slide={{ delay, easing, duration, axis }}>
  <slot />
</div>

