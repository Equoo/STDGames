//! Save profiles: where a game's saves and configs live, and which set it uses.
//!
//! Every launch runs with one profile, stored in `<store>/<game>/<profile>/`:
//!
//! - `root/` receives every write to the game dir, which stays read-only.
//! - `paths/<placeholder>/...` is bind-mounted over each save location, found
//!   in the [Ludusavi manifest](https://github.com/mtkennerly/ludusavi-manifest)
//!   and the game's `[saves] paths`.
//!
//! ```
//! use stdg_saves::ProfileName;
//!
//! let name: ProfileName = "speedrun".parse()?;
//! assert_eq!(name.as_str(), "speedrun");
//! # Ok::<(), stdg_saves::SavesError>(())
//! ```
//!
//! This crate has no feature flags.

mod error;
mod fs_util;
mod layer;
mod location;
mod manifest;
mod profile;

pub use error::SavesError;
pub use layer::{CloudSync, SavesLayer};
pub use location::{Location, Os, PathContext, resolve};
pub use manifest::{Condition, MANIFEST_URL, ManifestIndex, SaveRule};
pub use profile::{GameSaves, ProfileLock, ProfileName};

#[cfg(test)]
mod tests;
