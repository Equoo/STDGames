use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct AppError {
	message: String
}

impl AppError {
	pub fn new(message: &str) -> AppError {
		AppError {message: String::from(message)}
	}
}

impl fmt::Display for AppError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "AppError {}", self.message)
	}
}

impl error::Error for AppError {}

