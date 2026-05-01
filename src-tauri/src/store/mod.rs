use std::process::ExitStatus;
use anyhow::Result;

// mod epic;
mod steam;
// mod switch;

use serde::{Deserialize, Serialize};
pub use steam::SteamStore;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum StoreId {
    Steam,
    Epic,
    Switch
}

pub trait Store {

}
