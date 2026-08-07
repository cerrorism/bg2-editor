//! Minimal `.2DA` (whitespace-delimited text table) reader — just enough
//! to look up a row by column value and read another column as a strref,
//! which is how the game itself resolves real in-game class/kit/race
//! display names (NearInfinity's own generic struct editor doesn't do
//! this at all; it just shows the raw `.IDS` symbol name, e.g. "THIEF"
//! instead of "Thief"/"盗贼" — confirmed by reading `CreResource.java`,
//! which uses plain `IdsBitmap` for Class/Race/Gender/Alignment).
//!
//! Format (`IESDP`): line 1 = signature/version (ignored), line 2 =
//! default value for missing cells (ignored — this codebase only reads
//! cells that are actually present), line 3 = column headers (ignored —
//! looked up by fixed numeric index instead, since the header names
//! aren't needed once the layout is known), then one row per line:
//! `rowname col1 col2 ...`, whitespace-separated.

pub struct Table2da {
    rows: Vec<Vec<String>>,
}

impl Table2da {
    pub fn parse(text: &str) -> Table2da {
        let rows = text
            .lines()
            .skip(3)
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.split_whitespace().map(|s| s.to_owned()).collect())
            .collect();
        Table2da { rows }
    }

    /// The first row where `col_idx` (0 = the rowname itself, 1 = the
    /// first data column, ...) parses as the integer `value`.
    pub fn row_where(&self, col_idx: usize, value: i64) -> Option<&[String]> {
        self.rows.iter().map(|r| r.as_slice()).find(|r| r.get(col_idx).and_then(|c| c.parse::<i64>().ok()) == Some(value))
    }

    /// The first row whose rowname (column 0) case-insensitively matches
    /// `name` — used to join against an `.IDS` symbol (e.g. `KIT.IDS`'s
    /// `"BERSERKER"`), since some `.2DA` tables' own numeric columns use
    /// a different, table-local ID space than the corresponding `.IDS`
    /// file's real values.
    pub fn row_where_name(&self, name: &str) -> Option<&[String]> {
        self.rows.iter().map(|r| r.as_slice()).find(|r| r.first().is_some_and(|c| c.eq_ignore_ascii_case(name)))
    }
}
