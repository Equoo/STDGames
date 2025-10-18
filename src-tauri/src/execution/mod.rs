use std::process::Child;

use crate::library::Game;

pub mod setup;
pub mod build_command;
pub mod junest;
pub mod manager;

#[derive(Clone)]
pub struct Overlay {
	pub src: Vec<String>,
	pub dst: String,
}

//#[derive(Clone)]
pub struct GameProcess {
	pub process: Child,
	pub name: String,
}

pub struct GameExecution {
	library: Vec<Game>,
	pub running: Option<GameProcess>,
}

impl GameExecution {
	pub fn new( lib: Vec<Game>) -> Self {
		Self { library: lib, running: None }
	}
}