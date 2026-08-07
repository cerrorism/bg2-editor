//! `.BAM` item/spell icon decoder (v1 + zlib-compressed `BAMC`),
//! single-frame-to-RGBA only — enough for a static inventory icon.
//! Verified directly against
//! `NearInfinity/src/org/infinity/resource/graphics/BamV1Decoder.java`
//! (frame/palette layout, RLE semantics) and `Compressor.java`
//! (`BAMC` header + zlib framing).

use std::io::Read;

use crate::format::primitives::{read_i32, read_u16, read_u32, read_u8};

/// Decodes frame `frame_index` of a BAM v1 (or `BAMC`-compressed v1)
/// resource to RGBA.
pub fn decode_bam_frame(bytes: &[u8], frame_index: usize) -> Result<egui::ColorImage, String> {
    let owned;
    let buf: &[u8] = if bytes.len() >= 4 && &bytes[0..4] == b"BAMC" {
        if bytes.len() < 12 {
            return Err("BAMC header too short".to_owned());
        }
        let decompressed_len = read_u32(bytes, 8) as usize;
        let mut decoder = flate2::read::ZlibDecoder::new(&bytes[12..]);
        let mut out = Vec::with_capacity(decompressed_len);
        decoder.read_to_end(&mut out).map_err(|e| format!("inflate BAMC: {e}"))?;
        owned = out;
        &owned
    } else {
        bytes
    };

    if buf.len() < 0x18 || &buf[0..4] != b"BAM " || &buf[4..8] != b"V1  " {
        return Err("not a BAM V1 resource".to_owned());
    }

    let frame_count = read_u16(buf, 0x08) as usize;
    if frame_index >= frame_count {
        return Err(format!("frame {frame_index} out of range (0..{frame_count})"));
    }
    let ofs_frames = read_i32(buf, 0x0C) as usize;
    let ofs_palette = read_i32(buf, 0x10) as usize;
    let rle_index = read_u8(buf, 0x0B);

    let frame_off = ofs_frames + frame_index * 12;
    if frame_off + 12 > buf.len() {
        return Err("frame table entry out of bounds".to_owned());
    }
    let width = read_u16(buf, frame_off) as usize;
    let height = read_u16(buf, frame_off + 2) as usize;
    let data_field = read_u32(buf, frame_off + 8);
    let compressed = data_field & 0x8000_0000 == 0;
    let ofs_data = (data_field & 0x7FFF_FFFF) as usize;

    if width == 0 || height == 0 {
        return Ok(egui::ColorImage::new([1, 1], egui::Color32::TRANSPARENT));
    }

    // Palette: 256 entries x 4 bytes (B,G,R,A). A stored 0 means "alpha
    // channel unused" (pre-EE convention) rather than "transparent" — NI
    // forces such entries opaque, so a plain 8bpp icon isn't invisible.
    if ofs_palette + 256 * 4 > buf.len() {
        return Err("palette out of bounds".to_owned());
    }
    let mut palette = [(0u8, 0u8, 0u8, 0u8); 256];
    for (i, entry) in palette.iter_mut().enumerate() {
        let o = ofs_palette + i * 4;
        let (b, g, r, mut a) = (buf[o], buf[o + 1], buf[o + 2], buf[o + 3]);
        if a == 0 {
            a = 255;
        }
        *entry = (r, g, b, a);
    }
    // Infinity Engine's "magic green" transparency convention: the first
    // palette entry whose RGB is pure green (0,255,0) is transparent;
    // failing that, palette index 0 is (matches BamV1Control::preparePalette).
    let transparent_idx = (0..256).find(|&i| palette[i] == (0, 255, 0, 255)).unwrap_or(0);
    palette[transparent_idx].3 = 0;

    // Decode the flat width*height stream of palette indices. RLE runs
    // are not confined to row boundaries (NI's decoder doesn't reset
    // `count` per row either), so this is decoded as one flat sequence.
    let want = width * height;
    let mut indices: Vec<u8> = Vec::with_capacity(want);
    let mut pos = ofs_data;
    while indices.len() < want {
        if pos >= buf.len() {
            return Err("ran out of pixel data before filling the frame".to_owned());
        }
        let pixel = buf[pos];
        pos += 1;
        if compressed && pixel == rle_index {
            if pos >= buf.len() {
                return Err("RLE run missing count byte".to_owned());
            }
            let run = buf[pos] as usize + 1;
            pos += 1;
            for _ in 0..run {
                if indices.len() >= want {
                    break;
                }
                indices.push(pixel);
            }
        } else {
            indices.push(pixel);
        }
    }

    let mut rgba = Vec::with_capacity(want * 4);
    for idx in indices {
        let (r, g, b, a) = palette[idx as usize];
        rgba.extend_from_slice(&[r, g, b, a]);
    }
    Ok(egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba))
}
