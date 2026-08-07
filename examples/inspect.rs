//! Read-only diagnostic: loads a real save folder, prints key fields per
//! party member, then does an in-memory serialize+reparse round trip
//! (never writing back to disk) to check the parser against real game
//! data rather than only synthetic test fixtures.

use std::path::PathBuf;

use bg2_editor::save_file;

fn main() {
    let folder = std::env::args().nth(1).expect("usage: inspect <save-folder>");
    let folder = PathBuf::from(folder);

    let gam = match save_file::load(&folder) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("FAILED TO LOAD: {e}");
            std::process::exit(1);
        }
    };

    println!("Loaded OK.");
    println!("game_time={} party_gold={} reputation={}", gam.game_time, gam.party_gold, gam.reputation);
    println!("party members: {}", gam.party.len());
    println!("non-party members: {}", gam.non_party.len());
    println!("globals: {}  journal: {}", gam.globals.len(), gam.journal.len());
    println!("stored_locations: {}  pocket_plane: {}", gam.stored_locations.len(), gam.pocket_plane_locations.len());
    println!("familiar section present: {}", gam.familiar.is_some());

    for (i, m) in gam.party.iter().enumerate() {
        let end = m.name.iter().position(|&b| b == 0).unwrap_or(m.name.len());
        let name = String::from_utf8_lossy(&m.name[..end]);
        print!("[{i}] name={name:?} ");
        match &m.cre {
            Some(cre) => {
                println!(
                    "STR={} DEX={} CON={} INT={} WIS={} CHA={} HP={}/{} AC(nat/eff)={}/{} THAC0={} lvl={}/{}/{} class={} race={} align={} kit={:#06x} XP={} gold={} known_spells={} mem_info={} memorized={} items={}",
                    cre.str_score, cre.dex_score, cre.con_score, cre.int_score, cre.wis_score, cre.cha_score,
                    cre.hp_current, cre.hp_max, cre.ac_natural, cre.ac_effective, cre.thac0,
                    cre.level1, cre.level2, cre.level3, cre.class, cre.race, cre.alignment, cre.kit,
                    cre.xp, cre.gold, cre.known_spells.len(), cre.mem_info.len(), cre.memorized_spells.len(), cre.items.len()
                );
                println!(
                    "    profs raw bytes: large_sword={:#010b} small_sword={:#010b} bow={:#010b} spear={:#010b} blunt={:#010b} spiked={:#010b} axe={:#010b} missile={:#010b}",
                    cre.prof_large_sword.0, cre.prof_small_sword.0, cre.prof_bow.0, cre.prof_spear.0,
                    cre.prof_blunt.0, cre.prof_spiked.0, cre.prof_axe.0, cre.prof_missile.0
                );
                println!(
                    "    profs rank (low 3 bits): large_sword={} small_sword={} bow={} spear={} blunt={} spiked={} axe={} missile={}",
                    cre.prof_large_sword.rank(), cre.prof_small_sword.rank(), cre.prof_bow.rank(), cre.prof_spear.rank(),
                    cre.prof_blunt.rank(), cre.prof_spiked.rank(), cre.prof_axe.rank(), cre.prof_missile.rank()
                );
            }
            None => println!("(external CRE reference, not embedded)"),
        }
    }

    // In-memory round trip check against the real file — never written to disk.
    let bytes = gam.serialize();
    match bg2_editor::format::gam::GamFile::parse(&bytes) {
        Ok(reparsed) => {
            if reparsed == gam {
                println!("\nROUND TRIP: OK (serialize -> reparse produced an identical structure)");
            } else {
                println!("\nROUND TRIP: MISMATCH (serialize -> reparse differs from the original parse)");
            }
        }
        Err(e) => println!("\nROUND TRIP: reparse FAILED: {e}"),
    }

    let orig_bytes = std::fs::read(folder.join("BALDUR.gam"))
        .or_else(|_| std::fs::read(folder.join("baldur.gam")))
        .unwrap_or_default();
    println!("original file size = {}, our serialized size = {}", orig_bytes.len(), bytes.len());
    if orig_bytes == bytes {
        println!("BYTE-FOR-BYTE IDENTICAL to the original file.");
    } else if orig_bytes.len() == bytes.len() {
        let diff_count = orig_bytes.iter().zip(bytes.iter()).filter(|(a, b)| a != b).count();
        println!("Same length, but {diff_count} byte(s) differ. Contiguous diff runs (offset, len):");
        let mut i = 0usize;
        let mut runs = 0;
        while i < orig_bytes.len() {
            if orig_bytes[i] != bytes[i] {
                let start = i;
                while i < orig_bytes.len() && orig_bytes[i] != bytes[i] {
                    i += 1;
                }
                let len = i - start;
                let orig_slice = &orig_bytes[start..i];
                let new_slice = &bytes[start..i];
                println!("  offset {start:#06x} ({start}), len {len}: orig={orig_slice:02x?} new={new_slice:02x?}");
                runs += 1;
                if runs > 40 {
                    println!("  ...(truncated)");
                    break;
                }
            } else {
                i += 1;
            }
        }
    } else {
        println!("Lengths differ.");
    }
}
