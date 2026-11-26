//! Profiles on disk: `<store>/<game>/<profile>/`.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use crate::SavesError;
use crate::fs_util::copy_tree;

/// The profile used when none was ever picked.
const DEFAULT_PROFILE: &str = "default";

/// File holding a game's current profile.
const CURRENT_FILE: &str = "current";

/// Hidden dir holding the overlay work dirs, outside the profiles.
const WORK_DIR: &str = ".work";

/// A name usable as one path component: letters, digits, `.`, `_`, `-`.
fn check_name(name: &str) -> Result<(), SavesError> {
    let valid = !name.is_empty()
        && !name.starts_with('.')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(SavesError::InvalidName(name.to_string()))
    }
}

/// A profile name, safe to use as a dir name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileName(String);

impl ProfileName {
    /// The `default` profile, which adopts saves already on disk.
    pub fn default_profile() -> Self {
        Self(DEFAULT_PROFILE.to_string())
    }

    /// Whether this is the `default` profile.
    pub fn is_default(&self) -> bool {
        self.0 == DEFAULT_PROFILE
    }

    /// The name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ProfileName {
    type Err = SavesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        check_name(s)?;
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for ProfileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Exclusive use of a profile; released on drop, or when the process dies.
#[derive(Debug)]
pub struct ProfileLock {
    _file: std::fs::File,
}

/// One game's profiles: `<store>/<game>/`.
#[derive(Debug, Clone)]
pub struct GameSaves {
    dir: PathBuf,
}

impl GameSaves {
    /// The profiles of `game_id` under `store`.
    pub fn new(store: &Path, game_id: &str) -> Result<Self, SavesError> {
        check_name(game_id)?;
        Ok(Self {
            dir: store.join(game_id),
        })
    }

    /// `<store>/<game>/<profile>`.
    pub fn profile_dir(&self, profile: &ProfileName) -> PathBuf {
        self.dir.join(profile.as_str())
    }

    /// Receives every write to the game dir.
    pub fn root_upper(&self, profile: &ProfileName) -> PathBuf {
        self.profile_dir(profile).join("root")
    }

    /// The overlay work dir, kept out of the profile so copies skip it.
    pub fn root_work(&self, profile: &ProfileName) -> PathBuf {
        self.dir.join(WORK_DIR).join(profile.as_str())
    }

    /// Locks `profile` so no other launch or command uses it meanwhile.
    pub fn lock(&self, profile: &ProfileName) -> Result<ProfileLock, SavesError> {
        let dir = self.dir.join(WORK_DIR);
        std::fs::create_dir_all(&dir).map_err(SavesError::io("creating", &dir))?;
        let path = dir.join(format!("{profile}.lock"));
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(SavesError::io("opening", &path))?;
        match file.try_lock() {
            Ok(()) => Ok(ProfileLock { _file: file }),
            Err(std::fs::TryLockError::WouldBlock) => {
                Err(SavesError::ProfileInUse(profile.to_string()))
            }
            Err(std::fs::TryLockError::Error(e)) => Err(SavesError::io("locking", &path)(e)),
        }
    }

    /// Where a bound save location lives in the profile.
    pub fn path_dir(&self, profile: &ProfileName, store_rel: &Path) -> PathBuf {
        self.profile_dir(profile).join("paths").join(store_rel)
    }

    /// Every profile, sorted.
    pub fn list(&self) -> Result<Vec<ProfileName>, SavesError> {
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(SavesError::io("reading", &self.dir)(e)),
        };
        let mut profiles = Vec::new();
        for entry in entries {
            let entry = entry.map_err(SavesError::io("reading", &self.dir))?;
            let is_dir = entry
                .file_type()
                .map_err(SavesError::io("reading", &entry.path()))?
                .is_dir();
            if let (true, Some(Ok(name))) = (is_dir, entry.file_name().to_str().map(str::parse)) {
                profiles.push(name);
            }
        }
        profiles.sort();
        Ok(profiles)
    }

