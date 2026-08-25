use slint::{Rgba8Pixel, SharedPixelBuffer};
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Fetches game artwork off the UI thread and hands decoded images back via
/// `slint::invoke_from_event_loop`. Backed by a plain `std::thread` pool (via `thread::spawn`
/// per request) rather than a tokio runtime, since this is just a handful of HTTP GETs.
#[derive(Clone)]
pub struct ImageCache {
    inner: Arc<Mutex<HashMap<String, SharedPixelBuffer<Rgba8Pixel>>>>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Loads `url`, invoking `on_ready` with the decoded pixel buffer on the Slint UI thread.
    /// Hands back raw `SharedPixelBuffer` data (not `slint::Image`) since `on_ready` — via
    /// `invoke_from_event_loop` — must be `Send`, and callers generally want to fold the result
    /// into their own `Send`-safe state before constructing a `slint::Image` on the UI thread.
    /// If already cached, `on_ready` still runs through the event loop rather than inline, so
    /// callers can always assume "later, on the UI thread". Retries with `fallback` (caching the
    /// result under `url` either way) if `url` fails — pass an empty string to skip the retry.
    /// Used for `img_url`'s resized-image URLs: the CDN's `/img/` resizing endpoint
    /// isn't live on every deployment yet, so a 404 there should still fall back to the original
    /// `/cdn/` asset rather than leaving the image permanently blank.
    pub fn load_with_fallback(
        &self,
        url: String,
        fallback: String,
        on_ready: impl FnOnce(SharedPixelBuffer<Rgba8Pixel>) + Send + 'static,
    ) {
        if url.is_empty() {
            return;
        }

        if let Some(buf) = self.inner.lock().unwrap().get(&url).cloned() {
            let _ = slint::invoke_from_event_loop(move || {
                on_ready(buf);
            });
            return;
        }

        let cache = self.inner.clone();
        std::thread::spawn(move || {
            let buffer = match fetch_and_decode(&url) {
                Ok(buf) => buf,
                Err(err) => {
                    if fallback.is_empty() || fallback == url {
                        eprintln!("[image_cache] failed to load {url}: {err}");
                        return;
                    }
                    match fetch_and_decode(&fallback) {
                        Ok(buf) => buf,
                        Err(fallback_err) => {
                            eprintln!(
                                "[image_cache] failed to load {url} ({err}), fallback {fallback} also failed: {fallback_err}"
                            );
                            return;
                        }
                    }
                }
            };

            // Cached under the originally-requested `url` (not `fallback`) so `image_for`'s
            // lookup — which only ever knows the sized URL it asked for — still finds it.
            cache.lock().unwrap().insert(url, buffer.clone());

            let _ = slint::invoke_from_event_loop(move || {
                on_ready(buffer);
            });
        });
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

fn agent() -> ureq::Agent {
    // Several mock game entries point at a dead/unreachable IP; without an explicit timeout
    // a black-holed connection can hang for minutes (default OS TCP retransmission timeout),
    // pinning a thread per stuck request.
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .build()
}

fn fetch_and_decode(url: &str) -> Result<SharedPixelBuffer<Rgba8Pixel>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    agent().get(url).call()?.into_reader().read_to_end(&mut bytes)?;

    let decoded = image::load_from_memory(&bytes)?.to_rgba8();
    let (width, height) = (decoded.width(), decoded.height());
    Ok(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(decoded.as_raw(), width, height))
}
