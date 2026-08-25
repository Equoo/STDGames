use super::GameSource;
use crate::data::{mock_data::mock_games, GameDisplay};
use async_trait::async_trait;
use std::sync::Mutex;

/// Mock backend: serves the bundled mock data immediately and simulates launch/kill
/// locally so Play/Kill can actually be exercised in this UI-only rewrite.
pub struct MockGameSource {
    running: Mutex<Option<String>>,
}

impl MockGameSource {
    pub fn new() -> Self {
        Self { running: Mutex::new(None) }
    }
}

impl Default for MockGameSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameSource for MockGameSource {
    async fn fetch_library(&self) -> Vec<GameDisplay> {
        mock_games()
    }

    async fn launch_game(&self, slug: &str) -> bool {
        *self.running.lock().unwrap() = Some(slug.to_string());
        true
    }

    async fn get_running_game(&self) -> Option<String> {
        self.running.lock().unwrap().clone()
    }

    async fn kill_running_game(&self) {
        *self.running.lock().unwrap() = None;
    }

    fn add_desktop_icon(&self) {
        eprintln!("[DEV] Mock add desktop icon");
    }

    fn open_url(&self, url: &str) {
        // Best-effort: shell out to the platform opener, matching `window.open` in the browser sandbox.
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(url).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/C", "start", url]).spawn();
    }
}
