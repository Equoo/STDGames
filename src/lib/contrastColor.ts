// lib/contrastColor.ts
import { sampleBehind } from '$lib/sampleBackground';

interface Options {
  container?: HTMLElement;   // defaults to document.body
  scroll?: HTMLElement;   // defaults to document.body
  dark?:  string;            // defaults to '#111111'
  light?: string;            // defaults to '#ffffff'
}

export function autoContrast(el: HTMLElement, opts: Options = {}) {
  let { container = document.body, scroll = document.body, dark = '#111111', light = '#ffffff' } = opts;
  let pending = false;
  let rafId: number;
	
  console.log("Load auto contrast", el, container, scroll);

  async function refresh() {
    if (pending) return;
    pending = true;
	console.log("Try to get back");
    const lum = await sampleBehind(el, container);
	console.log("Luminance is ", lum);
    el.style.color = lum > 0.179 ? dark : light;
    pending = false;
  }

  // debounce resize via rAF
  function onResize() {
    cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(refresh);
  }
  function onScroll() {
    cancelAnimationFrame(rafId);
    rafId = requestAnimationFrame(refresh);
  }

  const ro = new ResizeObserver(onResize);
  ro.observe(container);
  
  scroll.addEventListener('scroll', onScroll, { passive: true });

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

function getScrollParent(el: HTMLElement): HTMLElement | Window {
  let node: HTMLElement | null = el.parentElement;
  while (node) {
    const { overflow, overflowY, overflowX } = getComputedStyle(node);
    if (/auto|scroll/.test(overflow + overflowY + overflowX)) return node;
    node = node.parentElement;
  }
  return window; // fallback: page scroll
}
