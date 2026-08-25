mod backdrop;
mod backend;
mod data;
mod favorites;
mod html;
mod image_cache;
mod img_url;
mod state;
mod theme;

mod slint_gen {
    slint::include_modules!();
}

use backend::mock::MockGameSource;
use backend::real::RealGameSource;
use backend::GameSource;
use data::mock_data::mock_games;
use image_cache::ImageCache;
use slint::{ComponentHandle, Timer, TimerMode};
use slint_gen::{AppWindow, SplashWindow, Theme, ViewKind};
use state::{AppState, SharedState};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    let splash = SplashWindow::new().expect("failed to create splash window");
    splash.set_progress(0.0);
    splash.show().expect("failed to show splash window");

    let app = AppWindow::new().expect("failed to create main window");

    // Prefer `games.toml` (or the bundled `games.toml.exemple`) + the CDN's `/api/data` for real
    // library metadata; `RealGameSource::fetch_library` never panics on a missing/corrupt file or
    // an unreachable network, but if it comes back empty we still fall back to the bundled mock
    // library below so the UI never launches to an empty grid. `STDGAMES_FORCE_MOCK=1` skips the
    // real backend entirely, for offline dev work.
    let source: Arc<dyn GameSource> = if std::env::var("STDGAMES_FORCE_MOCK").is_ok() {
        Arc::new(MockGameSource::new())
    } else {
        Arc::new(RealGameSource::new(RealGameSource::default_path()))
    };
    let images = ImageCache::new();
    let state: SharedState = Arc::new(Mutex::new(AppState::new()));

    app.global::<Theme>().set_dark(theme::load());

    {
        let mut library = pollster::block_on(source.fetch_library());
        if library.is_empty() {
            eprintln!("[startup] real game library was empty, falling back to mock data");
            library = mock_games();
        }
        let mut st = state.lock().unwrap();
        st.library = library;
        st.reshuffle_home();
    }

    // Kick off async artwork loading (background threads); each resolved image triggers a
    // full model rebuild so it appears wherever it's referenced (sidebar/grid/hero/etc).
    // Only `Send`-safe data (`Arc<Mutex<AppState>>`, `slint::Weak`, raw pixel buffers) ever
    // crosses the thread boundary here — see the doc comment on `AppState`.
    {
        let urls = state.lock().unwrap().card_image_urls();
        for (url, fallback) in urls {
            let state = state.clone();
            let app_weak = app.as_weak();
            let url_for_cb = url.clone();
            images.load_with_fallback(url, fallback, move |buf| {
                state.lock().unwrap().images.insert(url_for_cb.clone(), buf);
                if let Some(app) = app_weak.upgrade() {
                    refresh(&app, &state);
                }
            });
        }
    }

    refresh(&app, &state);

    // Home for one-shot timers created inside callbacks (e.g. the launch delay, and the
    // splash-hide delay below): `Timer` cancels itself on drop, so anything started inside a
    // closure must be parked somewhere that outlives that single callback invocation.
    let pending_timers: Rc<RefCell<Vec<Timer>>> = Rc::new(RefCell::new(Vec::new()));

    wire_callbacks(&app, &state, &source, &images, &pending_timers);

    // Debounced dynamic-backdrop rebuild (see `backdrop.rs`'s doc comment). `refresh()` invokes
    // `backdrop-dirty` on every data change, and `app.slint` invokes it on window resize / view
    // switches — all of that funnels through this one closure, which restarts a single-shot timer
    // each time rather than rebuilding immediately, so a burst of changes (images streaming in,
    // a resize drag) only triggers one rebuild after things settle. Storing the new `Timer` into
    // `pending_backdrop` drops (and per `Timer`'s own semantics, cancels) whatever was pending.
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        let pending_backdrop: Rc<RefCell<Option<Timer>>> = Rc::new(RefCell::new(None));
        app.on_backdrop_dirty(move || {
            let state = state.clone();
            let app_weak = app_weak.clone();
            let timer = Timer::default();
            timer.start(
                TimerMode::SingleShot,
                Duration::from_millis(400),
                move || {
                    if let Some(app) = app_weak.upgrade() {
                        rebuild_backdrop(&app, &state);
                    }
                },
            );
            *pending_backdrop.borrow_mut() = Some(timer);
        });
    }

    // Poll for the running game every second, exactly like the Svelte `setInterval`.
    let poll_timer = Timer::default();
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        let source = source.clone();
        poll_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            let running = pollster::block_on(source.get_running_game());
            let changed = {
                let mut st = state.lock().unwrap();
                if st.running_slug != running {
                    st.running_slug = running;
                    true
                } else {
                    false
                }
            };
            if changed {
                if let Some(app) = app_weak.upgrade() {
                    refresh(&app, &state);
                }
            }
        });
    }

    // Splash progress + handoff to the main window, simulating real progress events.
    let splash_timer = Timer::default();
    {
        let splash_weak = splash.as_weak();
        let app_weak = app.as_weak();
        let progress = Rc::new(RefCell::new(0.0f32));
        let handed_off = Rc::new(RefCell::new(false));
        let pending_timers = pending_timers.clone();
        splash_timer.start(TimerMode::Repeated, Duration::from_millis(40), move || {
            if *handed_off.borrow() {
                return;
            }
            let mut p = progress.borrow_mut();
            *p = (*p + 0.06).min(1.0);
            if let Some(splash) = splash_weak.upgrade() {
                splash.set_progress(*p);
            }
            if *p >= 1.0 {
                *handed_off.borrow_mut() = true;
                if let Some(app) = app_weak.upgrade() {
                    let _ = app.show();
                }
                // Hiding the splash in the same tick as showing the main window — even after
                // it — leaves the main window mapped but never actually composited (a winit/
                // Slint multi-window redraw-scheduling quirk, confirmed empirically: skipping
                // the hide entirely let the main window render fine). Deferring the hide to
                // the next tick gives the main window's first frame a chance to actually paint
                // before splash goes away. The timer must be parked in `pending_timers` since
                // `Timer` cancels itself on drop.
                let splash_weak = splash_weak.clone();
                let hide_timer = Timer::default();
                hide_timer.start(
                    TimerMode::SingleShot,
                    Duration::from_millis(80),
                    move || {
                        if let Some(splash) = splash_weak.upgrade() {
                            let _ = splash.hide();
                        }
                    },
                );
                pending_timers.borrow_mut().push(hide_timer);
            }
        });
    }

    slint::run_event_loop().expect("event loop failed");
}

