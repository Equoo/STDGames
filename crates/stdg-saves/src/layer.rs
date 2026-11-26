//! The `saves` layer: game dir read-only, writes and saves in the profile.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use stdg_core::{
    BindMode, BindPurpose, Binding, CoreError, Diagnostic, LaunchCtx, Layer, LayerId, PathValue,
    SAVES_LAYER_ID, SessionGuard, Slot,
};

use crate::fs_util::copy_tree;
use crate::{GameSaves, Location, Os, PathContext, ProfileLock, ProfileName, SavesError, resolve};

/// Steam Cloud sync around a launch; the hook for a future Steam Cloud helper.
pub trait CloudSync: Send + Sync + std::fmt::Debug {
    /// Brings cloud files into `profile_dir` before the game starts.
    fn pull(&self, profile_dir: &Path) -> Result<(), SavesError>;
    /// Sends `profile_dir`'s cloud files back after the game exits.
    fn push(&self, profile_dir: &Path) -> Result<(), SavesError>;
}

/// Keeps the game dir read-only and binds the profile's save dirs.
#[derive(Debug)]
pub struct SavesLayer {
    /// The game dir, overlaid so writes land in the profile.
    pub game_root: PathBuf,
    /// The OS the game runs as.
    pub os: Os,
    /// This game's profiles.
    pub saves: GameSaves,
    /// The profile of this launch.
    pub profile: ProfileName,
    /// Save locations in Ludusavi syntax: manifest rules, then `[saves] paths`.
    pub rules: Vec<String>,
    /// Shown as a preflight warning (e.g. no manifest index yet).
    pub notice: Option<String>,
    /// Steam Cloud sync; `None` until it is implemented.
    pub cloud: Option<Arc<dyn CloudSync>>,
}

/// One save dir bound from the profile.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SaveBind {
    guest: PathBuf,
    store_rel: PathBuf,
}

impl SavesLayer {
    /// The dirs to bind, nested ones dropped since their parent covers them.
    fn binds(&self, ctx: &LaunchCtx) -> Vec<SaveBind> {
        let cx = PathContext::from_env(self.os, ctx.wine_prefix.clone());
        let mut binds: Vec<SaveBind> = Vec::new();
        for rule in &self.rules {
            match resolve(rule, &cx) {
                Location::Bind { guest, store_rel } => binds.push(SaveBind { guest, store_rel }),
                Location::GameDir => {}
                Location::SteamCloud => {
                    tracing::debug!("{rule}: Steam Cloud files are not handled yet");
                }
                Location::Skipped(reason) => tracing::debug!("{rule}: skipped, {reason}"),
            }
        }
        binds.sort_by(|a, b| a.guest.cmp(&b.guest));
        let mut kept: Vec<SaveBind> = Vec::new();
        for bind in binds {
            if !kept.iter().any(|k| bind.guest.starts_with(&k.guest)) {
                kept.push(bind);
            }
        }
        kept
    }

    fn failure(&self, e: &SavesError) -> CoreError {
        CoreError::LayerFailure {
            layer: self.id(),
            reason: e.to_string(),
        }
    }

    /// Creates the profile's dirs and the mount points.
    fn create_dirs(&self, binds: &[SaveBind]) -> Result<(), SavesError> {
        for dir in [
            self.saves.root_upper(&self.profile),
            self.saves.root_work(&self.profile),
        ] {
            std::fs::create_dir_all(&dir).map_err(SavesError::io("creating", &dir))?;
        }
        for bind in binds {
            let stored = self.saves.path_dir(&self.profile, &bind.store_rel);
            if !stored.exists() {
                // The default profile adopts saves made before the save manager.
                if self.profile.is_default() && bind.guest.is_dir() {
                    tracing::info!(
                        "adopting {} into profile {}",
                        bind.guest.display(),
                        self.profile
                    );
                    copy_tree(&bind.guest, &stored)?;
                } else {
                    std::fs::create_dir_all(&stored)
                        .map_err(SavesError::io("creating", &stored))?;
                }
            }
            std::fs::create_dir_all(&bind.guest)
                .map_err(SavesError::io("creating", &bind.guest))?;
        }
        Ok(())
    }
}

/// Holds the profile lock while the game runs, then pushes to the cloud.
#[derive(Debug)]
struct ProfileGuard {
    profile_dir: PathBuf,
    cloud: Option<Arc<dyn CloudSync>>,
    /// Released after the push, when the guard is dropped.
    _lock: Option<ProfileLock>,
}

impl SessionGuard for ProfileGuard {
    fn label(&self) -> &'static str {
        "saves-profile"
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        if let Some(cloud) = &self.cloud
            && let Err(e) = cloud.push(&self.profile_dir)
        {
            tracing::error!("Steam Cloud push failed: {e}");
        }
    }
}

impl Layer for SavesLayer {
    fn id(&self) -> LayerId {
        LayerId(SAVES_LAYER_ID.to_string())
    }

    fn slot(&self) -> Slot {
        Slot::Saves
    }

    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        match &self.notice {
            Some(notice) => Err(Diagnostic::warning(notice.clone())),
            None => Ok(()),
        }
    }

    fn container_needs(&self, ctx: &LaunchCtx) -> Vec<Binding> {
        let mut needs = vec![Binding {
            source: PathValue::Host(self.game_root.clone()),
            mode: BindMode::Overlay {
                upper: self.saves.root_upper(&self.profile),
                work: self.saves.root_work(&self.profile),
            },
            purpose: BindPurpose(format!("saves-root ({})", self.profile)),
        }];
        for bind in self.binds(ctx) {
            needs.push(Binding {
                source: PathValue::Translated {
                    host: self.saves.path_dir(&self.profile, &bind.store_rel),
                    guest: bind.guest,
                },
                mode: BindMode::ReadWrite,
                purpose: BindPurpose(format!("saves ({})", bind.store_rel.display())),
            });
        }
        needs
    }

    fn prepare(&self, ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        let profile_dir = self.saves.profile_dir(&self.profile);
        let mut lock = None;
        if !ctx.is_dry_run() {
            lock = Some(
                self.saves
                    .lock(&self.profile)
                    .map_err(|e| self.failure(&e))?,
            );
            self.create_dirs(&self.binds(ctx))
                .map_err(|e| self.failure(&e))?;
            if let Some(cloud) = &self.cloud {
                cloud.pull(&profile_dir).map_err(|e| self.failure(&e))?;
            }
        }
        Ok(Box::new(ProfileGuard {
            profile_dir,
            cloud: self.cloud.clone(),
            _lock: lock,
        }))
    }
}
