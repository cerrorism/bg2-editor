use std::path::PathBuf;

use egui::{DragValue, ScrollArea, Ui};

use crate::format::cre::CreV1;
use crate::format::gam::GamFile;
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
}

impl Default for Bg2EditorApp {
    fn default() -> Self {
        let save_root = save_file::default_save_roots().into_iter().find(|p| p.is_dir());
        let save_folders = save_root
            .as_deref()
            .map(save_file::list_save_folders)
            .unwrap_or_default();
        Self {
            save_root,
            save_folders,
            selected_folder_idx: None,
            gam: None,
            selected_char_idx: 0,
            tab: Tab::Abilities,
            is_dirty: false,
            status_msg: String::new(),
        }
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
            ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Abilities => tab_abilities(ui, cre, dirty),
                Tab::ClassSkills => tab_class_skills(ui, cre, dirty),
                Tab::Inventory => {
                    ui.label("Inventory editing is not implemented yet.");
                }
                Tab::Spells => {
                    ui.label("Spellbook editing is not implemented yet.");
                }
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

fn tab_class_skills(ui: &mut Ui, cre: &mut CreV1, dirty: &mut bool) {
    ui.columns(2, |cols| {
        egui::Grid::new("class_grid").num_columns(2).spacing([16.0, 6.0]).striped(true).show(&mut cols[0], |ui| {
            ui.heading("Class & Levels");
            ui.end_row();
            drag_u8(ui, "Level (class 1)", &mut cre.level1, 1..=99, dirty);
            drag_u8(ui, "Level (class 2)", &mut cre.level2, 0..=99, dirty);
            drag_u8(ui, "Level (class 3)", &mut cre.level3, 0..=99, dirty);

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
