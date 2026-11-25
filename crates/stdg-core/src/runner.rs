//! The `Runner` trait.

use crate::{
    command::CommandSpec, ctx::LaunchCtx, error::CoreError, ids::RunnerId, target::TargetKind,
};

/// What actually gets executed at the core.
pub trait Runner: Send + Sync + std::fmt::Debug {
    /// The id this runner is registered under.
    fn id(&self) -> RunnerId;
    /// Whether this runner can launch `target`.
    fn accepts(&self, target: &TargetKind) -> bool;
    /// Builds the bare, un-wrapped command for `ctx.plan`.
    fn build(&self, ctx: &LaunchCtx) -> Result<CommandSpec, CoreError>;
}
