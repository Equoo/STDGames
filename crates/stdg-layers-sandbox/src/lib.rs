//! Sandbox slot: bwrap.
//!
//! Mandatory in every mode (see `stdg_plan::validate::MANDATORY_SLOTS`) —
//! there is no unsandboxed launch, only a choice of profile:
//!
//!   - [`SandboxProfile::Normal`]: bind the host's own `/usr` (and its
//!     `bin`/`sbin`/`lib*` layout) read-only. The game sees the host's own
//!     libraries; the namespace still isolates the process and gives inner
//!     layers (a Wine prefix, an injected SteamApi library...) a controlled
//!     place to land via `ctx.bindings`, without ever writing into the
//!     game's own install directory.
//!   - [`SandboxProfile::SuperCompat`]: bind a self-contained root
//!     filesystem image (an Arch Linux install, in practice) as `/` instead
//!     of the host's. For sessions where the user cannot install anything
//!     themselves — no Steam, no system packages, a locked-down shared
//!     machine — this ships everything the game or its compat layers need
//!     without depending on what the host has.
//!
//! Every flag and ordering decision below was checked against a real
//! `bwrap` rather than guessed — see `tests.rs`, whose end-to-end test is
//! skipped automatically wherever `bwrap` isn't on `PATH`.

use std::path::{Path, PathBuf};

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx, Layer, LayerId,
    Outcome, PathValue, Slot,
};

/// Host environment passed through to every profile: display/audio so a
/// game can actually render and produce sound, and identity variables that
/// Python, Wine, and Proton's own bookkeeping assume are set to something
/// real (paired with `bind_home` actually binding the directory `HOME`
/// names). Missing on a headless or Wayland-only host is fine — the
/// corresponding bwrap `-try` bind/var is simply skipped.
const PASSTHROUGH_ENV_VARS: [&str; 7] = [
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "XDG_RUNTIME_DIR",
    "PULSE_SERVER",
    "HOME",
    "USER",
    "LOGNAME",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxProfile {
    Normal,
    SuperCompat,
}

pub struct BwrapLayer {
    pub profile: SandboxProfile,
    /// Root of the self-contained image used by [`SandboxProfile::SuperCompat`].
    /// Ignored for `Normal`; required (and checked in `preflight`) for
    /// `SuperCompat`. Must already be an unpacked directory — this layer
    /// does not fetch or extract an image itself.
    pub image_root: Option<PathBuf>,
}

impl BwrapLayer {
    pub fn normal() -> Self {
        Self {
            profile: SandboxProfile::Normal,
            image_root: None,
        }
    }

    pub fn super_compat(image_root: PathBuf) -> Self {
        Self {
            profile: SandboxProfile::SuperCompat,
            image_root: Some(image_root),
        }
    }
}

impl Layer for BwrapLayer {
    fn id(&self) -> LayerId {
        LayerId("bwrap".to_string())
    }

    fn slot(&self) -> Slot {
        Slot::Sandbox
    }

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::of([capabilities::SANDBOXED])
    }

    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        if find_bwrap().is_none() {
            return Err(Diagnostic::error("bwrap was not found on PATH")
                .with_hint("install bubblewrap (package `bubblewrap` on most distros)"));
        }
        if self.profile == SandboxProfile::SuperCompat {
            match &self.image_root {
                Some(path) if path.is_dir() => {}
                Some(path) => {
                    return Err(Diagnostic::error(format!(
                        "super-compat image root {} does not exist or is not a directory",
                        path.display()
                    )));
                }
                None => {
                    return Err(Diagnostic::error("profile=super-compat requires an `image_root` parameter"));
                }
            }
        }
        Ok(())
    }

    fn wrap(&self, inner: CommandSpec, ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        let bwrap_path = find_bwrap().unwrap_or_else(|| PathBuf::from("bwrap"));
        let mut spec = CommandSpec::new(PathValue::Host(bwrap_path));

        // Namespace shape: full isolation except network (games need it for
        // online features/Steam), supervised so an orphaned sandbox can't
        // outlive the launcher.
        spec.push_arg_literal("--unshare-all");
        spec.push_arg_literal("--share-net");
        spec.push_arg_literal("--die-with-parent");
        spec.push_arg_literal("--proc");
        spec.push_arg_literal("/proc");
        spec.push_arg_literal("--dev");
        spec.push_arg_literal("/dev");
        spec.push_arg_literal("--tmpfs");
        spec.push_arg_literal("/tmp");

        match self.profile {
            SandboxProfile::Normal => bind_host_userspace(&mut spec),
            SandboxProfile::SuperCompat => {
                let image_root = self
                    .image_root
                    .clone()
                    .expect("checked by preflight before wrap is ever called");
                spec.push_arg_literal("--ro-bind");
                spec.push_arg_path(PathValue::Host(image_root));
                spec.push_arg_literal("/");
            }
        }

        // The game's own files: read-write, since saves/config commonly
        // live alongside the install directory.
        spec.push_arg_literal("--bind");
        spec.push_arg_path(PathValue::Host(ctx.plan.config.root.clone()));
        spec.push_arg_path(PathValue::Host(ctx.plan.config.root.clone()));

        bind_gpu_and_display(&mut spec);
        bind_home(&mut spec);

        // Whatever inner layers declared they need across the container
        // boundary (an injected library, an IPC socket...) — this is what
        // `Layer::container_needs()` exists to feed.
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

/// Locates `bwrap` on `PATH` the same way a shell would, since `CommandSpec`
/// always carries an absolute or explicit program path rather than relying
/// on the executor to search `PATH` itself.
fn find_bwrap() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var).map(|dir| dir.join("bwrap")).find(|p| p.is_file())
}