fn wire_callbacks(
    app: &AppWindow,
    state: &SharedState,
    source: &Arc<dyn GameSource>,
    images: &ImageCache,
    pending_timers: &Rc<RefCell<Vec<Timer>>>,
) {
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        app.on_search_changed(move |text| {
            state.lock().unwrap().search = text.to_string();
            if let Some(app) = app_weak.upgrade() {
                refresh(&app, &state);
            }
        });
    }
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        app.on_tag_toggled(move |tag| {
            let tag = tag.to_string();
            let mut st = state.lock().unwrap();
            st.active_tag = if st.active_tag.as_deref() == Some(tag.as_str()) {
                None
            } else {
                Some(tag)
            };
            drop(st);
            if let Some(app) = app_weak.upgrade() {
                refresh(&app, &state);
            }
        });
    }
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        app.on_sort_changed(move |descending| {
            state.lock().unwrap().sort_order = if descending {
                data::SortOrder::Descending
            } else {
                data::SortOrder::Ascending
            };
            if let Some(app) = app_weak.upgrade() {
                refresh(&app, &state);
            }
        });
    }
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        let images = images.clone();
        app.on_game_selected(move |slug| {
            state.lock().unwrap().selected_slug = Some(slug.to_string());
            if let Some(app) = app_weak.upgrade() {
                app.set_current_view(ViewKind::Preview);
                refresh(&app, &state);
            }

            // Full-size hero/logo/screenshots/movie-thumbnails for this one game only — see
            // `AppState::selected_game_media_urls`'s doc comment for why these aren't preloaded
            // for the whole library up front.
            let urls = state.lock().unwrap().selected_game_media_urls();
            for (url, fallback) in urls {
                let state = state.clone();
                let app_weak = app_weak.clone();
                let url_for_cb = url.clone();
                images.load_with_fallback(url, fallback, move |buf| {
                    state.lock().unwrap().images.insert(url_for_cb.clone(), buf);
                    if let Some(app) = app_weak.upgrade() {
                        refresh(&app, &state);
                    }
                });
            }
        });
    }
    {
        let app_weak = app.as_weak();
        app.on_back_to_library(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_current_view(ViewKind::Library);
            }
        });
    }
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        app.on_reshuffle(move || {
            state.lock().unwrap().reshuffle_home();
            if let Some(app) = app_weak.upgrade() {
                refresh(&app, &state);
            }
        });
    }
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        app.on_favorite_toggled(move || {
            let slug = state.lock().unwrap().selected_slug.clone();
            if let Some(slug) = slug {
                state.lock().unwrap().toggle_favorite(&slug);
            }
            if let Some(app) = app_weak.upgrade() {
                refresh(&app, &state);
            }
        });
    }
    {
        let app_weak = app.as_weak();
        app.on_theme_changed(move |dark| {
            theme::save(dark);
            if let Some(app) = app_weak.upgrade() {
                app.global::<Theme>().set_dark(dark);
            }
        });
    }
    {
        let source = source.clone();
        app.on_add_desktop_icon(move || {
            source.add_desktop_icon();
        });
    }
    {
        let source = source.clone();
        app.on_open_url(move |url| {
            source.open_url(&url);
        });
    }
    {
        let state = state.clone();
        let app_weak = app.as_weak();
        let source = source.clone();
        let pending_timers = pending_timers.clone();
        app.on_play_clicked(move || {
            let slug = state.lock().unwrap().selected_slug.clone();
            let Some(slug) = slug else { return };
            let is_running = state.lock().unwrap().running_slug.as_deref() == Some(slug.as_str());

            if is_running {
                pollster::block_on(source.kill_running_game());
                state.lock().unwrap().running_slug = None;
                if let Some(app) = app_weak.upgrade() {
                    refresh(&app, &state);
                }
                return;
            }

            state.lock().unwrap().is_launching = true;
            if let Some(app) = app_weak.upgrade() {
                refresh(&app, &state);
            }

            let state = state.clone();
            let app_weak = app_weak.clone();
            let source = source.clone();
            // `Timer` cancels itself on drop, so park it in `pending_timers` (owned by `main`)
            // rather than letting this local go out of scope when the callback returns.
            let launch_timer = Timer::default();
            launch_timer.start(
                TimerMode::SingleShot,
                Duration::from_millis(450),
                move || {
                    pollster::block_on(source.launch_game(&slug));
                    let mut st = state.lock().unwrap();
                    st.running_slug = Some(slug.clone());
                    st.is_launching = false;
                    drop(st);
                    if let Some(app) = app_weak.upgrade() {
                        refresh(&app, &state);
                    }
                },
            );
            pending_timers.borrow_mut().push(launch_timer);
        });
    }
}

