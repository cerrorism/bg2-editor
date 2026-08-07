//! Verifies class/race/kit name localization against real game data, in
//! both English and Chinese, before trusting it in the UI. Read-only.

use std::path::PathBuf;

use bg2_editor::gamedata::GameData;

fn main() {
    let game_root = PathBuf::from(std::env::args().nth(1).expect("usage: test_localize <game-root>"));

    {
        let gd = GameData::load(&game_root).expect("load gamedata");
        if let Some(t) = gd.ids("KIT.IDS") {
            println!("KIT.IDS entries (first 15): {:?}", &t.entries[..t.entries.len().min(15)]);
        }
    }

    for locale in [None, Some("zh_CN")] {
        let gd = GameData::load_with_locale(&game_root, locale).expect("load gamedata");
        println!("=== locale: {locale:?} ===");
        for class_id in [1u32, 2, 3, 4, 5, 6, 11, 12] {
            println!("  class {class_id}: {:?}", gd.class_name(class_id));
        }
        for race_id in [1u32, 2, 3, 4, 5, 6, 7] {
            println!("  race {race_id}: {:?}", gd.race_name(race_id));
        }
        for kit_id in [0u32, 0x4001, 0x4003, 0x4004, 0x4014] {
            println!("  kit {kit_id:#06x}: {:?}", gd.kit_name(kit_id));
        }
    }
}
