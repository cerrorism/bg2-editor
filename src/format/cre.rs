//! CRE V1.0 creature structure (BG1/BG2/BG2:EE), as embedded in a GAM party
//! member record. Byte offsets verified directly against
//! `NearInfinity/src/org/infinity/resource/cre/CreResource.java` `readOther()`
//! for the non-PSTEE, Enhanced-Edition branch (offsets relative to the start
//! of the CRE header, i.e. right after the 8-byte "CRE V1.0" signature).

use super::primitives::{read_i16, read_i8, read_resref, read_text, read_u16, read_u32, read_u8, ResRef, Writer};

/// Fixed size of the CRE header (signature+version + all fixed fields,
/// through the trailing section offset/count table and dialog ref).
const HEADER_LEN: usize = 8 + 716;
/// Number of item slots preceding the "selected weapon" fields (BG2/EE,
/// non-PSTEE layout): 9 armor slots + 4 weapons + 4 quivers + cloak + 3
/// quick slots + 16 general inventory + 1 magic weapon slot.
const ITEM_SLOT_COUNT: usize = 38;

// Reserved for the not-yet-built Inventory UI (equipped-slot labels).
#[allow(dead_code)]
pub const ITEM_SLOT_NAMES: [&str; ITEM_SLOT_COUNT] = [
    "Helmet", "Armor", "Shield", "Gloves", "Left Ring", "Right Ring", "Amulet", "Belt", "Boots",
    "Weapon 1", "Weapon 2", "Weapon 3", "Weapon 4",
    "Quiver 1", "Quiver 2", "Quiver 3", "Quiver 4",
    "Cloak",
    "Quick Slot 1", "Quick Slot 2", "Quick Slot 3",
    "Inventory 1", "Inventory 2", "Inventory 3", "Inventory 4", "Inventory 5", "Inventory 6",
    "Inventory 7", "Inventory 8", "Inventory 9", "Inventory 10", "Inventory 11", "Inventory 12",
    "Inventory 13", "Inventory 14", "Inventory 15", "Inventory 16",
    "Magic Weapon",
];

