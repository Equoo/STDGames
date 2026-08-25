

use anyhow::Result;
use tracing::{info, debug};

#[tracing::instrument]
impl RuntimeBuilder {
    pub fn reaper(mut self, appid: int) -> Result<Self> {
        info!("running…");

        // TODO: find it from steam installed
        let reaper_path

        debug!("Reaper: {}", reaper_path.display())
        self.arg(&reaper_path.into_os_string().into_string()?);
        self.arg(&format!("AppId={}", appid));
        self.arg("--");

        info!("✓");
        Ok(self)
    }
}
