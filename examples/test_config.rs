//! Verifies config save/load round-trips at the new %APPDATA%-based
//! location (not next to the executable).

use bg2_editor::config;

fn main() {
    let game = std::path::PathBuf::from("D:\\SteamLibrary\\steamapps\\common\\Baldur's Gate Enhanced Edition");
    let save = std::path::PathBuf::from("C:\\Users\\cerro\\OneDrive\\Documents\\Baldur's Gate - Enhanced Edition\\save");

    config::save_game_root(&game);
    config::save_save_root(&save);

    let loaded_game = config::load_game_root();
    let loaded_save = config::load_save_root();

    println!("saved game root:  {}", game.display());
    println!("loaded game root: {:?}", loaded_game);
    println!("saved save root:  {}", save.display());
    println!("loaded save root: {:?}", loaded_save);

    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let cfg_path = std::path::Path::new(&appdata).join("bg2-editor").join("bg2-editor.cfg");
    println!("\nconfig file: {}", cfg_path.display());
    println!("exists: {}", cfg_path.is_file());
    if let Ok(text) = std::fs::read_to_string(&cfg_path) {
        println!("contents:\n{text}");
    }

    let ok = loaded_game.as_deref() == Some(game.as_path()) && loaded_save.as_deref() == Some(save.as_path());
    if ok {
        println!("CONFIG ROUND TRIP OK");
    } else {
        println!("CONFIG ROUND TRIP FAILED");
        std::process::exit(1);
    }
}
