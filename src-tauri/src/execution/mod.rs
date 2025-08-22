use std::process::Child;

mod build_command;
mod junest;

#[derive(Clone)]
pub struct Overlay {
	pub src: Vec<String>,
	pub dst: String,
}

#[derive(Clone)]
pub struct GameProcess {
	pub process: Child,
	pub game: String,
}

pub struct GameExecution {
	pub running: Option<GameProcess>,
}