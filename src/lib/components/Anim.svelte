<!-- BlurIn.svelte -->
<script>
  import { blur } from 'svelte/transition';
  export let amount = '2rem';
  export let opacity = 0.5;
  export let duration = 400;
</script>

<div in:blur={{ amount, opacity, duration }}>
  <slot />
</div>

