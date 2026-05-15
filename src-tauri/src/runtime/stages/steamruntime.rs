
use anyhow::Result;
use tracing::{info, debug};

fn detect_slr_root() -> Option<PathBuf> {
    let candidates = [
        dirs::home_dir()?.join(".steam/steam/steamapps/common/SteamLinuxRuntime_sniper"),
        dirs::home_dir()?.join(".local/share/Steam/steamapps/common/SteamLinuxRuntime_sniper"),
        // UMU can also use a standalone copy
        dirs::data_local_dir()?.join("umu/runtime/SteamLinuxRuntime_sniper"),
    ];
    candidates.into_iter().find(|p| p.join("_v2-entry-point").exists())
}

#[tracing::instrument]
impl RuntimeBuilder {
    pub fn steamruntime(mut self, runtime: &PathBuf) -> Result<Self> {
        info!("running…");

        let root = if runtime.join("_v2-entry-point").exists() {
            debug!("Runtime already downloaded");
            runtime
        } else {
            if let Some(path) = detect_slr_root() {
                debug!("Found existing steamruntime");
                path
            } else {
                debug!("Downloading…");
                // TODO: Download steamruntime
                runtime
            }
        }

        let entry_point = root.join("_v2-entry-point");
        anyhow::ensure!(entry_point.exists(),
            "SLR entry point not found: {}", entry_point.display());

        debug!("Using steamruntime from: {:?}", root);
        self.arg(entry_point.into_os_string().into_string()?);
        self.arg("--verb=waitforexitandrun");
        self.arg("--");

        self.env("STEAM_COMPAT_INSTALL_PATH", self.workdir.into_os_string().into_string()?);
        self.env("PRESSURE_VESSEL_RUNTIME", root.into_os_string().into_string()?);

        info!("✓");
        Ok(self)
    }
}
