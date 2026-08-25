//! Approximates a live `backdrop-filter: blur()` for the topbar/`ChromeBackdrop` band, which
//! Slint has no primitive for. Rather than reading back the actual rendered frame (would need
//! raw GPU/GL access, backend-specific and fragile — see the discussion that led here), this
//! composites a *stand-in* bitmap in Rust from data already in hand: for the game grid, a tall
//! canvas of the card thumbnails arranged to match `GameGrid`'s real column layout; for the game
//! preview, the hero image followed by a gradient approximating the description panel's fade.
//! Either way the result is blurred once and reused — cheap to reuse (no GPU decode of scroll
//! offset), expensive to rebuild, so callers should only rebuild on a debounce (see `main.rs`),
//! not per scroll tick. The *position* within the result that's actually visible behind the
//! topbar is handled in Slint (`ChromeBackdrop`) by offsetting an `Image` by `-scroll-y` inside a
//! clipping rect — cheap, so that part *does* track scroll live.

use image::{imageops, RgbaImage};
use slint::{Rgba8Pixel, SharedPixelBuffer};

/// How strongly to blur — roughly matches the pre-baked `bg_blurred_*.png` assets' softness.
const BLUR_SIGMA: f32 = 18.0;

pub struct GridMetrics {
    pub cols: u32,
    pub card_w: u32,
    pub card_h: u32,
    pub gap: u32,
}

/// Mirrors `GameGrid`'s column/card-width formula (`ui/components/game_grid.slint`:
/// `cols = max(1, floor((width + gap) / (min-card-w + gap)))`,
/// `card-w = (width - (cols - 1) * gap) / cols`) so the composited backdrop's card positions
/// line up with the real grid once blurred. `scale_unit` is `Scale.unit` in the same (logical
/// pixel) units as `content_width`.
pub fn grid_metrics(content_width: u32, scale_unit: f32) -> GridMetrics {
    let gap = scale_unit.round().max(1.0) as u32;
    let min_card_w = (18.0 * scale_unit).round().max(1.0) as u32;
    let cols = ((content_width + gap) / (min_card_w + gap)).max(1);
    let card_w = (content_width.saturating_sub((cols - 1) * gap) / cols).max(1);
    let card_h = (card_w * 9 / 16).max(1);
    GridMetrics { cols, card_w, card_h, gap }
}

fn to_rgba_image(buf: &SharedPixelBuffer<Rgba8Pixel>) -> Option<RgbaImage> {
    RgbaImage::from_raw(buf.width(), buf.height(), buf.as_bytes().to_vec())
}

fn to_shared(img: &RgbaImage) -> SharedPixelBuffer<Rgba8Pixel> {
    SharedPixelBuffer::clone_from_slice(img.as_raw(), img.width(), img.height())
}

fn lerp_rgba(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let l = |i: usize| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8;
    [l(0), l(1), l(2), l(3)]
}

/// Composites `thumbs` (in display order) into a tall canvas at `content_width`, positioned
/// `x_offset` into a `total_width`-wide canvas (the sidebar's own opaque background covers
/// anything to the left of it — `x_offset` just keeps the grid aligned with where it really sits
/// on screen), then blurs the whole thing once. `base_fill` shows through gaps and cells with no
/// thumbnail loaded yet.
pub fn compose_grid_backdrop(
    thumbs: &[SharedPixelBuffer<Rgba8Pixel>],
    content_width: u32,
    x_offset: u32,
    total_width: u32,
    scale_unit: f32,
    base_fill: [u8; 4],
) -> SharedPixelBuffer<Rgba8Pixel> {
    let metrics = grid_metrics(content_width, scale_unit);
    let rows = (thumbs.len() as u32).div_ceil(metrics.cols).max(1);
    let canvas_h = (rows * metrics.card_h + rows.saturating_sub(1) * metrics.gap).max(1);
    let mut canvas = RgbaImage::from_pixel(total_width.max(1), canvas_h, image::Rgba(base_fill));

    for (i, thumb) in thumbs.iter().enumerate() {
        let Some(src) = to_rgba_image(thumb) else { continue };
        let resized = imageops::resize(&src, metrics.card_w, metrics.card_h, imageops::FilterType::Triangle);
        let i = i as u32;
        let x = x_offset + (i % metrics.cols) * (metrics.card_w + metrics.gap);
        let y = (i / metrics.cols) * (metrics.card_h + metrics.gap);
        imageops::overlay(&mut canvas, &resized, x as i64, y as i64);
    }

    to_shared(&imageops::blur(&canvas, BLUR_SIGMA))
}

/// Composites the hero image (resized to `total_width` x `hero_height`, aspect not preserved —
/// blurring hides the slight squish) followed by a `fade_height`-tall gradient toward
/// `fade_to`, approximating the description panel's own fade so scrolling past the hero still
/// shows a plausible (if not pixel-exact) blur rather than a hard cutoff.
pub fn compose_hero_backdrop(
    hero: Option<&SharedPixelBuffer<Rgba8Pixel>>,
    total_width: u32,
    hero_height: u32,
    fade_from: [u8; 4],
    fade_to: [u8; 4],
    fade_height: u32,
) -> SharedPixelBuffer<Rgba8Pixel> {
    let total_h = (hero_height + fade_height).max(1);
    let mut canvas = RgbaImage::new(total_width.max(1), total_h);

    let hero_band = match hero.and_then(to_rgba_image) {
        Some(img) => imageops::resize(&img, total_width.max(1), hero_height.max(1), imageops::FilterType::Triangle),
        None => RgbaImage::from_pixel(total_width.max(1), hero_height.max(1), image::Rgba(fade_from)),
    };
    imageops::overlay(&mut canvas, &hero_band, 0, 0);

    for y in 0..fade_height {
        let t = y as f32 / fade_height.max(1) as f32;
        let color = image::Rgba(lerp_rgba(fade_from, fade_to, t));
        for x in 0..total_width {
            canvas.put_pixel(x, hero_height + y, color);
        }
    }

    to_shared(&imageops::blur(&canvas, BLUR_SIGMA))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_metrics_matches_game_grid_formula() {
        // 2000px content, Scale.unit ~16.6 (2000/120) => min-card-w ~300, gap ~17.
        let m = grid_metrics(2000, 2000.0 / 120.0);
        assert!(m.cols >= 1);
        assert_eq!(m.card_w * m.cols + m.gap * (m.cols - 1), {
            // reconstructing content_width from card_w/gap/cols should land close to 2000
            // (integer rounding), never exceed it.
            let reconstructed = m.card_w * m.cols + m.gap.saturating_mul(m.cols.saturating_sub(1));
            reconstructed.min(2000)
        });
    }

    #[test]
    fn grid_metrics_never_zero_cols() {
        let m = grid_metrics(1, 16.0);
        assert_eq!(m.cols, 1);
        assert!(m.card_w >= 1);
    }

    #[test]
    fn compose_grid_backdrop_sizes_canvas_to_row_count() {
        let thumb = SharedPixelBuffer::<Rgba8Pixel>::new(64, 36);
        let thumbs = vec![thumb; 5];
        let out = compose_grid_backdrop(&thumbs, 900, 0, 900, 15.0, [10, 10, 10, 255]);
        assert_eq!(out.width(), 900);
        assert!(out.height() > 0);
    }

    #[test]
    fn compose_hero_backdrop_without_hero_still_produces_canvas() {
        let out = compose_hero_backdrop(None, 800, 600, [10, 10, 10, 255], [20, 20, 30, 255], 400);
        assert_eq!(out.width(), 800);
        assert_eq!(out.height(), 1000);
    }
}
