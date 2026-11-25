//! Raw (partial) and resolved game configuration.

mod raw;
mod resolved;

pub use raw::{PartialGameConfig, PartialModeConfig};
pub use resolved::ResolvedConfig;
