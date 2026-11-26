//! Sandbox slot: wraps the command in [Conty](https://github.com/Kron4ek/Conty).

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, Binding, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx, Layer,
    LayerId, Outcome, PathValue, SessionGuard, Slot,
};

/// Host linker vars dropped before the inner env is applied.
const STRIP_ENV_VARS: [&str; 2] = ["LD_LIBRARY_PATH", "LD_PRELOAD"];

/// Conty names looked up on `PATH` when `conty_path` is unset.
const CONTY_BINARY_NAMES: [&str; 2] = ["conty", "conty.sh"];

/// Forces `userns` on (`1`/`true`) or off (`0`/`false`); unset, it follows [`fusermount_is_setuid_root`].
pub const USERNS_ENV: &str = "STDG_CONTY_USERNS";

/// fusermount names in Conty's order: it uses `fusermount3` when present.
const FUSERMOUNT_NAMES: [&str; 2] = ["fusermount3", "fusermount"];

/// A [`USERNS_ENV`] value that is not a boolean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidUserNsFlag(pub String);

impl fmt::Display for InvalidUserNsFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{USERNS_ENV} must be 1, 0, true or false (got {:?})",
            self.0
        )
    }
}

impl std::error::Error for InvalidUserNsFlag {}

/// Parses a [`USERNS_ENV`] value.
///
/// # Errors
///
/// Returns [`InvalidUserNsFlag`] for anything but `1`, `0`, `true` or `false`.
pub fn parse_userns_flag(value: &str) -> Result<bool, InvalidUserNsFlag> {
    match value.trim() {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        other => Err(InvalidUserNsFlag(other.to_string())),
    }
}

/// Whether the fusermount Conty will use is setuid root, so Conty can mount without `userns`.
#[must_use]
pub fn fusermount_is_setuid_root() -> bool {
    FUSERMOUNT_NAMES
        .iter()
        .find_map(|name| find_on_path(name))
        .is_some_and(|path| is_setuid_root(&path))
}

fn is_setuid_root(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path).is_ok_and(|meta| meta.uid() == 0 && meta.mode() & 0o4000 != 0)
}

/// Runs Conty as root in a new user namespace, then maps the game back to this user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserNs {
    /// uid the game runs as.
    pub uid: u32,
    /// gid the game runs as.
    pub gid: u32,
}

impl UserNs {
    /// The current process's uid and gid.
    pub fn current() -> std::io::Result<Self> {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::metadata("/proc/self")?;
        Ok(Self {
            uid: meta.uid(),
            gid: meta.gid(),
        })
    }
}

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
    /// Lets Conty mount its image without setuid `fusermount3` (e.g. inside `JuNest`).
    pub userns: Option<UserNs>,
}

impl ContyLayer {
    /// Conty resolved from `PATH`, no extra sandbox, default base dir.
    #[must_use]
    pub fn new() -> Self {
        Self {
            conty_path: None,
            sandbox_level: None,
            base_dir: None,
            userns: None,
        }
    }

