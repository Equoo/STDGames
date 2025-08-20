use std::env;
use std::error::Error;

pub struct Config {
	pub username: String,
    pub user_home: String,
    pub user_save_dir: String,
    pub resources_dir: String,
	pub junest_bin: String,
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
        let resources_dir = "/sgoinfre/stdgames/.resources".to_string();
        let temp_dir = format!("/tmp/stdgames/{}", username).to_string();
        Ok(Config {
			user_home:					format!("/home/{username}").to_string(),
            user_save_dir:				format!("/sgoinfre/{username}/.stdgames_saves").to_string(),
			username:					username,
			junest_bin:					format!("{resources_dir}/junest/bin/junest").to_string(),
            resources_desktop_file:		format!("{resources_dir}/stdgames.desktop").to_string(),
            resources_junest_home_dir:	format!("{resources_dir}/junest_home").to_string(),
            resource_umu_archive_file:	format!("{resources_dir}/umu.zip").to_string(),
            resources_dir:				resources_dir,
            temp_junest_home_dir:		format!("{temp_dir}/junest").to_string(),
            temp_umu_dir:				format!("{temp_dir}/umu").to_string(),
            temp_dir:					temp_dir,
        })
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::default().expect("Failed to initialize Config")
});
