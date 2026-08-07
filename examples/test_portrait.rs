//! Validates portrait BMP resolution + decoding against a real game
//! install: resolves a known portrait resref (or, if `--save <folder>` is
//! given, the first party member's actual small portrait resref from a
//! real save), decodes it, prints dimensions, and dumps a BMP to the
//! given path for visual inspection. Read-only (writes only the output
//! BMP, never touches game/save files).

use std::path::PathBuf;

use bg2_editor::gamedata::{portrait, GameData};

fn main() {
    let mut args = std::env::args().skip(1);
    let game_root = PathBuf::from(args.next().expect("usage: test_portrait <game-root> (<portrait-resref> | --save <save-folder>) [out.bmp]"));
    let mode = args.next().expect("usage: test_portrait <game-root> (<portrait-resref> | --save <save-folder>) [out.bmp]");

    let mut gd = GameData::load(&game_root).expect("load gamedata");

    let resref = if mode == "--save" {
        let save_folder = PathBuf::from(args.next().expect("--save requires a folder path"));
        // save_folder is a specific save-slot dir; its parent is the
        // "save" root that `portraits_dir` expects.
        if let Some(save_root) = save_folder.parent() {
            if let Some(dir) = bg2_editor::save_file::portraits_dir(save_root) {
                println!("using extra portraits dir: {}", dir.display());
                gd = gd.with_extra_portraits_dir(Some(dir));
            }
        }
        let gam = bg2_editor::save_file::load(&save_folder).expect("load save");
        let member = gam.party.first().expect("save has no party members");
        let cre = member.cre.as_ref().expect("first member has no embedded CRE");
        println!("first party member portrait_small = {:?}", cre.portrait_small.as_str());
        cre.portrait_small.as_str()
    } else {
        mode
    };
    let out = args.next().unwrap_or_else(|| "portrait_out.bmp".to_owned());

    let bytes = gd.portrait_bytes(&resref).unwrap_or_else(|| panic!("portrait {resref} not found"));
    println!("{resref}: {} bytes", bytes.len());

    let img = portrait::decode_bmp_to_color_image(&bytes).expect("decode BMP");
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
