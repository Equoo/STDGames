
use anyhow::Result;
use tracing::{info, debug};

#[tracing::instrument]
impl RuntimeBuilder {
    pub fn proton(mut self, version: &String, compatdata: Option<PathBuf>) -> Result<Self> {
        info!("running…");

        let proton_path

        let proton_bin = self.proton_path.join("proton");
        anyhow::ensure!(proton_bin.exists(),
            "proton script not found at: {}", proton_bin.display());

        // Proton needs to know where its own files are
        self.env("PROTON_PATH", self.proton_path.to_str().unwrap());

        // Also tell the SLR where to find Proton (used by _v2-entry-point)
        self.env(
            "STEAM_COMPAT_TOOL_PATHS",
            self.proton_path.to_str().unwrap()
        );

        self.env("PROTON_VERB", "waitforexitandrun");

        debug!("Proton: {}", version, proton_path.display());
        self.arg(proton_bin.into_os_string().into_string()?);
        self.arg("waitforexitandrun");


        let compatdata_path = compatdata.unwrap_or()

        let pfx = compat_data.join("pfx");
        std::fs::create_dir_all(&pfx)?;

        debug!("CompatData: {}", compatdata_path.display());
        debug!("WINEPREFIX: {}", pfx.display());

        self.env("STEAM_COMPAT_DATA_PATH", compatdata_path.into_os_string().into_string()?);
        self.env("WINEPREFIX", pfx.into_os_string().into_string()?);

        info!("✓");
        Ok(self)
    }
}
