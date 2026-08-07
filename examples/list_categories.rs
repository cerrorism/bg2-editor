//! Lists every distinct item category present in the real catalog, in
//! the same sorted order the picker's dropdown uses, with a count per
//! category. Read-only.

use std::path::PathBuf;

use bg2_editor::gamedata::GameData;

fn main() {
    let game_root = PathBuf::from(std::env::args().nth(1).expect("usage: list_categories <game-root>"));
    let gd = GameData::load(&game_root).expect("load gamedata");
    let catalog = gd.item_catalog_full();

    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for e in catalog.iter() {
        *counts.entry(e.stats.category_label()).or_insert(0) += 1;
    }
    println!("{} distinct categories present, {} total items:\n", counts.len(), catalog.len());
    for (cat, count) in &counts {
        println!("  {cat:30} {count}");
    }
}
