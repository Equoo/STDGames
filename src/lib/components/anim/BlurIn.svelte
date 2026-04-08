<!-- BlurIn.svelte -->
<script>
  import { blur } from 'svelte/transition';
  export let amount = '1rem';
  export let delay = 25;
  export let opacity = 0;
  export let duration = 400;
</script>

<div in:blur={{ amount, delay, opacity, duration }}>
  <slot />
</div>

