//! Turns Ludusavi paths (`<winAppData>/Studio/Game/*.sav`) into dirs to bind.

use std::path::{Component, Path, PathBuf};

use stdg_core::{TargetKind, WinePrefix};

/// The OS a game runs as, which decides the manifest rules and placeholders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    /// Windows games, run in a Wine prefix.
    Windows,
    /// Native Linux games and emulators.
    Linux,
}

impl Os {
    /// The OS of `target`.
    pub fn of(target: &TargetKind) -> Self {
        match target {
            TargetKind::Windows => Os::Windows,
            TargetKind::NativeLinux | TargetKind::Rom(_) => Os::Linux,
        }
    }

    /// The name the manifest's `when.os` uses.
    pub fn manifest_name(self) -> &'static str {
        match self {
            Os::Windows => "windows",
            Os::Linux => "linux",
        }
    }
}

/// What the placeholders stand for in this launch.
#[derive(Debug, Clone)]
pub struct PathContext {
    /// The OS the game runs as.
    pub os: Os,
    /// `$HOME`.
    pub home: Option<PathBuf>,
    /// `$XDG_DATA_HOME`, else `~/.local/share`.
    pub xdg_data: Option<PathBuf>,
    /// `$XDG_CONFIG_HOME`, else `~/.config`.
    pub xdg_config: Option<PathBuf>,
    /// The Wine prefix of a Windows game.
    pub wine: Option<WinePrefix>,
}

impl PathContext {
    /// Reads `HOME` and the XDG dirs from the environment.
    pub fn from_env(os: Os, wine: Option<WinePrefix>) -> Self {
        let var = |name: &str| std::env::var_os(name).map(PathBuf::from);
        let home = var("HOME");
        Self {
            os,
            xdg_data: var("XDG_DATA_HOME")
                .or_else(|| home.as_ref().map(|h| h.join(".local/share"))),
            xdg_config: var("XDG_CONFIG_HOME").or_else(|| home.as_ref().map(|h| h.join(".config"))),
            home,
            wine,
        }
    }

    /// The OS user name the game sees.
    fn user(&self) -> Option<String> {
        match (self.os, &self.wine) {
            (Os::Windows, Some(wine)) => Some(wine.user.clone()),
            _ => std::env::var("USER").ok(),
        }
    }

    /// The dir a leading placeholder stands for.
    fn root(&self, placeholder: &str) -> Option<PathBuf> {
        let wine = self.wine.as_ref().filter(|_| self.os == Os::Windows);
        let user_dir = wine.map(WinePrefix::user_dir);
        let linux = |p: &Option<PathBuf>| p.clone().filter(|_| self.os == Os::Linux);
        match placeholder {
            "home" => match self.os {
                Os::Windows => user_dir,
                Os::Linux => self.home.clone(),
            },
            "winAppData" => user_dir.map(|d| d.join("AppData/Roaming")),
            "winLocalAppData" => user_dir.map(|d| d.join("AppData/Local")),
            "winLocalAppDataLow" => user_dir.map(|d| d.join("AppData/LocalLow")),
            "winDocuments" => user_dir.map(|d| d.join("Documents")),
            "winPublic" => wine.map(|w| w.drive_c.join("users/Public")),
            "winProgramData" => wine.map(|w| w.drive_c.join("ProgramData")),
            "winDir" => wine.map(|w| w.drive_c.join("windows")),
            "xdgData" => linux(&self.xdg_data),
            "xdgConfig" => linux(&self.xdg_config),
            _ => None,
        }
    }
}

/// Where a save rule lands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// Bind `store_rel` (inside the profile) over `guest`.
    Bind {
        /// The dir the game reads and writes.
        guest: PathBuf,
        /// The same dir inside the profile, e.g. `winAppData/Studio/Game`.
        store_rel: PathBuf,
    },
    /// Inside the game dir: already kept by the profile's `root/`.
    GameDir,
    /// Steam Cloud files in the Steam client (`<root>/userdata/...`): not handled yet.
    SteamCloud,
    /// Cannot be bound; the reason is logged.
    Skipped(String),
}

/// Resolves one rule. Globs are cut off: the dir before the first one is bound.
pub fn resolve(rule: &str, cx: &PathContext) -> Location {
    let mut parts = rule.split(['/', '\\']).filter(|p| !p.is_empty());
    let Some(first) = parts.next() else {
        return Location::Skipped("empty path".to_string());
    };
    let Some(placeholder) = first.strip_prefix('<').and_then(|p| p.strip_suffix('>')) else {
        return Location::Skipped("does not start with a placeholder".to_string());
    };
    match placeholder {
        "base" => return Location::GameDir,
        "root" => return Location::SteamCloud,
        _ => {}
    }
    let Some(root) = cx.root(placeholder) else {
        return Location::Skipped(format!("<{placeholder}> means nothing for this game"));
    };

    let user = cx.user();
    let mut literal = Vec::new();
    for part in parts {
        let part = match (part, &user) {
            ("<osUserName>", Some(user)) => user.as_str(),
            _ => part,
        };
        if part.contains(['*', '?', '[', '{', '<']) {
            break;
        }
        if Path::new(part)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Location::Skipped(format!("unsafe component {part:?}"));
        }
        literal.push(part);
    }
    if literal.is_empty() {
        return Location::Skipped(format!("would bind all of <{placeholder}>"));
    }

    let rel: PathBuf = literal.iter().collect();
    Location::Bind {
        guest: root.join(&rel),
        store_rel: Path::new(placeholder).join(rel),
    }
}
