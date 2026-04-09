// lib/contrastColor.ts
import { sampleBehind } from '$lib/sampleBackground';

interface Options {
  container?: HTMLElement;   // defaults to document.body
  dark?:  string;            // defaults to '#111111'
  light?: string;            // defaults to '#ffffff'
}

export function autoContrast(el: HTMLElement, opts: Options = {}) {
  let { container = document.body, dark = '#111111', light = '#ffffff' } = opts;
  let pending = false;
  let rafId: number;

  async function refresh() {
    if (pending) return;
    pending = true;
    const lum = await sampleBehind(el, container);
    el.style.color = lum > 0.179 ? dark : light;
    pending = false;
  }

  // debounce resize via rAF
  function onResize() {
    cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(refresh);
  }

  const ro = new ResizeObserver(onResize);
  ro.observe(container);
  refresh();   // initial sample

  return {
    update(newOpts: Options) {
      ({ container = document.body, dark = '#111111', light = '#ffffff' } = newOpts);
      refresh();
    },
    destroy() {
      ro.disconnect();
      cancelAnimationFrame(rafId);
    }
  };
}
