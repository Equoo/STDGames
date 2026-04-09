<!-- ContrastText.svelte -->
<script lang="ts">
  import { autoContrast } from '$lib/contrastColor';

  export let container: HTMLElement | undefined = undefined;
  export let tag = 'span';

  // resolved after mount — falls back to body if not provided
  $: opts = { container };
</script>

<svelte:element this={tag} use:autoContrast={opts}>
  <slot />
</svelte:element>
