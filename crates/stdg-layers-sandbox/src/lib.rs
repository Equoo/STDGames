//! Sandbox slot: wraps the command in [Conty](https://github.com/Kron4ek/Conty).

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx, Layer, LayerId,
    Outcome, PathValue, Slot,
};

/// Host linker vars dropped before the inner env is applied.
const STRIP_ENV_VARS: [&str; 2] = ["LD_LIBRARY_PATH", "LD_PRELOAD"];

/// Conty names looked up on `PATH` when `conty_path` is unset.
const CONTY_BINARY_NAMES: [&str; 2] = ["conty", "conty.sh"];

/// Conty's `SANDBOX_LEVEL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxLevel {
    /// `1`: throwaway home.
    UserFiles,
    /// `2`: also hides processes and dbus.
    Processes,
    /// `3`: also no network or X11.
    Network,
}

impl SandboxLevel {
    /// The value Conty expects in `SANDBOX_LEVEL`.
    pub fn as_conty_value(self) -> &'static str {
        match self {
            SandboxLevel::UserFiles => "1",
            SandboxLevel::Processes => "2",
            SandboxLevel::Network => "3",
        }
    }
}

/// A `sandbox_level` that is not `1`, `2` or `3`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidSandboxLevel(pub String);

impl fmt::Display for InvalidSandboxLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sandbox level must be 1, 2 or 3 (got {:?})", self.0)
    }
}

impl std::error::Error for InvalidSandboxLevel {}

impl FromStr for SandboxLevel {
    type Err = InvalidSandboxLevel;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "1" => Ok(SandboxLevel::UserFiles),
            "2" => Ok(SandboxLevel::Processes),
            "3" => Ok(SandboxLevel::Network),
            other => Err(InvalidSandboxLevel(other.to_string())),
        }
    }
}

/// Wraps the command in Conty.
#[derive(Debug)]
pub struct ContyLayer {
    /// Conty executable; looked up on `PATH` when `None`.
    pub conty_path: Option<PathBuf>,
    /// Extra isolation; `None` keeps Conty's default.
    pub sandbox_level: Option<SandboxLevel>,
    /// Conty's `BASE_DIR`; `None` means `/tmp`.
    pub base_dir: Option<PathBuf>,
}

impl ContyLayer {
    /// Conty resolved from `PATH`, no extra sandbox, default base dir.
    #[must_use]
    pub fn new() -> Self {
        Self {
            conty_path: None,
            sandbox_level: None,
            base_dir: None,
        }
    }

    /// Conty at an explicit path.
    #[must_use]
    pub fn at(conty_path: PathBuf) -> Self {
        Self {
            conty_path: Some(conty_path),
            sandbox_level: None,
            base_dir: None,
        }
    }

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

        Ok(())
    }

    fn wrap(&self, inner: CommandSpec, ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        let conty = self.resolve().unwrap_or_else(|| PathBuf::from("conty"));
        let mut spec = CommandSpec::new(PathValue::Host(conty));

        // Conty reads its settings from its own environment.
        spec.set_env_literal("QUIET_MODE", "1");
        if let Some(level) = self.sandbox_level {
            spec.set_env_literal("SANDBOX", "1");
            spec.set_env_literal("SANDBOX_LEVEL", level.as_conty_value());
        }
        if let Some(base_dir) = &self.base_dir {
            spec.set_env_path("BASE_DIR", PathValue::Host(base_dir.clone()));
        }

        // Remaining args go straight to bwrap. Game root is read-write for saves.
        spec.push_arg_literal("--bind");
        spec.push_arg_path(PathValue::Host(ctx.plan.config.root.clone()));
        spec.push_arg_path(PathValue::Host(ctx.plan.config.root.clone()));

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

fn find_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(name))
        .find(|p| p.is_file())
}

/// Strips host linker vars, then applies `inner.env` so it wins.
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
