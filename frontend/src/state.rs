use crate::data::{GameDisplay, SortOrder};
use crate::slint_gen::{CarouselMedia, CarouselMediaKind, DescriptionBlock, DescriptionBlockKind, GameCardData};
use crate::{html, img_url};
use rand::seq::SliceRandom;
use slint::{Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Owns the library + all filter/sort/favorite/running/selection state, mirroring
/// `gameStore.ts`. Rebuilds and pushes fresh `ModelRc<GameCardData>`s into the Slint UI
/// whenever anything that affects a displayed list changes (search, tag, sort, favorites,
/// running slug, or a newly-resolved image).
///
/// Images are stored as raw `SharedPixelBuffer`s (plain pixel data, `Send`) rather than
/// `slint::Image` handles, because a resolved image arrives via a background-thread callback
/// hopping onto the UI thread through `invoke_from_event_loop` — the closure making that hop
/// must be `Send`, which rules out anything backed by `Rc`. `slint::Image` values are only ever
/// constructed on the UI thread, inside `to_card()`.
pub struct AppState {
    pub library: Vec<GameDisplay>,
    pub favorites: Vec<String>,
    pub images: HashMap<String, SharedPixelBuffer<Rgba8Pixel>>,
    pub search: String,
    pub active_tag: Option<String>,
    pub sort_order: SortOrder,
    pub selected_slug: Option<String>,
    pub running_slug: Option<String>,
    pub home_pick_slugs: Vec<String>,
    pub is_launching: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            library: Vec::new(),
            favorites: crate::favorites::load(),
            images: HashMap::new(),
            search: String::new(),
            active_tag: None,
            sort_order: SortOrder::Descending,
            selected_slug: None,
            running_slug: None,
            home_pick_slugs: Vec::new(),
            is_launching: false,
        }
    }

    pub fn find(&self, slug: &str) -> Option<&GameDisplay> {
        self.library.iter().find(|g| g.slug == slug)
    }

    /// `url` is expected pre-sized (see `img_url`) — the cache key has to match exactly what
    /// `card_image_urls`/`selected_game_media_urls` fetched, or this silently misses and shows
    /// a blank image.
    fn image_for(&self, url: Option<String>) -> Image {
        match url {
            Some(u) if !u.is_empty() => {
                self.images.get(&u).map(|buf| Image::from_rgba8(buf.clone())).unwrap_or_default()
            }
            _ => Image::default(),
        }
    }

    pub fn to_card(&self, g: &GameDisplay) -> GameCardData {
        let has_logo = g.logo.as_deref().is_some_and(|s| !s.is_empty());
        let hero_source = g.hero_source();
        GameCardData {
            slug: g.slug.clone().into(),
            name: SharedString::from(g.display_name()),
            icon: self.image_for(g.icon.as_deref().filter(|s| !s.is_empty()).map(img_url::icon)),
            hero: self.image_for(hero_source.map(img_url::hero_full)),
            hero_thumb: self.image_for(hero_source.map(img_url::card_thumb)),
            logo: self.image_for(g.logo.as_deref().filter(|s| !s.is_empty()).map(img_url::logo)),
            has_logo,
            tags: ModelRc::new(VecModel::from(
                g.tags.clone().unwrap_or_default().into_iter().map(SharedString::from).collect::<Vec<_>>(),
            )),
            short_description: html::to_text(g.short_description.as_deref().unwrap_or_default()).into(),
            is_favorite: self.favorites.iter().any(|s| s == &g.slug),
            is_running: self.running_slug.as_deref() == Some(g.slug.as_str()),
        }
    }

    /// Search + tag filter + name sort, ported 1:1 from `gameStore.ts`'s `filteredGames`.
    pub fn filtered_games(&self) -> Vec<&GameDisplay> {
        let query = self.search.to_lowercase();
        let mut games: Vec<&GameDisplay> = self
            .library
            .iter()
            .filter(|g| query.is_empty() || g.display_name().to_lowercase().contains(&query))
            .filter(|g| match &self.active_tag {
                None => true,
                Some(tag) => g.tags.as_ref().is_some_and(|tags| tags.iter().any(|t| t == tag)),
            })
            .collect();

        games.sort_by(|a, b| {
            let na = a.display_name().to_lowercase();
            let nb = b.display_name().to_lowercase();
            match self.sort_order {
                SortOrder::Descending => na.cmp(&nb),
                SortOrder::Ascending => nb.cmp(&na),
            }
        });
        games
    }

    pub fn filtered_cards(&self) -> ModelRc<GameCardData> {
        ModelRc::new(VecModel::from(self.filtered_games().into_iter().map(|g| self.to_card(g)).collect::<Vec<_>>()))
    }

    pub fn home_pick_cards(&self) -> ModelRc<GameCardData> {
        let cards: Vec<GameCardData> =
            self.home_pick_slugs.iter().filter_map(|s| self.find(s)).map(|g| self.to_card(g)).collect();
        ModelRc::new(VecModel::from(cards))
    }

    pub fn favorite_cards(&self) -> ModelRc<GameCardData> {
        let cards: Vec<GameCardData> =
            self.library.iter().filter(|g| self.favorites.iter().any(|s| s == &g.slug)).map(|g| self.to_card(g)).collect();
        ModelRc::new(VecModel::from(cards))
    }

    pub fn selected_card(&self) -> Option<GameCardData> {
        self.selected_slug.as_deref().and_then(|s| self.find(s)).map(|g| self.to_card(g))
    }

    pub fn carousel_media(&self) -> ModelRc<CarouselMedia> {
        let Some(slug) = self.selected_slug.clone() else {
            return ModelRc::new(VecModel::from(Vec::<CarouselMedia>::new()));
        };
        let Some(game) = self.find(&slug) else {
            return ModelRc::new(VecModel::from(Vec::<CarouselMedia>::new()));
        };

        // Interleave screenshots/videos, ported from Carousel.svelte's `mediaItems` derived.
        let screenshots = game.screenshots.clone().unwrap_or_default();
        let videos = game.movies.clone().unwrap_or_default();
        let thumbnails = game.movies_thumbnails.clone().unwrap_or_default();
        let total = screenshots.len() + videos.len();
        let mut items = Vec::with_capacity(total);
        if total > 0 {
            let videos_interval = if !videos.is_empty() { screenshots.len() / videos.len() } else { usize::MAX };
            let mut video_i = 0usize;
            let mut shot_i = 0usize;
            for i in 0..total {
                let is_video =
                    !videos.is_empty() && videos_interval > 0 && i % (videos_interval + 1) == 0 && video_i < videos.len();
                if is_video {
                    let src = thumbnails.get(video_i).or(videos.get(video_i)).cloned().unwrap_or_default();
                    items.push(CarouselMedia {
                        kind: CarouselMediaKind::Video,
                        src: self.image_for(Some(img_url::carousel_media(&src))),
                    });
                    video_i += 1;
                } else if shot_i < screenshots.len() {
                    let src = screenshots[shot_i].clone();
                    items.push(CarouselMedia {
                        kind: CarouselMediaKind::Image,
                        src: self.image_for(Some(img_url::carousel_media(&src))),
                    });
                    shot_i += 1;
                }
            }
        }
        ModelRc::new(VecModel::from(items))
    }

    /// The selected game's `description` (Steam store-page HTML) parsed into renderable blocks —
    /// see `DescriptionBlock` in `ui/model.slint` and `html::parse_blocks`. Slint's `Text` can't
    /// render HTML directly, so this is what stands in for it.
    pub fn description_blocks(&self) -> ModelRc<DescriptionBlock> {
        let Some(game) = self.selected_slug.as_deref().and_then(|s| self.find(s)) else {
            return ModelRc::new(VecModel::from(Vec::<DescriptionBlock>::new()));
        };

        let items: Vec<DescriptionBlock> = html::parse_blocks(game.description.as_deref().unwrap_or_default())
            .into_iter()
            .map(|block| match block {
                html::Block::Heading(text) => DescriptionBlock {
                    kind: DescriptionBlockKind::Heading,
                    bold_prefix: SharedString::default(),
                    text: text.into(),
                    image: Image::default(),
                },
                html::Block::Paragraph { bold_prefix, text } => DescriptionBlock {
                    kind: DescriptionBlockKind::Paragraph,
                    bold_prefix: bold_prefix.into(),
                    text: text.into(),
                    image: Image::default(),
                },
                html::Block::ListItem { bold_prefix, text } => DescriptionBlock {
                    kind: DescriptionBlockKind::ListItem,
                    bold_prefix: bold_prefix.into(),
                    text: text.into(),
                    image: Image::default(),
                },
                html::Block::Image { src } => DescriptionBlock {
                    kind: DescriptionBlockKind::Image,
                    bold_prefix: SharedString::default(),
                    text: SharedString::default(),
                    image: self.image_for(Some(img_url::carousel_media(&src))),
                },
            })
            .collect();
        ModelRc::new(VecModel::from(items))
    }

    pub fn reshuffle_home(&mut self) {
        let mut slugs: Vec<String> = self.library.iter().map(|g| g.slug.clone()).collect();
        slugs.shuffle(&mut rand::thread_rng());
        slugs.truncate(6);
        self.home_pick_slugs = slugs;
    }

    pub fn toggle_favorite(&mut self, slug: &str) {
        if let Some(pos) = self.favorites.iter().position(|s| s == slug) {
            self.favorites.remove(pos);
        } else {
            self.favorites.push(slug.to_string());
        }
        crate::favorites::save(&self.favorites);
    }

    /// Icon + a small hero/cover thumbnail for every game, pre-sized via `img_url`, for the
    /// initial image-loading pass. Each entry is `(sized_url, original_url)` — the CDN's `/img/`
    /// resizing endpoint isn't necessarily live on every deployment, so `ImageCache::load_with_
    /// fallback` retries the original `/cdn/` URL if the sized one 404s, rather than leaving the
    /// image permanently blank.
    ///
    /// Deliberately excludes the full-size hero, logo, and screenshots/movie thumbnails: the
    /// mock library never populated the latter, so eagerly loading them for the *entire* library
    /// was harmless dead code — but real data from `RealGameSource` can carry dozens of
    /// screenshots per game (and a multi-megapixel hero), and spawning a background thread per
    /// URL across ~90 games at once pegs every core and balloons memory. Those only matter for
    /// whichever game is actually being previewed, so they're fetched on demand instead — see
    /// `selected_game_media_urls`.
    pub fn card_image_urls(&self) -> Vec<(String, String)> {
        let mut urls = Vec::new();
        for g in &self.library {
            if let Some(icon) = g.icon.as_deref().filter(|s| !s.is_empty()) {
                urls.push((img_url::icon(icon), icon.to_string()));
            }
            if let Some(hero) = g.hero_source() {
                urls.push((img_url::card_thumb(hero), hero.to_string()));
            }
        }
        urls.sort();
        urls.dedup();
        urls
    }

    /// Full-size hero, logo, and screenshots/movie-thumbnails for the currently selected game
    /// only, as `(sized_url, original_url)` pairs — see `card_image_urls`'s doc comment for both
    /// the fallback behavior and why these aren't preloaded for the whole library.
    pub fn selected_game_media_urls(&self) -> Vec<(String, String)> {
        let Some(game) = self.selected_slug.as_deref().and_then(|s| self.find(s)) else {
            return Vec::new();
        };

        let mut urls: Vec<(String, String)> = Vec::new();
        if let Some(hero) = game.hero_source() {
            urls.push((img_url::hero_full(hero), hero.to_string()));
        }
        if let Some(logo) = game.logo.as_deref().filter(|s| !s.is_empty()) {
            urls.push((img_url::logo(logo), logo.to_string()));
        }
        urls.extend(
            [&game.screenshots, &game.movies_thumbnails]
                .into_iter()
                .flatten()
                .flatten()
                .map(|u| (img_url::carousel_media(u), u.clone())),
        );
        urls.extend(html::parse_blocks(game.description.as_deref().unwrap_or_default()).into_iter().filter_map(
            |block| match block {
                html::Block::Image { src } => Some((img_url::carousel_media(&src), src)),
                _ => None,
            },
        ));
        urls.sort();
        urls.dedup();
        urls
    }
}

pub type SharedState = Arc<Mutex<AppState>>;
