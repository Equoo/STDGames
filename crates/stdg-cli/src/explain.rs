//! `stdgames explain --game <id> --mode <mode>`
//!
//! The primary debugging tool: resolves and prints the full plan for a
//! game/mode pair, including the final command, environment, and container
//! bindings, without launching anything.
//!
//! This has to keep working even while some layers are still `todo!()`
//! stubs, so building the display command walks the pipeline itself
//! (instead of reusing `stdg_exec::run_pipeline`) and catches any stub
//! panic per layer, reporting "not yet implemented" for that step instead
//! of crashing the whole command.

use std::any::Any;
use std::panic::{self, AssertUnwindSafe};

use stdg_core::{ArgValue, Binding, CommandSpec, LaunchCtx, LayerCatalog, RunnerCatalog, SessionId, SessionInfo};

use crate::loading::load_plan;

pub fn run(game_id_str: &str, mode_id_str: &str) -> Result<(), String> {
    let (registry, plan) = load_plan(game_id_str, mode_id_str)?;

    println!("game:   {}", plan.game_id);
    println!("mode:   {}", plan.mode_id);
    println!("target: {:?}", plan.target);
    println!("runner: {}", plan.runner);
    println!();

    println!("slots (application order, innermost to outermost):");
    for (slot, layer_ref) in plan.layers_inside_out() {
        let params: Vec<String> = layer_ref.params.iter().map(|(k, v)| format!("{k}={v}")).collect();
        if params.is_empty() {
            println!("  {slot:<12?} {}", layer_ref.id);
        } else {
            println!("  {slot:<12?} {} ({})", layer_ref.id, params.join(", "));
        }
    }
    println!();

    let runner = registry
        .resolve_runner(&plan.runner)
        .ok_or_else(|| format!("runner `{}` not found in the registry", plan.runner))?;

    let ctx = LaunchCtx {
        plan: plan.clone(),
        session: SessionInfo {
            id: SessionId("explain".to_string()),
            tmp_dir: std::env::temp_dir().join("stdgames-explain"),
        },
        bindings: Vec::new(),
        dry_run: true,
    };

    let mut spec: CommandSpec = runner.build(&ctx).map_err(|e| e.to_string())?;
    let mut bindings: Vec<Binding> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    for (slot, layer_ref) in plan.layers_inside_out() {
        let layer = match registry.resolve_layer(layer_ref, &plan.config) {
            Ok(layer) => layer,
            Err(e) => {
                notes.push(format!("{slot:?}/{}: could not resolve layer: {e}", layer_ref.id));
                continue;
            }
        };

        match panic::catch_unwind(AssertUnwindSafe(|| layer.container_needs())) {
            Ok(needs) => bindings.extend(needs),
            Err(payload) => notes.push(format!(
                "{slot:?}/{}: container_needs() {}",
                layer_ref.id,
                describe_panic(payload.as_ref())
            )),
        }

        let before_patch = spec.clone();
        match panic::catch_unwind(AssertUnwindSafe(|| {
            let mut s = before_patch.clone();
            let result = layer.patch(&mut s, &ctx);
            (s, result)
        })) {
            Ok((patched, Ok(()))) => spec = patched,
            Ok((_, Err(e))) => notes.push(format!("{slot:?}/{}: patch() failed: {e}", layer_ref.id)),
            Err(payload) => notes.push(format!(
                "{slot:?}/{}: patch() {}",
                layer_ref.id,
                describe_panic(payload.as_ref())
            )),
        }

        let before_wrap = spec.clone();
        match panic::catch_unwind(AssertUnwindSafe(|| layer.wrap(before_wrap.clone(), &ctx))) {
            Ok(Ok(outcome)) => spec = outcome.into_command(),
            Ok(Err(e)) => notes.push(format!("{slot:?}/{}: wrap() failed: {e}", layer_ref.id)),
            Err(payload) => notes.push(format!(
                "{slot:?}/{}: wrap() {}",
                layer_ref.id,
                describe_panic(payload.as_ref())
            )),
        }
    }

    panic::set_hook(previous_hook);

    println!("resolved command (as far as implemented layers allow):");
    print_command(&spec);
    println!();

    if !bindings.is_empty() {
        println!("container bindings:");
        for b in &bindings {
            println!("  {:?} {} <- {}", b.mode, b.purpose.0, b.source.host().display());
        }
        println!();
    }

    if !notes.is_empty() {
        println!("not yet implemented for this plan:");
        for note in &notes {
            println!("  - {note}");
        }
    }

    Ok(())
}

fn describe_panic(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "not yet implemented".to_string()
    }
}

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
