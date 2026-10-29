use crate::color::Color;
use crate::geom::{Point, Rect, smoothstep};
use crate::image::{Image, over, scale};
use crate::raster::{Mask, capsule, convex_hull, fill_polygons, hash2, value_noise};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BrushKind {
    Pencil,
    Brush,
    Calligraphy45,
    Calligraphy135,
    Airbrush,
    Oil,
    Crayon,
    Marker,
    NaturalPencil,
    Watercolor,
    EraserRound,
    EraserSquare,
}

impl BrushKind {
    pub fn size_range(self) -> (u32, u32, u32) {
        match self {
            BrushKind::Pencil => (1, 1, 248),
            BrushKind::Brush => (3, 1, 248),
            BrushKind::Calligraphy45 | BrushKind::Calligraphy135 => (5, 1, 248),
            BrushKind::Airbrush => (8, 4, 496),
            BrushKind::Oil | BrushKind::Crayon | BrushKind::Marker | BrushKind::Watercolor => (30, 4, 496),
            BrushKind::NaturalPencil => (4, 1, 248),
            BrushKind::EraserRound | BrushKind::EraserSquare => (8, 1, 248),
        }
    }

    pub fn is_eraser(self) -> bool {
        matches!(self, BrushKind::EraserRound | BrushKind::EraserSquare)
    }

    fn base_alpha(self) -> f32 {
        match self {
            BrushKind::Marker => 0.55,
            BrushKind::Watercolor => 0.75,
            BrushKind::NaturalPencil => 0.85,
            _ => 1.0,
        }
    }

    fn depth_field(self) -> bool {
        matches!(self, BrushKind::Oil | BrushKind::Marker | BrushKind::Watercolor)
    }

