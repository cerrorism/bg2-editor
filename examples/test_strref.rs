//! Ad-hoc: resolves a raw strref in both en_US and a given locale, to
//! verify class/race/kit name strrefs found in clastext.2DA/racetext.2DA
//! actually resolve to sane text before wiring them into the app.

use std::path::PathBuf;

use bg2_editor::gamedata::GameData;

fn main() {
    let mut args = std::env::args().skip(1);
    let game_root = PathBuf::from(args.next().expect("usage: test_strref <game-root> <strref> [locale]"));
    let strref: i32 = args.next().expect("usage: test_strref <game-root> <strref> [locale]").parse().expect("strref must be an int");
    let locale = args.next();

    let gd_en = GameData::load(&game_root).expect("load en");
    println!("en_US: {:?}", gd_en.tlk_string(strref));

    if let Some(loc) = locale {
        let gd_loc = GameData::load_with_locale(&game_root, Some(&loc)).expect("load locale");
        println!("{loc}: {:?}", gd_loc.tlk_string(strref));
    }
}
