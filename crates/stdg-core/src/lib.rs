//! Core types and traits for the STDGames launch runtime.
//!
//! This crate performs no I/O and spawns no processes. It defines the data
//! model shared by the planner (`stdg-plan`), the executor (`stdg-exec`),
//! and every runner/layer implementation crate.

pub mod binding;
pub mod capability;
pub mod catalog;
pub mod command;
pub mod config;
pub mod ctx;
pub mod error;
pub mod guard;
pub mod ids;
pub mod layer;
pub mod outcome;
pub mod plan;
pub mod runner;
pub mod slot;
pub mod target;

pub use binding::{BindMode, BindPurpose, Binding, PathValue};
pub use capability::{Capability, CapabilitySet};
pub use catalog::{LayerCatalog, RunnerCatalog};
pub use command::{ArgValue, CommandSpec, EnvValue, PathListSeparator};
pub use config::{PartialGameConfig, PartialModeConfig, ResolvedConfig};
pub use ctx::{LaunchCtx, SessionInfo};
pub use error::{ConfigError, CoreError, Diagnostic, Result, Severity};
pub use guard::SessionGuard;
pub use ids::{GameId, LayerId, ModeId, RunnerId, SessionId};
pub use layer::{Layer, LayerRef};
pub use outcome::Outcome;
pub use plan::Plan;
pub use runner::Runner;
pub use slot::Slot;
pub use target::{PlatformId, TargetKind};
