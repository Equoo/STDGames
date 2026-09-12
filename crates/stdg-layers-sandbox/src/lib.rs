//! Sandbox slot: Conty.
//!
//! Mandatory in every mode (see `stdg_plan::validate::MANDATORY_SLOTS`) —
//! there is no unsandboxed launch.
//!
//! This layer wraps the inner command in [Conty][conty], a single-file
//! container runtime that bundles its own (Arch Linux) userspace and, on
//! every run, sets up the user/pid/uts namespaces, `/proc`, `/dev`, the GPU
//! devices and the X11/Wayland/PulseAudio sockets itself. That makes the old
//! "bind the host's `/usr`" vs. "bind a self-contained image as `/`" profile
//! split unnecessary: every launch now runs against Conty's bundled root
//! filesystem, so nothing on the host needs to be installed or mirrored.
//!
//! What this layer still owns is the **bind wiring**:
//!
//!   - the game's own install directory, read-write, since saves/config
//!     commonly live next to the binary;
//!   - whatever inner layers declared they need across the container
//!     boundary via `Layer::container_needs()` / `ctx.bindings` — an
//!     injected library, a Wine prefix, an IPC socket...
//!
//! Both become Conty `--bind` / `--ro-bind` arguments, which Conty forwards
//! straight through to bubblewrap.
//!
//! Extra isolation (a throwaway home, no dbus, no network) is delegated to
//! Conty's own `SANDBOX` / `SANDBOX_LEVEL` mechanism via [`ContyLayer::sandbox_level`]
//! rather than reproduced here.
//!
//! [conty]: https://github.com/Kron4ek/Conty

use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx, Layer, LayerId,
    Outcome, PathValue, Slot,
};

/// Dynamic-linker variables that must never reach the game with a value
/// inherited from the launcher's own environment. Conty does not
/// `--clearenv`, so we drop these explicitly; an inner layer that genuinely
/// needs one (Proton/Wine setting `LD_LIBRARY_PATH`) puts it back through
/// `inner.env`, which is applied afterwards and wins.
const STRIP_ENV_VARS: [&str; 2] = ["LD_LIBRARY_PATH", "LD_PRELOAD"];

/// Candidate names for the Conty executable, tried in order on `PATH` when
/// no explicit `conty_path` is configured.
const CONTY_BINARY_NAMES: [&str; 2] = ["conty", "conty.sh"];

pub struct ContyLayer {
    /// Path to the Conty executable (typically a `conty.sh`). When `None`,
    /// it is looked up on `PATH` (`conty`, then `conty.sh`). Either way the
    /// result is checked in `preflight`.
    pub conty_path: Option<PathBuf>,
    /// When set, Conty runs with `SANDBOX=1` and this `SANDBOX_LEVEL`
    /// (`1` isolates user files, `2` also hides processes/dbus, `3` also
    /// drops network and X11 — see Conty's docs). `None` leaves Conty at its
    /// default: no throwaway home on top of the namespace it always creates.
    pub sandbox_level: Option<u8>,
    /// Maps to Conty's `BASE_DIR` — where it unpacks its helpers and mounts
    /// the image. `None` uses Conty's default (`/tmp`).
    pub base_dir: Option<PathBuf>,
}

impl ContyLayer {
    /// Conty resolved from `PATH`, no extra sandbox, default base dir.
    pub fn new() -> Self {
        Self {
            conty_path: None,
            sandbox_level: None,
            base_dir: None,
        }
    }

    /// Conty at an explicit path.
    pub fn at(conty_path: PathBuf) -> Self {
        Self {
            conty_path: Some(conty_path),
            sandbox_level: None,
            base_dir: None,
        }
    }

    /// The Conty executable to invoke: the configured path if any, else the
    /// first of [`CONTY_BINARY_NAMES`] found on `PATH`.
    fn resolve(&self) -> Option<PathBuf> {
        if let Some(path) = &self.conty_path {
            return Some(path.clone());
        }
        CONTY_BINARY_NAMES
            .iter()
            .find_map(|name| find_on_path(name))
    }
}

impl Default for ContyLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl Layer for ContyLayer {
    fn id(&self) -> LayerId {
        LayerId("conty".to_string())
    }

