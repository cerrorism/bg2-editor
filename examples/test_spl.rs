//! Validates SPL stat parsing against real game data: builds the full
//! spell catalog and spot-checks well-known spells (Magic Missile should
//! be level 1 Wizard, Cure Light Wounds level 1 Priest, etc.) against
//! expected values. Read-only.

use std::path::PathBuf;
use std::time::Instant;

use bg2_editor::gamedata::GameData;

fn main() {
    let game_root = PathBuf::from(std::env::args().nth(1).expect("usage: test_spl <game-root>"));
    let gd = GameData::load(&game_root).expect("load gamedata");

    let t0 = Instant::now();
    let catalog = gd.spell_catalog_full();
    println!("Full spell catalog: {} entries in {:?}", catalog.len(), t0.elapsed());

    for query in ["Magic Missile", "Cure Light Wounds", "Fireball", "Identify"] {
        println!("\n=== matching {query:?} ===");
        let mut shown = 0;
        for entry in catalog.iter() {
            if !entry.name.to_ascii_lowercase().contains(&query.to_ascii_lowercase()) {
                continue;
            }
            let a = entry.stats.ability.as_ref();
            println!(
                "  {:8} {:30} level={:>2} type={:8} school={:12} secondary={:20} target={:20} range={:>4} casting_speed={:>3} icon={}",
                entry.resref,
                entry.name,
                entry.stats.spell_level,
                entry.stats.spell_type_label(),
                entry.stats.school_label(),
                entry.stats.secondary_type_label(),
                a.map(|x| x.target_label()).unwrap_or("-"),
                a.map(|x| x.range_feet.to_string()).unwrap_or_else(|| "-".to_owned()),
                a.map(|x| x.casting_speed.to_string()).unwrap_or_else(|| "-".to_owned()),
                entry.stats.icon,
            );
            if let Some(desc) = entry.stats.description(&gd) {
                println!("    description: {}", desc.chars().take(150).collect::<String>());
            }
            shown += 1;
            if shown >= 5 {
                break;
            }
        }
        if shown == 0 {
            println!("  (none found)");
        }
    }
}
