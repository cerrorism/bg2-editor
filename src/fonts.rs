//! egui's bundled default fonts (Hack + emoji) only cover Latin script.
//! Party member names and (for some localizations) item/spell names can
//! be CJK text, which would otherwise render as empty boxes. This loads
//! a system CJK font, if one can be found, as a fallback so those glyphs
//! actually show up — without bundling/licensing a font ourselves.

use std::path::PathBuf;

/// Common Windows CJK font files, most-preferred first. `.ttc` (font
/// collection) files are supported by egui's underlying font parser,
/// which uses the first face in the collection.
const CANDIDATES: &[&str] = &[
    // Simplified Chinese
    "msyh.ttc",
    "msyh.ttf",
    "simhei.ttf",
    "simsun.ttc",
    // Traditional Chinese
    "msjh.ttc",
    // Japanese
    "meiryo.ttc",
    "YuGothM.ttc",
    // Korean
    "malgun.ttf",
];

pub fn setup(ctx: &egui::Context) {
    let Some(font_dir) = windows_fonts_dir() else {
        return;
    };
    for name in CANDIDATES {
        let path = font_dir.join(name);
        if let Ok(bytes) = std::fs::read(&path) {
            install(ctx, "cjk", bytes);
            return;
        }
    }
}

fn windows_fonts_dir() -> Option<PathBuf> {
    let windir = std::env::var_os("WINDIR").or_else(|| std::env::var_os("SystemRoot"))?;
    Some(PathBuf::from(windir).join("Fonts"))
}

fn install(ctx: &egui::Context, key: &str, bytes: Vec<u8>) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(key.to_owned(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts.families.entry(family).or_default().push(key.to_owned());
    }
    ctx.set_fonts(fonts);
}
