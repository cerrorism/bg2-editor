//! Verifies the party-member name fallback (GAM literal name -> CRE
//! name_strref -> TLK) against a real save with a recruited companion
//! whose literal name field is blank. Read-only.

use std::path::PathBuf;

use bg2_editor::gamedata::GameData;
use bg2_editor::save_file;

fn main() {
    let mut args = std::env::args().skip(1);
    let game_root = PathBuf::from(args.next().expect("usage: test_member_name <game-root> <save-folder>"));
    let save_folder = PathBuf::from(args.next().expect("usage: test_member_name <game-root> <save-folder>"));

    // detect_active_language expects the save *root* (containing save
    // slot folders), not an individual slot folder.
    let save_root = save_folder.parent().expect("save folder should have a parent");
    let locale = save_file::detect_active_language(save_root);
    println!("detected active language: {locale:?}");
    let gd = GameData::load_with_locale(&game_root, locale.as_deref()).expect("load gamedata");
    let gam = save_file::load(&save_folder).expect("load save");

    for (i, m) in gam.party.iter().enumerate() {
        let end = m.name.iter().position(|&b| b == 0).unwrap_or(m.name.len());
        let literal = String::from_utf8_lossy(&m.name[..end]).trim().to_string();
        let resolved = m.cre.as_ref().and_then(|cre| gd.tlk_string(cre.name_strref));
        println!("[{i}] literal_name={literal:?} cre_name_strref={:?} resolved_via_tlk={resolved:?}",
            m.cre.as_ref().map(|c| c.name_strref));
    }
}
