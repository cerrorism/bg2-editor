//! Read-only: loads a real game install's chitin.key/dialog.tlk, then
//! resolves item/spell/class/race/alignment/kit names for a real
//! character loaded from an actual save, exercising the full
//! KEY -> BIF -> resource -> TLK / IDS pipeline against real data.

use std::path::PathBuf;

use bg2_editor::gamedata::GameData;
use bg2_editor::save_file;

fn main() {
    let mut args = std::env::args().skip(1);
    let game_root = PathBuf::from(args.next().expect("usage: test_gamedata <game-root> <save-folder>"));
    let save_folder = PathBuf::from(args.next().expect("usage: test_gamedata <game-root> <save-folder>"));

    let data = GameData::load(&game_root).expect("load gamedata");
    println!("Loaded chitin.key + dialog.tlk from {}", game_root.display());

    let gam = save_file::load(&save_folder).expect("load save");
    let cre = gam.party[0].cre.as_ref().expect("embedded cre");

    println!(
        "\nClass={} ({})  Race={} ({})  Gender={} ({})  Alignment={} ({})  Allegiance={} ({})  Kit={:#06x} ({})",
        cre.class, data.ids_label("CLASS.IDS", cre.class as u32),
        cre.race, data.ids_label("RACE.IDS", cre.race as u32),
        cre.gender, data.ids_label("GENDER.IDS", cre.gender as u32),
        cre.alignment, data.ids_label("ALIGNMEN.IDS", cre.alignment as u32),
        cre.allegiance, data.ids_label("EA.IDS", cre.allegiance as u32),
        cre.kit, data.ids_label("KIT.IDS", cre.kit),
    );

    println!("\nItems ({}):", cre.items.len());
    for it in &cre.items {
        let resref = it.item.as_str();
        let name = data.item_name(&resref).unwrap_or_else(|| "??? (unresolved)".to_string());
        println!("  {resref:8} -> {name}");
    }

    println!("\nKnown spells ({}):", cre.known_spells.len());
    for sp in &cre.known_spells {
        let resref = sp.spell.as_str();
        let name = data.spell_name(&resref).unwrap_or_else(|| "??? (unresolved)".to_string());
        println!("  {resref:8} -> {name}");
    }

    println!("\nMemorized spells ({}):", cre.memorized_spells.len());
    for sp in &cre.memorized_spells {
        let resref = sp.spell.as_str();
        let name = data.spell_name(&resref).unwrap_or_else(|| "??? (unresolved)".to_string());
        println!("  {resref:8} -> {name}");
    }

    // Sanity check: also try a couple of well-known BG1 IDS symbols/values
    // to make sure IDS parsing isn't accidentally empty.
    let class_table = data.ids("CLASS.IDS").expect("CLASS.IDS should resolve");
    println!("\nCLASS.IDS entries loaded: {}", class_table.entries.len());

    let t0 = std::time::Instant::now();
    let items = data.item_catalog();
    let item_elapsed = t0.elapsed();
    let t1 = std::time::Instant::now();
    let spells = data.spell_catalog();
    let spell_elapsed = t1.elapsed();
    println!(
        "\nItem catalog: {} entries in {:?}. Spell catalog: {} entries in {:?}.",
        items.len(), item_elapsed, spells.len(), spell_elapsed
    );
    println!("First 5 items alphabetically:");
    for (rr, name) in items.iter().take(5) {
        println!("  {rr:8} {name}");
    }
    println!("First 5 spells alphabetically:");
    for (rr, name) in spells.iter().take(5) {
        println!("  {rr:8} {name}");
    }
    // Cache should make the second call effectively instant.
    let t2 = std::time::Instant::now();
    let _ = data.item_catalog();
    println!("Second item_catalog() call (cached): {:?}", t2.elapsed());
}
