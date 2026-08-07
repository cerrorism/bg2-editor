use std::path::PathBuf;
use std::rc::Rc;

use egui::{DragValue, ScrollArea, Ui};

use crate::config;
use crate::format::cre::{CreV1, InvItem, KnownSpell, MemorizedSpell, ITEM_SLOT_NAMES};
use crate::format::gam::GamFile;
use crate::format::primitives::ResRef;
use crate::gamedata::GameData;
use crate::save_file;

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Abilities,
    ClassSkills,
    Inventory,
    Spells,
}

pub struct Bg2EditorApp {
    save_root: Option<PathBuf>,
    save_folders: Vec<PathBuf>,
    selected_folder_idx: Option<usize>,
    gam: Option<GamFile>,
    selected_char_idx: usize,
    tab: Tab,
    is_dirty: bool,
    status_msg: String,
    game_root: Option<PathBuf>,
    game_data: Option<Rc<GameData>>,
}

impl Default for Bg2EditorApp {
    fn default() -> Self {
        let save_root = save_file::default_save_roots().into_iter().find(|p| p.is_dir());
        let save_folders = save_root
            .as_deref()
            .map(save_file::list_save_folders)
            .unwrap_or_default();

        let mut app = Self {
            save_root,
            save_folders,
            selected_folder_idx: None,
            gam: None,
            selected_char_idx: 0,
            tab: Tab::Abilities,
            is_dirty: false,
            status_msg: String::new(),
            game_root: None,
            game_data: None,
        };
        if let Some(root) = config::load_game_root() {
            if let Ok(data) = GameData::load(&root) {
                app.game_data = Some(Rc::new(data));
                app.game_root = Some(root);
            }
        }
        app
    }
}

impl eframe::App for Bg2EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_panel(ctx);
        self.side_panel(ctx);
        self.central_panel(ctx);
    }
}