    /// Conty at an explicit path.
    #[must_use]
    pub fn at(conty_path: PathBuf) -> Self {
        Self {
            conty_path: Some(conty_path),
            sandbox_level: None,
            base_dir: None,
            userns: None,
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

        if self.userns.is_some() && find_on_path("unshare").is_none() {
            return Err(Diagnostic::error("unshare was not found on PATH")
                .with_hint("install util-linux, or remove `userns`"));
        }
        Ok(())
    }

    fn prepare(&self, ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        let dir = status_dir(ctx);
        if !ctx.is_dry_run() {
            std::fs::create_dir_all(&dir).map_err(|e| CoreError::LayerFailure {
                layer: self.id(),
                reason: format!("could not create {}: {e}", dir.display()),
            })?;
        }
        Ok(Box::new(StatusDir(dir)))
    }

    fn wrap(&self, inner: CommandSpec, ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        let conty = self.resolve().unwrap_or_else(|| PathBuf::from("conty"));
        let mut spec = if self.userns.is_some() {
            let unshare = find_on_path("unshare").unwrap_or_else(|| PathBuf::from("unshare"));
            let mut spec = CommandSpec::new(PathValue::Host(unshare));
            spec.push_arg_literal("--map-root-user");
            spec.push_arg_literal("--mount");
            spec.push_arg_literal("--");
            spec.push_arg_path(PathValue::Host(conty));
            spec.set_env_literal("ALLOW_ROOT", "1");
            spec
        } else {
            CommandSpec::new(PathValue::Host(conty))
        };

        // Conty reads its settings from its own environment.
        spec.set_env_literal("QUIET_MODE", "1");
        if let Some(level) = self.sandbox_level {
            spec.set_env_literal("SANDBOX", "1");
            spec.set_env_literal("SANDBOX_LEVEL", level.as_conty_value());
        }
        if let Some(base_dir) = &self.base_dir {
            spec.set_env_path("BASE_DIR", PathValue::Host(base_dir.clone()));
        }

        // Remaining args go straight to bwrap.
        if let Some(userns) = self.userns {
            spec.push_arg_literal("--unshare-user");
            spec.push_arg_literal("--uid");
            spec.push_arg_literal(userns.uid.to_string());
            spec.push_arg_literal("--gid");
            spec.push_arg_literal(userns.gid.to_string());
        }
        let root = &ctx.plan.config.root;
        let overlays = overlay_mounts(&ctx.bindings).map_err(|reason| CoreError::LayerFailure {
            layer: self.id(),
            reason,
        })?;
        if !overlays.iter().any(|o| o.dest == *root) {
            spec.push_arg_literal("--bind");
            spec.push_arg_path(PathValue::Host(root.clone()));
            spec.push_arg_path(PathValue::Host(root.clone()));
        }
        for overlay in &overlays {
            push_overlay(&mut spec, overlay);
        }

        // Plain binds after overlays, so they land on top.
        for binding in &ctx.bindings {
            let flag = match binding.mode {
                BindMode::ReadOnly => "--ro-bind",
                BindMode::ReadWrite => "--bind",
                BindMode::Overlay { .. } | BindMode::OverlayLower => continue,
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

        // Conty always exits 0, so the inner command records its own code.
        let dir = status_dir(ctx);
        let status_file = dir.join("status");
        spec.push_arg_literal("--bind");
        spec.push_arg_path(PathValue::Host(dir.clone()));
        spec.push_arg_path(PathValue::Host(dir));
        spec.push_arg_literal("--");
        spec.push_arg_literal("/bin/sh");
        spec.push_arg_literal("-c");
        spec.push_arg_literal(r#""$@"; echo $? > "$0""#);
        spec.push_arg_path(PathValue::Host(status_file.clone()));
        spec.status_file = Some(status_file);
        if let Some(program) = &inner.program {
            spec.push_arg_path(program.clone());
        }
        for arg in &inner.args {
            spec.push_arg(arg.clone());
        }

        Ok(Outcome::Direct(spec))
    }
}

fn status_dir(ctx: &LaunchCtx) -> PathBuf {
    ctx.session.tmp_dir.join("conty")
}

/// Removes the exit-status dir at session end.
#[derive(Debug)]
struct StatusDir(PathBuf);

impl SessionGuard for StatusDir {
    fn label(&self) -> &'static str {
        "conty-status-dir"
    }
}

impl Drop for StatusDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(name))
        .find(|p| p.is_file())
}

/// One overlay mount: the dest dir, the dirs laid over it, and its writable top.
#[derive(Debug, PartialEq, Eq)]
struct OverlayMount {
    dest: PathBuf,
    /// Laid over `dest` in order; the last one wins.
    lowers: Vec<PathBuf>,
    /// `(upper, work)`; `None` mounts the overlay read-only.
    upper: Option<(PathBuf, PathBuf)>,
}

/// Groups overlay bindings by dest, in first-seen order.
fn overlay_mounts(bindings: &[Binding]) -> Result<Vec<OverlayMount>, String> {
    let mut mounts: Vec<OverlayMount> = Vec::new();
    for binding in bindings {
        if !matches!(
            binding.mode,
            BindMode::Overlay { .. } | BindMode::OverlayLower
        ) {
            continue;
        }
        let dest = binding.source.effective();
        let index = if let Some(index) = mounts.iter().position(|m| m.dest == dest) {
            index
        } else {
            mounts.push(OverlayMount {
                dest: dest.to_path_buf(),
                lowers: Vec::new(),
                upper: None,
            });
            mounts.len() - 1
        };
        let mount = &mut mounts[index];
        match &binding.mode {
            BindMode::OverlayLower => mount.lowers.push(binding.source.host().to_path_buf()),
            BindMode::Overlay { upper, work } => {
                if mount.upper.is_some() {
                    return Err(format!(
                        "two writable overlays on {} ({})",
                        dest.display(),
                        binding.purpose.0
                    ));
                }
                mount.upper = Some((upper.clone(), work.clone()));
            }
            BindMode::ReadOnly | BindMode::ReadWrite => {}
        }
    }
    Ok(mounts)
}

/// Mounts the overlay: `dest` itself at the bottom, then its lowers, writes to `upper`.
fn push_overlay(spec: &mut CommandSpec, overlay: &OverlayMount) {
    for lower in std::iter::once(&overlay.dest).chain(&overlay.lowers) {
        spec.push_arg_literal("--overlay-src");
        spec.push_arg_path(PathValue::Host(lower.clone()));
    }
    match &overlay.upper {
        Some((upper, work)) => {
            spec.push_arg_literal("--overlay");
            spec.push_arg_path(PathValue::Host(upper.clone()));
            spec.push_arg_path(PathValue::Host(work.clone()));
        }
        None => spec.push_arg_literal("--ro-overlay"),
    }
    spec.push_arg_path(PathValue::Host(overlay.dest.clone()));
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
