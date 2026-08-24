interface ResizeOptions {
	w?: number;
	h?: number;
	q?: number;
}

const CDN_SEGMENT = '/cdn/';

/**
 * Rewrites a `/cdn/...` asset URL into a `/img/...` URL served by the
 * server's resize endpoint, so views rendering many assets at once (the
 * library grid, screenshot carousels) request appropriately-sized variants
 * instead of full-resolution originals.
 *
 * Leaves the URL untouched if it isn't a `/cdn/` asset (mock data, external
 * URLs) or no resize options are given.
 */
export function resizedUrl(url: string | undefined, opts: ResizeOptions): string | undefined {
	if (!url) return url;

	const cdnIndex = url.indexOf(CDN_SEGMENT);
	if (cdnIndex === -1) return url;

	const params = new URLSearchParams();
	if (opts.w) params.set('w', Math.round(opts.w).toString());
	if (opts.h) params.set('h', Math.round(opts.h).toString());
	if (opts.q) params.set('q', Math.round(opts.q).toString());
	if ([...params].length === 0) return url;

	const rewritten =
		url.slice(0, cdnIndex) + '/img/' + url.slice(cdnIndex + CDN_SEGMENT.length);
	return `${rewritten}?${params.toString()}`;
}