    fn textured(self) -> bool {
        matches!(
            self,
            BrushKind::Oil | BrushKind::Crayon | BrushKind::Marker | BrushKind::NaturalPencil | BrushKind::Watercolor
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StrokeParams {
    pub kind: BrushKind,
    pub size: f32,
    pub color: Color,
    pub opacity: f32,
    pub recolor: Option<(Color, Color)>,
}

pub struct Stroke {
    pub params: StrokeParams,
    pub mask: Mask,
    last: Option<(Point, f32)>,
    travelled: f32,
    next_stamp: f32,
    seed: u32,
    rng: u64,
    pub dirty: Rect,
}

impl Stroke {
    pub fn new(params: StrokeParams, w: u32, h: u32, seed: u32) -> Self {
        Stroke {
            params,
            mask: Mask::new(w, h),
            last: None,
            travelled: 0.0,
            next_stamp: 0.0,
            seed,
            rng: 0x9e3779b97f4a7c15 ^ seed as u64,
            dirty: Rect::EMPTY,
        }
    }

    fn rand(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 33) as u32
    }

    pub fn add_point(&mut self, p: Point, pressure: f32) -> Rect {
        let prev = self.last;
        self.last = Some((p, pressure));
        let (a, pa) = prev.unwrap_or((p, pressure));
        let r = match self.params.kind {
            BrushKind::Airbrush => self.spray(p),
            k if k.depth_field() => self.depth(a, p, pa, pressure),
            k if k.textured() => self.stamps(a, p, pa, pressure, prev.is_none()),
            BrushKind::Calligraphy45 => self.nib(a, p, Point::new(-1.0, 1.0)),
            BrushKind::Calligraphy135 => self.nib(a, p, Point::new(1.0, 1.0)),
            BrushKind::EraserSquare => self.square(a, p),
            k => {
                let rad = self.params.size * 0.5;
                let ra = rad * pressure_curve(pa);
                let rb = rad * pressure_curve(pressure);
                let g = if k == BrushKind::Pencil { 0.0 } else { 1.0 };
                capsule(&mut self.mask, a, p, ra, rb, g)
            }
        };
        self.dirty = self.dirty.union(&r);
        r
    }

    pub fn tick(&mut self) -> Rect {
        match (self.params.kind, self.last) {
            (BrushKind::Airbrush, Some((p, _))) => {
                let r = self.spray(p);
                self.dirty = self.dirty.union(&r);
                r
            }
            _ => Rect::EMPTY,
        }
    }

    fn spray(&mut self, c: Point) -> Rect {
        let mut d = self.params.size.round() as i32;
        d = ((d + 1) & !1).max(4);
        let half = d / 2;
        let r2 = half * half;
        let (cx, cy) = (c.x.floor() as i32, c.y.floor() as i32);
        for _ in 0..10 {
            loop {
                let dx = (self.rand() % (d as u32 + 1)) as i32 - half;
                let dy = (self.rand() % (d as u32 + 1)) as i32 - half;
                if dx * dx + dy * dy <= r2 {
                    self.mask.set(cx + dx, cy + dy, 1.0);
                    break;
                }
            }
        }
        Rect::new(cx - half, cy - half, cx + half + 1, cy + half + 1)
    }

    fn nib(&mut self, a: Point, b: Point, diag: Point) -> Rect {
        let r = self.params.size * 0.5;
        let off = diag.normalized() * (1.42 * r * std::f32::consts::FRAC_1_SQRT_2);
        let ext = if (b - a).len() > 1e-3 { (b - a).normalized() * 0.75 } else { Point::new(0.0, 0.0) };
        let (a, b) = (a - ext, b + ext);
        let hull = convex_hull(vec![a - off, a + off, b - off, b + off]);
        let bounds = self.mask.bounds();
        let mask = &mut self.mask;
        fill_polygons(&[hull], bounds, |x, y, c| mask.max_at(x, y, c));
        Rect::from_points(a, b).inflate(r.ceil() as i32 + 2).intersect(&bounds)
    }

    fn square(&mut self, a: Point, b: Point) -> Rect {
        let r = self.params.size * 0.5;
        let mut pts = Vec::with_capacity(8);
        for p in [a, b] {
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                pts.push(p + Point::new(sx * r, sy * r));
            }
        }
        let hull = convex_hull(pts);
        let bounds = self.mask.bounds();
        let mask = &mut self.mask;
        fill_polygons(&[hull], bounds, |x, y, c| mask.max_at(x, y, if c > 0.5 { 1.0 } else { 0.0 }));
        Rect::from_points(a, b).inflate(r.ceil() as i32 + 2).intersect(&bounds)
    }

    fn depth(&mut self, a: Point, b: Point, pa: f32, pb: f32) -> Rect {
        let rad = self.params.size * 0.5;
        let ra = rad * pressure_curve(pa).max(0.2);
        let rb = rad * pressure_curve(pb).max(0.2);
        let bb = Rect::from_points(a, b).inflate(ra.max(rb).ceil() as i32 + 1).intersect(&self.mask.bounds());
        for y in bb.y0..bb.y1 {
            for x in bb.x0..bb.x1 {
                let p = Point::new(x as f32 + 0.5, y as f32 + 0.5);
                let (d, t) = crate::raster::seg_dist(p, a, b);
                let rr = ra + (rb - ra) * t;
                if d < rr {
                    self.mask.max_at(x, y, (1.0 - d / rr).max(1e-4));
                }
            }
        }
        bb
    }

    fn profile(&self, depth: f32, x: i32, y: i32) -> f32 {
        let r = self.params.size * 0.5;
        let edge_px = depth * r;
        let aa = edge_px.clamp(0.0, 1.0);
        let seed = self.seed;
        match self.params.kind {
            BrushKind::Marker => aa,
            BrushKind::Watercolor => {
                let rim = 1.0 - smoothstep(edge_px / (r * 0.35).max(2.0));
                let bleed = 0.85 + 0.15 * value_noise(x as f32 * 0.06, y as f32 * 0.06, seed ^ 0x55);
                aa * (0.55 + 0.4 * rim) * bleed
            }
            BrushKind::Oil => {
                let warp = value_noise(x as f32 * 0.015, y as f32 * 0.015, seed ^ 0x21) * 5.0;
                let bristle = value_noise(edge_px * 0.7 + warp, (x + y) as f32 * 0.02, seed ^ 0x33);
                let fine = value_noise(edge_px * 2.3, (x - y) as f32 * 0.01, seed ^ 0x44);
                let grain = hash2(x, y, seed ^ 0x99);
                aa * (0.62 + 0.25 * bristle + 0.08 * fine + 0.05 * grain).min(1.0)
            }
            _ => depth,
        }
    }

    fn stamps(&mut self, a: Point, b: Point, pa: f32, pb: f32, first: bool) -> Rect {
        let size = self.params.size;
        let spacing = (size * spacing_factor(self.params.kind)).max(1.0);
        let seg = (b - a).len();
        let dir = if seg > 1e-3 { (b - a).normalized() } else { Point::new(1.0, 0.0) };
        let mut dirty = Rect::EMPTY;
        if first {
            self.stamp(a, dir, pa);
            dirty = Rect::from_points(a, a).inflate(size.ceil() as i32);
        }
        let mut t = self.next_stamp - self.travelled;
        while t <= seg {
            if t >= 0.0 {
                let f = if seg > 0.0 { t / seg } else { 0.0 };
                let p = a.lerp(b, f);
                self.stamp(p, dir, pa + (pb - pa) * f);
                dirty = dirty.union(&Rect::from_points(p, p).inflate(size.ceil() as i32 + 1));
            }
            t += spacing;
        }
        self.travelled += seg;
        self.next_stamp = self.travelled + (t - seg);
        dirty.intersect(&self.mask.bounds())
    }

    fn stamp(&mut self, c: Point, dir: Point, pressure: f32) {
        let kind = self.params.kind;
        let r = self.params.size * 0.5 * pressure_curve(pressure).max(0.2);
        let nrm = dir.perp();
        let seed = self.seed;
        let bb = Rect::from_points(c, c).inflate(r.ceil() as i32 + 2).intersect(&self.mask.bounds());
        let wobble = hash2(c.x as i32, c.y as i32, seed);
        for y in bb.y0..bb.y1 {
            for x in bb.x0..bb.x1 {
                let p = Point::new(x as f32 + 0.5, y as f32 + 0.5);
                let d = p - c;
                let across = d.dot(nrm);
                let along = d.dot(dir);
                let dist = d.len();
                let cov = match kind {
                    BrushKind::Crayon => {
                        let edge = r * (0.82 + 0.18 * value_noise(along * 0.35 + wobble * 9.0, across * 0.35, seed));
                        if dist > edge {
                            0.0
                        } else {
                            let paper = value_noise(x as f32 * 0.9, y as f32 * 0.9, seed ^ 0x51) * 0.6
                                + hash2(x, y, seed ^ 0x77) * 0.4;
                            if paper > 0.42 { 1.0 } else { paper * 0.8 }
                        }
                    }
                    BrushKind::Oil => {
                        if dist > r {
                            0.0
                        } else {
                            let u = (across / r) * 0.5 + 0.5;
                            let bristle = value_noise(u * size_bristles(r), 0.5, seed ^ 0x33);
                            let streak = value_noise(x as f32 * 0.05 + u * 13.0, y as f32 * 0.05, seed ^ 0x99);
                            let edge = 1.0 - smoothstep((dist - r * 0.8) / (r * 0.2));
                            edge * (0.55 + 0.3 * bristle + 0.15 * streak)
                        }
                    }
                    BrushKind::Marker => {
                        let edge = r.min(1.5);
                        1.0 - smoothstep((dist - (r - edge)) / edge)
                    }
                    BrushKind::NaturalPencil => {
                        if dist > r {
                            0.0
                        } else {
                            let grain = value_noise(x as f32 * 1.3, y as f32 * 1.3, seed ^ 0x12) * 0.5
                                + hash2(x, y, seed ^ 0x34) * 0.5;
                            let soft = 1.0 - smoothstep((dist - r * 0.6) / (r * 0.4));
                            soft * (0.35 + 0.65 * grain)
                        }
                    }
                    BrushKind::Watercolor => {
                        if dist > r {
                            0.0
                        } else {
                            let ring = 0.55 + 0.45 * smoothstep((dist - r * 0.55) / (r * 0.4));
                            let edge = 1.0 - smoothstep((dist - r * 0.9) / (r * 0.1));
                            let bleed = 0.85 + 0.15 * value_noise(x as f32 * 0.08, y as f32 * 0.08, seed ^ 0x55);
                            ring * edge * bleed
                        }
                    }
                    _ => 0.0,
                };
                self.mask.max_at(x, y, cov);
            }
        }
    }

    pub fn composite(&self, base: &Image, out: &mut Image, rect: Rect) {
        let rect = rect.intersect(&base.bounds());
        let p = self.params;
        let alpha = p.opacity.clamp(0.0, 1.0) * p.kind.base_alpha();
        let src = p.color.premul();
        for y in rect.y0..rect.y1 {
            for x in rect.x0..rect.x1 {
                let i = base.idx(x, y);
                let mut m = self.mask.v[i];
                if m > 0.0 && p.kind.depth_field() {
                    m = self.profile(m, x, y);
                }
                let b = base.px[i];
                out.px[i] = if m <= 0.0 {
                    b
                } else if let Some((from, to)) = p.recolor {
                    recolor(b, from, to, m * alpha)
                } else if p.kind.is_eraser() {
                    scale(b, ((1.0 - m * alpha) * 255.0).round() as u8)
                } else {
                    over(b, scale(src, (m * alpha * 255.0).round() as u8))
                };
            }
        }
    }
}

fn recolor(b: [u8; 4], from: Color, to: Color, amount: f32) -> [u8; 4] {
    let c = Color::from_premul(b);
    let d = ((c.r as f32 - from.r as f32).powi(2) + (c.g as f32 - from.g as f32).powi(2) + (c.b as f32 - from.b as f32).powi(2))
        .sqrt()
        / 255.0;
    if c.a == 0 || d >= 0.01 {
        return b;
    }
    let target = to.with_alpha(c.a).premul();
    let t = amount.clamp(0.0, 1.0);
    [
        (b[0] as f32 + (target[0] as f32 - b[0] as f32) * t).round() as u8,
        (b[1] as f32 + (target[1] as f32 - b[1] as f32) * t).round() as u8,
        (b[2] as f32 + (target[2] as f32 - b[2] as f32) * t).round() as u8,
        b[3],
    ]
}

fn pressure_curve(p: f32) -> f32 {
    let s = smoothstep(p);
    if p >= 0.999 { 1.0 } else { s.max(0.05) }
}

fn spacing_factor(kind: BrushKind) -> f32 {
    match kind {
        BrushKind::Crayon => 0.12,
        BrushKind::Oil => 0.08,
        BrushKind::Marker => 0.1,
        BrushKind::NaturalPencil => 0.15,
        BrushKind::Watercolor => 0.1,
        _ => 0.25,
    }
}

fn size_bristles(r: f32) -> f32 {
    (r * 0.6).clamp(4.0, 40.0)
}
