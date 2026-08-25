pub mod mock;
pub mod real;

use crate::data::GameDisplay;
use async_trait::async_trait;

/// Seam for a future real backend (Tauri commands) to be wired in without touching UI code.
/// Mirrors `lib/api/games.ts` + `lib/api/system.ts`.
#[async_trait]
pub trait GameSource: Send + Sync {
    async fn fetch_library(&self) -> Vec<GameDisplay>;
    async fn launch_game(&self, slug: &str) -> bool;
    async fn get_running_game(&self) -> Option<String>;
    async fn kill_running_game(&self);
    fn add_desktop_icon(&self);
    fn open_url(&self, url: &str);
}
