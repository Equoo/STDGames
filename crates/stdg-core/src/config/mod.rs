//! Raw (partial) and resolved game configuration.

mod raw;
mod resolved;
mod saves;

pub use raw::{PartialGameConfig, PartialModeConfig};
pub use resolved::ResolvedConfig;
pub use saves::SavesConfig;