impl Bg2EditorApp {
    fn top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📂 Browse Saves Folder…").clicked() {
                    let start = self.save_root.clone();
                    let mut dialog = rfd::FileDialog::new();
                    if let Some(dir) = &start {
                        dialog = dialog.set_directory(dir);
                    }
                    if let Some(folder) = dialog.pick_folder() {
                        self.save_folders = save_file::list_save_folders(&folder);
                        self.save_root = Some(folder);
                        self.selected_folder_idx = None;
                        self.gam = None;
                        self.status_msg.clear();
                    }
                }

                let root_str = self
                    .save_root
                    .as_deref()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                ui.add(
                    egui::Label::new(egui::RichText::new(if root_str.is_empty() { "(no folder selected)" } else { &root_str }).weak())
                        .truncate(),
                );

                ui.separator();

                if ui.button("🎮 Set Game Folder…").on_hover_text(
                    "Point at your BG1:EE/BG2:EE install (the folder containing chitin.key) to show item/spell/class/race/kit names instead of raw codes."
                ).clicked() {
                    let mut dialog = rfd::FileDialog::new();
                    if let Some(dir) = &self.game_root {
                        dialog = dialog.set_directory(dir);
                    }
                    if let Some(folder) = dialog.pick_folder() {
                        match GameData::load(&folder) {
                            Ok(data) => {
                                config::save_game_root(&folder);
                                self.game_root = Some(folder);
                                self.game_data = Some(Rc::new(data));
                                self.status_msg = "Game data loaded.".to_owned();
                            }
                            Err(e) => self.status_msg = format!("Failed to load game folder: {e}"),
                        }
                    }
                }
                let game_str = self
                    .game_root
                    .as_deref()
                    .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
                    .unwrap_or_else(|| "(none — names shown as raw codes)".to_owned());
                ui.add(egui::Label::new(egui::RichText::new(game_str).weak()).truncate());

                ui.separator();

                ui.label("Party Gold:");
                ui.add_enabled_ui(self.gam.is_some(), |ui| {
                    if let Some(gam) = &mut self.gam {
                        let old = gam.party_gold;
                        let resp = ui.add(DragValue::new(&mut gam.party_gold).range(0u32..=4_294_967_295u32));
                        if resp.changed() && gam.party_gold != old {
                            self.is_dirty = true;
                        }
                    } else {
                        let mut dummy = 0u32;
                        ui.add(DragValue::new(&mut dummy));
                    }
                });

                ui.separator();

                ui.add_enabled_ui(self.is_dirty, |ui| {
                    if ui.button("💾 Save").clicked() {
                        if let (Some(gam), Some(idx)) = (&self.gam, self.selected_folder_idx) {
                            let folder = &self.save_folders[idx];
                            match save_file::save_with_backup(gam, folder) {
                                Ok(()) => {
                                    self.status_msg = "Saved. Backup written as baldur.gam.bak.".to_owned();
                                    self.is_dirty = false;
                                }
                                Err(e) => self.status_msg = format!("Save failed: {e}"),
                            }
                        }
                    }
                });

                if !self.status_msg.is_empty() {
                    ui.separator();
                    ui.label(egui::RichText::new(&self.status_msg).small());
                }
            });
        });
    }

    fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("save_list").min_width(200.0).max_width(320.0).show(ctx, |ui| {
            ui.heading("Saves");
            ui.separator();
            ScrollArea::vertical().show(ui, |ui| {
                for i in 0..self.save_folders.len() {
                    let name = self.save_folders[i]
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();
                    let selected = self.selected_folder_idx == Some(i);
                    if ui.selectable_label(selected, &name).clicked() && !selected {
                        match save_file::load(&self.save_folders[i]) {
                            Ok(gam) => {
                                self.gam = Some(gam);
                                self.selected_folder_idx = Some(i);
                                self.selected_char_idx = 0;
                                self.is_dirty = false;
                                self.status_msg.clear();
                            }
                            Err(e) => {
                                self.gam = None;
                                self.status_msg = format!("Failed to load save: {e}");
                            }
                        }
                    }
                }
            });
        });
    }

    fn central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(gam) = &mut self.gam else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a save on the left to begin editing.");
                });
                return;
            };

            if gam.party.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("This save has no party members.");
                });
                return;
            }
            if self.selected_char_idx >= gam.party.len() {
                self.selected_char_idx = 0;
            }

            ui.horizontal(|ui| {
                for (i, member) in gam.party.iter().enumerate() {
                    let name = member_display_name(member);
                    ui.selectable_value(&mut self.selected_char_idx, i, name);
                }
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Abilities, "Abilities & Combat");
                ui.selectable_value(&mut self.tab, Tab::ClassSkills, "Class & Skills");
                ui.selectable_value(&mut self.tab, Tab::Inventory, "Inventory");
                ui.selectable_value(&mut self.tab, Tab::Spells, "Spells");
            });
            ui.separator();

            let Some(cre) = gam.party[self.selected_char_idx].cre.as_mut() else {
                ui.label("This party member has no embedded character data (external reference).");
                return;
            };

            let dirty = &mut self.is_dirty;
            let gd = self.game_data.as_deref();
            ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Abilities => tab_abilities(ui, cre, dirty),
                Tab::ClassSkills => tab_class_skills(ui, cre, dirty, gd),
                Tab::Inventory => tab_inventory(ui, cre, dirty, gd),
                Tab::Spells => tab_spells(ui, cre, dirty, gd),
            });
        });
    }
}

fn member_display_name(member: &crate::format::gam::PartyMember) -> String {
    let end = member.name.iter().position(|&b| b == 0).unwrap_or(member.name.len());
    let s = String::from_utf8_lossy(&member.name[..end]).into_owned();
    if s.is_empty() {
        "(unnamed)".to_owned()
    } else {
        s
    }
}

fn drag_u8(ui: &mut Ui, label: &str, value: &mut u8, range: std::ops::RangeInclusive<u8>, dirty: &mut bool) {
    ui.label(label);
    let old = *value;
    let resp = ui.add(DragValue::new(value).range(range));
    if resp.changed() && *value != old {
        *dirty = true;
    }
    ui.end_row();
}

fn drag_i16(ui: &mut Ui, label: &str, value: &mut i16, dirty: &mut bool) {
    ui.label(label);
    let old = *value;
    let resp = ui.add(DragValue::new(value));
    if resp.changed() && *value != old {
        *dirty = true;
    }
    ui.end_row();
}

fn drag_u16(ui: &mut Ui, label: &str, value: &mut u16, dirty: &mut bool) {
    ui.label(label);
    let old = *value;
    let resp = ui.add(DragValue::new(value));
    if resp.changed() && *value != old {
        *dirty = true;
    }
    ui.end_row();
}

fn drag_u32(ui: &mut Ui, label: &str, value: &mut u32, dirty: &mut bool) {
    ui.label(label);
    let old = *value;
    let resp = ui.add(DragValue::new(value));
    if resp.changed() && *value != old {
        *dirty = true;
    }
    ui.end_row();
}