    fn slot(&self) -> Slot {
        Slot::Sandbox
    }

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::of([capabilities::SANDBOXED])
    }

    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        match &self.conty_path {
            Some(path) if path.is_file() => {}
            Some(path) => {
                return Err(Diagnostic::error(format!(
                    "conty executable {} does not exist or is not a file",
                    path.display()
                ))
                .with_hint("point `conty_path` at this deployment's Conty build"));
            }
            None => {
                if CONTY_BINARY_NAMES
                    .iter()
                    .all(|name| find_on_path(name).is_none())
                {
                    return Err(Diagnostic::error("conty was not found on PATH").with_hint(
                        "install Conty (https://github.com/Kron4ek/Conty) or set `conty_path`",
                    ));
                }
            }
        }

        if let Some(level) = self.sandbox_level {
            if !(1..=3).contains(&level) {
                return Err(Diagnostic::error(format!(
                    "conty sandbox_level must be 1, 2 or 3 (got {level})"
                )));
            }
        }

        Ok(())
    }

    fn wrap(&self, inner: CommandSpec, ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        let conty = self.resolve().unwrap_or_else(|| PathBuf::from("conty"));
        let mut spec = CommandSpec::new(PathValue::Host(conty));

        // Conty is configured through environment variables on the wrapper
        // process itself, not through arguments. Keep it quiet (the game's
        // own stdout/stderr is untouched) and forward the sandbox knobs.
        spec.set_env_literal("QUIET_MODE", "1");
        if let Some(level) = self.sandbox_level {
            spec.set_env_literal("SANDBOX", "1");
            spec.set_env_literal("SANDBOX_LEVEL", level.to_string());
        }
        if let Some(base_dir) = &self.base_dir {
            spec.set_env_path("BASE_DIR", PathValue::Host(base_dir.clone()));
        }

        // Everything from here on is a bubblewrap argument that Conty
        // appends verbatim to its own internal `bwrap` invocation.

        // The game's own files: read-write, since saves/config commonly
        // live alongside the install directory.
        spec.push_arg_literal("--bind");
        spec.push_arg_path(PathValue::Host(ctx.plan.config.root.clone()));
        spec.push_arg_path(PathValue::Host(ctx.plan.config.root.clone()));

        // Whatever inner layers declared they need across the container
        // boundary (an injected library, a Wine prefix, an IPC socket...) —
        // this is what `Layer::container_needs()` exists to feed.
        for binding in &ctx.bindings {
            let flag = match binding.mode {
                BindMode::ReadOnly => "--ro-bind",
                BindMode::ReadWrite => "--bind",
            };
            spec.push_arg_literal(flag);
            spec.push_arg_path(PathValue::Host(binding.source.host().to_path_buf()));
            spec.push_arg_path(PathValue::Host(binding.source.effective().to_path_buf()));
        }

        set_environment(&mut spec, &inner);

        if let Some(cwd) = &inner.cwd {
            spec.push_arg_literal("--chdir");
            spec.push_arg_path(cwd.clone());
        }

        spec.push_arg_literal("--");
        if let Some(program) = &inner.program {
            spec.push_arg_path(program.clone());
        }
        for arg in &inner.args {
            spec.push_arg(arg.clone());
        }

        Ok(Outcome::Direct(spec))
    }
}

/// Locates a binary on `PATH` the same way a shell would, since `CommandSpec`
/// always carries an explicit program path rather than relying on the
/// executor to search `PATH` itself.
fn find_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(name))
        .find(|p| p.is_file())
}

/// Conty inherits the launcher's environment and passes it into the
/// container (it does not `--clearenv`). We don't fight that wholesale, but
/// we do drop the dynamic-linker variables that must not leak from the host
/// ([`STRIP_ENV_VARS`]), then apply whatever the inner layers set on
/// `inner.env` (a Proton/Wine `LD_LIBRARY_PATH`, `SteamAppId`...) last — so
/// the deliberate value wins over both the host and Conty's own defaults.
fn set_environment(spec: &mut CommandSpec, inner: &CommandSpec) {
    for var in STRIP_ENV_VARS {
        spec.push_arg_literal("--unsetenv");
        spec.push_arg_literal(var);
    }

    for (key, value) in &inner.env {
        spec.push_arg_literal("--setenv");
        spec.push_arg_literal(key.clone());
        spec.push_arg_literal(value.render());
    }
}

#[cfg(test)]
mod tests;
