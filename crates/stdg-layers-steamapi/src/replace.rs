//! `steamapi-emu` and `steamapi-onlinefix`: replace the Steam API library and/or overlay the game dir.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, BindPurpose, Binding, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx,
    Layer, LayerId, PathValue, SessionGuard, Slot, TargetKind,
};

/// Which preset this layer is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceKind {
    /// Library replaced by default, overlay optional.
    Emu,
    /// Overlay by default, library replacement optional.
    OnlineFix,
}

/// Game platform, which decides the Steam API library names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// `steam_api64.dll` / `steam_api.dll`.
    Windows,
    /// `libsteam_api.so`.
    Linux,
}

impl Platform {
    /// The platform of `target`, if it has a Steam API library.
    pub fn of(target: &TargetKind) -> Option<Self> {
        match target {
            TargetKind::Windows => Some(Platform::Windows),
            TargetKind::NativeLinux => Some(Platform::Linux),
            TargetKind::Rom(_) => None,
        }
    }

    /// Library file names shipped by games of this platform.
    pub fn lib_names(self) -> &'static [&'static str] {
        match self {
            Platform::Windows => &["steam_api64.dll", "steam_api.dll"],
            Platform::Linux => &["libsteam_api.so"],
        }
    }
}

/// CPU architecture of a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// 32-bit x86.
    X86,
    /// x86-64.
    X64,
}

impl Arch {
    /// Layer param holding the replacement for this arch.
    pub fn param(self) -> &'static str {
        match self {
            Arch::X86 => "lib_x86",
            Arch::X64 => "lib_x64",
        }
    }

    /// Reads the ELF/PE header of `path`, falling back to the file name.
    pub fn detect(path: &Path) -> Option<Self> {
        read_header_arch(path)
            .ok()
            .flatten()
            .or_else(|| match path.file_name()?.to_str()? {
                "steam_api64.dll" => Some(Arch::X64),
                "steam_api.dll" => Some(Arch::X86),
                _ => None,
            })
    }
}

fn read_header_arch(path: &Path) -> std::io::Result<Option<Arch>> {
    let mut file = File::open(path)?;
    let mut head = [0u8; 64];
    file.read_exact(&mut head)?;
    if head.starts_with(b"\x7fELF") {
        return Ok(match head[4] {
            1 => Some(Arch::X86),
            2 => Some(Arch::X64),
            _ => None,
        });
    }
    if head.starts_with(b"MZ") {
        let pe_offset = u32::from_le_bytes([head[0x3c], head[0x3d], head[0x3e], head[0x3f]]);
        let mut pe = [0u8; 6];
        file.seek(SeekFrom::Start(u64::from(pe_offset)))?;
        file.read_exact(&mut pe)?;
        if pe.starts_with(b"PE\0\0") {
            return Ok(match u16::from_le_bytes([pe[4], pe[5]]) {
                0x014c => Some(Arch::X86),
                0x8664 => Some(Arch::X64),
                _ => None,
            });
        }
    }
    Ok(None)
}

/// Every Steam API library of `platform` under `root`, sorted.
pub fn find_steam_api_libs(root: &Path, platform: Platform) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if file_type.is_dir() {
                dirs.push(path);
            } else if entry
                .file_name()
                .to_str()
                .is_some_and(|n| platform.lib_names().contains(&n))
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// One game library to shadow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibReplace {
    /// The game's own library.
    pub target: PathBuf,
    /// Its architecture, if detected.
    pub arch: Option<Arch>,
    /// The library mounted over it, if one is configured for `arch`.
    pub replacement: Option<PathBuf>,
}

/// A directory overlaid on the game root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overlay {
    /// Files laid over the game, at their relative paths.
    pub source: PathBuf,
    /// Persistent dir holding the overlay's `upper` and `work` dirs.
    pub state_dir: PathBuf,
}

/// Replaces the Steam API library and/or overlays a directory on the game.
#[derive(Debug)]
pub struct SteamApiReplaceLayer {
    /// Which preset.
    pub kind: ReplaceKind,
    /// The game root (overlay destination).
    pub game_root: PathBuf,
    /// Whether libraries must be replaced.
    pub replace_lib: bool,
    /// Libraries found in the game.
    pub libs: Vec<LibReplace>,
    /// Optional overlay.
    pub overlay: Option<Overlay>,
    /// Exported to the game when set.
    pub app_id: Option<String>,
}

#[derive(Debug)]
struct PersistentState;

impl SessionGuard for PersistentState {
    fn label(&self) -> &'static str {
        "steamapi-overlay-state"
    }
}

impl Layer for SteamApiReplaceLayer {
    fn id(&self) -> LayerId {
        LayerId(
            match self.kind {
                ReplaceKind::Emu => "steamapi-emu",
                ReplaceKind::OnlineFix => "steamapi-onlinefix",
            }
            .to_string(),
        )
    }

    fn slot(&self) -> Slot {
        Slot::SteamApi
    }

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::of([capabilities::STEAM_HANDSHAKE])
    }

    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        if let Some(overlay) = &self.overlay {
            if !overlay.source.is_dir() {
                return Err(Diagnostic::error(format!(
                    "overlay directory {} does not exist",
                    overlay.source.display()
                ))
                .with_hint("set `overlay_dir`"));
            }
        }
        if self.replace_lib && self.libs.is_empty() {
            return Err(Diagnostic::error(format!(
                "no Steam API library found under {}",
                self.game_root.display()
            ))
            .with_hint("set `target_lib_relpath`, or `replace_lib = \"false\"`"));
        }
        for lib in &self.libs {
            let Some(replacement) = &lib.replacement else {
                let hint = lib
                    .arch
                    .map_or("cannot detect its architecture", Arch::param);
                return Err(Diagnostic::error(format!(
                    "no replacement configured for {}",
                    lib.target.display()
                ))
                .with_hint(format!("set `{hint}`")));
            };
            crate::preflight_replacement_lib(replacement)?;
        }
        Ok(())
    }

    fn container_needs(&self) -> Vec<Binding> {
        let mut needs = Vec::new();
        if let Some(overlay) = &self.overlay {
            needs.push(Binding {
                source: PathValue::Translated {
                    host: overlay.source.clone(),
                    guest: self.game_root.clone(),
                },
                mode: BindMode::Overlay {
                    upper: overlay.state_dir.join("upper"),
                    work: overlay.state_dir.join("work"),
                },
                purpose: BindPurpose(format!("{}-overlay", self.id())),
            });
        }
        for lib in &self.libs {
            if let Some(replacement) = &lib.replacement {
                needs.push(Binding {
                    source: PathValue::Translated {
                        host: replacement.clone(),
                        guest: lib.target.clone(),
                    },
                    mode: BindMode::ReadOnly,
                    purpose: BindPurpose(format!("{}-lib", self.id())),
                });
            }
        }
        needs
    }

    fn patch(&self, spec: &mut CommandSpec, _ctx: &LaunchCtx) -> Result<(), CoreError> {
        if let Some(app_id) = &self.app_id {
            crate::set_app_id_env(spec, app_id);
        }
        Ok(())
    }

    fn prepare(&self, ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        if let (Some(overlay), false) = (&self.overlay, ctx.is_dry_run()) {
            for dir in ["upper", "work"] {
                let path = overlay.state_dir.join(dir);
                std::fs::create_dir_all(&path).map_err(|e| CoreError::LayerFailure {
                    layer: self.id(),
                    reason: format!("could not create {}: {e}", path.display()),
                })?;
            }
        }
        Ok(Box::new(PersistentState))
    }
}
