//! Read-only resolver for a BG1:EE/BG2:EE game installation: resolves
//! item/spell resrefs to display names (via `chitin.key` -> `.bif` ->
//! the resource's Name strref -> `dialog.tlk`), and `.IDS` symbol tables
//! for class/race/kit/alignment/etc. labels. Not needed for editing raw
//! attributes — only for showing human-readable names instead of resrefs
//! and numeric IDs.

pub mod bam;
pub mod bif;
pub mod ids;
pub mod itm;
pub mod key;
pub mod portrait;
pub mod spl;
pub mod table2da;
pub mod tlk;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::format::primitives::read_u32;
use bif::BifArchive;
use ids::IdsTable;
use itm::ItemStats;
use key::KeyFile;
use spl::SpellStats;
use table2da::Table2da;
use tlk::TlkFile;

/// A (resref, display name) list for a full resource-type catalog (every
/// item or every spell in the game), sorted by name.
pub type Catalog = Rc<Vec<(String, String)>>;

/// One entry in the full item catalog: resref, display name, and parsed
/// combat/requirement stats (for category filtering and stat columns in
/// the item picker).
pub struct ItemEntry {
    pub resref: String,
    pub name: String,
    pub stats: ItemStats,
}
pub type ItemCatalog = Rc<Vec<ItemEntry>>;

/// One entry in the full spell catalog: resref, display name, and parsed
/// level/type/school/ability stats (for the spell picker's detail pane).
pub struct SpellEntry {
    pub resref: String,
    pub name: String,
    pub stats: SpellStats,
}
pub type SpellCatalog = Rc<Vec<SpellEntry>>;

pub struct GameData {
    root: PathBuf,
    /// The per-user "portraits" folder alongside the `save` folder in
    /// Documents (e.g. `.../Baldur's Gate - Enhanced Edition/portraits`)
    /// — where player-customized character portraits actually live, as
    /// opposed to a `Portraits/` folder in the game install itself (which
    /// holds the stock companion/NPC portraits and, on many installs,
    /// doesn't exist as a loose folder at all). Set via
    /// `with_extra_portraits_dir` once the save folder is known.
    extra_portraits_dir: Option<PathBuf>,
    key: KeyFile,
    tlk: TlkFile,
    bif_archives: RefCell<HashMap<u32, BifArchive>>,
    ids_cache: RefCell<HashMap<String, Rc<IdsTable>>>,
    table2da_cache: RefCell<HashMap<String, Option<Rc<Table2da>>>>,
    name_cache: RefCell<HashMap<(String, u16), Option<String>>>,
    item_catalog: RefCell<Option<Catalog>>,
    spell_catalog: RefCell<Option<Catalog>>,
    item_catalog_full: RefCell<Option<ItemCatalog>>,
    spell_catalog_full: RefCell<Option<SpellCatalog>>,
}

impl GameData {
    /// Loads `chitin.key` and `dialog.tlk` from a game install root (the
    /// folder directly containing `chitin.key`, e.g.
    /// `.../Baldur's Gate II - Enhanced Edition`), preferring `en_US` for
    /// text. Everything else (`.bif` archives, `.IDS` tables) is loaded
    /// lazily on first use. Prefer `load_with_locale` when the active
    /// in-game language is known — otherwise names may resolve in the
    /// wrong language (e.g. English instead of the game's actual
    /// Chinese/etc. text), which is still the right *character*, just
    /// not what the player actually sees on screen.
    pub fn load(root: &Path) -> Result<GameData, String> {
        Self::load_with_locale(root, None)
    }

