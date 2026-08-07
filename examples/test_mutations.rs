//! Exercises the inventory/spellbook mutation helpers (the ones the new
//! Inventory/Spells UI calls) against a real save's data, read-only —
//! verifies serialize/reparse stays internally consistent after each
//! mutation. Never writes to disk.

use std::path::PathBuf;

use bg2_editor::format::cre::{InvItem, KnownSpell, MemorizedSpell};
use bg2_editor::format::gam::GamFile;
use bg2_editor::format::primitives::ResRef;
use bg2_editor::save_file;

fn check(label: &str, gam: &GamFile) {
    let bytes = gam.serialize();
    match GamFile::parse(&bytes) {
        Ok(reparsed) if &reparsed == gam => println!("[ok]   {label}"),
        Ok(_) => {
            println!("[FAIL] {label}: reparse produced a different structure");
            std::process::exit(1);
        }
        Err(e) => {
            println!("[FAIL] {label}: reparse error: {e}");
            std::process::exit(1);
        }
    }
}

fn main() {
    let folder = std::env::args().nth(1).expect("usage: test_mutations <save-folder>");
    let mut gam = save_file::load(&PathBuf::from(folder)).expect("load");
    check("initial load", &gam);

    let cre = gam.party[0].cre.as_mut().expect("embedded cre");
    println!(
        "starting counts: items={} known_spells={} mem_info={} memorized={}",
        cre.items.len(), cre.known_spells.len(), cre.mem_info.len(), cre.memorized_spells.len()
    );

    cre.items.push(InvItem { item: ResRef::from_str("POTN08"), duration: 0, qty: [1, 0, 0], flags: 1 });
    check("add item", &gam);

    let cre = gam.party[0].cre.as_mut().unwrap();
    cre.remove_item(0);
    check("remove item 0 (shifts slots)", &gam);

    let cre = gam.party[0].cre.as_mut().unwrap();
    cre.known_spells.push(KnownSpell { spell: ResRef::from_str("SPWI104"), level: 1, kind: 1 });
    check("add known spell", &gam);

    let cre = gam.party[0].cre.as_mut().unwrap();
    cre.known_spells.remove(0);
    check("remove known spell 0", &gam);

    let cre = gam.party[0].cre.as_mut().unwrap();
    let existing_levels = cre.mem_info.len();
    cre.add_mem_level(9, 1, 1);
    check("add new mem level", &gam);

    let cre = gam.party[0].cre.as_mut().unwrap();
    cre.add_memorized_spell(existing_levels, MemorizedSpell { spell: ResRef::from_str("SPWI901"), flags: 0b010 });
    check("add memorized spell to new level", &gam);

    let cre = gam.party[0].cre.as_mut().unwrap();
    // Add a spell to an EARLY existing level too, to exercise the
    // table-index-shifting path against real (not synthetic) data.
    cre.add_memorized_spell(0, MemorizedSpell { spell: ResRef::from_str("SPWI112"), flags: 0b010 });
    check("add memorized spell to level 0 (shifts later levels)", &gam);

    let cre = gam.party[0].cre.as_mut().unwrap();
    cre.remove_memorized_spell(0, 0);
    check("remove memorized spell from level 0 (shifts later levels back)", &gam);

    let cre = gam.party[0].cre.as_mut().unwrap();
    cre.remove_mem_level(existing_levels); // remove the level we added
    check("remove mem level", &gam);

    println!("\nALL MUTATIONS PRODUCED INTERNALLY CONSISTENT DATA");
}