fn refresh(app: &AppWindow, state: &SharedState) {
    let st = state.lock().unwrap();

    let filtered = st.filtered_cards();
    app.set_sidebar_games(filtered.clone());
    app.set_library_view_games(filtered);
    app.set_home_picks(st.home_pick_cards());
    app.set_favorite_games(st.favorite_cards());
    app.set_active_tag(st.active_tag.clone().unwrap_or_default().into());
    app.set_sort_descending(st.sort_order == data::SortOrder::Descending);
    app.set_running_slug(st.running_slug.clone().unwrap_or_default().into());
    app.set_is_launching(st.is_launching);

    match st.selected_card() {
        Some(card) => {
            app.set_selected_game(card);
            app.set_has_selected_game(true);
            app.set_carousel_media(st.carousel_media());
            app.set_description_blocks(st.description_blocks());
        }
        None => app.set_has_selected_game(false),
    }

    app.invoke_backdrop_dirty();
}

/// Composites (in Rust) and blurs a stand-in for "what's actually behind the topbar right now"
/// — see `backdrop.rs`'s doc comment for why this exists instead of a live backdrop-filter.
/// Called only from the debounced timer in `main()`, never directly, so it's fine for this to be
/// a bit heavier than a per-frame operation.
fn rebuild_backdrop(app: &AppWindow, state: &SharedState) {
    let window = app.window();
    let physical = window.size();
    if physical.width == 0 || physical.height == 0 {
        return;
    }
    let scale = window.scale_factor();
    let logical_w = physical.width as f32 / scale;
    let logical_h = physical.height as f32 / scale;
    let scale_unit = logical_w / 120.0;
    let sidebar_w = app.get_sidebar_width();
    let content_w = (logical_w - sidebar_w).max(1.0);

    let fill = if app.global::<Theme>().get_dark() {
        [8u8, 8, 18, 255]
    } else {
        [230u8, 224, 236, 255]
    };

    let st = state.lock().unwrap();
    let buf = match app.get_current_view() {
        ViewKind::Library => backdrop::compose_grid_backdrop(
            &grid_thumbs(&st, st.filtered_games().into_iter()),
            content_w.round() as u32 - scale_unit as u32 * 3,
            sidebar_w.round() as u32 + scale_unit as u32 * 2,
            logical_w.round() as u32,
            scale_unit,
            fill,
        ),
        ViewKind::Home => backdrop::compose_grid_backdrop(
            &grid_thumbs(&st, st.home_pick_slugs.iter().filter_map(|s| st.find(s))),
            content_w.round() as u32 - scale_unit as u32 * 3,
            sidebar_w.round() as u32 + scale_unit as u32 * 2,
            logical_w.round() as u32,
            scale_unit,
            fill,
        ),
        ViewKind::Preview => {
            let hero = st
                .selected_slug
                .as_deref()
                .and_then(|s| st.find(s))
                .and_then(|g| g.hero_source())
                .map(img_url::hero_full)
                .and_then(|u| st.images.get(&u).cloned());
            backdrop::compose_hero_backdrop(
                hero.as_ref(),
                logical_w.round() as u32,
                logical_h.round() as u32,
                fill,
                fill,
                (logical_h * 2.0).round() as u32,
            )
        }
        // Settings' scrollable content is plain panels, not imagery worth compositing — the
        // static blurred-background asset (`ChromeBackdrop`'s base layer) covers it.
        ViewKind::Settings => return,
    };
    drop(st);

    app.set_dynamic_backdrop(slint::Image::from_rgba8(buf));
}

