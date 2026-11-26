//! The Ludusavi manifest, reduced to a small index of save rules.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Os, SavesError};

/// Where the full manifest is published.
pub const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/mtkennerly/ludusavi-manifest/master/data/manifest.yaml";

/// Bumped when the index format changes.
const INDEX_VERSION: u32 = 1;

/// The only store we launch from.
const STORE: &str = "steam";

/// One condition of a rule's `when` list; every set field must match.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    /// `windows`, `linux` or `mac`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// `steam`, `gog`...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<String>,
}

/// A save location of one game, in Ludusavi syntax.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveRule {
    /// e.g. `<winAppData>/Studio/Game/*.sav`.
    pub path: String,
    /// Applies when any condition matches; always when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub when: Vec<Condition>,
}

impl SaveRule {
    /// Whether the rule applies to a Steam game on `os`.
    pub fn applies_to(&self, os: Os) -> bool {
        self.when.is_empty()
            || self.when.iter().any(|c| {
                c.os.as_deref().is_none_or(|o| o == os.manifest_name())
                    && c.store.as_deref().is_none_or(|s| s == STORE)
            })
    }
}

/// The manifest's games with save rules, by name and by Steam app id.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ManifestIndex {
    version: u32,
    games: BTreeMap<String, Vec<SaveRule>>,
    aliases: BTreeMap<String, String>,
    steam_ids: BTreeMap<u32, String>,
}

#[derive(Deserialize)]
struct RawGame {
    #[serde(default)]
    files: BTreeMap<String, Option<RawFile>>,
    steam: Option<RawSteam>,
    id: Option<RawIds>,
    alias: Option<String>,
}

#[derive(Deserialize)]
struct RawFile {
    #[serde(default)]
    when: Vec<Condition>,
}

#[derive(Deserialize)]
struct RawSteam {
    id: Option<u32>,
}

#[derive(Deserialize)]
struct RawIds {
    #[serde(default, rename = "steamExtra")]
    steam_extra: Vec<u32>,
}

impl ManifestIndex {
    /// Builds the index from the manifest's YAML.
    pub fn from_manifest_yaml(yaml: &str) -> Result<Self, SavesError> {
        let raw: BTreeMap<String, RawGame> =
            serde_yaml_ng::from_str(yaml).map_err(SavesError::Manifest)?;
        let mut index = Self {
            version: INDEX_VERSION,
            ..Self::default()
        };
        for (name, game) in raw {
            let ids = game
                .steam
                .and_then(|s| s.id)
                .into_iter()
                .chain(game.id.map(|i| i.steam_extra).unwrap_or_default());
            for id in ids {
                index.steam_ids.entry(id).or_insert_with(|| name.clone());
            }
            if let Some(target) = game.alias {
                index.aliases.insert(name, target);
                continue;
            }
            let rules: Vec<SaveRule> = game
                .files
                .into_iter()
                .map(|(path, file)| SaveRule {
                    path,
                    when: file.map(|f| f.when).unwrap_or_default(),
                })
                .collect();
            if !rules.is_empty() {
                index.games.insert(name, rules);
            }
        }
        Ok(index)
    }

    /// Reads an index written by [`ManifestIndex::save`]; `None` if absent or outdated.
    pub fn load(path: &Path) -> Result<Option<Self>, SavesError> {
        let raw = match std::fs::read(path) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(SavesError::io("reading", path)(e)),
        };
        let index: Self = serde_json::from_slice(&raw).map_err(|source| SavesError::Index {
            path: path.to_path_buf(),
            source,
        })?;
        Ok((index.version == INDEX_VERSION).then_some(index))
    }

    /// Writes the index atomically.
    pub fn save(&self, path: &Path) -> Result<(), SavesError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(SavesError::io("creating", dir))?;
        }
        let json = serde_json::to_vec(self).map_err(|source| SavesError::Index {
            path: path.to_path_buf(),
            source,
        })?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json).map_err(SavesError::io("writing", &tmp))?;
        std::fs::rename(&tmp, path).map_err(SavesError::io("renaming", &tmp))
    }

    /// The rules of the game called `name`, else of Steam app `steam_id`.
    pub fn rules_for(
        &self,
        name: Option<&str>,
        steam_id: Option<u32>,
    ) -> Option<(&str, &[SaveRule])> {
        let name = match name {
            Some(name) => name,
            None => self.steam_ids.get(&steam_id?)?.as_str(),
        };
        let name = self.aliases.get(name).map_or(name, String::as_str);
        self.games
            .get_key_value(name)
            .map(|(name, rules)| (name.as_str(), rules.as_slice()))
    }

    /// Number of games with save rules.
    pub fn len(&self) -> usize {
        self.games.len()
    }

    /// Whether no game has save rules.
    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }
}
