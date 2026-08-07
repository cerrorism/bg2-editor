//! Tiny persisted config (just the last-used game install folder) so the
//! user doesn't have to re-pick it every launch. Stored as a one-line
//! text file next to the executable.

use std::path::{Path, PathBuf};

fn config_path() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(|d| d.join("bg2-editor.cfg"))
}

pub fn load_game_root() -> Option<PathBuf> {
    let path = config_path()?;
    let text = std::fs::read_to_string(path).ok()?;
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    let p = PathBuf::from(line);
    p.is_dir().then_some(p)
}

pub fn save_game_root(root: &Path) {
    if let Some(path) = config_path() {
        let _ = std::fs::write(path, root.to_string_lossy().as_bytes());
    }
}