#[derive(Clone, Debug, PartialEq)]
pub struct KnownSpell {
    pub spell: ResRef,
    pub level: u16,
    pub kind: u16, // 0=Priest, 1=Wizard, 2=Innate
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemInfo {
    pub level: u16,
    pub memorizable_total: u16,
    pub memorizable_current: u16,
    pub kind: u16,
    pub table_index: u32,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemorizedSpell {
    pub spell: ResRef,
    pub flags: u16, // bit0=cast, bit1=memorized, bit2=disabled
}

#[derive(Clone, Debug, PartialEq)]
pub struct InvItem {
    pub item: ResRef,
    pub duration: u16,
    pub qty: [u16; 3],
    pub flags: u32, // bit0=identified, bit1=not stealable, bit2=stolen, bit3=undroppable
}

/// A weapon-proficiency byte: low 3 bits = pip rank (0-5 in practice), the
/// rest is preserved as-is (dual-class "original/active class" bookkeeping).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProfByte(pub u8);

impl ProfByte {
    pub fn rank(&self) -> u8 {
        self.0 & 0b111
    }
    pub fn set_rank(&mut self, rank: u8) {
        self.0 = (self.0 & !0b111) | (rank & 0b111);
    }
}

fn swap_words(v: u32) -> u32 {
    ((v >> 16) & 0xFFFF) | ((v & 0xFFFF) << 16)
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreV1 {
    // --- preserved, not exposed for editing (round-tripped byte-for-byte) ---
    pub name_strref: i32,
    pub tooltip_strref: i32,
    pub flags: u32,
    pub xp_value: u32,
    pub status: u32,
    pub animation: u16,
    pub unused1: u16,
    pub colors: [u8; 7],
    pub effect_version: u8,
    pub portrait_small: ResRef,
    pub portrait_large: ResRef,
    pub fatigue: u8,
    pub intoxication: u8,
    pub tracking: u8,
    pub target_text: [u8; 32],
    pub sound_slots: [u8; 400],
    pub morale: u8,
    pub morale_break: u8,
    pub racial_enemy: u8,
    pub morale_recovery: u16,
    pub scripts: [ResRef; 5], // override, class, race, general, default
    pub general: u8,
    pub specifics: u8,
    pub object_specifiers: [u8; 5],
    pub identifier_global: u16,
    pub identifier_local: u16,
    pub script_name: [u8; 32],
    pub dialog: ResRef,

    // --- core attributes (editable) ---
    pub xp: u32,
    pub gold: u32,
    pub hp_current: i16,
    pub hp_max: i16,
    /// Per-creature reputation byte — not the same as party reputation
    /// (a separate GAM-level field); observed values in the wild go well
    /// past the nominal 0-20 in-game reputation display range, so this is
    /// treated as a plain unsigned byte rather than clamped to 0-20.
    pub reputation: u8,
    pub ac_natural: i16,
    pub ac_effective: i16,
    pub ac_mod_crushing: i16,
    pub ac_mod_missile: i16,
    pub ac_mod_piercing: i16,
    pub ac_mod_slashing: i16,
    pub thac0: i8,
    pub attacks_per_round: u8,
    pub save_death: i8,
    pub save_wand: i8,
    pub save_polymorph: i8,
    pub save_breath: i8,
    pub save_spell: i8,
    /// Signed: vulnerability (extra damage taken) is a real, legitimate
    /// negative resistance value, not just a display artifact.
    pub resist_fire: i8,
    pub resist_cold: i8,
    pub resist_electricity: i8,
    pub resist_acid: i8,
    pub resist_magic: i8,
    pub resist_magic_fire: i8,
    pub resist_magic_cold: i8,
    pub resist_slashing: i8,
    pub resist_crushing: i8,
    pub resist_piercing: i8,
    pub resist_missile: i8,
    pub hide_in_shadows: u8,
    pub detect_illusion: u8,
    pub set_traps: u8,
    pub lore: u8,
    pub open_locks: u8,
    pub move_silently: u8,
    pub find_traps: u8,
    pub pick_pockets: u8,
    /// Signed: curse effects can legitimately drive this negative.
    pub luck: i8,
    pub prof_large_sword: ProfByte,
    pub prof_small_sword: ProfByte,
    pub prof_bow: ProfByte,
    pub prof_spear: ProfByte,
    pub prof_blunt: ProfByte,
    pub prof_spiked: ProfByte,
    pub prof_axe: ProfByte,
    pub prof_missile: ProfByte,
    pub reserved_ee: [u8; 7],
    pub nightmare_mode: u8,
    pub translucency: u8,
    pub reputation_mod_killed: i8,
    pub reputation_mod_join: i8,
    pub reputation_mod_leave: i8,
    pub turn_undead_level: i8,
    pub level1: u8,
    pub level2: u8,
    pub level3: u8,
    pub sex: u8,
    pub str_score: u8,
    pub str_bonus: u8,
    pub int_score: u8,
    pub wis_score: u8,
    pub dex_score: u8,
    pub con_score: u8,
    pub cha_score: u8,
    /// Logical (word-unswapped) KIT.IDS value; 0 = No Kit.
    pub kit: u32,
    pub allegiance: u8,
    pub race: u8,
    pub class: u8,
    pub gender: u8,
    pub alignment: u8,

    // --- variable-length sections ---
    pub known_spells: Vec<KnownSpell>,
    pub mem_info: Vec<MemInfo>,
    pub memorized_spells: Vec<MemorizedSpell>,
    /// 38 equipped-item slots (indices into `items`, -1 = empty).
    pub item_slots: [i16; ITEM_SLOT_COUNT],
    pub selected_weapon_slot: i16,
    pub selected_weapon_ability: i16,
    pub items: Vec<InvItem>,
    /// Opaque effect records, each either 48 bytes (`effect_version==0`) or
    /// 264 bytes (`effect_version==1`); never edited, only round-tripped.
    pub effects_raw: Vec<u8>,
}

impl CreV1 {
    /// Parses a CRE V1.0 blob starting at `start` within `buf` (an absolute
    /// offset into the containing GAM file, or 0 for a standalone .cre).
    pub fn parse(buf: &[u8], start: usize) -> Result<CreV1, String> {
        let sig = read_text(buf, start, 4);
        let ver = read_text(buf, start + 4, 4);
        if sig != "CRE " || ver != "V1.0" {
            return Err(format!("unsupported CRE signature/version: {sig:?} {ver:?}"));
        }
        let b = start + 8; // base of readOther()'s offset space

        let effect_version = read_u8(buf, b + 43);

        let mut scripts = [ResRef::EMPTY; 5];
        for (i, s) in scripts.iter_mut().enumerate() {
            *s = read_resref(buf, b + 576 + i * 8);
        }

        let mut colors = [0u8; 7];
        colors.copy_from_slice(&buf[b + 36..b + 43]);

        let mut target_text = [0u8; 32];
        target_text.copy_from_slice(&buf[b + 124..b + 156]);

        let mut sound_slots = [0u8; 400];
        sound_slots.copy_from_slice(&buf[b + 156..b + 556]);

        let mut object_specifiers = [0u8; 5];
        object_specifiers.copy_from_slice(&buf[b + 622..b + 627]);

        let mut script_name = [0u8; 32];
        script_name.copy_from_slice(&buf[b + 632..b + 664]);

        let mut reserved_ee = [0u8; 7];
        reserved_ee.copy_from_slice(&buf[b + 110..b + 117]);

        let kit_raw = read_u32(buf, b + 572);

        let off_known_spells = read_u32(buf, b + 664) as usize;
        let num_known_spells = read_u32(buf, b + 668) as usize;
        let off_mem_info = read_u32(buf, b + 672) as usize;
        let num_mem_info = read_u32(buf, b + 676) as usize;
        let off_mem_spells = read_u32(buf, b + 680) as usize;
        let off_item_slots = read_u32(buf, b + 688) as usize;
        let off_items = read_u32(buf, b + 692) as usize;
        let num_items = read_u32(buf, b + 696) as usize;
        let off_effects = read_u32(buf, b + 700) as usize;
        let num_effects = read_u32(buf, b + 704) as usize;
        let dialog = read_resref(buf, b + 708);

        let mut known_spells = Vec::with_capacity(num_known_spells);
        for i in 0..num_known_spells {
            let o = start + off_known_spells + i * 12;
            known_spells.push(KnownSpell {
                spell: read_resref(buf, o),
                level: read_u16(buf, o + 8),
                kind: read_u16(buf, o + 10),
            });
        }

        let mut mem_info = Vec::with_capacity(num_mem_info);
        for i in 0..num_mem_info {
            let o = start + off_mem_info + i * 16;
            mem_info.push(MemInfo {
                level: read_u16(buf, o),
                memorizable_total: read_u16(buf, o + 2),
                memorizable_current: read_u16(buf, o + 4),
                kind: read_u16(buf, o + 6),
                table_index: read_u32(buf, o + 8),
                count: read_u32(buf, o + 12),
            });
        }

        // The flat memorized-spells table is sliced per MemInfo entry via
        // table_index/count, but in practice it's one contiguous array
        // covering the full range referenced by all MemInfo entries; we
        // just read it as one flat array sized to cover the furthest
        // (table_index+count) referenced, matching how the game writes it.
        let mem_spell_count = mem_info
            .iter()
            .map(|m| m.table_index + m.count)
            .max()
            .unwrap_or(0) as usize;
        let mut memorized_spells = Vec::with_capacity(mem_spell_count);
        for i in 0..mem_spell_count {
            let o = start + off_mem_spells + i * 12;
            memorized_spells.push(MemorizedSpell {
                spell: read_resref(buf, o),
                flags: read_u16(buf, o + 8),
            });
        }

        let mut items = Vec::with_capacity(num_items);
        for i in 0..num_items {
            let o = start + off_items + i * 20;
            items.push(InvItem {
                item: read_resref(buf, o),
                duration: read_u16(buf, o + 8),
                qty: [read_u16(buf, o + 10), read_u16(buf, o + 12), read_u16(buf, o + 14)],
                flags: read_u32(buf, o + 16),
            });
        }

        let mut item_slots = [0i16; ITEM_SLOT_COUNT];
        for (i, slot) in item_slots.iter_mut().enumerate() {
            *slot = read_i16(buf, start + off_item_slots + i * 2);
        }
        let selected_weapon_slot = read_i16(buf, start + off_item_slots + ITEM_SLOT_COUNT * 2);
        let selected_weapon_ability = read_i16(buf, start + off_item_slots + ITEM_SLOT_COUNT * 2 + 2);

        let entry_size = if effect_version == 1 { 264 } else { 48 };
        let effects_raw = buf[start + off_effects..start + off_effects + num_effects * entry_size].to_vec();

        Ok(CreV1 {
            name_strref: read_u32(buf, b) as i32,
            tooltip_strref: read_u32(buf, b + 4) as i32,
            flags: read_u32(buf, b + 8),
            xp_value: read_u32(buf, b + 12),
            status: read_u32(buf, b + 24),
            animation: read_u16(buf, b + 32),
            unused1: read_u16(buf, b + 34),
            colors,
            effect_version,
            portrait_small: read_resref(buf, b + 44),
            portrait_large: read_resref(buf, b + 52),
            fatigue: read_u8(buf, b + 99),
            intoxication: read_u8(buf, b + 100),
            tracking: read_u8(buf, b + 123),
            target_text,
            sound_slots,
            morale: read_u8(buf, b + 567),
            morale_break: read_u8(buf, b + 568),
            racial_enemy: read_u8(buf, b + 569),
            morale_recovery: read_u16(buf, b + 570),
            scripts,
            general: read_u8(buf, b + 617),
            specifics: read_u8(buf, b + 620),
            object_specifiers,
            identifier_global: read_u16(buf, b + 628),
            identifier_local: read_u16(buf, b + 630),
            script_name,
            dialog,

            xp: read_u32(buf, b + 16),
            gold: read_u32(buf, b + 20),
            hp_current: read_i16(buf, b + 28),
            hp_max: read_i16(buf, b + 30),
            reputation: read_u8(buf, b + 60),
            ac_natural: read_i16(buf, b + 62),
            ac_effective: read_i16(buf, b + 64),
            ac_mod_crushing: read_i16(buf, b + 66),
            ac_mod_missile: read_i16(buf, b + 68),
            ac_mod_piercing: read_i16(buf, b + 70),
            ac_mod_slashing: read_i16(buf, b + 72),
            thac0: read_i8(buf, b + 74),
            attacks_per_round: read_u8(buf, b + 75),
            save_death: read_i8(buf, b + 76),
            save_wand: read_i8(buf, b + 77),
            save_polymorph: read_i8(buf, b + 78),
            save_breath: read_i8(buf, b + 79),
            save_spell: read_i8(buf, b + 80),
            resist_fire: read_i8(buf, b + 81),
            resist_cold: read_i8(buf, b + 82),
            resist_electricity: read_i8(buf, b + 83),
            resist_acid: read_i8(buf, b + 84),
            resist_magic: read_i8(buf, b + 85),
            resist_magic_fire: read_i8(buf, b + 86),
            resist_magic_cold: read_i8(buf, b + 87),
            resist_slashing: read_i8(buf, b + 88),
            resist_crushing: read_i8(buf, b + 89),
            resist_piercing: read_i8(buf, b + 90),
            resist_missile: read_i8(buf, b + 91),
            hide_in_shadows: read_u8(buf, b + 61),
            detect_illusion: read_u8(buf, b + 92),
            set_traps: read_u8(buf, b + 93),
            lore: read_u8(buf, b + 94),
            open_locks: read_u8(buf, b + 95),
            move_silently: read_u8(buf, b + 96),
            find_traps: read_u8(buf, b + 97),
            pick_pockets: read_u8(buf, b + 98),
            luck: read_i8(buf, b + 101),
            prof_large_sword: ProfByte(read_u8(buf, b + 102)),
            prof_small_sword: ProfByte(read_u8(buf, b + 103)),
            prof_bow: ProfByte(read_u8(buf, b + 104)),
            prof_spear: ProfByte(read_u8(buf, b + 105)),
            prof_blunt: ProfByte(read_u8(buf, b + 106)),
            prof_spiked: ProfByte(read_u8(buf, b + 107)),
            prof_axe: ProfByte(read_u8(buf, b + 108)),
            prof_missile: ProfByte(read_u8(buf, b + 109)),
            reserved_ee,
            nightmare_mode: read_u8(buf, b + 117),
            translucency: read_u8(buf, b + 118),
            reputation_mod_killed: read_i8(buf, b + 119),
            reputation_mod_join: read_i8(buf, b + 120),
            reputation_mod_leave: read_i8(buf, b + 121),
            turn_undead_level: read_i8(buf, b + 122),
            level1: read_u8(buf, b + 556),
            level2: read_u8(buf, b + 557),
            level3: read_u8(buf, b + 558),
            sex: read_u8(buf, b + 559),
            str_score: read_u8(buf, b + 560),
            str_bonus: read_u8(buf, b + 561),
            int_score: read_u8(buf, b + 562),
            wis_score: read_u8(buf, b + 563),
            dex_score: read_u8(buf, b + 564),
            con_score: read_u8(buf, b + 565),
            cha_score: read_u8(buf, b + 566),
            kit: swap_words(kit_raw),
            allegiance: read_u8(buf, b + 616),
            race: read_u8(buf, b + 618),
            class: read_u8(buf, b + 619),
            gender: read_u8(buf, b + 621),
            alignment: read_u8(buf, b + 627),

            known_spells,
            mem_info,
            memorized_spells,
            item_slots,
            selected_weapon_slot,
            selected_weapon_ability,
            items,
            effects_raw,
        })
    }

    /// Rebuilds the full CRE blob (header + all sections) from scratch,
    /// recomputing every section offset/count field. The returned buffer's
    /// length is the new `GAM_NPC_CRE_SIZE`.
    pub fn serialize(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.text("CRE ", 4);
        w.text("V1.0", 4);
        debug_assert_eq!(w.len(), 8);

        w.i32(self.name_strref);
        w.i32(self.tooltip_strref);
        w.u32(self.flags);
        w.u32(self.xp_value);
        w.u32(self.xp);
        w.u32(self.gold);
        w.u32(self.status);
        w.i16(self.hp_current);
        w.i16(self.hp_max);
        w.u16(self.animation);
        w.u16(self.unused1);
        w.bytes(&self.colors);
        w.u8(self.effect_version);
        w.resref(self.portrait_small);
        w.resref(self.portrait_large);
        w.u8(self.reputation);
        w.u8(self.hide_in_shadows);
        w.i16(self.ac_natural);
        w.i16(self.ac_effective);
        w.i16(self.ac_mod_crushing);
        w.i16(self.ac_mod_missile);
        w.i16(self.ac_mod_piercing);
        w.i16(self.ac_mod_slashing);
        w.i8(self.thac0);
        w.u8(self.attacks_per_round);
        w.i8(self.save_death);
        w.i8(self.save_wand);
        w.i8(self.save_polymorph);
        w.i8(self.save_breath);
        w.i8(self.save_spell);
        w.i8(self.resist_fire);
        w.i8(self.resist_cold);
        w.i8(self.resist_electricity);
        w.i8(self.resist_acid);
        w.i8(self.resist_magic);
        w.i8(self.resist_magic_fire);
        w.i8(self.resist_magic_cold);
        w.i8(self.resist_slashing);
        w.i8(self.resist_crushing);
        w.i8(self.resist_piercing);
        w.i8(self.resist_missile);
        w.u8(self.detect_illusion);
        w.u8(self.set_traps);
        w.u8(self.lore);
        w.u8(self.open_locks);
        w.u8(self.move_silently);
        w.u8(self.find_traps);
        w.u8(self.pick_pockets);
        w.u8(self.fatigue);
        w.u8(self.intoxication);
        w.i8(self.luck);
        w.u8(self.prof_large_sword.0);
        w.u8(self.prof_small_sword.0);
        w.u8(self.prof_bow.0);
        w.u8(self.prof_spear.0);
        w.u8(self.prof_blunt.0);
        w.u8(self.prof_spiked.0);
        w.u8(self.prof_axe.0);
        w.u8(self.prof_missile.0);
        w.bytes(&self.reserved_ee);
        w.u8(self.nightmare_mode);
        w.u8(self.translucency);
        w.i8(self.reputation_mod_killed);
        w.i8(self.reputation_mod_join);
        w.i8(self.reputation_mod_leave);
        w.i8(self.turn_undead_level);
        w.u8(self.tracking);
        w.bytes(&self.target_text);
        w.bytes(&self.sound_slots);
        debug_assert_eq!(w.len(), 8 + 556);

        w.u8(self.level1);
        w.u8(self.level2);
        w.u8(self.level3);
        w.u8(self.sex);
        w.u8(self.str_score);
        w.u8(self.str_bonus);
        w.u8(self.int_score);
        w.u8(self.wis_score);
        w.u8(self.dex_score);
        w.u8(self.con_score);
        w.u8(self.cha_score);
        w.u8(self.morale);
        w.u8(self.morale_break);
        w.u8(self.racial_enemy);
        w.u16(self.morale_recovery);
        w.u32(swap_words(self.kit));
        for s in &self.scripts {
            w.resref(*s);
        }
        w.u8(self.allegiance);
        w.u8(self.general);
        w.u8(self.race);
        w.u8(self.class);
        w.u8(self.specifics);
        w.u8(self.gender);
        w.bytes(&self.object_specifiers);
        w.u8(self.alignment);
        w.u16(self.identifier_global);
        w.u16(self.identifier_local);
        w.bytes(&self.script_name);
        debug_assert_eq!(w.len(), HEADER_LEN - 52);

        // Section offset/count table: reserve space, back-patch after we
        // know where each variable-length section lands.
        let table_start = w.len();
        w.zeros(6 * 4); // known-spells off/count, mem-info off/count, mem-spells off/count
        w.zeros(4); // item-slots offset
        w.zeros(2 * 4); // items off/count
        w.zeros(2 * 4); // effects off/count
        w.resref(self.dialog);
        debug_assert_eq!(w.len(), HEADER_LEN);

        // For each variable-length section, an empty section is written as
        // offset 0 (matching the game engine's own convention observed in
        // real save files) rather than a valid-but-unused pointer — the
        // reader never dereferences it when the paired count is 0 either
        // way, but matching the convention keeps output closer to what the
        // game itself produces.
        let off_known_spells = if self.known_spells.is_empty() { 0 } else { w.len() as u32 };
        for k in &self.known_spells {
            w.resref(k.spell);
            w.u16(k.level);
            w.u16(k.kind);
        }

        let off_mem_info = if self.mem_info.is_empty() { 0 } else { w.len() as u32 };
        for m in &self.mem_info {
            w.u16(m.level);
            w.u16(m.memorizable_total);
            w.u16(m.memorizable_current);
            w.u16(m.kind);
            w.u32(m.table_index);
            w.u32(m.count);
        }

        let off_mem_spells = if self.memorized_spells.is_empty() { 0 } else { w.len() as u32 };
        for m in &self.memorized_spells {
            w.resref(m.spell);
            w.u16(m.flags);
            w.u16(0);
        }

        let off_item_slots = w.len() as u32;
        for slot in &self.item_slots {
            w.i16(*slot);
        }
        w.i16(self.selected_weapon_slot);
        w.i16(self.selected_weapon_ability);

        let off_items = if self.items.is_empty() { 0 } else { w.len() as u32 };
        for it in &self.items {
            w.resref(it.item);
            w.u16(it.duration);
            w.u16(it.qty[0]);
            w.u16(it.qty[1]);
            w.u16(it.qty[2]);
            w.u32(it.flags);
        }

        let off_effects = if self.effects_raw.is_empty() { 0 } else { w.len() as u32 };
        w.bytes(&self.effects_raw);
        let entry_size = if self.effect_version == 1 { 264 } else { 48 };
        let num_effects = (self.effects_raw.len() / entry_size) as u32;

        w.patch_u32(table_start, off_known_spells);
        w.patch_u32(table_start + 4, self.known_spells.len() as u32);
        w.patch_u32(table_start + 8, off_mem_info);
        w.patch_u32(table_start + 12, self.mem_info.len() as u32);
        w.patch_u32(table_start + 16, off_mem_spells);
        w.patch_u32(table_start + 20, self.memorized_spells.len() as u32);
        w.patch_u32(table_start + 24, off_item_slots);
        w.patch_u32(table_start + 28, off_items);
        w.patch_u32(table_start + 32, self.items.len() as u32);
        w.patch_u32(table_start + 36, off_effects);
        w.patch_u32(table_start + 40, num_effects);

        w.0
    }

    /// Removes `items[idx]`, clearing any equipped-slot reference to it
    /// (setting that slot to empty) and shifting every other slot index
    /// that pointed past it down by one, keeping `item_slots` consistent
    /// with the new (shorter) `items` array.
    pub fn remove_item(&mut self, idx: usize) {
        self.items.remove(idx);
        let idx = idx as i16;
        for slot in self.item_slots.iter_mut() {
            if *slot == idx {
                *slot = -1;
            } else if *slot > idx {
                *slot -= 1;
            }
        }
    }

    /// Appends a spell to the flat `memorized_spells` table within the
    /// block owned by `mem_info[level_idx]`, updating that entry's `count`
    /// and shifting every other level's `table_index` that falls after the
    /// insertion point, so the table stays contiguous and correctly sliced.
    pub fn add_memorized_spell(&mut self, level_idx: usize, spell: MemorizedSpell) {
        let insert_pos = (self.mem_info[level_idx].table_index + self.mem_info[level_idx].count) as usize;
        self.memorized_spells.insert(insert_pos, spell);
        self.mem_info[level_idx].count += 1;
        for (i, m) in self.mem_info.iter_mut().enumerate() {
            // >= (not >): a block starting exactly at insert_pos is the
            // common contiguous-layout case (e.g. the next spell level's
            // block starts right where this one ends) and must shift too.
            if i != level_idx && m.table_index as usize >= insert_pos {
                m.table_index += 1;
            }
        }
    }

    /// Removes the `local_idx`-th spell memorized at `mem_info[level_idx]`,
    /// with the same table-index bookkeeping as `add_memorized_spell`.
    pub fn remove_memorized_spell(&mut self, level_idx: usize, local_idx: usize) {
        let abs_pos = self.mem_info[level_idx].table_index as usize + local_idx;
        self.memorized_spells.remove(abs_pos);
        self.mem_info[level_idx].count -= 1;
        for (i, m) in self.mem_info.iter_mut().enumerate() {
            if i != level_idx && m.table_index as usize > abs_pos {
                m.table_index -= 1;
            }
        }
    }

    /// Adds a new (initially empty) spell-memorization level/class block,
    /// e.g. for a character gaining access to a spell level they didn't
    /// have before.
    pub fn add_mem_level(&mut self, level: u16, kind: u16, memorizable: u16) {
        self.mem_info.push(MemInfo {
            level,
            memorizable_total: memorizable,
            memorizable_current: memorizable,
            kind,
            table_index: self.memorized_spells.len() as u32,
            count: 0,
        });
    }

    /// Removes `mem_info[level_idx]` and every spell memorized in it,
    /// rebasing every other level's `table_index` that pointed past this
    /// block's start.
    pub fn remove_mem_level(&mut self, level_idx: usize) {
        let entry = self.mem_info.remove(level_idx);
        let start = entry.table_index as usize;
        let count = entry.count as usize;
        self.memorized_spells.drain(start..start + count);
        for m in self.mem_info.iter_mut() {
            if m.table_index as usize > start {
                m.table_index -= count as u32;
            }
        }
    }
}

#[cfg(test)]
/// Builds a CreV1 with every field set to a distinguishable value (and
/// non-empty variable-length sections), to catch offset mistakes that a
/// mostly-zeroed struct would silently paper over. Shared with the GAM
/// round-trip test.
pub(crate) fn sample_cre() -> CreV1 {
        CreV1 {
            name_strref: -1,
            tooltip_strref: 12345,
            flags: 0x0000_0201,
            xp_value: 100,
            status: 0x40,
            animation: 0x6001,
            unused1: 0,
            colors: [1, 2, 3, 4, 5, 6, 7],
            effect_version: 1,
            portrait_small: ResRef::from_str("PORTS"),
            portrait_large: ResRef::from_str("PORTL"),
            fatigue: 3,
            intoxication: 4,
            tracking: 5,
            target_text: [7u8; 32],
            sound_slots: [9u8; 400],
            morale: 10,
            morale_break: 11,
            racial_enemy: 12,
            morale_recovery: 300,
            scripts: [
                ResRef::from_str("SCROVR"),
                ResRef::from_str("SCCLS"),
                ResRef::from_str("SCRACE"),
                ResRef::from_str("SCGEN"),
                ResRef::from_str("SCDEF"),
            ],
            general: 13,
            specifics: 14,
            object_specifiers: [1, 2, 3, 4, 5],
            identifier_global: 111,
            identifier_local: 222,
            script_name: {
                let mut a = [0u8; 32];
                let s = b"MYCHARNAME";
                a[..s.len()].copy_from_slice(s);
                a
            },
            dialog: ResRef::from_str("MYDLG"),

            xp: 123456,
            gold: 500,
            hp_current: 42,
            hp_max: 55,
            reputation: 10,
            ac_natural: -2,
            ac_effective: -5,
            ac_mod_crushing: 1,
            ac_mod_missile: 2,
            ac_mod_piercing: 3,
            ac_mod_slashing: 4,
            thac0: 15,
            attacks_per_round: 2,
            save_death: 6,
            save_wand: 7,
            save_polymorph: 8,
            save_breath: 9,
            save_spell: 10,
            resist_fire: 1,
            resist_cold: 2,
            resist_electricity: 3,
            resist_acid: 4,
            resist_magic: 5,
            resist_magic_fire: 6,
            resist_magic_cold: 7,
            resist_slashing: 8,
            resist_crushing: 9,
            resist_piercing: 10,
            resist_missile: 11,
            hide_in_shadows: 20,
            detect_illusion: 21,
            set_traps: 22,
            lore: 23,
            open_locks: 24,
            move_silently: 25,
            find_traps: 26,
            pick_pockets: 27,
            luck: 2,
            prof_large_sword: ProfByte(0b11_010),
            prof_small_sword: ProfByte(2),
            prof_bow: ProfByte(3),
            prof_spear: ProfByte(0),
            prof_blunt: ProfByte(0),
            prof_spiked: ProfByte(0),
            prof_axe: ProfByte(1),
            prof_missile: ProfByte(0),
            reserved_ee: [0; 7],
            nightmare_mode: 0,
            translucency: 0,
            reputation_mod_killed: -1,
            reputation_mod_join: 1,
            reputation_mod_leave: -2,
            turn_undead_level: 0,
            level1: 12,
            level2: 0,
            level3: 0,
            sex: 1,
            str_score: 18,
            str_bonus: 76,
            int_score: 10,
            wis_score: 11,
            dex_score: 16,
            con_score: 14,
            cha_score: 9,
            kit: 0x4000,
            allegiance: 2,
            race: 1,
            class: 6,
            gender: 1,
            alignment: 5,

            known_spells: vec![
                KnownSpell { spell: ResRef::from_str("SPWI112"), level: 1, kind: 1 },
                KnownSpell { spell: ResRef::from_str("SPWI204"), level: 2, kind: 1 },
            ],
            mem_info: vec![MemInfo {
                level: 1,
                memorizable_total: 4,
                memorizable_current: 4,
                kind: 1,
                table_index: 0,
                count: 2,
            }],
            memorized_spells: vec![
                MemorizedSpell { spell: ResRef::from_str("SPWI112"), flags: 0b010 },
                MemorizedSpell { spell: ResRef::from_str("SPWI112"), flags: 0b010 },
            ],
            item_slots: {
                let mut s = [-1i16; ITEM_SLOT_COUNT];
                s[0] = 0;
                s[9] = 1;
                s
            },
            selected_weapon_slot: 9,
            selected_weapon_ability: 0,
            items: vec![
                InvItem { item: ResRef::from_str("HELM01"), duration: 0, qty: [1, 0, 0], flags: 1 },
                InvItem { item: ResRef::from_str("SW1H01"), duration: 0, qty: [1, 0, 0], flags: 1 },
            ],
            effects_raw: vec![0xABu8; 264 * 2],
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cre_round_trip() {
        let original = sample_cre();
        let bytes = original.serialize();
        let parsed = CreV1::parse(&bytes, 0).expect("parse should succeed");
        assert_eq!(original, parsed);
    }

    #[test]
    fn cre_round_trip_empty_sections() {
        let mut original = sample_cre();
        original.known_spells.clear();
        original.mem_info.clear();
        original.memorized_spells.clear();
        original.items.clear();
        original.effects_raw.clear();
        let bytes = original.serialize();
        let parsed = CreV1::parse(&bytes, 0).expect("parse should succeed");
        assert_eq!(original, parsed);
    }

    #[test]
    fn remove_item_clears_and_shifts_slots() {
        let mut cre = sample_cre();
        // sample_cre: item_slots[0] = 0 (HELM01), item_slots[9] = 1 (SW1H01)
        cre.remove_item(0);
        assert_eq!(cre.items.len(), 1);
        assert_eq!(cre.items[0].item.as_str(), "SW1H01");
        assert_eq!(cre.item_slots[0], -1, "slot referencing the removed item should clear");
        assert_eq!(cre.item_slots[9], 0, "slot referencing a later item should shift down");

        let bytes = cre.serialize();
        let parsed = CreV1::parse(&bytes, 0).expect("parse should succeed");
        assert_eq!(cre, parsed);
    }

    #[test]
    fn memorized_spell_add_remove_keeps_other_levels_consistent() {
        let mut cre = sample_cre();
        // Add a second spell-level block after the existing one (level 1,
        // table_index 0, count 2) so we can verify it shifts correctly.
        cre.mem_info.push(MemInfo {
            level: 2,
            memorizable_total: 2,
            memorizable_current: 2,
            kind: 1,
            table_index: 2,
            count: 1,
        });
        cre.memorized_spells.push(MemorizedSpell { spell: ResRef::from_str("SPWI204"), flags: 0b010 });

        // Add a spell to level 1 (index 0): should insert at absolute
        // position 2 (end of level 1's block) and push level 2's
        // table_index from 2 to 3.
        cre.add_memorized_spell(0, MemorizedSpell { spell: ResRef::from_str("SPWI113"), flags: 0b010 });
        assert_eq!(cre.mem_info[0].count, 3);
        assert_eq!(cre.mem_info[1].table_index, 3);
        assert_eq!(cre.memorized_spells[2].spell.as_str(), "SPWI113");
        assert_eq!(cre.memorized_spells[3].spell.as_str(), "SPWI204");

        let bytes = cre.serialize();
        let parsed = CreV1::parse(&bytes, 0).expect("parse should succeed");
        assert_eq!(cre, parsed);

        // Remove the spell we just added from level 1: level 2's
        // table_index should drop back to 2.
        cre.remove_memorized_spell(0, 2);
        assert_eq!(cre.mem_info[0].count, 2);
        assert_eq!(cre.mem_info[1].table_index, 2);
        assert_eq!(cre.memorized_spells[2].spell.as_str(), "SPWI204");

        let bytes = cre.serialize();
        let parsed = CreV1::parse(&bytes, 0).expect("parse should succeed");
        assert_eq!(cre, parsed);
    }

    #[test]
    fn mem_level_add_remove() {
        let mut cre = sample_cre();
        // sample_cre already has one level (table_index=0, count=2).
        cre.add_mem_level(2, 1, 2);
        assert_eq!(cre.mem_info.len(), 2);
        assert_eq!(cre.mem_info[1].table_index, 2, "new level starts after the existing 2 memorized spells");

        cre.add_memorized_spell(1, MemorizedSpell { spell: ResRef::from_str("SPWI204"), flags: 0b010 });
        assert_eq!(cre.memorized_spells.len(), 3);
        assert_eq!(cre.mem_info[1].count, 1);

        let bytes = cre.serialize();
        let parsed = CreV1::parse(&bytes, 0).expect("parse should succeed");
        assert_eq!(cre, parsed);

        // Removing level 0 (the original 2-spell block) should drop those
        // 2 spells and rebase level 1's table_index from 2 down to 0.
        cre.remove_mem_level(0);
        assert_eq!(cre.mem_info.len(), 1);
        assert_eq!(cre.mem_info[0].table_index, 0);
        assert_eq!(cre.memorized_spells.len(), 1);
        assert_eq!(cre.memorized_spells[0].spell.as_str(), "SPWI204");

        let bytes = cre.serialize();
        let parsed = CreV1::parse(&bytes, 0).expect("parse should succeed");
        assert_eq!(cre, parsed);
    }
}
