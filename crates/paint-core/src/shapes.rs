use std::f32::consts::{PI, TAU};

use crate::geom::Point;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapeKind {
    Line,
    Curve,
    Oval,
    Rectangle,
    RoundedRectangle,
    Polygon,
    Triangle,
    RightTriangle,
    Diamond,
    Pentagon,
    Hexagon,
    RightArrow,
    LeftArrow,
    UpArrow,
    DownArrow,
    FourPointStar,
    FivePointStar,
    SixPointStar,
    RoundRectCallout,
    OvalCallout,
    CloudCallout,
    Heart,
    Lightning,
}

impl ShapeKind {
    pub const ALL: [ShapeKind; 23] = [
        ShapeKind::Line,
        ShapeKind::Curve,
        ShapeKind::Oval,
        ShapeKind::Rectangle,
        ShapeKind::RoundedRectangle,
        ShapeKind::Polygon,
        ShapeKind::Triangle,
        ShapeKind::RightTriangle,
        ShapeKind::Diamond,
        ShapeKind::Pentagon,
        ShapeKind::Hexagon,
        ShapeKind::RightArrow,
        ShapeKind::LeftArrow,
        ShapeKind::UpArrow,
        ShapeKind::DownArrow,
        ShapeKind::FourPointStar,
        ShapeKind::FivePointStar,
        ShapeKind::SixPointStar,
        ShapeKind::RoundRectCallout,
        ShapeKind::OvalCallout,
        ShapeKind::CloudCallout,
        ShapeKind::Heart,
        ShapeKind::Lightning,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ShapeKind::Line => "Line",
            ShapeKind::Curve => "Curve",
            ShapeKind::Oval => "Oval",
            ShapeKind::Rectangle => "Rectangle",
            ShapeKind::RoundedRectangle => "Rounded rectangle",
            ShapeKind::Polygon => "Polygon",
            ShapeKind::Triangle => "Triangle",
            ShapeKind::RightTriangle => "Right triangle",
            ShapeKind::Diamond => "Diamond",
            ShapeKind::Pentagon => "Pentagon",
            ShapeKind::Hexagon => "Hexagon",
            ShapeKind::RightArrow => "Right arrow",
            ShapeKind::LeftArrow => "Left arrow",
            ShapeKind::UpArrow => "Up arrow",
            ShapeKind::DownArrow => "Down arrow",
            ShapeKind::FourPointStar => "Four-point star",
            ShapeKind::FivePointStar => "Five-point star",
            ShapeKind::SixPointStar => "Six-point star",
            ShapeKind::RoundRectCallout => "Rounded rectangular callout",
            ShapeKind::OvalCallout => "Oval callout",
            ShapeKind::CloudCallout => "Cloud callout",
            ShapeKind::Heart => "Heart",
            ShapeKind::Lightning => "Lightning",
        }
    }

    pub fn is_box_shape(self) -> bool {
        !matches!(self, ShapeKind::Line | ShapeKind::Curve | ShapeKind::Polygon)
    }

    pub fn unit_paths(self) -> Vec<Vec<Point>> {
        let p = |x: f32, y: f32| Point::new(x, y);
        match self {
            ShapeKind::Line | ShapeKind::Curve | ShapeKind::Polygon => vec![],
            ShapeKind::Oval => vec![ellipse(0.5, 0.5, 0.5, 0.5, 96)],
            ShapeKind::Rectangle => vec![vec![p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0)]],
            ShapeKind::RoundedRectangle => vec![],
            ShapeKind::Triangle => vec![vec![p(0.5, 0.0), p(1.0, 1.0), p(0.0, 1.0)]],
            ShapeKind::RightTriangle => vec![vec![p(0.0, 0.0), p(1.0, 1.0), p(0.0, 1.0)]],
            ShapeKind::Diamond => vec![vec![p(0.5, 0.0), p(1.0, 0.5), p(0.5, 1.0), p(0.0, 0.5)]],
            ShapeKind::Pentagon => vec![fit(regular(5, 1.0, -PI / 2.0))],
            ShapeKind::Hexagon => vec![vec![p(0.25, 0.0), p(0.75, 0.0), p(1.0, 0.5), p(0.75, 1.0), p(0.25, 1.0), p(0.0, 0.5)]],
            ShapeKind::RightArrow => vec![arrow()],
            ShapeKind::LeftArrow => vec![arrow().into_iter().map(|q| p(1.0 - q.x, q.y)).collect()],
            ShapeKind::UpArrow => vec![arrow().into_iter().map(|q| p(q.y, 1.0 - q.x)).collect()],
            ShapeKind::DownArrow => vec![arrow().into_iter().map(|q| p(q.y, q.x)).collect()],
            ShapeKind::FourPointStar => vec![fit(star(4, 0.38))],
            ShapeKind::FivePointStar => vec![fit(star(5, 0.382))],
            ShapeKind::SixPointStar => vec![fit(star(6, 0.577))],
            ShapeKind::RoundRectCallout => vec![],
            ShapeKind::OvalCallout => vec![oval_callout()],
            ShapeKind::CloudCallout => cloud_callout(),
            ShapeKind::Heart => vec![heart()],
            ShapeKind::Lightning => vec![vec![
                p(0.38, 0.0),
                p(0.66, 0.0),
                p(0.52, 0.34),
                p(0.78, 0.34),
                p(0.30, 1.0),
                p(0.42, 0.52),
                p(0.18, 0.52),
            ]],
        }
    }

    pub fn paths(self, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<Vec<Point>> {
        let (w, h) = (x1 - x0, y1 - y0);
        match self {
            ShapeKind::RoundedRectangle => vec![round_rect(x0, y0, w, h, (w.abs().min(h.abs()) * 0.15).min(48.0), None)],
            ShapeKind::RoundRectCallout => {
                let bh = h * 0.78;
                vec![round_rect(x0, y0, w, bh, (w.abs().min(bh.abs()) * 0.15).min(40.0), Some((x0 + w * 0.2, x0 + w * 0.35, Point::new(x0 + w * 0.12, y0 + h))))]
            }
            _ => self
                .unit_paths()
                .into_iter()
                .map(|path| path.into_iter().map(|q| Point::new(x0 + q.x * w, y0 + q.y * h)).collect())
                .collect(),
        }
    }
}

fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32, n: usize) -> Vec<Point> {
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32 * TAU;
            Point::new(cx + rx * t.cos(), cy + ry * t.sin())
        })
        .collect()
}

fn regular(n: usize, r: f32, start: f32) -> Vec<Point> {
    (0..n)
        .map(|i| {
            let t = start + i as f32 / n as f32 * TAU;
            Point::new(r * t.cos(), r * t.sin())
        })
        .collect()
}

fn star(n: usize, inner: f32) -> Vec<Point> {
    (0..n * 2)
        .map(|i| {
            let r = if i % 2 == 0 { 1.0 } else { inner };
            let t = -PI / 2.0 + i as f32 / (n * 2) as f32 * TAU;
            Point::new(r * t.cos(), r * t.sin())
        })
        .collect()
}

fn fit(pts: Vec<Point>) -> Vec<Point> {
    let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for p in &pts {
        minx = minx.min(p.x);
        miny = miny.min(p.y);
        maxx = maxx.max(p.x);
        maxy = maxy.max(p.y);
    }
    pts.into_iter().map(|p| Point::new((p.x - minx) / (maxx - minx), (p.y - miny) / (maxy - miny))).collect()
}

fn arrow() -> Vec<Point> {
    let p = Point::new;
    vec![p(0.0, 0.25), p(0.5, 0.25), p(0.5, 0.0), p(1.0, 0.5), p(0.5, 1.0), p(0.5, 0.75), p(0.0, 0.75)]
}

fn heart() -> Vec<Point> {
    let pts: Vec<Point> = (0..128)
        .map(|i| {
            let t = i as f32 / 128.0 * TAU;
            let x = 16.0 * t.sin().powi(3);
            let y = -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
            Point::new(x, y)
        })
        .collect();
    fit(pts)
}

fn oval_callout() -> Vec<Point> {
    let (cx, cy, rx, ry) = (0.5, 0.4, 0.5, 0.4);
    let n = 96;
    let a0 = 1.95f32;
    let a1 = 2.35f32;
    let mut out = Vec::new();
    let mut tail_done = false;
    for i in 0..n {
        let t = i as f32 / n as f32 * TAU;
        if t > a0 && t < a1 {
            if !tail_done {
                out.push(Point::new(cx + rx * a0.cos(), cy + ry * a0.sin()));
                out.push(Point::new(0.1, 1.0));
                out.push(Point::new(cx + rx * a1.cos(), cy + ry * a1.sin()));
                tail_done = true;
            }
            continue;
        }
        out.push(Point::new(cx + rx * t.cos(), cy + ry * t.sin()));
    }
    out
}

fn cloud_callout() -> Vec<Vec<Point>> {
    let mut body = Vec::new();
    let bumps = 9;
    let (cx, cy, rx, ry) = (0.5, 0.38, 0.42, 0.3);
    for b in 0..bumps {
        let t0 = b as f32 / bumps as f32 * TAU;
        let t1 = (b + 1) as f32 / bumps as f32 * TAU;
        let c0 = Point::new(cx + rx * t0.cos(), cy + ry * t0.sin());
        let c1 = Point::new(cx + rx * t1.cos(), cy + ry * t1.sin());
        let mid = c0.lerp(c1, 0.5);
        let out = (mid - Point::new(cx, cy)).normalized();
        let bulge = (c1 - c0).len() * 0.55;
        for k in 0..8 {
            let s = k as f32 / 8.0;
            let q = c0.lerp(c1, s) + out * (bulge * (s * PI).sin());
            body.push(q);
        }
    }
    vec![
        body,
        ellipse(0.2, 0.8, 0.06, 0.045, 32),
        ellipse(0.12, 0.92, 0.035, 0.028, 24),
    ]
}

fn round_rect(x: f32, y: f32, w: f32, h: f32, r: f32, tail: Option<(f32, f32, Point)>) -> Vec<Point> {
    let (x0, x1) = (x.min(x + w), x.max(x + w));
    let (y0, y1) = (y.min(y + h), y.max(y + h));
    let r = r.min((x1 - x0) / 2.0).min((y1 - y0) / 2.0).max(0.0);
    let mut out = Vec::new();
    let corner = |out: &mut Vec<Point>, cx: f32, cy: f32, a0: f32| {
        for i in 0..=8 {
            let t = a0 + i as f32 / 8.0 * (PI / 2.0);
            out.push(Point::new(cx + r * t.cos(), cy + r * t.sin()));
        }
    };
    corner(&mut out, x1 - r, y0 + r, -PI / 2.0);
    corner(&mut out, x1 - r, y1 - r, 0.0);
    if let Some((ta, tb, tip)) = tail {
        out.push(Point::new(ta.max(tb), y1));
        out.push(tip);
        out.push(Point::new(ta.min(tb), y1));
    }
    corner(&mut out, x0 + r, y1 - r, PI / 2.0);
    corner(&mut out, x0 + r, y0 + r, PI);
    out
}

pub fn bezier(p0: Point, c1: Point, c2: Point, p3: Point, n: usize) -> Vec<Point> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            p0 * (u * u * u) + c1 * (3.0 * u * u * t) + c2 * (3.0 * u * t * t) + p3 * (t * t * t)
        })
        .collect()
}
