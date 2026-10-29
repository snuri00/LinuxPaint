#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub const fn from_hex(v: u32) -> Self {
        Color::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }

    pub fn to_hex(self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32
    }

    pub fn to_argb(self) -> u32 {
        (self.a as u32) << 24 | self.to_hex()
    }

    pub fn from_argb(v: u32) -> Self {
        Color::rgba((v >> 16) as u8, (v >> 8) as u8, v as u8, (v >> 24) as u8)
    }

    pub fn premul(self) -> [u8; 4] {
        let a = self.a as u32;
        [mul8(self.r as u32, a), mul8(self.g as u32, a), mul8(self.b as u32, a), self.a]
    }

    pub fn from_premul(p: [u8; 4]) -> Self {
        let a = p[3] as u32;
        if a == 0 {
            return Color::TRANSPARENT;
        }
        let un = |c: u8| ((c as u32 * 255 + a / 2) / a).min(255) as u8;
        Color::rgba(un(p[0]), un(p[1]), un(p[2]), p[3])
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Color { a, ..self }
    }

    pub fn to_hsv(self) -> (f32, f32, f32) {
        let (r, g, b) = (self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let h = if d == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / d).rem_euclid(6.0))
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        let s = if max == 0.0 { 0.0 } else { d / max };
        (h, s, max)
    }

    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let c = v * s;
        let hp = (h.rem_euclid(360.0)) / 60.0;
        let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
        let (r, g, b) = match hp as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = v - c;
        let f = |t: f32| ((t + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        Color::rgb(f(r), f(g), f(b))
    }
}

#[inline]
pub fn mul8(a: u32, b: u32) -> u8 {
    let t = a * b + 128;
    ((t + (t >> 8)) >> 8) as u8
}

pub const BASIC_PALETTE: [u32; 20] = [
    0x000000, 0x7F7F7F, 0x880015, 0xED1C24, 0xFF7F27, 0xFFF200, 0x22B14C, 0x00A2E8, 0x3F48CC,
    0xA349A4, 0xFFFFFF, 0xC3C3C3, 0xB97A57, 0xFFAEC9, 0xFFC90E, 0xEFE4B0, 0xB5E61D, 0x99D9EA,
    0x7092BE, 0xC8BFE7,
];