fn tab_abilities(ui: &mut Ui, cre: &mut CreV1, dirty: &mut bool) {
    ui.columns(2, |cols| {
        egui::Grid::new("abilities_grid").num_columns(2).spacing([16.0, 6.0]).striped(true).show(&mut cols[0], |ui| {
            ui.heading("Ability Scores");
            ui.end_row();
            drag_u8(ui, "Strength", &mut cre.str_score, 1..=25, dirty);
            drag_u8(ui, "Strength % (18/xx)", &mut cre.str_bonus, 0..=100, dirty);
            drag_u8(ui, "Dexterity", &mut cre.dex_score, 1..=25, dirty);
            drag_u8(ui, "Constitution", &mut cre.con_score, 1..=25, dirty);
            drag_u8(ui, "Intelligence", &mut cre.int_score, 1..=25, dirty);
            drag_u8(ui, "Wisdom", &mut cre.wis_score, 1..=25, dirty);
            drag_u8(ui, "Charisma", &mut cre.cha_score, 1..=25, dirty);

            ui.heading("Hit Points");
            ui.end_row();
            drag_u16(ui, "Current HP", &mut cre.hp_current, dirty);
            drag_u16(ui, "Max HP", &mut cre.hp_max, dirty);

            ui.heading("Experience & Gold");
            ui.end_row();
            drag_u32(ui, "Experience Points", &mut cre.xp, dirty);
            drag_u32(ui, "Gold (carried)", &mut cre.gold, dirty);
        });

        egui::Grid::new("combat_grid").num_columns(2).spacing([16.0, 6.0]).striped(true).show(&mut cols[1], |ui| {
            ui.heading("Armor Class & THAC0");
            ui.end_row();
            drag_i16(ui, "AC (natural)", &mut cre.ac_natural, dirty);
            drag_i16(ui, "AC (effective)", &mut cre.ac_effective, dirty);
            drag_i16(ui, "AC vs Crushing", &mut cre.ac_mod_crushing, dirty);
            drag_i16(ui, "AC vs Missile", &mut cre.ac_mod_missile, dirty);
            drag_i16(ui, "AC vs Piercing", &mut cre.ac_mod_piercing, dirty);
            drag_i16(ui, "AC vs Slashing", &mut cre.ac_mod_slashing, dirty);
            drag_u8(ui, "THAC0", &mut cre.thac0, 0..=25, dirty);

            ui.heading("Saving Throws");
            ui.end_row();
            drag_u8(ui, "vs Death", &mut cre.save_death, 0..=20, dirty);
            drag_u8(ui, "vs Wands", &mut cre.save_wand, 0..=20, dirty);
            drag_u8(ui, "vs Polymorph", &mut cre.save_polymorph, 0..=20, dirty);
            drag_u8(ui, "vs Breath", &mut cre.save_breath, 0..=20, dirty);
            drag_u8(ui, "vs Spell", &mut cre.save_spell, 0..=20, dirty);

            ui.heading("Resistances (%)");
            ui.end_row();
            drag_u8(ui, "Fire", &mut cre.resist_fire, 0..=100, dirty);
            drag_u8(ui, "Cold", &mut cre.resist_cold, 0..=100, dirty);
            drag_u8(ui, "Electricity", &mut cre.resist_electricity, 0..=100, dirty);
            drag_u8(ui, "Acid", &mut cre.resist_acid, 0..=100, dirty);
            drag_u8(ui, "Magic", &mut cre.resist_magic, 0..=100, dirty);
            drag_u8(ui, "Magic Fire", &mut cre.resist_magic_fire, 0..=100, dirty);
            drag_u8(ui, "Magic Cold", &mut cre.resist_magic_cold, 0..=100, dirty);
            drag_u8(ui, "Slashing", &mut cre.resist_slashing, 0..=100, dirty);
            drag_u8(ui, "Crushing", &mut cre.resist_crushing, 0..=100, dirty);
            drag_u8(ui, "Piercing", &mut cre.resist_piercing, 0..=100, dirty);
            drag_u8(ui, "Missile", &mut cre.resist_missile, 0..=100, dirty);
        });
    });
}

