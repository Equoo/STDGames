// lib/sampleBackground.ts
import html2canvas from 'html2canvas-pro';

function toLinear(v: number) {
  v /= 255;
  return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
}

function luminance(r: number, g: number, b: number) {
  return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
}

// Cache: container → { canvas, expiry }
const snapshotCache = new WeakMap<HTMLElement, { canvas: HTMLCanvasElement; expiry: number }>();
const CACHE_TTL = 200; // ms

// Debug overlay: one preview per container
const debugPreviews = new WeakMap<HTMLElement, HTMLCanvasElement>();
const DEBUG = false; // flip to false to disable
const DEBUG_SCALE = 0.25;

function upsertDebugPreview(container: HTMLElement, snapshot: HTMLCanvasElement) {
  let preview = debugPreviews.get(container);

  if (!preview) {
    preview = document.createElement('canvas');
    preview.style.cssText = `
      position: fixed;
      bottom: 8px;
      right: 8px;
      z-index: 99999;
      border: 2px solid magenta;
      border-radius: 4px;
      opacity: 0.85;
      pointer-events: none;
      image-rendering: pixelated;
    `;
    document.body.appendChild(preview);
    debugPreviews.set(container, preview);
  }

  preview.width  = Math.round(snapshot.width  * DEBUG_SCALE);
  preview.height = Math.round(snapshot.height * DEBUG_SCALE);
  const ctx = preview.getContext('2d')!;
  ctx.drawImage(snapshot, 0, 0, preview.width, preview.height);
}

const snapshotQueues = new WeakMap<HTMLElement, Promise<void>>();

function withContainerLock<T>(container: HTMLElement, fn: () => Promise<T>): Promise<T> {
  const current = snapshotQueues.get(container) ?? Promise.resolve();
  let resolve!: () => void;
  const next = new Promise<void>(r => resolve = r);
  snapshotQueues.set(container, next);

  return current.then(fn).finally(resolve);
}

// usage
async function getSnapshot(container: HTMLElement): Promise<HTMLCanvasElement> {
  const cached = snapshotCache.get(container);
  if (cached && Date.now() < cached.expiry) return cached.canvas;

  return withContainerLock(container, async () => {
    // re-check cache — a previous waiter may have already populated it
    const cached = snapshotCache.get(container);
    if (cached && Date.now() < cached.expiry) return cached.canvas;

	console.debug("HTML2CANVAS", cached);

    const canvas = await html2canvas(container, {
		allowTaint: true,
		useCORS: true,
		logging: false,
		imageSmoothing: false,
		ignoreElements: (element) => {
			return element.classList.contains('no-capture');
		},
		onclone: (clonedDoc, clonedEl) => {
		  const scrollables = document.getElementsByClassName('scrollable');
		  const clonedScrollables = clonedDoc.getElementsByClassName('scrollable');

		  for (let i = 0; i < scrollables.length; i++) {
			const real = scrollables[i] as HTMLElement;
			const clone = clonedScrollables[i] as HTMLElement;
			const scrollTop  = real.scrollTop;
			const scrollLeft = real.scrollLeft;

			// Force overflow visible so clipped content is rendered
			clone.style.overflow = 'visible';

			// Shift the inner content up by the scroll offset
			// const inner = clone.firstElementChild as HTMLElement | null;
			// if (inner) {
			//   inner.style.transform = `translate(${-scrollLeft}px, ${-scrollTop}px)`;
			// }
			clone.style.transform = `translate(${-scrollLeft}px, ${-scrollTop}px)`;
		  }
		},
		scale: 1,
	  });
    snapshotCache.set(container, { canvas, expiry: Date.now() + CACHE_TTL });
	if (DEBUG) upsertDebugPreview(container, canvas)
    return canvas;
  });
}

function sampleLuminance(
  canvas: HTMLCanvasElement,
  el: HTMLElement,
  container: HTMLElement
): number {
  const cRect = container.getBoundingClientRect();
  const eRect = el.getBoundingClientRect();
  const x = Math.round(eRect.left - cRect.left);
  const y = Math.round(eRect.top  - cRect.top);
  const w = Math.max(1, Math.round(eRect.width));
  const h = Math.max(1, Math.round(eRect.height));

  const ctx = canvas.getContext('2d')!;
  const { data } = ctx.getImageData(x, y, w, h);
  let rS = 0, gS = 0, bS = 0, n = 0;
  for (let i = 0; i < data.length; i += 4) {
    rS += data[i]; gS += data[i + 1]; bS += data[i + 2]; n++;
  }

  const lum = luminance(rS / n, gS / n, bS / n);

  if (DEBUG) {
    // Draw a magenta rect on the debug preview showing the sampled region
    const preview = debugPreviews.get(container);
    if (preview) {
      const pCtx = preview.getContext('2d')!;
      pCtx.strokeStyle = 'magenta';
      pCtx.lineWidth = 1;
      pCtx.strokeRect(
        x * DEBUG_SCALE,
        y * DEBUG_SCALE,
        w * DEBUG_SCALE,
        h * DEBUG_SCALE
      );
      // Label with luminance value
      pCtx.fillStyle = 'magenta';
      pCtx.font = `${Math.max(8, 10 * DEBUG_SCALE * 4)}px monospace`;
      pCtx.fillText(lum.toFixed(3), x * DEBUG_SCALE + 2, y * DEBUG_SCALE + 10);
    }
  }

  return lum;
}

export async function sampleBehind(
  el: HTMLElement,
  container: HTMLElement = document.body
): Promise<number> {
  const canvas = await getSnapshot(container);
  return sampleLuminance(canvas, el, container);
}

export function invalidateSnapshot(container: HTMLElement) {
  snapshotCache.delete(container);
}
