use crate::{command::CommandSpec, ctx::LaunchCtx, error::CoreError, ids::RunnerId, target::TargetKind};

/// What actually gets executed at the core. Exactly one per launch,
/// determined by the target kind, never by the mode.
pub trait Runner: Send + Sync {
    fn id(&self) -> RunnerId;
    fn accepts(&self, target: &TargetKind) -> bool;
    fn build(&self, ctx: &LaunchCtx) -> Result<CommandSpec, CoreError>;
}