    /// Same as `load`, but tries `preferred_locale` (e.g. `"zh_CN"`)
    /// before falling back to `en_US` / whatever's available.
    pub fn load_with_locale(root: &Path, preferred_locale: Option<&str>) -> Result<GameData, String> {
        let key_bytes = fs::read(root.join("chitin.key")).map_err(|e| format!("read chitin.key: {e}"))?;
        let key = KeyFile::parse(&key_bytes)?;

        let tlk_path = find_tlk_path(root, preferred_locale)?;
        let tlk_bytes = fs::read(&tlk_path).map_err(|e| format!("read {}: {e}", tlk_path.display()))?;
        let tlk = TlkFile::parse(&tlk_bytes)?;

        Ok(GameData {
            root: root.to_path_buf(),
            extra_portraits_dir: None,
            key,
            tlk,
            bif_archives: RefCell::new(HashMap::new()),
            ids_cache: RefCell::new(HashMap::new()),
            table2da_cache: RefCell::new(HashMap::new()),
            name_cache: RefCell::new(HashMap::new()),
            item_catalog: RefCell::new(None),
            spell_catalog: RefCell::new(None),
            item_catalog_full: RefCell::new(None),
            spell_catalog_full: RefCell::new(None),
        })
    }

    /// Attaches the per-user Documents `portraits` folder (sibling of the
    /// `save` folder) so player-customized portraits — which live there,
    /// not in the game install — can be resolved. Builder-style since
    /// `GameData` is usually held as `Rc<GameData>` once loaded, so a
    /// plain `&mut self` setter wouldn't be usable after construction.
    pub fn with_extra_portraits_dir(mut self, dir: Option<PathBuf>) -> Self {
        self.extra_portraits_dir = dir;
        self
    }

    /// Raw bytes of a portrait BMP. Tries, in order: the per-user
    /// Documents `portraits` folder (player-customized portraits),
    /// a loose `Portraits/` folder in the game install (stock companion
    /// portraits, on installs that ship them loose rather than bif'd),
    /// then the ordinary KEY/BIF resource path.
    pub fn portrait_bytes(&self, resref: &str) -> Option<Vec<u8>> {
        let filename = format!("{}.BMP", resref.to_ascii_uppercase());
        if let Some(dir) = &self.extra_portraits_dir {
            if let Ok(bytes) = fs::read(dir.join(&filename)) {
                return Some(bytes);
            }
        }
        if let Ok(bytes) = fs::read(self.root.join("Portraits").join(&filename)) {
            return Some(bytes);
        }
        self.resource_bytes(resref, key::TYPE_BMP)
    }

