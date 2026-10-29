
use crate::geom::{Point, Rect, smoothstep};

#[derive(Clone, Debug)]
pub struct Mask {
    pub w: u32,
    pub h: u32,
    pub v: Vec<f32>,
}

impl Mask {
    pub fn new(w: u32, h: u32) -> Self {
        Mask { w, h, v: vec![0.0; (w * h) as usize] }
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.w as i32, self.h as i32)
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return 0.0;
        }
        self.v[y as usize * self.w as usize + x as usize]
    }

    #[inline]
    pub fn max_at(&mut self, x: i32, y: i32, c: f32) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 || c <= 0.0 {
            return;
        }
        let i = y as usize * self.w as usize + x as usize;
        if c > self.v[i] {
            self.v[i] = c.min(1.0);
        }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: f32) {
        if x >= 0 && y >= 0 && x < self.w as i32 && y < self.h as i32 {
            let i = y as usize * self.w as usize + x as usize;
            self.v[i] = c;
        }
    }

    pub fn clear(&mut self) {
        self.v.iter_mut().for_each(|v| *v = 0.0);
    }
}

#[inline]
pub fn seg_dist(p: Point, a: Point, b: Point) -> (f32, f32) {
    let ab = b - a;
    let l2 = ab.dot(ab);
    let t = if l2 < 1e-9 { 0.0 } else { ((p - a).dot(ab) / l2).clamp(0.0, 1.0) };
    ((p - (a + ab * t)).len(), t)
}

pub fn capsule(mask: &mut Mask, a: Point, b: Point, ra: f32, rb: f32, g: f32) -> Rect {
    let r = ra.max(rb);
    let bb = Rect::from_points(a, b).inflate(r.ceil() as i32 + 1).intersect(&mask.bounds());
    for y in bb.y0..bb.y1 {
        for x in bb.x0..bb.x1 {
            let p = Point::new(x as f32 + 0.5, y as f32 + 0.5);
            let (d, t) = seg_dist(p, a, b);
            let rr = ra + (rb - ra) * t;
            let c = if g <= 0.0 {
                if d <= rr.max(0.5) { 1.0 } else { 0.0 }
            } else {
                if d >= rr + 0.5 {
                    continue;
                }
                let g = g.min(rr.max(0.5));
                1.0 - smoothstep((d - (rr - g)) / g)
            };
            mask.max_at(x, y, c);
        }
    }
    bb
}

pub fn fill_polygons(polys: &[Vec<Point>], clip: Rect, mut f: impl FnMut(i32, i32, f32)) {
    const SS: usize = 4;
    let mut bb: Option<(f32, f32, f32, f32)> = None;
    for p in polys.iter().flatten() {
        bb = Some(match bb {
            None => (p.x, p.y, p.x, p.y),
            Some((a, b, c, d)) => (a.min(p.x), b.min(p.y), c.max(p.x), d.max(p.y)),
        });
    }
    let Some((minx, miny, maxx, maxy)) = bb else { return };
    let r = Rect::new(minx.floor() as i32, miny.floor() as i32, maxx.ceil() as i32 + 1, maxy.ceil() as i32 + 1)
        .intersect(&clip);
    if r.is_empty() {
        return;
    }
    let width = r.width() as usize;
    let mut row = vec![0.0f32; width + 1];
    let mut xs: Vec<(f32, i32)> = Vec::new();
    for y in r.y0..r.y1 {
        row.iter_mut().for_each(|v| *v = 0.0);
        for s in 0..SS {
            let sy = y as f32 + (s as f32 + 0.5) / SS as f32;
            xs.clear();
            for poly in polys {
                let n = poly.len();
                for i in 0..n {
                    let (p0, p1) = (poly[i], poly[(i + 1) % n]);
                    if (p0.y <= sy && p1.y > sy) || (p1.y <= sy && p0.y > sy) {
                        let t = (sy - p0.y) / (p1.y - p0.y);
                        xs.push((p0.x + (p1.x - p0.x) * t, if p1.y > p0.y { 1 } else { -1 }));
                    }
                }
            }
            xs.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut wind = 0;
            for k in 0..xs.len() {
                let prev = wind;
                wind += xs[k].1;
                if prev == 0 && wind != 0 {
                    let xa = xs[k].0;
                    let mut w2 = wind;
                    let mut j = k + 1;
                    while j < xs.len() {
                        w2 += xs[j].1;
                        if w2 == 0 {
                            break;
                        }
                        j += 1;
                    }
                    let xb = if j < xs.len() { xs[j].0 } else { xa };
                    add_span(&mut row, xa - r.x0 as f32, xb - r.x0 as f32, 1.0 / SS as f32);
                }
            }
        }
        for (i, &c) in row.iter().enumerate().take(width) {
            if c > 0.001 {
                f(r.x0 + i as i32, y, c.min(1.0));
            }
        }
    }
}

fn add_span(row: &mut [f32], xa: f32, xb: f32, w: f32) {
    let n = row.len() as f32 - 1.0;
    let xa = xa.clamp(0.0, n);
    let xb = xb.clamp(0.0, n);
    if xb <= xa {
        return;
    }
    let ia = xa.floor() as usize;
    let ib = xb.floor() as usize;
    if ia == ib {
        row[ia] += (xb - xa) * w;
        return;
    }
    row[ia] += (ia as f32 + 1.0 - xa) * w;
    for v in &mut row[ia + 1..ib] {
        *v += w;
    }
    if ib < row.len() {
        row[ib] += (xb - ib as f32) * w;
    }
}

pub fn convex_hull(mut pts: Vec<Point>) -> Vec<Point> {
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    pts.dedup_by(|a, b| (a.x - b.x).abs() < 1e-4 && (a.y - b.y).abs() < 1e-4);
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: Point, a: Point, b: Point| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut lower: Vec<Point> = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper: Vec<Point> = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

pub fn stroke_polyline(mask: &mut Mask, pts: &[Point], closed: bool, width: f32, aa: bool) -> Rect {
    let r = (width * 0.5).max(0.5);
    let g = if aa { 1.0 } else { 0.0 };
    let mut dirty = Rect::EMPTY;
    let n = pts.len();
    if n == 1 {
        return capsule(mask, pts[0], pts[0], r, r, g);
    }
    let segs = if closed { n } else { n - 1 };
    for i in 0..segs {
        let d = capsule(mask, pts[i], pts[(i + 1) % n], r, r, g);
        dirty = dirty.union(&d);
    }
    dirty
}

#[inline]
pub fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6b343) ^ (y as u32).wrapping_mul(0xd8163841) ^ seed.wrapping_mul(0xcb1ab31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1e995);
    h ^= h >> 15;
    (h & 0xffffff) as f32 / 16777215.0
}

pub fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - xi as f32, y - yi as f32);
    let (sx, sy) = (smoothstep(fx), smoothstep(fy));
    let a = hash2(xi, yi, seed);
    let b = hash2(xi + 1, yi, seed);
    let c = hash2(xi, yi + 1, seed);
    let d = hash2(xi + 1, yi + 1, seed);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}
