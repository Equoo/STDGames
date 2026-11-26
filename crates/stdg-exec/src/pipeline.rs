//! Runs every layer of a plan in order to build the final command.

use stdg_core::{
    Binding, CommandSpec, CoreError, LaunchCtx, LayerCatalog, Outcome, RunnerCatalog, SessionGuard,
    Severity,
};
use stdg_registry::Registry;

use crate::error::ExecError;

/// The result of [`run_pipeline`].
#[derive(Debug)]
pub struct PipelineOutput {
    /// The final, fully wrapped command.
    pub outcome: Outcome,
    /// Every container binding the layers requested.
    pub bindings: Vec<Binding>,
    /// Dropping these runs every layer's cleanup.
    pub guards: Vec<Box<dyn SessionGuard>>,
}

/// Runs every layer of `ctx.plan`, innermost first.
pub fn run_pipeline(registry: &Registry, mut ctx: LaunchCtx) -> Result<PipelineOutput, ExecError> {
    let runner = registry
        .resolve_runner(&ctx.plan.runner)
        .ok_or_else(|| CoreError::UnknownRunner(ctx.plan.runner.clone()))?;
    let mut spec: CommandSpec = runner.build(&ctx)?;

    let ordered: Vec<_> = ctx
        .plan
        .layers_inside_out()
        .map(|(slot, layer_ref)| (slot, layer_ref.clone()))
        .collect();

    let mut guards: Vec<Box<dyn SessionGuard>> = Vec::new();

    for (_slot, layer_ref) in ordered {
        let layer = registry.resolve_layer(&layer_ref, &ctx.plan.config)?;

        if let Err(diagnostic) = layer.preflight(&ctx) {
            match diagnostic.severity {
                Severity::Error => {
                    return Err(CoreError::Preflight {
                        layer: layer.id(),
                        diagnostic,
                    }
                    .into());
                }
                Severity::Warning => tracing::warn!(
                    "{}: {}{}",
                    layer.id(),
                    diagnostic.message,
                    diagnostic
                        .hint
                        .map(|h| format!(" (hint: {h})"))
                        .unwrap_or_default()
                ),
            }
        }

        if let Some(prefix) = layer.wine_prefix() {
            ctx.wine_prefix = Some(prefix);
        }

        let guard = layer.prepare(&mut ctx)?;
        guards.push(guard);

        let needs = layer.container_needs(&ctx);
        ctx.bindings.extend(needs);

        layer.patch(&mut spec, &ctx)?;
        let outcome = layer.wrap(spec, &ctx)?;
        spec = outcome.into_command();
    }

    Ok(PipelineOutput {
        outcome: Outcome::Direct(spec),
        bindings: ctx.bindings,
        guards,
    })
}
