//! `dialog.tlk` string table (strref -> localized text). Byte offsets
//! verified directly against
//! `NearInfinity/src/org/infinity/util/StringTable.java`.

use crate::format::primitives::{read_text, read_u32};

pub struct TlkFile {
    /// Text only; we don't need sound/volume/pitch for name display.
    texts: Vec<String>,
}

impl TlkFile {
    pub fn parse(buf: &[u8]) -> Result<TlkFile, String> {
        let sig = read_text(buf, 0, 4);
        let ver = read_text(buf, 4, 4);
        if sig != "TLK " || ver.trim() != "V1" {
            return Err(format!("unsupported TLK signature/version: {sig:?} {ver:?}"));
        }
        let num_entries = read_u32(buf, 10) as usize;
        let ofs_strings = read_u32(buf, 14) as usize;

        let mut texts = Vec::with_capacity(num_entries);
        for i in 0..num_entries {
            let o = 18 + i * 26;
            let ofs_string = read_u32(buf, o + 18) as usize;
            let len_string = read_u32(buf, o + 22) as usize;
            let start = ofs_strings + ofs_string;
            let text = buf
                .get(start..start + len_string)
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .unwrap_or_default();
            texts.push(text);
        }
        Ok(TlkFile { texts })
    }

    pub fn get(&self, strref: i32) -> Option<&str> {
        if strref < 0 {
            return None;
        }
        self.texts.get(strref as usize).map(|s| s.as_str())
    }
}
