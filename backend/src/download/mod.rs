
use std::{collections::{HashMap, HashSet}, net::SocketAddr, path::PathBuf, sync::RwLock};

use anyhow::Result;

use crate::library::Game;

pub enum DownloadType {
    GAME,
    TOOL,
    STORE
}

struct DownloadItem {
    typ: DownloadType,
    id: &'static str,
    progress: f32,
    complete: bool,
    destination: PathBuf
}

#[derive(Default)]
pub struct DownloadManager {
    downloads: HashMap<&'static str, DownloadItem>,
    order: Vec<&'static str>,
    downloading: bool,
}

static MANAGER: OnceLock<Mutex<DownloadManager>> = OnceLock::new();

impl DownloadManager {
    pub fn init(server: &'static str) -> Result<()> {
        MANAGER.set(Mutex::new(DownloadManager::default()))?;
        Ok(())
    }

    pub fn add(id: &'static str, typ: DownloadType, dest: PathBuf) -> Result<()> {
        Ok(())
    }

    pub fn start(id: &'static str) -> Result<()> {
        Ok(())
    }
    pub fn progress(id: &'static str) -> Option<f32> {
        None
    }
    pub fn is_completed(id: &'static str) -> Option<bool> {
        None
    }
    pub fn cancel(id: &'static str) -> Result<()> {
        Ok(())
    }
}
