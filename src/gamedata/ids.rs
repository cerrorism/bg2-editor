//! `.IDS` text symbol tables (e.g. `CLASS.IDS`: `6 FIGHTER_MAGE`). Parsing
//! rules verified directly against
//! `NearInfinity/src/org/infinity/util/IdsMap.java`: split each line at
//! its first space-or-tab, value before (decimal or `0x`-prefixed hex),
//! symbol after (trimmed, with an optional trailing `//` comment
//! stripped). A line with no space/tab (e.g. a lone leading item-count)
//! is silently skipped, which is what makes the traditional count-only
//! first line a no-op rather than a special case.

use std::collections::HashMap;

pub struct IdsTable {
    /// First symbol seen for a given value (duplicate values keep the
    /// first symbol, matching NearInfinity's `IdsMapEntry`).
    by_value: HashMap<u32, String>,
    /// In-file order, for populating dropdowns.
    pub entries: Vec<(u32, String)>,
}

impl IdsTable {
    pub fn parse(text: &str) -> IdsTable {
        let mut by_value = HashMap::new();
        let mut entries = Vec::new();

        for raw_line in text.split(['\r', '\n']) {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            let Some(sep) = line.find(|c: char| c == ' ' || c == '\t') else {
                continue;
            };
            let istr = line[..sep].trim();
            let mut vstr = line[sep..].trim();
            if istr.eq_ignore_ascii_case("IDS") {
                continue;
            }
            if let Some(c) = vstr.find("//") {
                vstr = vstr[..c].trim();
            }
            if istr.is_empty() || vstr.is_empty() {
                continue;
            }
            let value: Option<u32> = if istr.len() > 2 && istr[..2].eq_ignore_ascii_case("0x") {
                u32::from_str_radix(&istr[2..], 16).ok()
            } else {
                istr.parse::<i64>().ok().map(|v| v as u32)
            };
            let Some(value) = value else {
                continue;
            };
            by_value.entry(value).or_insert_with(|| vstr.to_string());
            entries.push((value, vstr.to_string()));
        }

        IdsTable { by_value, entries }
    }

    pub fn name(&self, value: u32) -> Option<&str> {
        self.by_value.get(&value).map(|s| s.as_str())
    }
}