    /// Raw bytes of a BAM icon (item/spell inventory icon resref).
    pub fn icon_bytes(&self, resref: &str) -> Option<Vec<u8>> {
        self.resource_bytes(resref, key::TYPE_BAM)
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

    /// Resolves a raw dialog.tlk string reference directly — e.g. a CRE's
    /// `name_strref` field (recruited companions typically have their
    /// display name here rather than in the GAM party record's literal
    /// name field, which is usually only populated for player-customized
    /// characters).
    pub fn tlk_string(&self, strref: i32) -> Option<String> {
        self.tlk.get(strref).filter(|s| !s.is_empty()).map(|s| s.to_string())
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

    /// Loads (and caches) a `.2DA` table by resref, e.g. `"CLASTEXT"`.
    /// `None` if the resource doesn't exist in this install (some tables,
    /// like the ones this editor uses, are EE-only).
    fn table2da(&self, resref: &str) -> Option<Rc<Table2da>> {
        let key = resref.to_ascii_uppercase();
        if let Some(cached) = self.table2da_cache.borrow().get(&key) {
            return cached.clone();
        }
        let table = self.resource_bytes(&key, key::TYPE_2DA).map(|bytes| Rc::new(Table2da::parse(&String::from_utf8_lossy(&bytes))));
        self.table2da_cache.borrow_mut().insert(key, table.clone());
        table
    }

    /// A handful of dialog.tlk strings are literal unsubstituted
    /// templates (e.g. `"<MAGESCHOOL>"`, `"<FIGHTERTYPE>"` — confirmed
    /// real entries in `clastext.2da`'s `MIXED`/`LOWER` columns for the
    /// Mage/Fighter base-class rows specifically, not a parsing bug),
    /// meant to be filled in by the engine rather than shown as-is.
    /// Treat those as "no real name available" so callers fall back
    /// cleanly instead of displaying the placeholder.
    fn filter_placeholder(s: Option<String>) -> Option<String> {
        s.filter(|s| !s.trim_start().starts_with('<'))
    }

    /// Real in-game class display name (e.g. "Thief"/"盗贼"), resolved
    /// via `clastext.2da` (columns: rowname, CLASSID, KITID, LOWER,
    /// DESCSTR, MIXED, ...) rather than the raw `CLASS.IDS` symbol name
    /// NearInfinity itself shows (e.g. "THIEF") — confirmed against real
    /// data that `clastext.2da`'s `MIXED` column is what the game
    /// actually displays, in whatever language `dialog.tlk` was loaded
    /// with. The base-class row for a given `CLASS.IDS` value has
    /// `KITID == 16384` (kitted variants of the same base class have
    /// their own rows with a real `KITID`, handled by `kit_name`
    /// instead). Returns `None` for Mage/Fighter's unkitted "true class"
    /// row specifically — see `filter_placeholder`.
    pub fn class_name(&self, class_id: u32) -> Option<String> {
        let table = self.table2da("CLASTEXT")?;
        let row = table.row_where(1, class_id as i64).filter(|r| r.get(2).and_then(|c| c.parse::<i64>().ok()) == Some(16384))?;
        let strref: i32 = row.get(5)?.parse().ok()?;
        Self::filter_placeholder(self.tlk_string(strref))
    }

    /// Real in-game race display name, resolved via `racetext.2da`
    /// (columns: rowname, ID, NAME, DESCSTR, UPPERCASE, BIOGRAPHY) —
    /// same rationale as `class_name`.
    pub fn race_name(&self, race_id: u32) -> Option<String> {
        let table = self.table2da("RACETEXT")?;
        let row = table.row_where(1, race_id as i64)?;
        let strref: i32 = row.get(4)?.parse().ok()?;
        Self::filter_placeholder(self.tlk_string(strref))
    }

    /// Real in-game kit display name, resolved via `clastext.2da`. Joined
    /// by *symbol* (e.g. `"BERSERKER"`) against `KIT.IDS`, not by
    /// `clastext.2da`'s own `KITID` column — confirmed those are two
    /// different numbering spaces: `KIT.IDS` stores each kit as a large
    /// bitmask-style constant (e.g. Berserker = `0x4001`), while
    /// `clastext.2da`'s `KITID` column is a small table-local sequence
    /// (Berserker = `1`) — so joining on the raw kit value against that
    /// column silently matches nothing (or the wrong row). `kit_id == 0`
    /// ("No Kit") has no in-game display string of its own — it's a
    /// synthetic value this editor uses for "no kit selected", not a
    /// real `KIT.IDS`/`clastext.2da` entry — so that case isn't looked
    /// up here at all; callers should special-case it themselves (as
    /// `ids_field_kit` in `app.rs` already did before this existed).
    pub fn kit_name(&self, kit_id: u32) -> Option<String> {
        if kit_id == 0 {
            return None;
        }
        let symbol = self.ids("KIT.IDS")?.name(kit_id)?.to_owned();
        let table = self.table2da("CLASTEXT")?;
        let row = table.row_where_name(&symbol)?;
        let strref: i32 = row.get(5)?.parse().ok()?;
        Self::filter_placeholder(self.tlk_string(strref))
    }

    /// Label for an ITM's weapon-proficiency byte. Prefers `PROFTYPE.IDS`,
    /// falling back to `STATS.IDS` (a much larger general stat-ID table
    /// that also happens to define the individual weapon-proficiency
    /// entries) if the former doesn't exist in this install — matching
    /// NearInfinity's own resolution order exactly.
    pub fn weapon_proficiency_label(&self, value: u8) -> String {
        if let Some(t) = self.ids("PROFTYPE.IDS") {
            if let Some(name) = t.name(value as u32) {
                return name.to_string();
            }
        }
        self.ids_label("STATS.IDS", value as u32)
    }

    /// Every item in the game as (resref, name), sorted by name. Built
    /// (and cached) on first call — resolves every ITM in the KEY index,
    /// so this can take a moment the first time it's needed.
    pub fn item_catalog(&self) -> Catalog {
        self.catalog(&self.item_catalog, key::TYPE_ITM, |gd, r| gd.item_name(r))
    }

    /// Every spell in the game as (resref, name), sorted by name.
    pub fn spell_catalog(&self) -> Catalog {
        self.catalog(&self.spell_catalog, key::TYPE_SPL, |gd, r| gd.spell_name(r))
    }

    /// Parses an item's category/proficiency/requirement/combat stats.
    /// `None` if the resref can't be resolved or its ITM can't be parsed.
    pub fn item_stats(&self, resref: &str) -> Option<ItemStats> {
        let bytes = self.resource_bytes(resref, key::TYPE_ITM)?;
        ItemStats::parse(&bytes).ok()
    }

    /// Every item in the game as (resref, name, stats), sorted by name —
    /// the richer counterpart to `item_catalog()` used by the item
    /// picker's category filter and stat columns. Cached like the other
    /// catalogs; building it parses every ITM's full stats, not just its
    /// name, but that's still just extra field reads on already-fetched
    /// bytes (no extra I/O), so it stays fast in practice.
    pub fn item_catalog_full(&self) -> ItemCatalog {
        if let Some(c) = self.item_catalog_full.borrow().as_ref() {
            return c.clone();
        }
        let mut list: Vec<ItemEntry> = self
            .key
            .names_of_type(key::TYPE_ITM)
            .filter_map(|entry| {
                let resref = entry.name.as_str();
                let name = self.item_name(&resref)?;
                if name.trim().is_empty() || name.trim().eq_ignore_ascii_case("<NO TEXT>") {
                    return None;
                }
                let stats = self.item_stats(&resref)?;
                Some(ItemEntry { resref, name, stats })
            })
            .collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        let rc: ItemCatalog = Rc::new(list);
        *self.item_catalog_full.borrow_mut() = Some(rc.clone());
        rc
    }

    /// Parses a spell's level/school/type/ability stats. `None` if the
    /// resref can't be resolved or its SPL can't be parsed.
    pub fn spell_stats(&self, resref: &str) -> Option<SpellStats> {
        let bytes = self.resource_bytes(resref, key::TYPE_SPL)?;
        SpellStats::parse(&bytes).ok()
    }

    /// Every spell in the game as (resref, name, stats), sorted by name —
    /// the richer counterpart to `spell_catalog()` used by the spell
    /// picker's detail pane. Cached like the other catalogs.
    pub fn spell_catalog_full(&self) -> SpellCatalog {
        if let Some(c) = self.spell_catalog_full.borrow().as_ref() {
            return c.clone();
        }
        let mut list: Vec<SpellEntry> = self
            .key
            .names_of_type(key::TYPE_SPL)
            .filter_map(|entry| {
                let resref = entry.name.as_str();
                let name = self.spell_name(&resref)?;
                if name.trim().is_empty() || name.trim().eq_ignore_ascii_case("<NO TEXT>") {
                    return None;
                }
                let stats = self.spell_stats(&resref)?;
                Some(SpellEntry { resref, name, stats })
            })
            .collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        let rc: SpellCatalog = Rc::new(list);
        *self.spell_catalog_full.borrow_mut() = Some(rc.clone());
        rc
    }

    fn catalog(
        &self,
        cache: &RefCell<Option<Catalog>>,
        restype: u16,
        resolve: impl Fn(&GameData, &str) -> Option<String>,
    ) -> Catalog {
        if let Some(c) = cache.borrow().as_ref() {
            return c.clone();
        }
        let mut list: Vec<(String, String)> = self
            .key
            .names_of_type(restype)
            .filter_map(|entry| {
                let resref = entry.name.as_str();
                let name = resolve(self, &resref)?;
                // Some internal/unused stub records resolve to literal
                // placeholder text rather than a real name — not
                // something a player should ever pick from a catalog.
                if name.trim().is_empty() || name.trim().eq_ignore_ascii_case("<NO TEXT>") {
                    return None;
                }
                Some((resref, name))
            })
            .collect();
        list.sort_by(|a, b| a.1.cmp(&b.1));
        let rc: Catalog = Rc::new(list);
        *cache.borrow_mut() = Some(rc.clone());
        rc
    }
}

fn find_tlk_path(root: &Path, preferred_locale: Option<&str>) -> Result<PathBuf, String> {
    let lang_dir = root.join("lang");
    if let Some(locale) = preferred_locale {
        let p = lang_dir.join(locale).join("dialog.tlk");
        if p.is_file() {
            return Ok(p);
        }
    }
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