fn tab_class_skills(ui: &mut Ui, cre: &mut CreV1, dirty: &mut bool, gd: Option<&GameData>) {
    ui.columns(2, |cols| {
        egui::Grid::new("class_grid").num_columns(2).spacing([16.0, 6.0]).striped(true).show(&mut cols[0], |ui| {
            ui.heading("Class & Levels");
            ui.end_row();
            drag_u8(ui, "Level (class 1)", &mut cre.level1, 1..=99, dirty);
            drag_u8(ui, "Level (class 2)", &mut cre.level2, 0..=99, dirty);
            drag_u8(ui, "Level (class 3)", &mut cre.level3, 0..=99, dirty);

            ui.heading("Class & Race");
            ui.end_row();
            ids_field_u8(ui, "Class", &mut cre.class, "CLASS.IDS", gd, dirty);
            ids_field_u8(ui, "Race", &mut cre.race, "RACE.IDS", gd, dirty);
            ids_field_u8(ui, "Gender", &mut cre.gender, "GENDER.IDS", gd, dirty);
            ids_field_u8(ui, "Alignment", &mut cre.alignment, "ALIGNMEN.IDS", gd, dirty);
            ids_field_u8(ui, "Allegiance", &mut cre.allegiance, "EA.IDS", gd, dirty);
            ids_field_kit(ui, "Kit", &mut cre.kit, gd, dirty);

            ui.heading("Weapon Proficiencies (pips)");
            ui.end_row();
            drag_prof(ui, "Large Sword", &mut cre.prof_large_sword, dirty);
            drag_prof(ui, "Small Sword", &mut cre.prof_small_sword, dirty);
            drag_prof(ui, "Bow", &mut cre.prof_bow, dirty);
            drag_prof(ui, "Spear", &mut cre.prof_spear, dirty);
            drag_prof(ui, "Blunt", &mut cre.prof_blunt, dirty);
            drag_prof(ui, "Spiked", &mut cre.prof_spiked, dirty);
            drag_prof(ui, "Axe", &mut cre.prof_axe, dirty);
            drag_prof(ui, "Missile", &mut cre.prof_missile, dirty);
        });

        egui::Grid::new("skills_grid").num_columns(2).spacing([16.0, 6.0]).striped(true).show(&mut cols[1], |ui| {
            ui.heading("Thief Skills (%)");
            ui.end_row();
            drag_u8(ui, "Pick Pockets", &mut cre.pick_pockets, 0..=255, dirty);
            drag_u8(ui, "Open Locks", &mut cre.open_locks, 0..=255, dirty);
            drag_u8(ui, "Find/Remove Traps", &mut cre.find_traps, 0..=255, dirty);
            drag_u8(ui, "Move Silently", &mut cre.move_silently, 0..=255, dirty);
            drag_u8(ui, "Hide in Shadows", &mut cre.hide_in_shadows, 0..=255, dirty);
            drag_u8(ui, "Detect Illusion", &mut cre.detect_illusion, 0..=255, dirty);
            drag_u8(ui, "Set Traps", &mut cre.set_traps, 0..=255, dirty);
            drag_u8(ui, "Lore", &mut cre.lore, 0..=255, dirty);

            ui.heading("Other");
            ui.end_row();
            drag_u8(ui, "Attacks / Round (raw)", &mut cre.attacks_per_round, 0..=10, dirty);
            drag_u8(ui, "Turn Undead Level", &mut cre.turn_undead_level, 0..=30, dirty);
            drag_u8(ui, "Luck", &mut cre.luck, 0..=100, dirty);
            drag_u8(ui, "Reputation (this creature)", &mut cre.reputation, 0..=20, dirty);
        });
    });
}

fn drag_prof(ui: &mut Ui, label: &str, prof: &mut crate::format::cre::ProfByte, dirty: &mut bool) {
    ui.label(label);
    let mut rank = prof.rank();
    let old = rank;
    let resp = ui.add(DragValue::new(&mut rank).range(0..=5u8));
    if resp.changed() && rank != old {
        prof.set_rank(rank);
        *dirty = true;
    }
    ui.end_row();
}

/// A dropdown of every symbol in `<ids_name>` (e.g. `"CLASS.IDS"`), when a
/// game folder is set; otherwise a plain raw-number field. Ends the row.
fn ids_field_u8(ui: &mut Ui, label: &str, value: &mut u8, ids_name: &str, gd: Option<&GameData>, dirty: &mut bool) {
    ui.label(label);
    if let Some(table) = gd.and_then(|gd| gd.ids(ids_name)) {
        let current_label = table.name(*value as u32).map(|s| s.to_owned()).unwrap_or_else(|| format!("(unknown: {value})"));
        let mut chosen = *value;
        egui::ComboBox::from_id_salt((ids_name, label)).selected_text(current_label).show_ui(ui, |ui| {
            for (val, name) in &table.entries {
                if *val <= u8::MAX as u32 {
                    ui.selectable_value(&mut chosen, *val as u8, name);
                }
            }
        });
        if chosen != *value {
            *value = chosen;
            *dirty = true;
        }
    } else {
        let old = *value;
        if ui.add(DragValue::new(value).range(0..=255u8)).changed() && *value != old {
            *dirty = true;
        }
    }
    ui.end_row();
}

