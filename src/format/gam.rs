//! GAM V2.0 (BG2/BG2:EE) save-game structure: header, party members (with
//! embedded CRE), non-party members, global variables, journal entries,
//! familiar info, and stored/pocket-plane locations. Byte offsets verified
//! directly against `NearInfinity/src/org/infinity/resource/gam/GamResource.java`
//! and `PartyNPC.java` for the `Profile.Engine.BG2 || Profile.isEnhancedEdition()`
//! branch.

use super::cre::CreV1;
use super::primitives::{read_i16, read_resref, read_text, read_u16, read_u32, read_u8, ResRef, Writer};

const PARTY_MEMBER_FIXED_LEN: usize = 352;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub name: [u8; 32],
    pub var_type: u16,
    pub reference: u16,
    pub dword: u32,
    pub int_value: i32,
    pub double_value: [u8; 8], // opaque (unused by the engine), round-tripped raw
    pub script_name: [u8; 32],
}

#[derive(Clone, Debug, PartialEq)]
pub struct JournalEntry {
    pub text_strref: i32,
    pub time: u32,
    pub chapter: u8,
    pub unused: u8,
    pub section: u8,
    pub source: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredLocation {
    pub area: ResRef,
    pub x: u16,
    pub y: u16,
}

/// The "Familiar info" section. In every real-world save observed, the
/// trailing variable-length "extra familiar resources" array is empty
/// (NearInfinity's own comment: "never seen these fields in use"); we only
/// support that common case and refuse to load anything else rather than
/// risk silently corrupting a save with the more complex relocation logic
/// a populated array would require.
#[derive(Clone, Debug, PartialEq)]
pub struct FamiliarSection {
    pub raw: [u8; 400],
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartyMember {
    pub selection_state: u16,
    pub party_position: u16,
    /// Raw 8 bytes: either a literal marker string or a ResourceRef to an
    /// external .CRE, byte-identical either way, so no need to distinguish.
    pub character_field: ResRef,
    pub orientation: u32,
    pub current_area: ResRef,
    pub location_x: i16,
    pub location_y: i16,
    pub viewport_x: i16,
    pub viewport_y: i16,
    pub modal_state: u16,
    pub happiness: u16,
    pub unused_96: [u8; 96],
    pub quick_weapon_slots: [u16; 4],
    pub quick_weapon_abilities: [u16; 4],
    pub quick_spells: [ResRef; 3],
    pub quick_item_slots: [u16; 3],
    pub quick_item_abilities: [u16; 3],
    pub name: [u8; 32],
    pub num_times_talked_to: u32,
    pub stat_foe_vanquished_strref: i32,
    pub stat_xp_foe_vanquished: u32,
    pub stat_time_in_party: u32,
    pub stat_join_time: u32,
    pub stat_in_party: u8,
    pub stat_unused: [u8; 2],
    pub stat_initial_char: u8,
    pub stat_kills_xp_chapter: u32,
    pub stat_num_kills_chapter: u32,
    pub stat_kills_xp_game: u32,
    pub stat_num_kills_game: u32,
    pub stat_fav_spell: [ResRef; 4],
    pub stat_fav_spell_count: [u16; 4],
    pub stat_fav_weapon: [ResRef; 4],
    pub stat_fav_weapon_count: [u16; 4],
    pub voice_set: [u8; 8],
    /// The embedded creature. `None` only for the rare case of an
    /// out-of-party reference-by-resref record (creOffset == 0).
    pub cre: Option<CreV1>,
}

impl PartyMember {
    fn parse(buf: &[u8], offset: usize) -> Result<PartyMember, String> {
        let cre_offset = read_u32(buf, offset + 4) as usize;
        let mut quick_weapon_slots = [0u16; 4];
        let mut quick_weapon_abilities = [0u16; 4];
        for i in 0..4 {
            quick_weapon_slots[i] = read_u16(buf, offset + 140 + i * 2);
            quick_weapon_abilities[i] = read_u16(buf, offset + 148 + i * 2);
        }
        let mut quick_spells = [ResRef::EMPTY; 3];
        let mut quick_item_slots = [0u16; 3];
        let mut quick_item_abilities = [0u16; 3];
        for i in 0..3 {
            quick_spells[i] = read_resref(buf, offset + 156 + i * 8);
            quick_item_slots[i] = read_u16(buf, offset + 180 + i * 2);
            quick_item_abilities[i] = read_u16(buf, offset + 186 + i * 2);
        }
        let mut name = [0u8; 32];
        name.copy_from_slice(&buf[offset + 192..offset + 224]);
        let mut unused_96 = [0u8; 96];
        unused_96.copy_from_slice(&buf[offset + 44..offset + 140]);

        let cs = offset + 228; // char-stats block start
        let mut stat_fav_spell = [ResRef::EMPTY; 4];
        let mut stat_fav_spell_count = [0u16; 4];
        let mut stat_fav_weapon = [ResRef::EMPTY; 4];
        let mut stat_fav_weapon_count = [0u16; 4];
        for i in 0..4 {
            stat_fav_spell[i] = read_resref(buf, cs + 36 + i * 8);
            stat_fav_spell_count[i] = read_u16(buf, cs + 68 + i * 2);
            stat_fav_weapon[i] = read_resref(buf, cs + 76 + i * 8);
            stat_fav_weapon_count[i] = read_u16(buf, cs + 108 + i * 2);
        }
        let mut voice_set = [0u8; 8];
        voice_set.copy_from_slice(&buf[cs + 116..cs + 124]);

        let cre = if cre_offset != 0 {
            Some(CreV1::parse(buf, cre_offset)?)
        } else {
            None
        };

        Ok(PartyMember {
            selection_state: read_u16(buf, offset),
            party_position: read_u16(buf, offset + 2),
            character_field: read_resref(buf, offset + 12),
            orientation: read_u32(buf, offset + 20),
            current_area: read_resref(buf, offset + 24),
            location_x: read_i16(buf, offset + 32),
            location_y: read_i16(buf, offset + 34),
            viewport_x: read_i16(buf, offset + 36),
            viewport_y: read_i16(buf, offset + 38),
            modal_state: read_u16(buf, offset + 40),
            happiness: read_u16(buf, offset + 42),
            unused_96,
            quick_weapon_slots,
            quick_weapon_abilities,
            quick_spells,
            quick_item_slots,
            quick_item_abilities,
            name,
            num_times_talked_to: read_u32(buf, offset + 224),
            stat_foe_vanquished_strref: read_u32(buf, cs) as i32,
            stat_xp_foe_vanquished: read_u32(buf, cs + 4),
            stat_time_in_party: read_u32(buf, cs + 8),
            stat_join_time: read_u32(buf, cs + 12),
            stat_in_party: read_u8(buf, cs + 16),
            stat_unused: [read_u8(buf, cs + 17), read_u8(buf, cs + 18)],
            stat_initial_char: read_u8(buf, cs + 19),
            stat_kills_xp_chapter: read_u32(buf, cs + 20),
            stat_num_kills_chapter: read_u32(buf, cs + 24),
            stat_kills_xp_game: read_u32(buf, cs + 28),
            stat_num_kills_game: read_u32(buf, cs + 32),
            stat_fav_spell,
            stat_fav_spell_count,
            stat_fav_weapon,
            stat_fav_weapon_count,
            voice_set,
            cre,
        })
    }

