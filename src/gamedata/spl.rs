//! `.SPL` spell file: level/school/type, unidentified+identified
//! description strrefs, icon resref, and the primary ability's
//! target/range/casting-speed/damage. Byte offsets verified directly
//! against `NearInfinity/src/org/infinity/resource/spl/{SplResource,
//! Ability}.java` (114-byte header, identical shape to ITM through
//! offset 96; 40-byte ability structs, vs ITM's 56-byte).

use crate::format::primitives::{read_i16, read_resref, read_u16, read_u32, read_u8, ResRef};
use crate::gamedata::itm::ATTACK_TYPES;
use crate::gamedata::GameData;

/// Index = raw `Spell type` value (SPL offset 28). Source:
/// `SplResource.java` `SPELL_TYPE_ARRAY`.
pub const SPELL_TYPES: [&str; 6] = ["Special", "Wizard", "Priest", "Psionic", "Innate", "Bard song"];

/// Index = raw `Primary type` (school) value (SPL offset 37). A
/// simplified fallback table (`PriTypeBitmap.java`'s own fallback when
/// `MSCHOOL.2DA`/`SCHOOL.IDS` aren't consulted) — good enough for
/// display without a full `.2DA` parser, which this codebase doesn't have.
pub const SCHOOLS: [&str; 10] =
    ["None", "Abjurer", "Conjurer", "Diviner", "Enchanter", "Illusionist", "Invoker", "Necromancer", "Transmuter", "Generalist"];

/// Index = raw `Secondary type` value (SPL offset 39). Fallback table
/// from `SecTypeBitmap.java` (`CATEGORY_ARRAY`), same simplification as
/// `SCHOOLS` above.
pub const SECONDARY_TYPES: [&str; 14] = [
    "None",
    "Spell protections",
    "Specific protections",
    "Illusionary protections",
    "Magic attack",
    "Divination attack",
    "Conjuration",
    "Combat protections",
    "Contingency",
    "Battleground",
    "Offensive damage",
    "Disabling",
    "Combination",
    "Non-combat",
];

/// Index = raw `Target` value (ability offset +12). Source:
/// `AbstractAbility.java` `TARGET_TYPE_ARRAY`.
pub const TARGET_TYPES: [&str; 8] =
    ["", "Living actor", "Inventory", "Dead actor", "Any point within range", "Caster", "", "Caster (keep spell, no animation)"];

#[derive(Clone, Copy, Default)]
pub struct SpellAbilitySummary {
    pub attack_type: u8,
    pub target: u8,
    pub num_targets: u8,
    pub range_feet: i16,
    pub min_level: u16,
    pub casting_speed: u16,
    // Note: unlike ITM, the SPL ability struct's dice_size/dice_count/
    // damage_bonus/damage_type fields are *not* read here — NearInfinity
    // itself labels them "(unused)" for spells (`SPL_ABIL_DICE_SIZE` etc.
    // are `AbstractAbility.ABILITY_DICE_SIZE + SUFFIX_UNUSED`); actual
    // spell damage is expressed via Effect blocks, not these leftover
    // ability-struct fields inherited from ITM's shared layout. Surfacing
    // them would show garbage numbers, not real damage.
}

impl SpellAbilitySummary {
    pub fn attack_type_label(&self) -> &'static str {
        ATTACK_TYPES.get(self.attack_type as usize).copied().unwrap_or("?")
    }
    pub fn target_label(&self) -> &'static str {
        TARGET_TYPES.get(self.target as usize).copied().unwrap_or("?")
    }
}

#[derive(Clone)]
pub struct SpellStats {
    pub spell_type: u16,
    pub spell_level: u32,
    pub primary_type: u8,
    pub secondary_type: u8,
    pub icon: ResRef,
    pub unidentified_desc_strref: i32,
    pub identified_desc_strref: i32,
    /// First ability with a Melee/Ranged/Magical/Launcher attack type —
    /// same "practical primary ability" convention as `itm::ItemStats`.
    pub ability: Option<SpellAbilitySummary>,
}

impl SpellStats {
    pub fn spell_type_label(&self) -> &'static str {
        SPELL_TYPES.get(self.spell_type as usize).copied().unwrap_or("Unknown")
    }
    pub fn school_label(&self) -> &'static str {
        SCHOOLS.get(self.primary_type as usize).copied().unwrap_or("Unknown")
    }
    pub fn secondary_type_label(&self) -> &'static str {
        SECONDARY_TYPES.get(self.secondary_type as usize).copied().unwrap_or("Unknown")
    }

    pub fn description(&self, gd: &GameData) -> Option<String> {
        gd.tlk_string(self.identified_desc_strref).or_else(|| gd.tlk_string(self.unidentified_desc_strref))
    }

    pub fn parse(buf: &[u8]) -> Result<SpellStats, String> {
        if buf.len() < 114 {
            return Err("SPL too short for a standard header".to_owned());
        }
        let spell_type = read_u16(buf, 28);
        let primary_type = read_u8(buf, 37);
        let secondary_type = read_u8(buf, 39);
        let spell_level = read_u32(buf, 52);
        let icon = read_resref(buf, 58);
        let unidentified_desc_strref = read_u32(buf, 80) as i32;
        let identified_desc_strref = read_u32(buf, 84) as i32;
        let off_abilities = read_u32(buf, 100) as usize;
        let num_abilities = read_u16(buf, 104) as usize;

        let mut ability = None;
        for i in 0..num_abilities {
            let o = off_abilities + i * 40;
            if o + 40 > buf.len() {
                break;
            }
            let attack_type = read_u8(buf, o);
            if !(1..=4).contains(&attack_type) {
                continue;
            }
            ability = Some(SpellAbilitySummary {
                attack_type,
                target: read_u8(buf, o + 12),
                num_targets: read_u8(buf, o + 13),
                range_feet: read_i16(buf, o + 14),
                min_level: read_u16(buf, o + 16),
                casting_speed: read_u16(buf, o + 18),
            });
            break;
        }

        Ok(SpellStats {
            spell_type,
            spell_level,
            primary_type,
            secondary_type,
            icon,
            unidentified_desc_strref,
            identified_desc_strref,
            ability,
        })
    }
}
