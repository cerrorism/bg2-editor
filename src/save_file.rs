use std::fs;
use std::path::{Path, PathBuf};

use crate::format::gam::GamFile;

/// Lists save-slot folders under `root` (each containing a `baldur.gam`),
/// newest first by the GAM file's modified time.
pub fn list_save_folders(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root) else {
        return vec![];
    };
    let mut folders: Vec<(PathBuf, std::time::SystemTime)> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter_map(|p| {
            let gam = p.join("baldur.gam");
            let modified = fs::metadata(&gam).ok()?.modified().ok()?;
            Some((p, modified))
        })
        .collect();
    folders.sort_by(|a, b| b.1.cmp(&a.1));
    folders.into_iter().map(|(p, _)| p).collect()
}

/// The default save-folder locations for BG1:EE and BG2:EE, in the order
/// they should be tried. Checks both a plain `Documents` folder and
/// OneDrive-redirected `Documents` (a common Windows setup where
/// `USERPROFILE\Documents` isn't where Documents actually lives).
pub fn default_save_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for docs in dirs_documents_candidates() {
        roots.push(docs.join("Baldur's Gate II - Enhanced Edition").join("save"));
        roots.push(docs.join("Baldur's Gate - Enhanced Edition").join("save"));
    }
    roots
}

fn dirs_documents_candidates() -> Vec<PathBuf> {
    let Some(profile) = std::env::var_os("USERPROFILE").map(PathBuf::from) else {
        return Vec::new();
    };
    let mut candidates = vec![profile.join("Documents")];
    // OneDrive can redirect Documents; "OneDrive" is the default folder
    // name, but a custom account name (e.g. "OneDrive - Company") is also
    // common, so check every OneDrive* folder directly under the profile.
    if let Ok(entries) = fs::read_dir(&profile) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("OneDrive") {
                candidates.push(entry.path().join("Documents"));
            }
        }
    }
    candidates
}

/// Detects the game's currently-active text language (e.g. `"zh_CN"`)
/// from `Baldur.lua` in the Documents-folder game-data directory (the
/// parent of `save_folder`'s root, i.e.
/// `.../Baldur's Gate - Enhanced Edition/Baldur.lua`, containing a line
/// like `SetPrivateProfileString('Language','Text','zh_CN')`). Used to
/// pick the matching `dialog.tlk` locale so resolved names match what
/// the player actually sees in-game, rather than always English.
pub fn detect_active_language(save_root: &Path) -> Option<String> {
    let docs_game_dir = save_root.parent()?;
    let lua_path = find_case_insensitive(docs_game_dir, "Baldur.lua")?;
    let text = fs::read_to_string(lua_path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("SetPrivateProfileString('Language','Text','") {
            if let Some(end) = rest.find('\'') {
                return Some(rest[..end].to_owned());
            }
        }
    }
    None
}

/// The per-user Documents `portraits` folder (sibling of `save`), where
/// player-customized character portraits live — as opposed to the game
/// install's own `Portraits/` folder, which holds only the stock
/// companion/NPC portraits (and on many installs doesn't exist loose at
/// all, since those are bif'd).
pub fn portraits_dir(save_root: &Path) -> Option<PathBuf> {
    let docs_game_dir = save_root.parent()?;
    let dir = docs_game_dir.join("portraits");
    dir.is_dir().then_some(dir)
}

fn find_case_insensitive(dir: &Path, name: &str) -> Option<PathBuf> {
    let direct = dir.join(name);
    if direct.is_file() {
        return Some(direct);
    }
    let entries = fs::read_dir(dir).ok()?;
    entries.flatten().map(|e| e.path()).find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.eq_ignore_ascii_case(name)))
}

pub fn load(save_folder: &Path) -> Result<GamFile, String> {
    let gam_path = save_folder.join("baldur.gam");
    let buf = fs::read(&gam_path).map_err(|e| format!("failed to read {}: {e}", gam_path.display()))?;
    GamFile::parse(&buf)
}

/// Writes the (possibly edited) GAM file back to its save folder, first
/// copying the existing `baldur.gam` to `baldur.gam.bak` (not overwriting
/// an existing backup, to avoid clobbering the pre-session original).
pub fn save_with_backup(gam: &GamFile, save_folder: &Path) -> Result<(), String> {
    let gam_path = save_folder.join("baldur.gam");
    let backup_path = make_backup_path(save_folder);
    fs::copy(&gam_path, &backup_path).map_err(|e| format!("backup failed: {e}"))?;

    let bytes = gam.serialize();
    fs::write(&gam_path, &bytes).map_err(|e| format!("write failed: {e}"))?;
    Ok(())
}

fn make_backup_path(save_folder: &Path) -> PathBuf {
    let candidate = save_folder.join("baldur.gam.bak");
    if !candidate.exists() {
        return candidate;
    }
    let mut n = 2u32;
    loop {
        let c = save_folder.join(format!("baldur.gam.bak{n}"));
        if !c.exists() {
            return c;
        }
        n += 1;
    }
}
