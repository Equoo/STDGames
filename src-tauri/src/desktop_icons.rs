
use std::env;
use users::get_current_username;
use std::path::Path;

// TODO: remove the old desktop file if it exist
// TODO: add an desktop icon action to remove itself
// update the database entry using `update-desktop-database ~/.local/share/applications`


#[tauri::command]
pub fn add_launcher_desktop_icon() -> String
{

    let home = env::var("HOME").unwrap_or(format!("/home/{}", get_current_username().unwrap_or(std::ffi::OsString::from("hoadjwao")).to_string_lossy().into_owned()));

    println!("home is {:#?}", home);


    let desktop_file = Path::new(&home).join(".local/share/applications/stdgames.desktop");


    println!("desktop is {:#?}", desktop_file);

    if desktop_file.exists() {
        return "".to_string();
    }


    match std::os::unix::fs::symlink("/sgoinfre/stdgames/.ressources/stdgames.desktop", desktop_file)
    {
        Ok(()) => return "".to_string(),
        Err(e) => return e.to_string(),
    }

}

