use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};

use crate::color::Color;
use crate::image::Image;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub size_pt: f32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub align: Align,
    pub color: Color,
    pub background: Option<Color>,
}

pub fn layout_lines(font: &FontArc, text: &str, px: f32, max_w: f32) -> Vec<String> {
    let sf = font.as_scaled(PxScale::from(px));
    let width = |s: &str| -> f32 { s.chars().map(|c| sf.h_advance(font.glyph_id(c))).sum() };
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let mut cur = String::new();
        for word in para.split_inclusive(' ') {
            let cand = format!("{cur}{word}");
            if !cur.is_empty() && width(cand.trim_end()) > max_w {
                lines.push(std::mem::take(&mut cur));
                cur = word.to_string();
            } else {
                cur = cand;
            }
        }
        lines.push(cur);
    }
    lines
}

pub fn render(font: &FontArc, text: &str, style: &TextStyle, w: u32, h: u32) -> Image {
    let mut img = match style.background {
        Some(bg) => Image::filled(w, h, bg),
        None => Image::new(w, h),
    };
    let px = style.size_pt * 96.0 / 72.0;
    let scale = PxScale::from(px);
    let sf = font.as_scaled(scale);
    let line_h = sf.ascent() - sf.descent() + sf.line_gap();
    let src = style.color.premul();
    let slant = if style.italic { 0.2 } else { 0.0 };
    let mut y = sf.ascent();
    for line in layout_lines(font, text, px, w as f32 - 4.0) {
        let line_w: f32 = line.trim_end().chars().map(|c| sf.h_advance(font.glyph_id(c))).sum();
        let mut x = match style.align {
            Align::Left => 2.0,
            Align::Center => (w as f32 - line_w) / 2.0,
            Align::Right => w as f32 - line_w - 2.0,
        };
        let x_start = x;
        for ch in line.chars() {
            let gid = font.glyph_id(ch);
            let glyph = gid.with_scale_and_position(scale, point(x, y));
            if let Some(out) = font.outline_glyph(glyph) {
                let b = out.px_bounds();
                out.draw(|gx, gy, c| {
                    let yy = b.min.y as i32 + gy as i32;
                    let shift = ((y - yy as f32) * slant).round() as i32;
                    let xx = b.min.x as i32 + gx as i32 + shift;
                    let cov = (c.clamp(0.0, 1.0) * 255.0) as u8;
                    img.blend(xx, yy, src, cov);
                    if style.bold {
                        img.blend(xx + 1, yy, src, cov);
                    }
                });
            }
            x += sf.h_advance(gid);
        }
        let thick = (px / 14.0).max(1.0).round() as i32;
        let mut hline = |ly: f32| {
            for t in 0..thick {
                for xx in x_start.floor() as i32..(x_start + line_w).ceil() as i32 {
                    img.blend(xx, ly as i32 + t, src, 255);
                }
            }
        };
        if style.underline {
            hline(y + px * 0.12);
        }
        if style.strikethrough {
            hline(y - px * 0.3);
        }
        y += line_h;
    }
    img
}
