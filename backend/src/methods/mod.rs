use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::library::Game;

pub mod native;
// use  epic;
// mod onlinefix;
// mod steam;
// mod switch;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum ModeId {
    Native,
    Steam,
    SteamOnlineFix,
    Epic,
    Switch,
}

pub trait LaunchMode {
    fn build_cmd(&self, game: &Game) -> Command;
    fn mode_id(&self) -> ModeId;
    fn name(&self) -> &'static str;
}
