//! Ad-hoc research tool: dumps the raw text of a .2DA resource from the
//! real game install, to inspect real column layout before writing a
//! parser. Read-only.

use std::path::PathBuf;

use bg2_editor::gamedata::key::{self, KeyFile};
use bg2_editor::gamedata::bif::BifArchive;

fn main() {
    let mut args = std::env::args().skip(1);
    let game_root = PathBuf::from(args.next().expect("usage: dump_2da <game-root> <resref>"));
    let resref = args.next().expect("usage: dump_2da <game-root> <resref>");

    let key_bytes = std::fs::read(game_root.join("chitin.key")).expect("read chitin.key");
    let keyf = KeyFile::parse(&key_bytes).expect("parse key");
    if resref == "--list" {
        let mut names: Vec<String> = keyf.names_of_type(key::TYPE_2DA).map(|e| e.name.as_str()).collect();
        names.sort();
        for n in names {
            println!("{n}");
        }
        return;
    }
    let entry = match keyf.find(&resref, key::TYPE_2DA) {
        Some(e) => e,
        None => {
            eprintln!("{resref}.2DA not found; 2DA resrefs containing {resref:?}:");
            for e in keyf.names_of_type(key::TYPE_2DA) {
                let name = e.name.as_str();
                if name.to_ascii_uppercase().contains(&resref.to_ascii_uppercase()) {
                    eprintln!("  {name}");
                }
            }
            std::process::exit(1);
        }
    };
    let (bif_idx, sub_idx) = key::decode_locator(entry.locator, key::TYPE_2DA);
    let biff = &keyf.biffs[bif_idx as usize];
    let path = game_root.join(&biff.path);
    let mut archive = BifArchive::open(&path).expect("open bif");
    let bytes = archive.read_resource(sub_idx as usize).expect("read resource");
    print!("{}", String::from_utf8_lossy(&bytes));
}
