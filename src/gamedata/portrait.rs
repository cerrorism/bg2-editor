//! Character portraits — a plain, uncompressed Windows BMP (indexed or
//! truecolor), unlike item/spell icons which are the engine's own BAM
//! format. Decoded via the `image` crate rather than hand-rolled, since
//! BMP is a standard format with no Infinity-Engine-specific quirks.

pub fn decode_bmp_to_color_image(bytes: &[u8]) -> Result<egui::ColorImage, String> {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Bmp)
        .map_err(|e| format!("decode BMP: {e}"))?
        .to_rgba8();
    let (w, h) = img.dimensions();
    Ok(egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw()))
}
