//! Full dump of the first party member's parsed CRE fields, to precisely
//! map raw file-offset diffs to named fields when investigating a
//! reported issue. Read-only.

use std::path::PathBuf;

use bg2_editor::save_file;

fn main() {
    let folder = std::env::args().nth(1).expect("usage: dump_cre <save-folder>");
    let gam = save_file::load(&PathBuf::from(folder)).expect("load");
    let cre = gam.party[0].cre.as_ref().expect("embedded cre");

    println!("reputation={} hide_in_shadows={}", cre.reputation, cre.hide_in_shadows);
    println!("thac0={} attacks_per_round={}", cre.thac0, cre.attacks_per_round);
    println!(
        "saves: death={} wand={} poly={} breath={} spell={}",
        cre.save_death, cre.save_wand, cre.save_polymorph, cre.save_breath, cre.save_spell
    );
    println!(
        "profs: large_sword={:?} small_sword={:?} bow={:?} spear={:?} blunt={:?} spiked={:?} axe={:?} missile={:?}",
        cre.prof_large_sword, cre.prof_small_sword, cre.prof_bow, cre.prof_spear,
        cre.prof_blunt, cre.prof_spiked, cre.prof_axe, cre.prof_missile
    );
    println!(
        "nightmare={} translucency={} rep_mod_killed={} rep_mod_join={} rep_mod_leave={} turn_undead={}",
        cre.nightmare_mode, cre.translucency, cre.reputation_mod_killed, cre.reputation_mod_join,
        cre.reputation_mod_leave, cre.turn_undead_level
    );
    println!("\nknown_spells ({}):", cre.known_spells.len());
    for (i, k) in cre.known_spells.iter().enumerate() {
        println!("  [{i}] spell={:?} level={} kind={}", k.spell.as_str(), k.level, k.kind);
    }
    println!("\nmem_info ({}):", cre.mem_info.len());
    for (i, m) in cre.mem_info.iter().enumerate() {
        println!(
            "  [{i}] level={} total={} current={} kind={} table_index={} count={}",
            m.level, m.memorizable_total, m.memorizable_current, m.kind, m.table_index, m.count
        );
    }
    println!("\nmemorized_spells ({}):", cre.memorized_spells.len());
    for (i, m) in cre.memorized_spells.iter().enumerate() {
        println!("  [{i}] spell={:?} flags={:#05b}", m.spell.as_str(), m.flags);
    }
    println!("\nitems ({}):", cre.items.len());
    for (i, it) in cre.items.iter().enumerate() {
        println!("  [{i}] item={:?} duration={} qty={:?} flags={:#06b}", it.item.as_str(), it.duration, it.qty, it.flags);
    }
    println!("\nitem_slots: {:?}", cre.item_slots);
    println!("selected_weapon_slot={} selected_weapon_ability={}", cre.selected_weapon_slot, cre.selected_weapon_ability);
}
