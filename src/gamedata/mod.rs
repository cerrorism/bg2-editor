//! Read-only resolver for a BG1:EE/BG2:EE game installation: resolves
//! item/spell resrefs to display names (via `chitin.key` -> `.bif` ->
//! the resource's Name strref -> `dialog.tlk`), and `.IDS` symbol tables
//! for class/race/kit/alignment/etc. labels. Not needed for editing raw
//! attributes — only for showing human-readable names instead of resrefs
//! and numeric IDs.

pub mod bif;
pub mod ids;
pub mod key;
pub mod tlk;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::format::primitives::read_u32;
use bif::BifArchive;
use ids::IdsTable;
use key::KeyFile;
use tlk::TlkFile;

pub struct GameData {
    root: PathBuf,
    key: KeyFile,
    tlk: TlkFile,
    bif_archives: RefCell<HashMap<u32, BifArchive>>,
    ids_cache: RefCell<HashMap<String, std::rc::Rc<IdsTable>>>,
    name_cache: RefCell<HashMap<(String, u16), Option<String>>>,
}

impl GameData {
    /// Loads `chitin.key` and `dialog.tlk` from a game install root (the
    /// folder directly containing `chitin.key`, e.g.
    /// `.../Baldur's Gate II - Enhanced Edition`). Everything else
    /// (`.bif` archives, `.IDS` tables) is loaded lazily on first use.
    pub fn load(root: &Path) -> Result<GameData, String> {
        let key_bytes = fs::read(root.join("chitin.key")).map_err(|e| format!("read chitin.key: {e}"))?;
        let key = KeyFile::parse(&key_bytes)?;

        let tlk_path = find_tlk_path(root)?;
        let tlk_bytes = fs::read(&tlk_path).map_err(|e| format!("read {}: {e}", tlk_path.display()))?;
        let tlk = TlkFile::parse(&tlk_bytes)?;

        Ok(GameData {
            root: root.to_path_buf(),
            key,
            tlk,
            bif_archives: RefCell::new(HashMap::new()),
            ids_cache: RefCell::new(HashMap::new()),
            name_cache: RefCell::new(HashMap::new()),
        })
    }

    fn resource_bytes(&self, name: &str, restype: u16) -> Option<Vec<u8>> {
        let entry = self.key.find(name, restype)?;
        let (bif_idx, sub_idx) = key::decode_locator(entry.locator, restype);

        let mut archives = self.bif_archives.borrow_mut();
        if !archives.contains_key(&bif_idx) {
            let biff = self.key.biffs.get(bif_idx as usize)?;
            let path = self.root.join(&biff.path);
            let archive = BifArchive::open(&path).ok()?;
            archives.insert(bif_idx, archive);
        }
        let archive = archives.get_mut(&bif_idx)?;
        archive.read_resource(sub_idx as usize).ok()
    }

    /// Display name for an item resref: identified name if present, else
    /// the general/unidentified name. `None` if the resref can't be
    /// resolved (unknown item, or name lookup unavailable).
    pub fn item_name(&self, resref: &str) -> Option<String> {
        self.cached_name(resref, key::TYPE_ITM, 12, 8)
    }

    /// Display name for a spell resref.
    pub fn spell_name(&self, resref: &str) -> Option<String> {
        self.cached_name(resref, key::TYPE_SPL, 8, 8)
    }

    fn cached_name(&self, resref: &str, restype: u16, primary_off: usize, fallback_off: usize) -> Option<String> {
        let key = (resref.to_ascii_uppercase(), restype);
        if let Some(cached) = self.name_cache.borrow().get(&key) {
            return cached.clone();
        }
        let resolved = self.resolve_name(resref, restype, primary_off, fallback_off);
        self.name_cache.borrow_mut().insert(key, resolved.clone());
        resolved
    }

    fn resolve_name(&self, resref: &str, restype: u16, primary_off: usize, fallback_off: usize) -> Option<String> {
        let bytes = self.resource_bytes(resref, restype)?;
        if bytes.len() < fallback_off + 4 {
            return None;
        }
        let primary = read_u32(&bytes, primary_off) as i32;
        let fallback = read_u32(&bytes, fallback_off) as i32;
        self.tlk
            .get(primary)
            .filter(|s| !s.is_empty())
            .or_else(|| self.tlk.get(fallback))
            .map(|s| s.to_string())
    }

    /// Loads (and caches) an `.IDS` table by name, e.g. `"CLASS.IDS"` or
    /// just `"CLASS"` — resrefs never include the extension (the `.IDS`
    /// restype is a separate field), so a trailing `.IDS` is stripped
    /// before the resref lookup.
    pub fn ids(&self, name: &str) -> Option<std::rc::Rc<IdsTable>> {
        let key = name.to_ascii_uppercase();
        if let Some(cached) = self.ids_cache.borrow().get(&key) {
            return Some(cached.clone());
        }
        let resref = key.strip_suffix(".IDS").unwrap_or(&key);
        let bytes = self.resource_bytes(resref, key::TYPE_IDS)?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let table = std::rc::Rc::new(IdsTable::parse(&text));
        self.ids_cache.borrow_mut().insert(key, table.clone());
        Some(table)
    }

    /// Looks up a symbol name for a numeric ID in the given `.IDS` table,
    /// falling back to the raw number if the table or value isn't found.
    pub fn ids_label(&self, ids_name: &str, value: u32) -> String {
        self.ids(ids_name).and_then(|t| t.name(value).map(|s| s.to_string())).unwrap_or_else(|| value.to_string())
    }
}

fn find_tlk_path(root: &Path) -> Result<PathBuf, String> {
    let lang_dir = root.join("lang");
    let preferred = ["en_US", "en_us", "en_GB"];
    for lang in preferred {
        let p = lang_dir.join(lang).join("dialog.tlk");
        if p.is_file() {
            return Ok(p);
        }
    }
    // Fall back to whatever locale is actually present.
    if let Ok(entries) = fs::read_dir(&lang_dir) {
        for entry in entries.flatten() {
            let p = entry.path().join("dialog.tlk");
            if p.is_file() {
                return Ok(p);
            }
        }
    }
    // Classic (non-EE) layout: dialog.tlk sits directly in the game root.
    let classic = root.join("dialog.tlk");
    if classic.is_file() {
        return Ok(classic);
    }
    Err(format!("could not find dialog.tlk under {}", lang_dir.display()))
}
