//! Verifies that the system CJK font `fonts::setup` would pick actually
//! contains glyphs for a real Chinese character name from a real save,
//! using the same font-parsing crate (ab_glyph) egui/epaint uses
//! internally to rasterize glyphs — rather than just assuming a found
//! font file works.

use ab_glyph::Font as _;

fn windows_fonts_dir() -> Option<std::path::PathBuf> {
    let windir = std::env::var_os("WINDIR").or_else(|| std::env::var_os("SystemRoot"))?;
    Some(std::path::PathBuf::from(windir).join("Fonts"))
}

const CANDIDATES: &[&str] =
    &["msyh.ttc", "msyh.ttf", "simhei.ttf", "simsun.ttc", "msjh.ttc", "meiryo.ttc", "YuGothM.ttc", "malgun.ttf"];

fn main() {
    let font_dir = windows_fonts_dir().expect("WINDIR/SystemRoot should be set on Windows");
    println!("Fonts dir: {}", font_dir.display());

    let mut found = None;
    for name in CANDIDATES {
        let path = font_dir.join(name);
        if path.is_file() {
            println!("  candidate present: {name}");
            if found.is_none() {
                found = Some((name, path));
            }
        } else {
            println!("  candidate absent:  {name}");
        }
    }

    let (name, path) = found.expect("no candidate CJK font found — this is what fonts::setup would also hit");
    println!("\nUsing first available candidate: {name}");
    let bytes = std::fs::read(&path).expect("read font file");
    println!("Read {} bytes from {}", bytes.len(), path.display());

    // FontRef only reads face 0 of a collection by default, matching what
    // egui's FontData::from_owned + internal ab_glyph parsing will use.
    let font = ab_glyph::FontRef::try_from_slice(&bytes).expect("ab_glyph should parse this as a valid font/TTC face 0");

    // "阿尔德里克" — a real party member name from an actual save file.
    let test_string = "阿尔德里克 Test123";
    let mut all_ok = true;
    for ch in test_string.chars() {
        let id = font.glyph_id(ch);
        let has_glyph = id.0 != 0;
        println!("  {ch:?} -> glyph id {} {}", id.0, if has_glyph { "OK" } else { "MISSING (would render as tofu)" });
        if ch != ' ' && !has_glyph {
            all_ok = false;
        }
    }

    if all_ok {
        println!("\nALL CHARACTERS HAVE GLYPHS — this font should render the name correctly.");
    } else {
        println!("\nSOME CHARACTERS ARE MISSING GLYPHS — would still show as tofu boxes.");
        std::process::exit(1);
    }
}