    /// The profile picked with [`GameSaves::set_current`], else `default`.
    pub fn current(&self) -> Result<ProfileName, SavesError> {
        let file = self.dir.join(CURRENT_FILE);
        match std::fs::read_to_string(&file) {
            Ok(name) => name.trim().parse(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(ProfileName::default_profile())
            }
            Err(e) => Err(SavesError::io("reading", &file)(e)),
        }
    }

    /// Makes `profile` the one used by later launches.
    pub fn set_current(&self, profile: &ProfileName) -> Result<(), SavesError> {
        self.require(profile)?;
        let file = self.dir.join(CURRENT_FILE);
        std::fs::write(&file, format!("{profile}\n")).map_err(SavesError::io("writing", &file))
    }

    /// Creates `profile`, empty or as a copy of `from`.
    pub fn create(
        &self,
        profile: &ProfileName,
        from: Option<&ProfileName>,
    ) -> Result<(), SavesError> {
        let dir = self.profile_dir(profile);
        if dir.exists() {
            return Err(SavesError::ProfileExists(profile.to_string()));
        }
        match from {
            Some(from) => {
                self.require(from)?;
                let _lock = self.lock(from)?;
                copy_tree(&self.profile_dir(from), &dir)
            }
            None => std::fs::create_dir_all(&dir).map_err(SavesError::io("creating", &dir)),
        }
    }

    /// Writes `profile` to a `.tar.gz` archive.
    pub fn export(&self, profile: &ProfileName, archive: &Path) -> Result<(), SavesError> {
        self.require(profile)?;
        let _lock = self.lock(profile)?;
        run_tar(
            "export",
            Command::new("tar")
                .arg("-C")
                .arg(&self.dir)
                .arg("-czf")
                .arg(archive)
                .arg(profile.as_str()),
        )
    }

    /// Creates `profile` from an archive made by [`GameSaves::export`].
    pub fn import(&self, profile: &ProfileName, archive: &Path) -> Result<(), SavesError> {
        let dir = self.profile_dir(profile);
        if dir.exists() {
            return Err(SavesError::ProfileExists(profile.to_string()));
        }
        let staging = self.dir.join(format!("{WORK_DIR}/import-{profile}"));
        // Leftover from an interrupted import; nothing else lives there.
        let _ = std::fs::remove_dir_all(&staging);
        std::fs::create_dir_all(&staging).map_err(SavesError::io("creating", &staging))?;
        let result = run_tar(
            "import",
            Command::new("tar")
                .arg("-C")
                .arg(&staging)
                .arg("-xzf")
                .arg(archive)
                .arg("--no-same-owner"),
        )
        .and_then(|()| single_subdir(&staging, archive))
        .and_then(|src| std::fs::rename(&src, &dir).map_err(SavesError::io("moving", &src)));
        // Best effort: the import result is what matters.
        let _ = std::fs::remove_dir_all(&staging);
        result
    }

    fn require(&self, profile: &ProfileName) -> Result<(), SavesError> {
        if self.profile_dir(profile).is_dir() {
            Ok(())
        } else {
            Err(SavesError::NoSuchProfile(profile.to_string()))
        }
    }
}

fn run_tar(action: &'static str, cmd: &mut Command) -> Result<(), SavesError> {
    let status = cmd.status().map_err(|e| SavesError::Tar {
        action,
        reason: e.to_string(),
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(SavesError::Tar {
            action,
            reason: status.to_string(),
        })
    }
}

/// The one dir an archive extracted to.
fn single_subdir(staging: &Path, archive: &Path) -> Result<PathBuf, SavesError> {
    let mut dirs = std::fs::read_dir(staging)
        .map_err(SavesError::io("reading", staging))?
        .filter_map(Result::ok)
        .map(|e| e.path());
    match (dirs.next(), dirs.next()) {
        (Some(dir), None) if dir.is_dir() => Ok(dir),
        _ => Err(SavesError::BadArchive(archive.to_path_buf())),
    }
}
