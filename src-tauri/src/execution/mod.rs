use std::process::Child;

pub mod setup;
pub mod build_command;
pub mod junest;

#[derive(Clone)]
pub struct Overlay {
	pub src: Vec<String>,
	pub dst: String,
}

//#[derive(Clone)]
pub struct GameProcess {
	pub process: Child,
	pub game: String,
}

pub struct GameExecution {
	pub running: Option<GameProcess>,
}

impl GameExecution {
	pub fn new() -> Self {
		Self { running: None }
	}
}