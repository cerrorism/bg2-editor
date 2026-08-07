//! Finds items whose name contains a substring (case-insensitive) and
//! prints full stats — used to spot-check specific items. Read-only.

use std::path::PathBuf;

use bg2_editor::gamedata::GameData;

fn main() {
    let mut args = std::env::args().skip(1);
    let game_root = PathBuf::from(args.next().expect("usage: find_item <game-root> <name-substring> [locale]"));
    let query = args.next().expect("usage: find_item <game-root> <name-substring> [locale]").to_ascii_lowercase();
    let locale = args.next();

    let gd = GameData::load_with_locale(&game_root, locale.as_deref()).expect("load gamedata");
    let catalog = gd.item_catalog_full();

    for entry in catalog.iter() {
        if !entry.name.to_ascii_lowercase().contains(&query) {
            continue;
        }
        let a = entry.stats.ability.as_ref();
        println!(
            "{:8} {:30} category={:20} dmg={:8} type={:10} speed={:>3} range={:>3} STR={:>2} 2H={:5} enchant={} prof={}",
            entry.resref,
            entry.name,
            entry.stats.category_label(),
            a.map(|x| x.damage_string()).unwrap_or_else(|| "-".to_owned()),
            a.map(|x| x.damage_type_label()).unwrap_or("-"),
            a.map(|x| x.speed_factor.to_string()).unwrap_or_else(|| "-".to_owned()),
            a.map(|x| x.range.to_string()).unwrap_or_else(|| "-".to_owned()),
            entry.stats.min_strength,
            entry.stats.two_handed,
            entry.stats.enchantment,
            gd.weapon_proficiency_label(entry.stats.weapon_prof_id),
        );
    }
}
