//! `stdgames explain --game <id> --mode <mode>`

use anyhow::Context as _;
use stdg_core::{
    ArgValue, CommandSpec, LaunchCtx, LayerCatalog, RunKind, RunnerCatalog, SessionId, SessionInfo,
};

use crate::loading::load_plan;

#[expect(clippy::print_stdout, reason = "user-facing CLI output")]
pub(crate) fn run(game_id_str: &str, mode_id_str: &str) -> anyhow::Result<()> {
    let (registry, plan) = load_plan(game_id_str, mode_id_str)?;

    println!("game:   {}", plan.game_id);
    println!("mode:   {}", plan.mode_id);
    println!("target: {:?}", plan.target);
    println!("runner: {}", plan.runner);
    println!();

    println!("slots (application order, innermost to outermost):");
    for (slot, layer_ref) in plan.layers_inside_out() {
        let params: Vec<String> = layer_ref
            .params
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        if params.is_empty() {
            println!("  {slot:<12?} {}", layer_ref.id);
        } else {
            println!("  {slot:<12?} {} ({})", layer_ref.id, params.join(", "));
        }
    }
    println!();

    let runner = registry
        .resolve_runner(&plan.runner)
        .with_context(|| format!("runner `{}` not found in the registry", plan.runner))?;

    let mut ctx = LaunchCtx {
        plan: plan.clone(),
        session: SessionInfo {
            id: SessionId("explain".to_string()),
            tmp_dir: std::env::temp_dir().join("stdgames-explain"),
        },
        bindings: Vec::new(),
        run_kind: RunKind::DryRun,
    };

    let mut spec: CommandSpec = runner.build(&ctx)?;
    let mut notes: Vec<String> = Vec::new();

    for (slot, layer_ref) in plan.layers_inside_out() {
        let layer = match registry.resolve_layer(layer_ref, &plan.config) {
            Ok(layer) => layer,
            Err(e) => {
                notes.push(format!(
                    "{slot:?}/{}: could not resolve layer: {e}",
                    layer_ref.id
                ));
                continue;
            }
        };

        if let Err(d) = layer.preflight(&ctx) {
            notes.push(format!(
                "{slot:?}/{}: preflight {:?}: {}",
                layer_ref.id, d.severity, d.message
            ));
        }

        ctx.bindings.extend(layer.container_needs());

        let mut patched = spec.clone();
        match layer.patch(&mut patched, &ctx) {
            Ok(()) => spec = patched,
            Err(e) => notes.push(format!("{slot:?}/{}: patch() failed: {e}", layer_ref.id)),
        }

        match layer.wrap(spec.clone(), &ctx) {
            Ok(outcome) => spec = outcome.into_command(),
            Err(e) => notes.push(format!("{slot:?}/{}: wrap() failed: {e}", layer_ref.id)),
        }
    }

    println!("resolved command:");
    print_command(&spec);
    println!();

    if !ctx.bindings.is_empty() {
        println!("container bindings:");
        for b in &ctx.bindings {
            println!(
                "  {:?} {} <- {}",
                b.mode,
                b.purpose.0,
                b.source.host().display()
            );
        }
        println!();
    }

    if !notes.is_empty() {
        println!("problems found while resolving this plan:");
        for note in &notes {
            println!("  - {note}");
        }
    }

    Ok(())
}

#[expect(clippy::print_stdout, reason = "user-facing CLI output")]
fn print_command(spec: &CommandSpec) {
    match &spec.program {
        Some(program) => println!("  program: {}", program.effective().display()),
        None => println!("  program: <none>"),
    }
    if !spec.args.is_empty() {
        let args: Vec<String> = spec.args.iter().map(ArgValue::render).collect();
        println!("  args:    {}", args.join(" "));
    }
    if let Some(cwd) = &spec.cwd {
        println!("  cwd:     {}", cwd.effective().display());
    }
    if !spec.env.is_empty() {
        println!("  env:");
        for (key, value) in &spec.env {
            println!("    {key}={}", value.render());
        }
    }
}
