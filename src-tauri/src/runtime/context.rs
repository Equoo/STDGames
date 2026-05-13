use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::RwLock;

/// The shared mutable context passed through every pipeline stage.
/// Stages read from it, enrich it, and the final stage consumes it to exec.
#[derive(Debug, Clone)]
pub struct PipelineContext {
    
}

impl PipelineContext {
    pub fn new(app_id: u32, exe: PathBuf) -> Self {
        Self {
            app_id,
            exe,
            exe_args: Vec::new(),
            proton_path: PathBuf::new(),
            wine_prefix: PathBuf::new(),
            slr_entry_point: PathBuf::new(),
            reaper_path: None,
            runtime_root: PathBuf::new(),
            verb: "waitforexitandrun".to_string(),
            env: std::env::vars().collect(), // start from inherited env
            audit_log: Vec::new(),
            dry_run: false,
        }
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        let entry = msg.into();
        tracing::debug!("[pipeline] {}", entry);
        self.audit_log.push(entry);
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.insert(key.to_string(), val.to_string());
    }
}

MangoHud/etc
SSHFS - Docker
SteamEmu
SteamOnlineFix
overlay - savescript:
 - thread save and hook save at end
Reaper -> steam mode/online
SteamENV
SteamRuntime
Proton (contain SteamRuntime)
CompatibilityLayer


For all:
- MangoHUD/etc
- no download ? SSHFS-Docker

Native:
- overlay
- isSteam ? steam emu
- SteamRuntime
- Windows ? Proton

SteamOnline:
- overlay
- SteamOnlineFix
- SteamRuntime
- Windows ? Proton
- Reaper

Steam:
- overlay
- SteamRuntime
- WIndows ? Proton
- Reaper



