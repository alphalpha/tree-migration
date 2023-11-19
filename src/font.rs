use crate::util::Error;
use image::Rgb;
use rusttype::{point, PositionedGlyph, Scale};
use std::path::Path;

pub struct Font {
    pub font: rusttype::Font<'static>,
    pub scale: Scale,
    pub color: Rgb<u8>,
    pub pos: (i32, i32),
    pub background_color: Rgb<u8>,
}

impl Font {
    pub fn new(path: &Path, size: f32, background_color: Rgb<u8>) -> Result<Font, Error> {
        let data = std::fs::read(&path)
            .map_err(|_| Error::Custom(format!("Cannot read {}", path.display())))?;
        let font = rusttype::Font::try_from_vec(data)
            .ok_or(Error::Custom(format!("Cannot read {}", path.display())))?;
        Ok(Font {
            font: font,
            scale: Scale::uniform(size),
            color: Rgb([255, 255, 255]),
            pos: (0, 0),
            background_color: background_color,
        })
    }
}

pub fn text_width(scale: Scale, font: &rusttype::Font, text: &str) -> Option<u32> {
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);
    let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, offset).collect();
    if let Some(last_glyph) = glyphs.last() {
        let mut w = last_glyph.position().x as u32;
        if let Some(bb) = last_glyph.pixel_bounding_box() {
            w = w + bb.width() as u32;
        }
        return Some(w);
    }
    None
}
