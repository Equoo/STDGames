use anyhow::Result;
use tracing::{info, warn};
use crate::pipeline::{
    PipelineContext,
    stages::{PipelineStage, hooks::PostExitHookStage},
};

pub struct Pipeline {
    stages: Vec<Box<dyn PipelineStage>>,
    post_hooks: Option<PostExitHookStage>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new(), post_hooks: None }
    }

    /// Add a stage to the pipeline
    pub fn add<S: PipelineStage + 'static>(mut self, stage: S) -> Self {
        self.stages.push(Box::new(stage));
        self
    }

    /// Register post-exit hooks (run after game exits, outside pipeline)
    pub fn with_post_hooks(mut self, hooks: PostExitHookStage) -> Self {
        self.post_hooks = Some(hooks);
        self
    }

    /// Execute every stage in order against the shared context
    pub async fn execute(self, mut ctx: PipelineContext) -> Result<PipelineContext> {
        info!("=== Launch Pipeline Starting ({} stages) ===", self.stages.len());

        for stage in &self.stages {
            if stage.should_skip(&ctx) {
                info!("[{}] — skipped", stage.name());
                ctx.log(format!("[{}] skipped", stage.name()));
                continue;
            }

            info!("[{}] running…", stage.name());
            stage.run(&mut ctx).await.map_err(|e| {
                anyhow::anyhow!("Pipeline stage '{}' failed: {}", stage.name(), e)
            })?;
            info!("[{}] ✓", stage.name());
        }

        info!("=== Pipeline Complete ===");

        // Post-exit hooks run outside the pipeline (after game exits)
        if let Some(hooks) = self.post_hooks {
            hooks.run_after(&ctx.env).await;
        }

        Ok(ctx)
    }
}
