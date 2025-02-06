
use std::env;
use users::get_current_username;
use std::path::Path;
// TODO: remove the old desktop file if it exist

#[tauri::command]
pub fn add_launcher_desktop_icon() -> String// -> Result<(), Box<dyn std::error::Error>>
{

    // let home = env::var("HOME").unwrap_or(format!("/home/{}", get_current_username().unwrap().to_string_lossy().into_owned()));

    // println!("home is {:#?}", home);

    // let desktop = Path::new(home).join(".local/share/.application/stdgames.desktop");
    // std::fs::soft_link("/sgoinfre/stdgames/.ressources/stdgames.desktop", desktop).unwrap();






    "".to_string()

}