/// Same as `ids_field_u8` but for the 32-bit KIT.IDS value, and always
/// offers a synthetic "No Kit" (0) option since not every KIT.IDS
/// explicitly defines a symbol for 0.
fn ids_field_kit(ui: &mut Ui, label: &str, value: &mut u32, gd: Option<&GameData>, dirty: &mut bool) {
    ui.label(label);
    if let Some(table) = gd.and_then(|gd| gd.ids("KIT.IDS")) {
        let current_label = if *value == 0 {
            "No Kit".to_owned()
        } else {
            table.name(*value).map(|s| s.to_owned()).unwrap_or_else(|| format!("(unknown: {value:#06x})"))
        };
        let mut chosen = *value;
        egui::ComboBox::from_id_salt("kit_combo").selected_text(current_label).show_ui(ui, |ui| {
            ui.selectable_value(&mut chosen, 0u32, "No Kit");
            for (val, name) in &table.entries {
                ui.selectable_value(&mut chosen, *val, name);
            }
        });
        if chosen != *value {
            *value = chosen;
            *dirty = true;
        }
    } else {
        let old = *value;
        if ui.add(DragValue::new(value)).changed() && *value != old {
            *dirty = true;
        }
    }
    ui.end_row();
}

#[derive(Clone, Copy)]
enum CatalogKind {
    Item,
    Spell,
}

impl CatalogKind {
    fn name(&self, gd: &GameData, resref: &str) -> Option<String> {
        match self {
            CatalogKind::Item => gd.item_name(resref),
            CatalogKind::Spell => gd.spell_name(resref),
        }
    }
    fn catalog(&self, gd: &GameData) -> crate::gamedata::Catalog {
        match self {
            CatalogKind::Item => gd.item_catalog(),
            CatalogKind::Spell => gd.spell_catalog(),
        }
    }
    fn window_title(&self) -> &'static str {
        match self {
            CatalogKind::Item => "Pick an Item",
            CatalogKind::Spell => "Pick a Spell",
        }
    }
}

/// Edits a ResRef as an up-to-8-character text field. When `gd` is
/// `Some`, also renders one more grid cell containing the resolved
/// display name plus a "🔍" search-by-name picker button — callers must
/// account for that extra column in their `Grid::num_columns` when a game
/// folder is set.
fn edit_resref(ui: &mut Ui, id: egui::Id, resref: &mut ResRef, dirty: &mut bool, width: f32, gd: Option<&GameData>, kind: CatalogKind) {
    let mut s = resref.as_str();
    if ui.add(egui::TextEdit::singleline(&mut s).desired_width(width)).changed() {
        s.truncate(8);
        let new = ResRef::from_str(&s.to_ascii_uppercase());
        if new != *resref {
            *resref = new;
            *dirty = true;
        }
    }
    if let Some(gd) = gd {
        ui.horizontal(|ui| {
            if !resref.is_empty() {
                let label = kind.name(gd, &resref.as_str()).unwrap_or_else(|| "(unknown)".to_owned());
                ui.label(egui::RichText::new(label).weak());
            }
            let mut picked = None;
            picker_button(ui, id, "🔍", gd, kind, |rr| picked = Some(rr));
            if let Some(rr) = picked {
                *resref = rr;
                *dirty = true;
            }
        });
    }
}