    /// Writes the fixed 352-byte header only (not the embedded CRE, which
    /// is placed separately and referenced by absolute offset).
    fn serialize_header(&self, w: &mut Writer, cre_offset: u32, cre_size: u32) {
        let start = w.len();
        w.u16(self.selection_state);
        w.u16(self.party_position);
        w.u32(cre_offset);
        w.u32(cre_size);
        w.resref(self.character_field);
        w.u32(self.orientation);
        w.resref(self.current_area);
        w.i16(self.location_x);
        w.i16(self.location_y);
        w.i16(self.viewport_x);
        w.i16(self.viewport_y);
        w.u16(self.modal_state);
        w.u16(self.happiness);
        w.bytes(&self.unused_96);
        for v in self.quick_weapon_slots {
            w.u16(v);
        }
        for v in self.quick_weapon_abilities {
            w.u16(v);
        }
        for v in self.quick_spells {
            w.resref(v);
        }
        for v in self.quick_item_slots {
            w.u16(v);
        }
        for v in self.quick_item_abilities {
            w.u16(v);
        }
        w.bytes(&self.name);
        w.u32(self.num_times_talked_to);
        w.i32(self.stat_foe_vanquished_strref);
        w.u32(self.stat_xp_foe_vanquished);
        w.u32(self.stat_time_in_party);
        w.u32(self.stat_join_time);
        w.u8(self.stat_in_party);
        w.bytes(&self.stat_unused);
        w.u8(self.stat_initial_char);
        w.u32(self.stat_kills_xp_chapter);
        w.u32(self.stat_num_kills_chapter);
        w.u32(self.stat_kills_xp_game);
        w.u32(self.stat_num_kills_game);
        for v in self.stat_fav_spell {
            w.resref(v);
        }
        for v in self.stat_fav_spell_count {
            w.u16(v);
        }
        for v in self.stat_fav_weapon {
            w.resref(v);
        }
        for v in self.stat_fav_weapon_count {
            w.u16(v);
        }
        w.bytes(&self.voice_set);
        debug_assert_eq!(w.len() - start, PARTY_MEMBER_FIXED_LEN);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GamFile {
    pub game_time: u32,
    pub selected_formation: u16,
    pub formation_buttons: [u16; 5],
    pub party_gold: u32,
    pub view_player_area: u16,
    pub weather: u16,
    pub world_area: ResRef,
    pub current_link: u32,
    pub reputation: u32,
    pub master_area: ResRef,
    pub configuration: u32,
    pub save_version: u32,
    pub real_time: u32,
    pub zoom_level: u32,
    pub random_encounter_area: ResRef,
    pub worldmap: ResRef,
    pub campaign: [u8; 8],
    pub familiar_owner: u32,
    pub encounter_entry: [u8; 20],

    pub party: Vec<PartyMember>,
    pub non_party: Vec<PartyMember>,
    /// Opaque, unused-by-the-engine "party inventory" records (20 bytes
    /// each); preserved for round-tripping only.
    pub party_inventory_raw: Vec<u8>,
    pub globals: Vec<Variable>,
    pub journal: Vec<JournalEntry>,
    pub familiar: Option<FamiliarSection>,
    pub stored_locations: Vec<StoredLocation>,
    pub pocket_plane_locations: Vec<StoredLocation>,
}

impl GamFile {
    pub fn parse(buf: &[u8]) -> Result<GamFile, String> {
        let sig = read_text(buf, 0, 4);
        let ver = read_text(buf, 4, 4);
        if sig != "GAME" || ver != "V2.0" {
            return Err(format!(
                "unsupported save format: signature {sig:?} version {ver:?} (expected GAME/V2.0 — this editor targets BG2:EE / BG1:EE saves only)"
            ));
        }

        let off_party = read_u32(buf, 32) as usize;
        let num_party = read_u32(buf, 36) as usize;
        let off_inv = read_u32(buf, 40) as usize;
        let num_inv = read_u32(buf, 44) as usize;
        let off_non_party = read_u32(buf, 48) as usize;
        let num_non_party = read_u32(buf, 52) as usize;
        let off_globals = read_u32(buf, 56) as usize;
        let num_globals = read_u32(buf, 60) as usize;
        let num_journal = read_u32(buf, 76) as usize;
        let off_journal = read_u32(buf, 80) as usize;

        let off_familiar = read_u32(buf, 104) as usize;
        let off_locations = read_u32(buf, 108) as usize;
        let num_locations = read_u32(buf, 112) as usize;
        let off_pocket = read_u32(buf, 120) as usize;
        let num_pocket = read_u32(buf, 124) as usize;

        let mut party = Vec::with_capacity(num_party);
        for i in 0..num_party {
            party.push(PartyMember::parse(buf, off_party + i * PARTY_MEMBER_FIXED_LEN)?);
        }
        let mut non_party = Vec::with_capacity(num_non_party);
        for i in 0..num_non_party {
            non_party.push(PartyMember::parse(buf, off_non_party + i * PARTY_MEMBER_FIXED_LEN)?);
        }

        let party_inventory_raw = if off_inv > 0 {
            buf[off_inv..off_inv + num_inv * 20].to_vec()
        } else {
            Vec::new()
        };

        let mut globals = Vec::with_capacity(num_globals);
        for i in 0..num_globals {
            let o = off_globals + i * 84;
            let mut name = [0u8; 32];
            name.copy_from_slice(&buf[o..o + 32]);
            let mut double_value = [0u8; 8];
            double_value.copy_from_slice(&buf[o + 44..o + 52]);
            let mut script_name = [0u8; 32];
            script_name.copy_from_slice(&buf[o + 52..o + 84]);
            globals.push(Variable {
                name,
                var_type: read_u16(buf, o + 32),
                reference: read_u16(buf, o + 34),
                dword: read_u32(buf, o + 36),
                int_value: read_u32(buf, o + 40) as i32,
                double_value,
                script_name,
            });
        }

        let mut journal = Vec::with_capacity(num_journal);
        for i in 0..num_journal {
            let o = off_journal + i * 12;
            journal.push(JournalEntry {
                text_strref: read_u32(buf, o) as i32,
                time: read_u32(buf, o + 4),
                chapter: read_u8(buf, o + 8),
                unused: read_u8(buf, o + 9),
                section: read_u8(buf, o + 10),
                source: read_u8(buf, o + 11),
            });
        }

        let familiar = if off_familiar > 0 {
            let mut raw = [0u8; 400];
            raw.copy_from_slice(&buf[off_familiar..off_familiar + 400]);
            // Verify the "never used in practice" assumption: all 81
            // per-alignment/level familiar counts (bytes 76..400) must be
            // zero, or we'd need to relocate a trailing resref array we
            // don't support round-tripping.
            if raw[76..400].iter().any(|&b| b != 0) {
                return Err(
                    "save contains extended familiar data (unsupported edge case); refusing to load to avoid corrupting the save".into(),
                );
            }
            Some(FamiliarSection { raw })
        } else {
            None
        };

        let mut stored_locations = Vec::with_capacity(num_locations);
        for i in 0..num_locations {
            let o = off_locations + i * 12;
            stored_locations.push(StoredLocation {
                area: read_resref(buf, o),
                x: read_u16(buf, o + 8),
                y: read_u16(buf, o + 10),
            });
        }
        let mut pocket_plane_locations = Vec::with_capacity(num_pocket);
        for i in 0..num_pocket {
            let o = off_pocket + i * 12;
            pocket_plane_locations.push(StoredLocation {
                area: read_resref(buf, o),
                x: read_u16(buf, o + 8),
                y: read_u16(buf, o + 10),
            });
        }

        let mut campaign = [0u8; 8];
        campaign.copy_from_slice(&buf[148..156]);
        let mut encounter_entry = [0u8; 20];
        encounter_entry.copy_from_slice(&buf[160..180]);

        Ok(GamFile {
            game_time: read_u32(buf, 8),
            selected_formation: read_u16(buf, 12),
            formation_buttons: [
                read_u16(buf, 14),
                read_u16(buf, 16),
                read_u16(buf, 18),
                read_u16(buf, 20),
                read_u16(buf, 22),
            ],
            party_gold: read_u32(buf, 24),
            view_player_area: read_u16(buf, 28),
            weather: read_u16(buf, 30),
            world_area: read_resref(buf, 64),
            current_link: read_u32(buf, 72),
            reputation: read_u32(buf, 84),
            master_area: read_resref(buf, 88),
            configuration: read_u32(buf, 96),
            save_version: read_u32(buf, 100),
            real_time: read_u32(buf, 116),
            zoom_level: read_u32(buf, 128),
            random_encounter_area: read_resref(buf, 132),
            worldmap: read_resref(buf, 140),
            campaign,
            familiar_owner: read_u32(buf, 156),
            encounter_entry,

            party,
            non_party,
            party_inventory_raw,
            globals,
            journal,
            familiar,
            stored_locations,
            pocket_plane_locations,
        })
    }

    /// Rebuilds the entire GAM file from scratch in a fixed canonical
    /// section order, recomputing every offset/count field. Party-member
    /// CRE blobs are re-serialized (picking up any edits) and their
    /// offset/size fields updated accordingly.
    pub fn serialize(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.text("GAME", 4);
        w.text("V2.0", 4);
        w.u32(self.game_time);
        w.u16(self.selected_formation);
        for v in self.formation_buttons {
            w.u16(v);
        }
        w.u32(self.party_gold);
        w.u16(self.view_player_area);
        w.u16(self.weather);

        let at_off_party = w.len();
        w.zeros(4);
        let at_count_party = w.len();
        w.zeros(4);
        let at_off_inv = w.len();
        w.zeros(4);
        let at_count_inv = w.len();
        w.zeros(4);
        let at_off_non_party = w.len();
        w.zeros(4);
        let at_count_non_party = w.len();
        w.zeros(4);
        let at_off_globals = w.len();
        w.zeros(4);
        let at_count_globals = w.len();
        w.zeros(4);
        w.resref(self.world_area);
        w.u32(self.current_link);
        let at_count_journal = w.len();
        w.zeros(4);
        let at_off_journal = w.len();
        w.zeros(4);
        w.u32(self.reputation);
        w.resref(self.master_area);
        w.u32(self.configuration);
        w.u32(self.save_version);
        let at_off_familiar = w.len();
        w.zeros(4);
        let at_off_locations = w.len();
        w.zeros(4);
        let at_count_locations = w.len();
        w.zeros(4);
        w.u32(self.real_time);
        let at_off_pocket = w.len();
        w.zeros(4);
        let at_count_pocket = w.len();
        w.zeros(4);
        w.u32(self.zoom_level);
        w.resref(self.random_encounter_area);
        w.resref(self.worldmap);
        w.bytes(&self.campaign);
        w.u32(self.familiar_owner);
        w.bytes(&self.encounter_entry);
        debug_assert_eq!(w.len(), 180);

        // --- party members: fixed headers first, then embedded CRE blobs ---
        let off_party = w.len() as u32;
        let party_header_start = w.len();
        for m in &self.party {
            m.serialize_header(&mut w, 0, 0); // placeholder cre offset/size
        }
        let mut party_cre_ranges = Vec::with_capacity(self.party.len());
        for m in &self.party {
            if let Some(cre) = &m.cre {
                let bytes = cre.serialize();
                let cre_off = w.len() as u32;
                let cre_size = bytes.len() as u32;
                w.bytes(&bytes);
                party_cre_ranges.push(Some((cre_off, cre_size)));
            } else {
                party_cre_ranges.push(None);
            }
        }
        // back-patch each header's CRE offset/size fields (bytes 4/8 within
        // that member's 352-byte record)
        for (i, range) in party_cre_ranges.iter().enumerate() {
            if let Some((off, size)) = range {
                let rec = party_header_start + i * PARTY_MEMBER_FIXED_LEN;
                w.patch_u32(rec + 4, *off);
                w.patch_u32(rec + 8, *size);
            }
        }

        let off_non_party = w.len() as u32;
        let non_party_header_start = w.len();
        for m in &self.non_party {
            m.serialize_header(&mut w, 0, 0);
        }
        let mut non_party_cre_ranges = Vec::with_capacity(self.non_party.len());
        for m in &self.non_party {
            if let Some(cre) = &m.cre {
                let bytes = cre.serialize();
                let cre_off = w.len() as u32;
                let cre_size = bytes.len() as u32;
                w.bytes(&bytes);
                non_party_cre_ranges.push(Some((cre_off, cre_size)));
            } else {
                non_party_cre_ranges.push(None);
            }
        }
        for (i, range) in non_party_cre_ranges.iter().enumerate() {
            if let Some((off, size)) = range {
                let rec = non_party_header_start + i * PARTY_MEMBER_FIXED_LEN;
                w.patch_u32(rec + 4, *off);
                w.patch_u32(rec + 8, *size);
            }
        }

        let off_inv = if self.party_inventory_raw.is_empty() {
            0
        } else {
            let o = w.len() as u32;
            w.bytes(&self.party_inventory_raw);
            o
        };

        let off_globals = w.len() as u32;
        for v in &self.globals {
            w.bytes(&v.name);
            w.u16(v.var_type);
            w.u16(v.reference);
            w.u32(v.dword);
            w.i32(v.int_value);
            w.bytes(&v.double_value);
            w.bytes(&v.script_name);
        }

        let off_journal = w.len() as u32;
        for j in &self.journal {
            w.i32(j.text_strref);
            w.u32(j.time);
            w.u8(j.chapter);
            w.u8(j.unused);
            w.u8(j.section);
            w.u8(j.source);
        }

        let off_familiar = if let Some(fam) = &self.familiar {
            let o = w.len() as u32;
            w.bytes(&fam.raw);
            o
        } else {
            0
        };

        let mut off_locations = w.len() as u32;
        for l in &self.stored_locations {
            w.resref(l.area);
            w.u16(l.x);
            w.u16(l.y);
        }

        let mut off_pocket = w.len() as u32;
        for l in &self.pocket_plane_locations {
            w.resref(l.area);
            w.u16(l.x);
            w.u16(l.y);
        }

        // The game engine writes these two offsets as end-of-file (rather
        // than 0) when the corresponding section is empty; matching that
        // convention exactly (observed against a real save) rather than
        // just being functionally equivalent.
        let eof = w.len() as u32;
        if self.stored_locations.is_empty() {
            off_locations = eof;
        }
        if self.pocket_plane_locations.is_empty() {
            off_pocket = eof;
        }

        w.patch_u32(at_off_party, off_party);
        w.patch_u32(at_count_party, self.party.len() as u32);
        w.patch_u32(at_off_inv, off_inv);
        w.patch_u32(at_count_inv, (self.party_inventory_raw.len() / 20) as u32);
        w.patch_u32(at_off_non_party, off_non_party);
        w.patch_u32(at_count_non_party, self.non_party.len() as u32);
        w.patch_u32(at_off_globals, off_globals);
        w.patch_u32(at_count_globals, self.globals.len() as u32);
        w.patch_u32(at_count_journal, self.journal.len() as u32);
        w.patch_u32(at_off_journal, off_journal);
        w.patch_u32(at_off_familiar, off_familiar);
        w.patch_u32(at_off_locations, off_locations);
        w.patch_u32(at_count_locations, self.stored_locations.len() as u32);
        w.patch_u32(at_off_pocket, off_pocket);
        w.patch_u32(at_count_pocket, self.pocket_plane_locations.len() as u32);

        w.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::cre::sample_cre;

    fn sample_party_member(with_cre: bool) -> PartyMember {
        PartyMember {
            selection_state: 1,
            party_position: 0,
            character_field: ResRef::from_str("*"),
            orientation: 3,
            current_area: ResRef::from_str("AR0602"),
            location_x: 100,
            location_y: -50,
            viewport_x: 10,
            viewport_y: 20,
            modal_state: 0,
            happiness: 50,
            unused_96: [7u8; 96],
            quick_weapon_slots: [1, 2, 3, 4],
            quick_weapon_abilities: [0, 1, 0, 1],
            quick_spells: [ResRef::from_str("SPWI112"), ResRef::EMPTY, ResRef::EMPTY],
            quick_item_slots: [5, 6, 7],
            quick_item_abilities: [0, 0, 0],
            name: {
                let mut a = [0u8; 32];
                a[..6].copy_from_slice(b"Imoen2");
                a
            },
            num_times_talked_to: 4,
            stat_foe_vanquished_strref: -1,
            stat_xp_foe_vanquished: 0,
            stat_time_in_party: 12345,
            stat_join_time: 6789,
            stat_in_party: 1,
            stat_unused: [0, 0],
            stat_initial_char: b'I',
            stat_kills_xp_chapter: 100,
            stat_num_kills_chapter: 2,
            stat_kills_xp_game: 500,
            stat_num_kills_game: 10,
            stat_fav_spell: [ResRef::EMPTY; 4],
            stat_fav_spell_count: [0; 4],
            stat_fav_weapon: [ResRef::EMPTY; 4],
            stat_fav_weapon_count: [0; 4],
            voice_set: {
                let mut a = [0u8; 8];
                a[..4].copy_from_slice(b"IMO2");
                a
            },
            cre: if with_cre { Some(sample_cre()) } else { None },
        }
    }

    fn sample_gam() -> GamFile {
        GamFile {
            game_time: 987654,
            selected_formation: 2,
            formation_buttons: [1, 2, 3, 4, 5],
            party_gold: 15000,
            view_player_area: 0,
            weather: 1,
            world_area: ResRef::from_str("AR0602"),
            current_link: 0,
            reputation: 100,
            master_area: ResRef::from_str("AR0602"),
            configuration: 0,
            save_version: 0,
            real_time: 42,
            zoom_level: 0,
            random_encounter_area: ResRef::EMPTY,
            worldmap: ResRef::from_str("WORLDMAP"),
            campaign: [0u8; 8],
            familiar_owner: 0,
            encounter_entry: [0u8; 20],

            party: vec![sample_party_member(true), sample_party_member(true)],
            non_party: vec![sample_party_member(false)],
            party_inventory_raw: Vec::new(),
            globals: vec![Variable {
                name: {
                    let mut a = [0u8; 32];
                    a[..4].copy_from_slice(b"GLOB");
                    a
                },
                var_type: 0,
                reference: 0,
                dword: 0,
                int_value: 7,
                double_value: [0u8; 8],
                script_name: [0u8; 32],
            }],
            journal: vec![JournalEntry { text_strref: 42, time: 1000, chapter: 2, unused: 0, section: 3, source: 0xff }],
            familiar: Some(FamiliarSection { raw: [0u8; 400] }),
            stored_locations: vec![StoredLocation { area: ResRef::from_str("AR0000"), x: 1, y: 2 }],
            pocket_plane_locations: vec![],
        }
    }

    #[test]
    fn gam_round_trip() {
        let original = sample_gam();
        let bytes = original.serialize();
        let parsed = GamFile::parse(&bytes).expect("parse should succeed");
        assert_eq!(original, parsed);
    }
}