/// Resolves already-loaded thumbnail pixel buffers for `games`, in order — see
/// `AppState::card_image_urls`'s doc comment for why these are looked up by the same sized URL
/// they were cached under rather than re-deriving anything. Games whose thumbnail hasn't finished
/// loading yet are skipped (not padded), which can transiently shift later cards' backdrop
/// position until the next rebuild — acceptable for a blurred approximation.
fn grid_thumbs<'a>(
    st: &state::AppState,
    games: impl Iterator<Item = &'a data::GameDisplay>,
) -> Vec<slint::SharedPixelBuffer<slint::Rgba8Pixel>> {
    games
        .filter_map(|g| g.hero_source())
        .map(img_url::card_thumb)
        .filter_map(|u| st.images.get(&u).cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::slint_gen::{AppWindow, ViewKind};
    use slint::platform::{PointerEventButton, WindowEvent};
    use slint::{ComponentHandle, LogicalPosition};
    use std::cell::Cell;
    use std::rc::Rc;

    /// Regression test for the topbar swallowing clicks meant for the view underneath it.
    ///
    /// The topbar overlays the top of every view. It used to block input across its full
    /// 7.125-unit height, which silently ate every control living in the "header zone" below the
    /// nav row — Library's Multiplayer/Solo/Sort controls sit there, and GamePreview's Play/Back
    /// buttons dock there once scrolled (see `action-bar-y` in `game_preview_view.slint`). The
    /// blocker is now limited to the nav row itself; this asserts a click at Play's resting
    /// on-screen position (near the bottom of the hero, matching `action-bar-rest-y` at this
    /// window size) actually reaches it. It doesn't drive a real scroll gesture to also check the
    /// docked position — `WindowEvent::PointerScrolled` doesn't move a `Flickable`'s viewport
    /// under `init_no_event_loop()` (no event loop ticks the scroll-capture debounce it relies
    /// on) — but the docked zone sits well inside the nav row's already-covered non-blocked area,
    /// so this still exercises the same click-dispatch path the original bug broke.
    #[test]
    fn play_button_is_not_blocked_by_topbar() {
        i_slint_backend_testing::init_no_event_loop();

        let app = AppWindow::new().unwrap();
        // 1975px wide, 1398px tall => Scale.unit == 1975/120 == ~16.46px, matching the layout
        // coordinates below. Play rests at `height - 3.5 * Scale.unit` == 1398 - 57.6 == ~1340.4.
        app.window().set_size(slint::PhysicalSize::new(1975, 1398));
        app.set_current_view(ViewKind::Preview);
        app.set_has_selected_game(true);

        let clicked = Rc::new(Cell::new(false));
        let flag = clicked.clone();
        app.on_play_clicked(move || flag.set(true));

        // x is past the sidebar (15 units) plus the action bar's 2-unit inset; 300px is
        // comfortably inside the button regardless of how wide the "Play" label renders. y is
        // ~15px into the button's ~42.8px height, below its resting top edge.
        let pos = LogicalPosition::new(300.0, 1355.0);
        app.window()
            .dispatch_event(WindowEvent::PointerMoved { position: pos });
        app.window().dispatch_event(WindowEvent::PointerPressed {
            position: pos,
            button: PointerEventButton::Left,
        });
        app.window().dispatch_event(WindowEvent::PointerReleased {
            position: pos,
            button: PointerEventButton::Left,
        });

        assert!(
            clicked.get(),
            "click at the Play button was swallowed before reaching it"
        );
    }
}
