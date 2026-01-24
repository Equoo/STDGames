use clap::Parser;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::error::Error;

use crate::cli::Cli;

#[derive(Clone)]
pub struct Config {
    pub games_dir: String,
    pub username: String,
    pub user_home: String,
    pub user_save_dir: String,
    pub resources_dir: String,
    pub junest_bin: String,
    pub desktop_file: String,
    pub junest_home_dir: String,
    pub junest_bind: String,
    pub archive_file: String,
    pub protons_dir: String,
    pub overlay: String,
    pub temp_dir: String,
    pub temp_junest_home_dir: String,
    pub temp_umu_dir: String,
    pub library: String,
    pub log_file: String,
    pub umu_run: String,
}

impl Config {
    pub fn default() -> Result<Config, Box<dyn Error>> {
        let username = env::var("USER")?;
        let std_dir = "/sgoinfre/stdgames".to_string();
        let resources_dir = format!("{std_dir}/.resources").to_string();
        let temp_dir = format!("/tmp/{username}/stdgames").to_string();
        let games_dir = format!("{temp_dir}/stdgames_files").to_string();
        let user_save_dir = format!("/sgoinfre/{username}/.stdgames_saves").to_string();

        let cli = Cli::parse();
        let libdir = if let Some(path) = cli.config {
            path
        } else {
            format!("{resources_dir}/games.toml").to_string()
        };

        Ok(Config {
            games_dir: games_dir,
            user_home: format!("/home/{username}").to_string(),
            log_file: format!("{user_save_dir}/log.txt").to_string(),
            user_save_dir: user_save_dir,
            username: username,
            library: libdir,
            junest_bin: format!("{resources_dir}/junest/bin/junest").to_string(),
            desktop_file: format!("{resources_dir}/stdgames.desktop").to_string(),
            junest_home_dir: format!("{resources_dir}/junest_home").to_string(),
            junest_bind: format!("{resources_dir}/junest_bind/usr").to_string(),
            archive_file: format!("{resources_dir}/umu.zip").to_string(),
            protons_dir: format!("{resources_dir}/protons").to_string(),
            overlay: format!("{resources_dir}/overlay").to_string(),
            umu_run: format!("{resources_dir}/umu-launcher/umu-run").to_string(),
            resources_dir: resources_dir,
            temp_junest_home_dir: format!("{temp_dir}/junest").to_string(),
            temp_umu_dir: format!("{temp_dir}/umu").to_string(),
            temp_dir: temp_dir,
        })
    }

    pub fn build_vars(self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        env_vars.insert("GAMES_DIR".to_string(), self.games_dir);
        env_vars.insert("USER_HOME".to_string(), self.user_home);
        env_vars.insert("LOG_FILE".to_string(), self.log_file);
        env_vars.insert("USER_SAVE_DIR".to_string(), self.user_save_dir);
        env_vars.insert("USERNAME".to_string(), self.username);
        env_vars.insert("LIBRARY".to_string(), self.library);
        env_vars.insert("RESOURCES_DIR".to_string(), self.resources_dir);
        env_vars.insert("JUNEST_BIN".to_string(), self.junest_bin);
        env_vars.insert("DESKTOP_FILE".to_string(), self.desktop_file);
        env_vars.insert("JUNEST_HOME_DIR".to_string(), self.junest_home_dir);
        env_vars.insert("JUNEST_BIND".to_string(), self.junest_bind);
        env_vars.insert("ARCHIVE_FILE".to_string(), self.archive_file);
        env_vars.insert("PROTONS_DIR".to_string(), self.protons_dir);
        env_vars.insert("OVERLAY".to_string(), self.overlay);
        env_vars.insert("TEMP_DIR".to_string(), self.temp_dir);
        env_vars.insert(
            "TEMP_JUNEST_HOME_DIR".to_string(),
            self.temp_junest_home_dir,
        );
        env_vars.insert("TEMP_UMU_DIR".to_string(), self.temp_umu_dir);
        env_vars.insert("UMU_RUN".to_string(), self.umu_run);
        env_vars.extend(env::vars().map(|(k, v)| (k, v)));
        env_vars
    }
}

pub static CONFIG: Lazy<Config> =
    Lazy::new(|| Config::default().expect("Failed to initialize Config"));
