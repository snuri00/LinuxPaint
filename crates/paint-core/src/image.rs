use crate::color::{Color, mul8};
use crate::geom::Rect;

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub w: u32,
    pub h: u32,
    pub px: Vec<[u8; 4]>,
}

impl Image {
    pub fn new(w: u32, h: u32) -> Self {
        Image { w, h, px: vec![[0; 4]; (w * h) as usize] }
    }

    pub fn filled(w: u32, h: u32, c: Color) -> Self {
        Image { w, h, px: vec![c.premul(); (w * h) as usize] }
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.w as i32, self.h as i32)
    }

    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        y as usize * self.w as usize + x as usize
    }

    pub fn get(&self, x: i32, y: i32) -> [u8; 4] {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return [0; 4];
        }
        self.px[self.idx(x, y)]
    }

    pub fn put(&mut self, x: i32, y: i32, p: [u8; 4]) {
        if x >= 0 && y >= 0 && x < self.w as i32 && y < self.h as i32 {
            let i = self.idx(x, y);
            self.px[i] = p;
        }
    }

    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, src: [u8; 4], cov: u8) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 || cov == 0 {
            return;
        }
        let i = self.idx(x, y);
        self.px[i] = over(self.px[i], scale(src, cov));
    }

    pub fn crop(&self, r: Rect) -> Image {
        let mut out = Image::new(r.width() as u32, r.height() as u32);
        for y in 0..r.height() {
            for x in 0..r.width() {
                let p = self.get(r.x0 + x, r.y0 + y);
                out.put(x, y, p);
            }
        }
        out
    }

    pub fn paste_copy(&mut self, src: &Image, dx: i32, dy: i32) {
        for y in 0..src.h as i32 {
            for x in 0..src.w as i32 {
                self.put(dx + x, dy + y, src.get(x, y));
            }
        }
    }

    pub fn paste_over(&mut self, src: &Image, dx: i32, dy: i32, opacity: u8) {
        for y in 0..src.h as i32 {
            for x in 0..src.w as i32 {
                self.blend(dx + x, dy + y, src.get(x, y), opacity);
            }
        }
    }

    pub fn from_rgba8(w: u32, h: u32, straight: &[u8]) -> Image {
        let px = straight
            .chunks_exact(4)
            .map(|c| Color::rgba(c[0], c[1], c[2], c[3]).premul())
            .collect();
        Image { w, h, px }
    }

    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.px.len() * 4);
        for &p in &self.px {
            let c = Color::from_premul(p);
            out.extend_from_slice(&[c.r, c.g, c.b, c.a]);
        }
        out
    }

    pub fn to_dynamic(&self) -> image::DynamicImage {
        image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(self.w, self.h, self.to_rgba8()).expect("size matches"),
        )
    }

    pub fn from_dynamic(img: &image::DynamicImage) -> Image {
        let rgba = img.to_rgba8();
        Image::from_rgba8(rgba.width(), rgba.height(), rgba.as_raw())
    }
}

#[inline]
pub fn scale(p: [u8; 4], s: u8) -> [u8; 4] {
    if s == 255 {
        return p;
    }
    let s = s as u32;
    [mul8(p[0] as u32, s), mul8(p[1] as u32, s), mul8(p[2] as u32, s), mul8(p[3] as u32, s)]
}

#[inline]
pub fn over(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
    if src[3] == 255 {
        return src;
    }
    if src[3] == 0 {
        return dst;
    }
    let ia = 255 - src[3] as u32;
    [
        src[0].saturating_add(mul8(dst[0] as u32, ia)),
        src[1].saturating_add(mul8(dst[1] as u32, ia)),
        src[2].saturating_add(mul8(dst[2] as u32, ia)),
        src[3].saturating_add(mul8(dst[3] as u32, ia)),
    ]
}