/// Renders a button that opens a searchable popup window listing every
/// entry in `kind`'s catalog (filtered live by a search box); calls
/// `on_select` once when the user clicks a result. Open/closed state and
/// the search query persist in egui's own per-`id` memory, so this needs
/// no state in `Bg2EditorApp` and multiple pickers on screen at once
/// (one per row) don't interfere with each other.
fn picker_button(ui: &mut Ui, id: egui::Id, button_label: &str, gd: &GameData, kind: CatalogKind, mut on_select: impl FnMut(ResRef)) {
    let open_id = id.with("picker_open");
    let query_id = id.with("picker_query");

    if ui.button(button_label).clicked() {
        ui.memory_mut(|m| m.data.insert_temp(open_id, true));
    }
    let is_open = ui.memory(|m| m.data.get_temp::<bool>(open_id).unwrap_or(false));
    if !is_open {
        return;
    }

    let mut query: String = ui.memory(|m| m.data.get_temp::<String>(query_id)).unwrap_or_default();
    let mut still_open = true;
    let mut picked: Option<ResRef> = None;
    egui::Window::new(kind.window_title())
        .id(id.with("picker_window"))
        .open(&mut still_open)
        .default_width(420.0)
        .default_height(480.0)
        .show(ui.ctx(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut query).hint_text("Search by name or code…"));
            ui.separator();
            let catalog = kind.catalog(gd);
            let q = query.to_ascii_lowercase();
            ScrollArea::vertical().show(ui, |ui| {
                let mut shown = 0usize;
                for (rr, name) in catalog.iter() {
                    if !q.is_empty() && !name.to_ascii_lowercase().contains(&q) && !rr.to_ascii_lowercase().contains(&q) {
                        continue;
                    }
                    if ui.selectable_label(false, format!("{name}  [{rr}]")).clicked() {
                        picked = Some(ResRef::from_str(rr));
                    }
                    shown += 1;
                    if shown >= 300 {
                        ui.weak("(more than 300 matches — keep typing to narrow it down)");
                        break;
                    }
                }
            });
        });
    ui.memory_mut(|m| m.data.insert_temp(query_id, query));
    if let Some(rr) = picked {
        on_select(rr);
        still_open = false;
    }
    ui.memory_mut(|m| m.data.insert_temp(open_id, still_open));
}

fn drag_u16_inline(ui: &mut Ui, value: &mut u16, dirty: &mut bool) {
    let old = *value;
    if ui.add(DragValue::new(value).range(0..=9999u16)).changed() && *value != old {
        *dirty = true;
    }
}

const SPELL_TYPES: [&str; 3] = ["Priest", "Wizard", "Innate"];

fn spell_type_combo(ui: &mut Ui, kind: &mut u16, dirty: &mut bool, id: impl std::hash::Hash) {
    let current = SPELL_TYPES.get(*kind as usize).copied().unwrap_or("?");
    let mut chosen = *kind;
    egui::ComboBox::from_id_salt(id).selected_text(current).show_ui(ui, |ui| {
        for (i, name) in SPELL_TYPES.iter().enumerate() {
            ui.selectable_value(&mut chosen, i as u16, *name);
        }
    });
    if chosen != *kind {
        *kind = chosen;
        *dirty = true;
    }
}

