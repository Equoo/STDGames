use anyhow::{Result, anyhow};
use std::{collections::BTreeMap, path::PathBuf, str::FromStr, sync::Arc};
use tokio::process::{Child, Command};

use crate::{
    config::AppConfig,
    store::{Store, StoreId},
};

pub struct StoreProcessManager {
    stores: BTreeMap<StoreId, Box<dyn Store>>,
    child: Option<Child>,
    store_id: Option<StoreId>,
}

// TODO: Keep tracking of child after launcher closing
impl StoreProcessManager {
    pub fn launch(&self, store: &StoreId) -> Result<()> {
        Ok(())
    }

    pub async fn kill(&mut self) -> Result<()> {
        if let Some(child) = self.child.as_mut() {
            child.kill().await?;
        }
        Ok(())
    }
    pub fn is_running(&mut self) -> Result<bool> {
        if let Some(child) = self.child.as_mut() {
            Ok(child.try_wait()?.is_none())
        } else {
            Ok(false)
        }
    }
    pub fn get_running(&self) -> Option<StoreId> {
        self.store_id.clone()
    }
}
