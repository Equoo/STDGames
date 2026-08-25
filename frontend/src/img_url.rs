//! Builds resized-image URLs for the STDGames CDN: swap `/cdn/` for `/img/` and append
//! `w`/`h`/`p` (width/height/JPEG quality) query params. Each field of `GameCardData` is
//! displayed at a very different size (a 1-unit sidebar icon vs. a full-window hero background),
//! so requesting the original full-resolution asset for all of them wastes bandwidth, download
//! time, and decoded-pixel memory. The presets below are sized generously for this app's largest
//! observed window (~2500px wide) rather than any one exact on-screen size, since images are
//! fetched once at startup/selection and not re-fetched on resize.

/// `url` unchanged if it isn't one of this CDN's `/cdn/` URLs — the bundled `games.toml.exemple`
/// also carries plain third-party links (e.g. `imgs.search.brave.com`) that don't support this.
fn sized(url: &str, w: Option<u32>, h: Option<u32>, quality: Option<u32>) -> String {
    let Some(pos) = url.find("/cdn/") else {
        return url.to_string();
    };

    let mut out = String::with_capacity(url.len() + 24);
    out.push_str(&url[..pos]);
    out.push_str("/img/");
    out.push_str(&url[pos + "/cdn/".len()..]);

    let params: Vec<String> = [w.map(|v| format!("w={v}")), h.map(|v| format!("h={v}")), quality.map(|v| format!("p={v}"))]
        .into_iter()
        .flatten()
        .collect();
    if !params.is_empty() {
        out.push('?');
        out.push_str(&params.join("&"));
    }
    out
}

/// Sidebar/list row icons — rendered at ~1 `Scale.unit` (well under 64px even on a huge window).
pub fn icon(url: &str) -> String {
    sized(url, Some(64), None, Some(85))
}

/// Grid card thumbnails (`GameCard`'s background). Height-constrained by 16:9 crop, not the
/// original artwork's resolution, so a modest width is plenty even for a hover-scaled card.
pub fn card_thumb(url: &str) -> String {
    sized(url, Some(480), None, Some(78))
}

/// `GamePreviewView`'s full-bleed hero background — the one place a large image is warranted.
pub fn hero_full(url: &str) -> String {
    sized(url, Some(1920), None, Some(82))
}

/// Title logo (`GamePreviewView`), always displayed at a fixed small height regardless of
/// window size.
pub fn logo(url: &str) -> String {
    sized(url, None, Some(200), Some(85))
}

/// Carousel screenshots/movie thumbnails.
pub fn carousel_media(url: &str) -> String {
    sized(url, Some(1024), None, Some(78))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_cdn_prefix_and_appends_params() {
        assert_eq!(
            sized("http://37.59.106.4:2356/cdn/steam/367520/hero.jpg", Some(480), None, Some(78)),
            "http://37.59.106.4:2356/img/steam/367520/hero.jpg?w=480&p=78"
        );
    }

    #[test]
    fn omits_query_string_when_no_dimensions_given() {
        assert_eq!(sized("http://x/cdn/a.jpg", None, None, None), "http://x/img/a.jpg");
    }

    #[test]
    fn leaves_non_cdn_urls_untouched() {
        let url = "https://imgs.search.brave.com/abc.jpg";
        assert_eq!(sized(url, Some(480), None, Some(78)), url);
    }

    #[test]
    fn width_and_height_can_combine() {
        assert_eq!(sized("http://x/cdn/a.jpg", Some(200), Some(300), None), "http://x/img/a.jpg?w=200&h=300");
    }
}
