use std::path::PathBuf;

fn path() -> Option<PathBuf> {
    directories::ProjectDirs::from("dev", "STDGames", "stdgames-launcher")
        .map(|d| d.config_dir().join("favorites.json"))
}

/// Loads the persisted favorite slugs, replacing `localStorage.getItem('favorites')`.
pub fn load() -> Vec<String> {
    (|| -> Option<Vec<String>> {
        let data = std::fs::read_to_string(path()?).ok()?;
        serde_json::from_str(&data).ok()
    })()
    .unwrap_or_default()
}

pub fn save(favorites: &[String]) {
    let Some(p) = path() else { return };
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(favorites) {
        let _ = std::fs::write(p, json);
    }
}