fn tab_inventory(ui: &mut Ui, cre: &mut CreV1, dirty: &mut bool, gd: Option<&GameData>) {
    let mut to_remove: Option<usize> = None;

    ui.columns(2, |cols| {
        egui::Grid::new("equip_grid").num_columns(2).spacing([12.0, 4.0]).striped(true).show(&mut cols[0], |ui| {
            ui.heading("Equipped");
            ui.end_row();
            let item_count = cre.items.len();
            for i in 0..cre.item_slots.len() {
                ui.label(ITEM_SLOT_NAMES[i]);
                let current = cre.item_slots[i];
                let item_label = |idx: usize, it: &InvItem| -> String {
                    let resref = it.item.as_str();
                    match gd.and_then(|gd| gd.item_name(&resref)) {
                        Some(name) => format!("[{idx}] {resref} — {name}"),
                        None => format!("[{idx}] {resref}"),
                    }
                };
                let selected_text = if current < 0 {
                    "(empty)".to_owned()
                } else if (current as usize) < item_count {
                    item_label(current as usize, &cre.items[current as usize])
                } else {
                    format!("(invalid index {current})")
                };
                let mut chosen = current;
                egui::ComboBox::from_id_salt(("equip_slot", i)).selected_text(selected_text).show_ui(ui, |ui| {
                    ui.selectable_value(&mut chosen, -1i16, "(empty)");
                    for (idx, it) in cre.items.iter().enumerate() {
                        ui.selectable_value(&mut chosen, idx as i16, item_label(idx, it));
                    }
                });
                if chosen != current {
                    cre.item_slots[i] = chosen;
                    *dirty = true;
                }
                ui.end_row();
            }
        });

        cols[1].heading("All Items");
        cols[1].label(egui::RichText::new("Includes equipped items; slots on the left reference these by index.").small().weak());
        cols[1].horizontal(|ui| {
            if ui.button("+ Add Blank Item").clicked() {
                cre.items.push(InvItem { item: ResRef::EMPTY, duration: 0, qty: [1, 0, 0], flags: 1 });
                *dirty = true;
            }
            if let Some(gd) = gd {
                let mut picked = None;
                picker_button(ui, ui.id().with("add_item"), "🔍 Add via Search", gd, CatalogKind::Item, |rr| picked = Some(rr));
                if let Some(rr) = picked {
                    cre.items.push(InvItem { item: rr, duration: 0, qty: [1, 0, 0], flags: 1 });
                    *dirty = true;
                }
            }
        });
        let num_cols = if gd.is_some() { 8 } else { 7 };
        ScrollArea::vertical().id_salt("items_scroll").max_height(520.0).show(&mut cols[1], |ui| {
            egui::Grid::new("items_grid").num_columns(num_cols).spacing([6.0, 4.0]).striped(true).show(ui, |ui| {
                ui.strong("#");
                ui.strong("Item");
                if gd.is_some() {
                    ui.strong("Name");
                }
                ui.strong("Qty 1");
                ui.strong("Qty 2");
                ui.strong("Qty 3");
                ui.strong("ID'd");
                ui.strong("");
                ui.end_row();
                for i in 0..cre.items.len() {
                    ui.label(format!("{i}"));
                    let row_id = ui.id().with(("item_row", i));
                    edit_resref(ui, row_id, &mut cre.items[i].item, dirty, 70.0, gd, CatalogKind::Item);
                    drag_u16_inline(ui, &mut cre.items[i].qty[0], dirty);
                    drag_u16_inline(ui, &mut cre.items[i].qty[1], dirty);
                    drag_u16_inline(ui, &mut cre.items[i].qty[2], dirty);
                    let mut identified = cre.items[i].flags & 1 != 0;
                    if ui.checkbox(&mut identified, "").changed() {
                        if identified {
                            cre.items[i].flags |= 1;
                        } else {
                            cre.items[i].flags &= !1;
                        }
                        *dirty = true;
                    }
                    if ui.button("✖").clicked() {
                        to_remove = Some(i);
                    }
                    ui.end_row();
                }
            });
        });
    });

    if let Some(i) = to_remove {
        cre.remove_item(i);
        *dirty = true;
    }
}

enum SpellAction {
    RemoveKnown(usize),
    AddKnown(ResRef),
    AddLevel,
    RemoveLevel(usize),
    AddMemorized(usize, ResRef),
    RemoveMemorized(usize, usize),
}

