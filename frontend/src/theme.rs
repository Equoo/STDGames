use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct ThemeFile {
    theme: String,
}

fn path() -> Option<PathBuf> {
    directories::ProjectDirs::from("dev", "STDGames", "stdgames-launcher")
        .map(|d| d.config_dir().join("theme.json"))
}

/// Loads the persisted dark/light preference, replacing `localStorage.getItem('theme')`.
/// Defaults to dark, matching `gameStore.ts`'s `stored ?? 'dark'`.
pub fn load() -> bool {
    (|| -> Option<bool> {
        let data = std::fs::read_to_string(path()?).ok()?;
        let parsed: ThemeFile = serde_json::from_str(&data).ok()?;
        Some(parsed.theme == "dark")
    })()
    .unwrap_or(true)
}

pub fn save(dark: bool) {
    let Some(p) = path() else { return };
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = ThemeFile { theme: if dark { "dark" } else { "light" }.to_string() };
    if let Ok(json) = serde_json::to_string_pretty(&file) {
        let _ = std::fs::write(p, json);
    }
}
