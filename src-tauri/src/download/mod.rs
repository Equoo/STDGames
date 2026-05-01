
use std::{collections::{HashMap, HashSet}, net::SocketAddr, path::PathBuf, sync::RwLock};

use anyhow::Result;

use crate::library::Game;

// struct NodeInfo {
//     addr: SocketAddr,
//     game_ids: HashSet<&'static str>
// }

struct DownloadItem {
    game: &'static str,
    progress: f32,
    destination: PathBuf
}

// TODO: If TEMP installed check if owned or not
pub struct DownloadManager {
    // nodes: RwLock<HashMap<u32, NodeInfo>>,
    downloads: HashMap<&'static str, DownloadItem>,
    current: Option<&'static str>
}

impl DownloadManager {

}

pub trait DownloadMode {
    fn start(game: &Game);
    fn progress() -> f32;
    fn mode_id() -> DownloadModeId;
    fn is_ready() ->  bool;
    fn cancel();
}
