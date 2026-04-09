// lib/sampleBackground.ts
// Snapshots the DOM behind an element and returns
// the average WCAG luminance of that region.

import html2canvas from 'html2canvas';

function toLinear(v: number) {
  v /= 255;
  return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
}
function luminance(r: number, g: number, b: number) {
  return 0.2126 * toLinear(r) + 0.7152 * toLinear(g) + 0.0722 * toLinear(b);
}

export async function sampleBehind(
  el: HTMLElement,
  container: HTMLElement = document.body
): Promise<number> {

  // 1. hide the element so it doesn't occlude its own background
  const prev = el.style.visibility;
  el.style.visibility = 'hidden';

  // 2. snapshot the container
  const snapshot = await html2canvas(container, {
    useCORS: true,
    logging: false,
    scale: 1,           // scale:1 for speed; use devicePixelRatio for accuracy
  });

  el.style.visibility = prev;

  // 3. locate the element relative to the container
  const cRect = container.getBoundingClientRect();
  const eRect = el.getBoundingClientRect();
  const x = Math.round(eRect.left - cRect.left);
  const y = Math.round(eRect.top  - cRect.top);
  const w = Math.max(1, Math.round(eRect.width));
  const h = Math.max(1, Math.round(eRect.height));

  // 4. sample pixels and average
  const ctx = snapshot.getContext('2d')!;
  const { data } = ctx.getImageData(x, y, w, h);
  let rS = 0, gS = 0, bS = 0, n = 0;
  for (let i = 0; i < data.length; i += 4) {
    rS += data[i]; gS += data[i+1]; bS += data[i+2]; n++;
  }
  return luminance(rS/n, gS/n, bS/n);
}
