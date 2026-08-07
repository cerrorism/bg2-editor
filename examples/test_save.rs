//! Exercises the actual write path (`save_file::save_with_backup`) against
//! a scratch copy of a save folder: load, edit a stat, save, verify the
//! backup was created and matches the pre-edit original, and that
//! reloading picks up the edit.

use std::path::PathBuf;

use bg2_editor::save_file;

fn main() {
    let folder = std::env::args().nth(1).expect("usage: test_save <scratch-save-folder>");
    let folder = PathBuf::from(folder);

    let gam_path = folder.join("BALDUR.gam");
    let pre_edit_bytes = std::fs::read(&gam_path).expect("read original");

    let mut gam = save_file::load(&folder).expect("load");
    let original_str = gam.party[0].cre.as_ref().unwrap().str_score;
    let new_str = if original_str >= 25 { original_str - 1 } else { original_str + 1 };
    gam.party[0].cre.as_mut().unwrap().str_score = new_str;
    println!("editing STR: {original_str} -> {new_str}");

    save_file::save_with_backup(&gam, &folder).expect("save");

    let backup_path = folder.join("baldur.gam.bak");
    assert!(backup_path.exists(), "backup file should exist");
    let backup_bytes = std::fs::read(&backup_path).expect("read backup");
    assert_eq!(backup_bytes, pre_edit_bytes, "backup should match pre-edit original");
    println!("backup OK: baldur.gam.bak matches pre-edit original ({} bytes)", backup_bytes.len());

    let reloaded = save_file::load(&folder).expect("reload");
    let reloaded_str = reloaded.party[0].cre.as_ref().unwrap().str_score;
    assert_eq!(reloaded_str, new_str, "reloaded STR should reflect the edit");
    println!("reload OK: STR is now {reloaded_str}");

    println!("\nALL CHECKS PASSED");
}
