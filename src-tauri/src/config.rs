use std::error::Error;
use std::env;

pub struct Config {
	pub user_home: String,
	pub user_save_dir: String,
	pub resources_dir: String,
	pub resources_desktop_file: String,
	pub resources_junest_home_dir: String,
	pub resource_umu_archive_file: String,
	pub temp_dir: String,
	pub temp_junest_home_dir: String,
	pub temp_umu_dir: String,
}

impl Config {
	pub fn default() -> Result<Config, Box<dyn Error>> {
		let username = env::var("USER")?;
		let resources_dir = "/sgoinfre/stdgames/.ressources"; // TODO: fix the typo
		let temp_dir = format!("/tmp/stdgames/{}", username);
		Ok(Config {
			user_home: format!("/home/{}", username).to_string(),
			user_save_dir: format!("/sgoinfre/{}/.stdgames_saves", username).to_string(),
			resources_dir: format!("{}", resources_dir).to_string(),
			resources_desktop_file: format!("{}/stdgames.desktop", resources_dir).to_string(),
			resources_junest_home_dir: format!("{}/junest", resources_dir).to_string(),
			resource_umu_archive_file: format!("{}/umu.zip", resources_dir).to_string(),
			temp_dir: format!("{}", temp_dir).to_string(),
			temp_junest_home_dir: format!("{}/junest_home", temp_dir).to_string(),
			temp_umu_dir: format!("{}/umu", temp_dir).to_string(),
		})
	}
}
