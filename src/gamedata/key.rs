//! `chitin.key` — the master resource index for a game installation. Byte
//! offsets verified directly against
//! `NearInfinity/src/org/infinity/resource/key/Keyfile.java` and
//! `BIFFEntry.java`/`BIFFResourceEntry.java`.

use std::collections::HashMap;

use crate::format::primitives::{read_resref, read_text, read_u16, read_u32, ResRef};

pub const TYPE_ITM: u16 = 0x3ED;
pub const TYPE_SPL: u16 = 0x3EE;
pub const TYPE_IDS: u16 = 0x3F0;
/// Tileset resources use a different locator bit layout than everything
/// else; not otherwise used by this editor, kept for `decode_locator`.
pub const TYPE_TIS: u16 = 0x3EB;

pub struct BiffEntry {
    /// Game-root-relative path, forward-slash normalized (e.g. `data/25Items.bif`).
    pub path: String,
}

pub struct ResourceEntry {
    pub name: ResRef,
    pub restype: u16,
    pub locator: u32,
}

pub struct KeyFile {
    pub biffs: Vec<BiffEntry>,
    resources: Vec<ResourceEntry>,
    /// (uppercased resref, restype) -> index into `resources`, built once
    /// for fast lookups (a real KEY file has 50k+ resource entries).
    index: HashMap<(String, u16), usize>,
}

impl KeyFile {
    pub fn parse(buf: &[u8]) -> Result<KeyFile, String> {
        let sig = read_text(buf, 0, 4);
        let ver = read_text(buf, 4, 4);
        if sig != "KEY " || ver.trim() != "V1" {
            return Err(format!("unsupported KEY signature/version: {sig:?} {ver:?}"));
        }
        let num_bif = read_u32(buf, 8) as usize;
        let num_res = read_u32(buf, 12) as usize;
        let ofs_bif = read_u32(buf, 16) as usize;
        let ofs_res = read_u32(buf, 20) as usize;

        let mut biffs = Vec::with_capacity(num_bif);
        for i in 0..num_bif {
            let o = ofs_bif + i * 12;
            let str_off = read_u32(buf, o + 4) as usize;
            let str_len = read_u16(buf, o + 8) as usize; // includes the NUL terminator
            let name_len = str_len.saturating_sub(1);
            let raw = &buf[str_off..str_off + name_len];
            let path = String::from_utf8_lossy(raw).replace('\\', "/");
            let path = path.trim_start_matches('/').to_string();
            biffs.push(BiffEntry { path });
        }

        let mut resources = Vec::with_capacity(num_res);
        let mut index = HashMap::with_capacity(num_res);
        for i in 0..num_res {
            let o = ofs_res + i * 14;
            let entry = ResourceEntry {
                name: read_resref(buf, o),
                restype: read_u16(buf, o + 8),
                locator: read_u32(buf, o + 10),
            };
            index.insert((entry.name.as_str().to_ascii_uppercase(), entry.restype), i);
            resources.push(entry);
        }

        Ok(KeyFile { biffs, resources, index })
    }

    pub fn find(&self, name: &str, restype: u16) -> Option<&ResourceEntry> {
        let idx = *self.index.get(&(name.to_ascii_uppercase(), restype))?;
        Some(&self.resources[idx])
    }

    /// All resource names of a given type, for building a full catalog
    /// (e.g. every ITM in the game).
    pub fn names_of_type(&self, restype: u16) -> impl Iterator<Item = &ResourceEntry> {
        self.resources.iter().filter(move |r| r.restype == restype)
    }
}

/// Splits a locator into (bif index, index within that bif's file-entry
/// or tileset-entry table). Which table applies depends on `restype`,
/// not on any flag bit in the locator itself.
pub fn decode_locator(locator: u32, restype: u16) -> (u32, u32) {
    let bif_index = (locator >> 20) & 0xFFF;
    let sub_index = if restype == TYPE_TIS { (locator >> 14) & 0x3F } else { locator & 0x3FFF };
    (bif_index, sub_index)
}
