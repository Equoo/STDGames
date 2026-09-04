use stdg_core::{
    Binding, CommandSpec, CoreError, LaunchCtx, LayerCatalog, Outcome, RunnerCatalog, Severity,
    SessionGuard,
};
use stdg_registry::Registry;

use crate::error::ExecError;

pub struct PipelineOutput {
    pub outcome: Outcome,
    pub bindings: Vec<Binding>,
    /// Kept alive for the lifetime of the launched process; dropping this
    /// vector runs every layer's cleanup (tmpdir removal, cgroup teardown...).
    pub guards: Vec<Box<dyn SessionGuard>>,
}

/// Runs every layer in `ctx.plan` in application order (innermost to
/// outermost), threading the command spec and the accumulated container
/// bindings through `preflight -> prepare -> container_needs -> patch ->
/// wrap`.
///
/// When `ctx.dry_run` is set, layers are expected to skip real side effects
/// in `prepare` (see `Layer::prepare`'s contract) — `explain` relies on this
/// to produce an accurate plan without touching the filesystem.
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
            if diagnostic.severity == Severity::Error {
                return Err(CoreError::Preflight {
                    layer: layer.id(),
                    diagnostic,
                }
                .into());
            }
        }

        let guard = layer.prepare(&mut ctx)?;
        guards.push(guard);

        ctx.bindings.extend(layer.container_needs());

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
