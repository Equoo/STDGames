// lib/contrastColor.ts
import { sampleBehind } from '$lib/sampleBackground';
import { theme } from "$lib/stores/gameStore";
import type { Theme } from "$lib/stores/gameStore";

interface Options {
  container?: HTMLElement;   // defaults to document.body
  scroll?: HTMLElement;   // defaults to document.body
  dark?:  string;            // defaults to '#111111'
  light?: string;            // defaults to '#ffffff'
}

function isAttached(el: unknown): el is Element {
  return el instanceof Node && document.body.contains(el);
}

export function autoContrast(el: HTMLElement, opts: Options = {}) {
  let { container = document.body, scroll = document.body, dark = "var(--text-primary-light)", light = "var(--text-primary-dark)" } = opts;

  let pending = false;
  let rafId: number;

	if (!isAttached(el)) {
	  console.warn('Element not in DOM');
	  return;
	}
	if (!isAttached(container)) {
	  console.warn('Container not in DOM');
	  return;
	}

  async function refresh() {
    if (pending) return;
	if (!isAttached(el)) {
	  console.warn('Element not in DOM');
	  return;
	}
	if (!isAttached(container)) {
	  console.warn('Container not in DOM');
	  return;
	}
	console.debug("Pute de", container);
  	// if (container == document.body) return;
    pending = true;
    const lum = await sampleBehind(el, container);
	console.debug(lum);
    el.style.color = lum > 0.179 ? dark : light;
    el.style.transition = "color 0.2s ease";
    pending = false;
  }

  let time;
  function onScroll() {
	if (time) {
		clearTimeout(time);
	}
	time = setTimeout(() => {
		cancelAnimationFrame(rafId);
		rafId = requestAnimationFrame(refresh);
	}, 100);
  }

  let stopped = false;

  async function loop() {
    while (!stopped) {
	  const scrollables = document.getElementsByClassName('scrollable');

	  for (let i = 0; i < scrollables.length; i++) {
		const real = scrollables[i] as HTMLElement;
		real.removeEventListener('scroll', onScroll);
		real.addEventListener('scroll', onScroll, { passive: true });
	  }
      await new Promise(r => setTimeout(r, 500));
    }
  }

  loop();

  const observer = new MutationObserver(() => {
	  refresh();
  });

  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['class', 'data-theme'],
  });

  refresh();

  return {
    update(newOpts: Options) {
      ({ container = document.body, dark = "var(--text-primary-light)", light = "var(--text-primary-dark)" } = newOpts);
      refresh();
    },
    destroy() {
  		stopped = true;
		observer.disconnect();
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
