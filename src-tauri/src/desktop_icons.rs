
use std::env;
use users::get_current_username;


#[tauri::command]
pub fn add_launcher_desktop_icon()
{

    let home = env::var("HOME").unwrap_or(format!("/home/{}", get_current_username().unwrap().to_string_lossy().into_owned()));

    // println!("home is {:#?}", home);

    // let desktop = Path::new(home).join(".local/share/.application/STDGames.desktop");
    // std::fs::soft_link("/sgoinfre/42GamingNight/.STDGames/Ressources/STDGames.desktop", desktop).unwrap();


}