/// Binds the host's own `/usr` read-only and mirrors whatever `/bin`,
/// `/sbin`, `/lib*` layout the host actually has: a symlink into `/usr` on a
/// merged-usr host (the common case today), or a real bind of the
/// standalone directory on an older split layout.
fn bind_host_userspace(spec: &mut CommandSpec) {
    spec.push_arg_literal("--ro-bind");
    spec.push_arg_literal("/usr");
    spec.push_arg_literal("/usr");

    for (dest, usr_relative) in [
        ("/bin", "usr/bin"),
        ("/sbin", "usr/sbin"),
        ("/lib", "usr/lib"),
        ("/lib32", "usr/lib32"),
        ("/lib64", "usr/lib64"),
        ("/libx32", "usr/libx32"),
    ] {
        let host_path = Path::new(dest);
        if host_path.is_symlink() {
            spec.push_arg_literal("--symlink");
            spec.push_arg_literal(usr_relative);
            spec.push_arg_literal(dest);
        } else if host_path.is_dir() {
            spec.push_arg_literal("--ro-bind");
            spec.push_arg_literal(dest);
            spec.push_arg_literal(dest);
        }
        // Neither a symlink nor a directory: this host has no such path
        // (e.g. no /libx32 without multilib) — nothing to mirror.
    }
}

/// GPU device access and display/audio sockets, common to both profiles.
/// The `-try` bwrap flags make every one of these a no-op instead of an
/// error when the source doesn't exist (headless host, Wayland-only,
/// PipeWire instead of PulseAudio...).
fn bind_gpu_and_display(spec: &mut CommandSpec) {
    spec.push_arg_literal("--dev-bind-try");
    spec.push_arg_literal("/dev/dri");
    spec.push_arg_literal("/dev/dri");

    spec.push_arg_literal("--ro-bind-try");
    spec.push_arg_literal("/tmp/.X11-unix");
    spec.push_arg_literal("/tmp/.X11-unix");

    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        spec.push_arg_literal("--bind-try");
        spec.push_arg_literal(runtime_dir.clone());
        spec.push_arg_literal(runtime_dir);
    }
}

/// Binds the host's `$HOME` read-write. Plenty of what runs inside the
/// sandbox — Python's own `pathlib.Path.home()`, Wine, Proton's own
/// bookkeeping — assumes a real, writable home directory exists, not just
/// that the `HOME` variable is set to some string; setting the variable
/// without binding the directory it names leaves lookups like `~/.cache`
/// resolving to a path that doesn't exist inside the sandbox at all.
///
/// This does expose the real host home directory rather than a private
/// synthetic one scoped to the session — an accepted simplification for
/// now (consistent with `Normal` already reusing the host's own `/usr`),
/// not a deliberate security stance; a session-scoped fake `$HOME` would be
/// the harder-isolation follow-up.
fn bind_home(spec: &mut CommandSpec) {
    if let Ok(home) = std::env::var("HOME") {
        spec.push_arg_literal("--bind-try");
        spec.push_arg_literal(home.clone());
        spec.push_arg_literal(home);
    }
}

/// Starts from a clean environment rather than inheriting the launcher's
/// own — sandboxing is meant to isolate, and a stray host env var is exactly
/// the kind of thing it should stop leaking into the game. Only a small,
/// deliberate set gets reintroduced: a sane `PATH`, host display/audio
/// wiring, and whatever the inner layers themselves set on `inner.env`
/// (e.g. SteamAppId) — which, being the most specific, is applied last.
fn set_environment(spec: &mut CommandSpec, inner: &CommandSpec) {
    spec.push_arg_literal("--clearenv");
    spec.push_arg_literal("--setenv");
    spec.push_arg_literal("PATH");
    spec.push_arg_literal("/usr/bin:/bin:/usr/sbin:/sbin");

    for var in PASSTHROUGH_ENV_VARS {
        if let Ok(val) = std::env::var(var) {
            spec.push_arg_literal("--setenv");
            spec.push_arg_literal(var);
            spec.push_arg_literal(val);
        }
    }

    for (key, value) in &inner.env {
        spec.push_arg_literal("--setenv");
        spec.push_arg_literal(key.clone());
        spec.push_arg_literal(value.render());
    }
}

#[cfg(test)]
mod tests;
