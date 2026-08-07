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
