//! Validates ITM stat parsing against real game data: builds the full
//! item catalog (with stats) and prints a sample of weapons per category,
//! to sanity-check damage/speed/STR-requirement/proficiency values look
//! like real BG2 weapon data. Read-only.

use std::path::PathBuf;
use std::time::Instant;

use bg2_editor::gamedata::GameData;

fn main() {
    let game_root = PathBuf::from(std::env::args().nth(1).expect("usage: test_itm <game-root>"));
    let gd = GameData::load(&game_root).expect("load gamedata");

    let t0 = Instant::now();
    let catalog = gd.item_catalog_full();
    println!("Full item catalog: {} entries in {:?}", catalog.len(), t0.elapsed());

    match gd.ids("PROFTYPE.IDS") {
        Some(t) => println!("PROFTYPE.IDS: {} entries, e.g. {:?}", t.entries.len(), &t.entries[..t.entries.len().min(5)]),
        None => println!("PROFTYPE.IDS: not found"),
    }
    match gd.ids("STATS.IDS") {
        Some(t) => {
            println!("STATS.IDS: {} entries, e.g. {:?}", t.entries.len(), &t.entries[..t.entries.len().min(5)]);
            for v in [0u32, 89, 90, 91, 92, 104] {
                println!("  STATS.IDS[{v}] = {:?}", t.name(v));
            }
        }
        None => println!("STATS.IDS: not found"),
    }

    for cat_name in ["Large swords", "Small swords", "Bows", "Two-Handed swords", "Bastard swords", "Axes"] {
        println!("\n=== {cat_name} ===");
        let mut shown = 0;
        for entry in catalog.iter() {
            if entry.stats.category_label() != cat_name {
                continue;
            }
            let prof = gd.weapon_proficiency_label(entry.stats.weapon_prof_id);
            let ability = entry.stats.ability.as_ref();
            println!(
                "  {:8} {:30} dmg={:8} type={:10} speed={:>3} range={:>3} STR={:>2} 2H={:5} prof={} ({})",
                entry.resref,
                entry.name,
                ability.map(|a| a.damage_string()).unwrap_or_else(|| "-".to_owned()),
                ability.map(|a| a.damage_type_label()).unwrap_or("-"),
                ability.map(|a| a.speed_factor.to_string()).unwrap_or_else(|| "-".to_owned()),
                ability.map(|a| a.range.to_string()).unwrap_or_else(|| "-".to_owned()),
                entry.stats.min_strength,
                entry.stats.two_handed,
                entry.stats.weapon_prof_id,
                prof,
            );
            shown += 1;
            if shown >= 5 {
                break;
            }
        }
        if shown == 0 {
            println!("  (none found)");
        }
    }

    // Cache should make a second call effectively instant.
    let t1 = Instant::now();
    let _ = gd.item_catalog_full();
    println!("\nSecond item_catalog_full() call (cached): {:?}", t1.elapsed());
}
