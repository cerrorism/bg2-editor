//! Little-endian binary read/write helpers and shared field types used by
//! the GAM and CRE parsers/serializers.
//!
//! Some helpers here (`ResRef::is_empty`, `read_i32`) aren't used yet but
//! are generic building blocks for the not-yet-built inventory/spellbook
//! UI and gamedata resolver.
#![allow(dead_code)]

/// Fixed 8-byte resource reference (filename without extension), as used
/// throughout the Infinity Engine's binary formats. Null-padded on disk.
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct ResRef(pub [u8; 8]);

impl ResRef {
    pub const EMPTY: ResRef = ResRef([0; 8]);

    pub fn from_bytes(buf: &[u8]) -> ResRef {
        let mut b = [0u8; 8];
        b.copy_from_slice(&buf[..8]);
        ResRef(b)
    }

    pub fn from_str(s: &str) -> ResRef {
        let mut b = [0u8; 8];
        for (i, c) in s.bytes().take(8).enumerate() {
            b[i] = c;
        }
        ResRef(b)
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    pub fn as_str(&self) -> String {
        let end = self.0.iter().position(|&b| b == 0).unwrap_or(8);
        String::from_utf8_lossy(&self.0[..end]).into_owned()
    }
}

impl std::fmt::Display for ResRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for ResRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResRef({:?})", self.as_str())
    }
}

pub fn read_u8(buf: &[u8], off: usize) -> u8 {
    buf[off]
}

pub fn read_i8(buf: &[u8], off: usize) -> i8 {
    buf[off] as i8
}

pub fn read_u16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off + 1]])
}

pub fn read_i16(buf: &[u8], off: usize) -> i16 {
    i16::from_le_bytes([buf[off], buf[off + 1]])
}

pub fn read_u32(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

pub fn read_i32(buf: &[u8], off: usize) -> i32 {
    i32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

pub fn read_resref(buf: &[u8], off: usize) -> ResRef {
    ResRef::from_bytes(&buf[off..off + 8])
}

pub fn read_text(buf: &[u8], off: usize, len: usize) -> String {
    let end = buf[off..off + len]
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(len);
    String::from_utf8_lossy(&buf[off..off + end]).into_owned()
}

/// A growable byte buffer with push helpers, used to serialize sections
/// before their final absolute offsets in the output file are known.
#[derive(Default, Clone)]
pub struct Writer(pub Vec<u8>);

impl Writer {
    pub fn new() -> Self {
        Writer(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }

    pub fn i8(&mut self, v: i8) {
        self.0.push(v as u8);
    }

    pub fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i16(&mut self, v: i16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    pub fn resref(&mut self, v: ResRef) {
        self.0.extend_from_slice(&v.0);
    }

    pub fn text(&mut self, s: &str, len: usize) {
        let mut b = vec![0u8; len];
        for (i, c) in s.bytes().take(len).enumerate() {
            b[i] = c;
        }
        self.0.extend_from_slice(&b);
    }

    pub fn bytes(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b);
    }

    pub fn zeros(&mut self, n: usize) {
        self.0.resize(self.0.len() + n, 0);
    }

    /// Overwrite a u32 already written at `at` (used to back-patch
    /// offset/count fields once section placement is known).
    pub fn patch_u32(&mut self, at: usize, v: u32) {
        self.0[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
}
