<script>
  export let image = '';
  export let text = '';
  export let className = '';
  export let direction = 'bottom'; // 👈 new
</script>

<div
  class={`prog-blur ${className}`}
  data-direction={direction}
  style={`--image: url(${image})`}
>
  <div class="text">{text}</div>
</div>

<style>
  .prog-blur {
    width: 240px;
    aspect-ratio: 3 / 4;

    border-radius: 1.5rem;
    position: relative;
    overflow: clip;
    isolation: isolate;

    backdrop-filter: contrast(0);
    transition: all 300ms;

    display: inline-block;
  }

  .text {
    position: absolute;
    left: 1.5rem;
    z-index: 99;
    pointer-events: none;

    font-size: 1.5rem;
    font-weight: 800;
    color: white;
    mix-blend-mode: overlay;

    filter: blur(24px);
    opacity: 0;

    transition: all 300ms;
  }

  /* 👇 FROM BOTTOM (default) */
  .prog-blur[data-direction="bottom"] .text {
    bottom: 1.5rem;
    transform: translate3d(0, calc(100% + 0.5rem), 0);
  }

  /* 👇 FROM TOP (reverse) */
  .prog-blur[data-direction="top"] .text {
    top: 1.5rem;
    transform: translate3d(0, calc(-100% - 0.5rem), 0);
  }

  /* 👇 Hover animation */
  .prog-blur:hover .text {
    transform: translate3d(0, 0, 0);
    filter: blur(0);
    opacity: 1;
  }

  .prog-blur::before {
    content: "";
    position: absolute;
    inset: 0;
    z-index: -9;

    background: var(--image) no-repeat center / cover;
    transition: all 200ms;
  }

  .prog-blur::after {
    content: "";
    position: absolute;
    inset: 0;
    z-index: 9;

    background: var(--image) no-repeat center / cover;

    mask: linear-gradient(to top, transparent 4rem, white 50%);
    transition: all 200ms;
  }

  .prog-blur:hover::before {
    filter: blur(24px);
  }
</style>