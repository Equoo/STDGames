<!-- FlyIn.svelte -->
<script>
  import { fly } from 'svelte/transition';
  import { linear, bounceIn, cubicOut } from 'svelte/easing';
  export let delay = 0;
  export let easing = cubicOut;
  export let x = -300;
  export let y = 0;
  export let opacity = 0;
  export let duration = 400;
</script>
<div 
  in:fly={{ delay, easing, x, y , opacity, duration }}>
  <slot />
</div>

