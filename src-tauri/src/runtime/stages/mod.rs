pub mod reaper;
pub mod runtime_discovery;
pub mod proton_setup;
pub mod env_steam;
pub mod env_feature_flags;
pub mod compat_data;
pub mod hooks;
pub mod exec;

use async_trait::async_trait;
use anyhow::Result;
use crate::pipeline::PipelineContext;

/// Every stage in the launch pipeline implements this trait.
/// Stages are async, ordered, and composable.
#[async_trait]
pub trait PipelineStage: Send + Sync {
    /// Human-readable name shown in logs and dry-run output
    fn name(&self) -> &'static str;

    /// Execute the stage, mutating the context
    async fn run(&self, ctx: &mut PipelineContext) -> Result<()>;

    /// Whether to skip this stage (allows conditional stages)
    fn should_skip(&self, ctx: &PipelineContext) -> bool {
        false
    }
}