fn tab_spells(ui: &mut Ui, cre: &mut CreV1, dirty: &mut bool, gd: Option<&GameData>) {
    let mut action: Option<SpellAction> = None;
    let known_cols = if gd.is_some() { 5 } else { 4 };
    let mem_cols = if gd.is_some() { 6 } else { 5 };

    ui.columns(2, |cols| {
        cols[0].heading("Known Spells");
        cols[0].horizontal(|ui| {
            if ui.button("+ Add Blank").clicked() {
                action = Some(SpellAction::AddKnown(ResRef::EMPTY));
            }
            if let Some(gd) = gd {
                let mut picked = None;
                picker_button(ui, ui.id().with("add_known"), "🔍 Add via Search", gd, CatalogKind::Spell, |rr| picked = Some(rr));
                if let Some(rr) = picked {
                    action = Some(SpellAction::AddKnown(rr));
                }
            }
        });
        ScrollArea::vertical().id_salt("known_scroll").max_height(520.0).show(&mut cols[0], |ui| {
            egui::Grid::new("known_grid").num_columns(known_cols).spacing([6.0, 4.0]).striped(true).show(ui, |ui| {
                ui.strong("Spell");
                if gd.is_some() {
                    ui.strong("Name");
                }
                ui.strong("Level");
                ui.strong("Type");
                ui.strong("");
                ui.end_row();
                for i in 0..cre.known_spells.len() {
                    let row_id = ui.id().with(("known_row", i));
                    edit_resref(ui, row_id, &mut cre.known_spells[i].spell, dirty, 80.0, gd, CatalogKind::Spell);
                    let mut lvl = cre.known_spells[i].level;
                    if ui.add(DragValue::new(&mut lvl).range(1..=9u16)).changed() {
                        cre.known_spells[i].level = lvl;
                        *dirty = true;
                    }
                    spell_type_combo(ui, &mut cre.known_spells[i].kind, dirty, ("known_kind", i));
                    if ui.button("✖").clicked() {
                        action = Some(SpellAction::RemoveKnown(i));
                    }
                    ui.end_row();
                }
            });
        });

        cols[1].heading("Memorized Spells");
        if cols[1].button("+ Add Spell Level").clicked() {
            action = Some(SpellAction::AddLevel);
        }
        ScrollArea::vertical().id_salt("mem_scroll").max_height(520.0).show(&mut cols[1], |ui| {
            for level_idx in 0..cre.mem_info.len() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Level:");
                        let mut lvl = cre.mem_info[level_idx].level;
                        if ui.add(DragValue::new(&mut lvl).range(1..=9u16)).changed() {
                            cre.mem_info[level_idx].level = lvl;
                            *dirty = true;
                        }
                        ui.label("Type:");
                        spell_type_combo(ui, &mut cre.mem_info[level_idx].kind, dirty, ("mem_kind", level_idx));
                        ui.label("Max:");
                        let mut total = cre.mem_info[level_idx].memorizable_total;
                        if ui.add(DragValue::new(&mut total).range(0..=99u16)).changed() {
                            cre.mem_info[level_idx].memorizable_total = total;
                            *dirty = true;
                        }
                        ui.label("Currently castable:");
                        let mut current = cre.mem_info[level_idx].memorizable_current;
                        if ui.add(DragValue::new(&mut current).range(0..=99u16)).changed() {
                            cre.mem_info[level_idx].memorizable_current = current;
                            *dirty = true;
                        }
                        if ui.button("✖ Remove Level").clicked() {
                            action = Some(SpellAction::RemoveLevel(level_idx));
                        }
                    });

                    let start = cre.mem_info[level_idx].table_index as usize;
                    let count = cre.mem_info[level_idx].count as usize;
                    egui::Grid::new(("mem_grid", level_idx)).num_columns(mem_cols).spacing([6.0, 4.0]).striped(true).show(ui, |ui| {
                        for local_idx in 0..count {
                            let row_id = ui.id().with(("mem_row", level_idx, local_idx));
                            let spell = &mut cre.memorized_spells[start + local_idx];
                            edit_resref(ui, row_id, &mut spell.spell, dirty, 80.0, gd, CatalogKind::Spell);
                            let mut cast = spell.flags & 0b001 != 0;
                            let mut memorized = spell.flags & 0b010 != 0;
                            let mut disabled = spell.flags & 0b100 != 0;
                            if ui.checkbox(&mut memorized, "Memorized").changed()
                                || ui.checkbox(&mut cast, "Cast").changed()
                                || ui.checkbox(&mut disabled, "Disabled").changed()
                            {
                                spell.flags = (cast as u16) | ((memorized as u16) << 1) | ((disabled as u16) << 2);
                                *dirty = true;
                            }
                            if ui.button("✖").clicked() {
                                action = Some(SpellAction::RemoveMemorized(level_idx, local_idx));
                            }
                            ui.end_row();
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("+ Add Blank Spell").clicked() {
                            action = Some(SpellAction::AddMemorized(level_idx, ResRef::EMPTY));
                        }
                        if let Some(gd) = gd {
                            let mut picked = None;
                            picker_button(ui, ui.id().with(("add_mem", level_idx)), "🔍 Add via Search", gd, CatalogKind::Spell, |rr| picked = Some(rr));
                            if let Some(rr) = picked {
                                action = Some(SpellAction::AddMemorized(level_idx, rr));
                            }
                        }
                    });
                });
            }
        });
    });

    match action {
        Some(SpellAction::RemoveKnown(i)) => {
            cre.known_spells.remove(i);
            *dirty = true;
        }
        Some(SpellAction::AddKnown(spell)) => {
            cre.known_spells.push(KnownSpell { spell, level: 1, kind: 1 });
            *dirty = true;
        }
        Some(SpellAction::AddLevel) => {
            cre.add_mem_level(1, 1, 1);
            *dirty = true;
        }
        Some(SpellAction::RemoveLevel(i)) => {
            cre.remove_mem_level(i);
            *dirty = true;
        }
        Some(SpellAction::AddMemorized(level_idx, spell)) => {
            cre.add_memorized_spell(level_idx, MemorizedSpell { spell, flags: 0b010 });
            *dirty = true;
        }
        Some(SpellAction::RemoveMemorized(level_idx, local_idx)) => {
            cre.remove_memorized_spell(level_idx, local_idx);
            *dirty = true;
        }
        None => {}
    }
}
