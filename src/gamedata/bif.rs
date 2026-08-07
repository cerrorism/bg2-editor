//! `.bif` resource archives referenced by `chitin.key`. Supports plain
//! `BIFFV1  ` (the common case for modern EE installs — read directly via
//! seek, no decompression), whole-file-compressed `BIF V1.0`, and
//! block-compressed `BIFCV1.0` (older/CD-era formats, decompressed fully
//! into memory on open). Byte offsets verified directly against
//! `NearInfinity/src/org/infinity/resource/key/{BIFFReader,BIFReader,BIFCReader}.java`.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use flate2::read::ZlibDecoder;

use crate::format::primitives::read_u32;

struct FileEntry {
    offset: u32,
    size: u32,
}

enum Source {
    Plain(File),
    Decompressed(Vec<u8>),
}

pub struct BifArchive {
    source: Source,
    file_entries: Vec<FileEntry>,
}

impl BifArchive {
    pub fn open(path: &Path) -> Result<BifArchive, String> {
        let mut f = File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
        let mut sig = [0u8; 8];
        f.read_exact(&mut sig).map_err(|e| e.to_string())?;
        let sig_str = String::from_utf8_lossy(&sig).into_owned();

        match sig_str.as_str() {
            "BIFFV1  " => {
                let mut header = [0u8; 20];
                f.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
                f.read_exact(&mut header).map_err(|e| e.to_string())?;
                let num_files = read_u32(&header, 8) as usize;
                let ofs_files = read_u32(&header, 16) as u64;
                let mut table = vec![0u8; num_files * 16];
                f.seek(SeekFrom::Start(ofs_files)).map_err(|e| e.to_string())?;
                f.read_exact(&mut table).map_err(|e| e.to_string())?;
                let file_entries = parse_file_entries(&table, 0, num_files);
                Ok(BifArchive { source: Source::Plain(f), file_entries })
            }
            "BIF V1.0" => {
                let mut all = Vec::new();
                f.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
                f.read_to_end(&mut all).map_err(|e| e.to_string())?;
                let name_len = read_u32(&all, 8) as usize;
                let unc_size = read_u32(&all, 12 + name_len) as usize;
                let comp_start = 20 + name_len;
                let mut decoder = ZlibDecoder::new(&all[comp_start..]);
                let mut out = Vec::with_capacity(unc_size);
                decoder.read_to_end(&mut out).map_err(|e| format!("inflate {}: {e}", path.display()))?;
                Self::from_plain_bytes(out)
            }
            "BIFCV1.0" => {
                let mut all = Vec::new();
                f.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
                f.read_to_end(&mut all).map_err(|e| e.to_string())?;
                let unc_size = read_u32(&all, 8) as usize;
                let mut out = Vec::with_capacity(unc_size);
                let mut pos = 12usize;
                while out.len() < unc_size && pos + 8 <= all.len() {
                    let block_unc = read_u32(&all, pos) as usize;
                    let block_comp = read_u32(&all, pos + 4) as usize;
                    pos += 8;
                    let mut decoder = ZlibDecoder::new(&all[pos..pos + block_comp]);
                    let mut block_out = Vec::with_capacity(block_unc);
                    decoder.read_to_end(&mut block_out).map_err(|e| format!("inflate block in {}: {e}", path.display()))?;
                    out.extend_from_slice(&block_out);
                    pos += block_comp;
                }
                Self::from_plain_bytes(out)
            }
            other => Err(format!("{}: unrecognized BIF signature {other:?}", path.display())),
        }
    }

    fn from_plain_bytes(bytes: Vec<u8>) -> Result<BifArchive, String> {
        let num_files = read_u32(&bytes, 8) as usize;
        let ofs_files = read_u32(&bytes, 16) as usize;
        let file_entries = parse_file_entries(&bytes, ofs_files, num_files);
        Ok(BifArchive { source: Source::Decompressed(bytes), file_entries })
    }

    pub fn read_resource(&mut self, file_index: usize) -> Result<Vec<u8>, String> {
        let entry = self.file_entries.get(file_index).ok_or_else(|| format!("bad BIF file index {file_index}"))?;
        match &mut self.source {
            Source::Plain(f) => {
                let mut buf = vec![0u8; entry.size as usize];
                f.seek(SeekFrom::Start(entry.offset as u64)).map_err(|e| e.to_string())?;
                f.read_exact(&mut buf).map_err(|e| e.to_string())?;
                Ok(buf)
            }
            Source::Decompressed(bytes) => {
                let start = entry.offset as usize;
                let end = start + entry.size as usize;
                Ok(bytes.get(start..end).ok_or("resource range out of bounds")?.to_vec())
            }
        }
    }
}

/// File-entries table: 16 bytes each — locator(4, ignored, we index by
/// position instead) + offset(4) + size(4) + type(2) + padding(2).
fn parse_file_entries(buf: &[u8], base: usize, count: usize) -> Vec<FileEntry> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let o = base + i * 16;
        out.push(FileEntry { offset: read_u32(buf, o + 4), size: read_u32(buf, o + 8) });
    }
    out
}
