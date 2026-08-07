//! `.ITM` item file: category, weapon-proficiency slot, ability-requirement
//! stats, and the primary combat ability's damage/speed/range. Scoped to
//! the BG2/BG1:EE layout specifically (114-byte header with KIT.IDS-style
//! 1-byte stat-requirement fields at offsets 40-49, 56-byte BG2/EE-packed
//! ability structs) — verified directly against
//! `NearInfinity/src/org/infinity/resource/itm/{ItmResource,Ability}.java`
//! and `resource/AbstractAbility.java`.

use crate::format::primitives::{read_i16, read_u16, read_u32, read_u8};

/// Index = raw `Category` byte value (ITM offset 28). Source, verified
/// directly: `ItemTypeBitmap.java` `CATEGORIES_ARRAY` (78 entries).
pub const CATEGORIES: [&str; 78] = [
    "Miscellaneous", "Amulets and necklaces", "Armor", "Belts and girdles", "Boots", "Arrows",
    "Bracers and gauntlets", "Headgear", "Keys", "Potions", "Rings", "Scrolls", "Shields", "Food",
    "Bullets", "Bows", "Daggers", "Maces", "Slings", "Small swords", "Large swords", "Hammers",
    "Morning stars", "Flails", "Darts", "Axes", "Quarterstaves", "Crossbows",
    "Hand-to-hand weapons", "Spears", "Halberds", "Bolts", "Cloaks and robes", "Gold pieces",
    "Gems", "Wands", "Containers", "Books", "Familiars", "Tattoos", "Lenses", "Bucklers",
    "Candles", "Child bodies", "Clubs", "Female bodies", "Keys (old)", "Large shields",
    "Male bodies", "Medium shields", "Notes", "Rods", "Skulls", "Small shields", "Spider bodies",
    "Telescopes", "Bottles", "Greatswords", "Bags", "Furs and pelts", "Leather armor",
    "Studded leather", "Chain mail", "Splint mail", "Plate mail", "Full plate", "Hide armor",
    "Robes", "Scale mail", "Bastard swords", "Scarves", "Rations", "Hats", "Gloves", "Eyeballs",
    "Earrings", "Teeth", "Bracelets",
];

pub const ATTACK_TYPES: [&str; 5] = ["None", "Melee", "Ranged", "Magical", "Launcher"];
pub const LAUNCHER_TYPES: [&str; 4] = ["None", "Bow", "Crossbow", "Sling"];
/// Index = raw `Damage type` value (ability offset +28). Source:
/// `AbstractAbility.java` `DMG_TYPE_ARRAY`.
pub const DAMAGE_TYPES: [&str; 10] = [
    "None", "Piercing", "Crushing", "Slashing", "Missile", "Fist", "Piercing/Crushing",
    "Piercing/Slashing", "Crushing/Slashing", "Blunt missile",
];

#[derive(Clone, Copy, Default)]
pub struct AbilitySummary {
    pub attack_type: u8,
    pub range: u16,
    pub launcher_required: u8,
    pub speed_factor: i8,
    pub dice_count: u8,
    pub dice_size: u8,
    pub damage_bonus: i16,
    pub damage_type: u16,
}

impl AbilitySummary {
    pub fn attack_type_label(&self) -> &'static str {
        ATTACK_TYPES.get(self.attack_type as usize).copied().unwrap_or("?")
    }
    pub fn damage_type_label(&self) -> &'static str {
        DAMAGE_TYPES.get(self.damage_type as usize).copied().unwrap_or("?")
    }
    /// e.g. "1d8+1", or "1d6" with no bonus, or "-" if there's no damage
    /// (a pure launcher, whose damage comes from its ammo).
    pub fn damage_string(&self) -> String {
        if self.dice_count == 0 && self.dice_size == 0 && self.damage_bonus == 0 {
            return "-".to_owned();
        }
        if self.damage_bonus == 0 {
            format!("{}d{}", self.dice_count, self.dice_size)
        } else if self.damage_bonus > 0 {
            format!("{}d{}+{}", self.dice_count, self.dice_size, self.damage_bonus)
        } else {
            format!("{}d{}{}", self.dice_count, self.dice_size, self.damage_bonus)
        }
    }
}

#[derive(Clone)]
pub struct ItemStats {
    pub category_id: u16,
    pub weapon_prof_id: u8,
    pub min_level: u16,
    pub min_strength: u16,
    pub min_strength_bonus: u8,
    pub min_dexterity: u8,
    pub min_constitution: u8,
    pub min_intelligence: u8,
    pub min_wisdom: u8,
    pub min_charisma: u16,
    pub price: u32,
    pub weight: u32,
    pub enchantment: u32,
    pub two_handed: bool,
    /// The first ability whose type is Melee/Ranged/Launcher (skips
    /// pure on-equip-effect "Magical" entries) — the practical "basic
    /// attack" stats for display, per general Infinity Engine convention
    /// (not something the file format itself designates canonically).
    pub ability: Option<AbilitySummary>,
}

impl ItemStats {
    pub fn category_label(&self) -> &'static str {
        CATEGORIES.get(self.category_id as usize).copied().unwrap_or("Unknown")
    }

    pub fn parse(buf: &[u8]) -> Result<ItemStats, String> {
        if buf.len() < 114 {
            return Err("ITM too short for a standard header".to_owned());
        }
        let flags = read_u32(buf, 24);
        let category_id = read_u16(buf, 28);
        let min_level = read_u16(buf, 36);
        let min_strength = read_u16(buf, 38); // 2 bytes even in the KIT.IDS branch; only 40+ shrinks to 1-byte fields
        let min_strength_bonus = read_u8(buf, 40);
        let min_intelligence = read_u8(buf, 42);
        let min_dexterity = read_u8(buf, 44);
        let min_wisdom = read_u8(buf, 46);
        let min_constitution = read_u8(buf, 48);
        let weapon_prof_id = read_u8(buf, 49);
        let min_charisma = read_u16(buf, 50);
        let price = read_u32(buf, 52);
        let weight = read_u32(buf, 76);
        let enchantment = read_u32(buf, 96);
        let off_abilities = read_u32(buf, 100) as usize;
        let num_abilities = read_u16(buf, 104) as usize;

        let mut ability = None;
        for i in 0..num_abilities {
            let o = off_abilities + i * 56;
            if o + 56 > buf.len() {
                break;
            }
            let attack_type = read_u8(buf, o);
            if !(1..=4).contains(&attack_type) {
                continue; // skip "None"/purely-magical-effect abilities
            }
            ability = Some(AbilitySummary {
                attack_type,
                range: read_u16(buf, o + 14),
                launcher_required: read_u8(buf, o + 16),
                speed_factor: read_i8_at(buf, o + 18),
                dice_count: read_u8(buf, o + 24),
                dice_size: read_u8(buf, o + 22),
                damage_bonus: read_i16(buf, o + 26),
                damage_type: read_u16(buf, o + 28),
            });
            break;
        }

        Ok(ItemStats {
            category_id,
            weapon_prof_id,
            min_level,
            min_strength,
            min_strength_bonus,
            min_dexterity,
            min_constitution,
            min_intelligence,
            min_wisdom,
            min_charisma,
            price,
            weight,
            enchantment,
            two_handed: flags & 0b100 != 0, // bit 2 = "Two-handed"
            ability,
        })
    }
}

fn read_i8_at(buf: &[u8], off: usize) -> i8 {
    buf[off] as i8
}
