//! Validates BAM icon decoding against real game data: resolves a known
//! icon resref, decodes frame 0, prints dimensions, and dumps a BMP to
//! the given path for visual inspection. Read-only.

use std::path::PathBuf;

use bg2_editor::gamedata::{bam, GameData};

fn main() {
    let mut args = std::env::args().skip(1);
    let game_root = PathBuf::from(args.next().expect("usage: test_bam <game-root> <icon-resref> [out.bmp]"));
    let resref = args.next().expect("usage: test_bam <game-root> <icon-resref> [out.bmp]");
    let out = args.next().unwrap_or_else(|| "bam_out.bmp".to_owned());

    let gd = GameData::load(&game_root).expect("load gamedata");
    let bytes = gd.icon_bytes(&resref).unwrap_or_else(|| panic!("icon {resref} not found"));
    println!("{resref}: {} bytes, sig={:?}", bytes.len(), String::from_utf8_lossy(&bytes[..4.min(bytes.len())]));

    let img = bam::decode_bam_frame(&bytes, 0).expect("decode BAM");
    println!("decoded: {}x{} pixels", img.size[0], img.size[1]);

    let rgba: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
    image::save_buffer_with_format(
        &out,
        &rgba,
        img.size[0] as u32,
        img.size[1] as u32,
        image::ColorType::Rgba8,
        image::ImageFormat::Bmp,
    )
    .expect("write bmp");
    println!("wrote {out}");
}
