use std::f32::consts::{PI, TAU};

use eframe::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, pos2, vec2};
use eframe::epaint::{CubicBezierShape, PathShape, PathStroke};
use paint_core::shapes::ShapeKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    RectSelect,
    FreeSelect,
    Crop,
    Resize,
    Rotate,
    Flip,
    Pencil,
    Fill,
    Text,
    Eraser,
    Picker,
    Magnifier,
    Brush,
    Size,
    Layers,
    EditColors,
    Undo,
    Redo,
    Settings,
    ChevronDown,
    ZoomIn,
    ZoomOut,
    Fit,
    Cursor,
    SelectionSize,
    CanvasSize,
    Save,
    Eye,
    EyeOff,
    Plus,
    Trash,
    Duplicate,
    Merge,
    ArrowUp,
    ArrowDown,
    More,
    Outline,
    FillShape,
    ScrollUp,
    ScrollDown,
    AlignLeft,
    AlignCenter,
    AlignRight,
}

struct G<'a> {
    pa: &'a Painter,
    r: Rect,
    c: Color32,
    w: f32,
}

impl G<'_> {
    fn p(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.r.left() + self.r.width() * x / 16.0, self.r.top() + self.r.height() * y / 16.0)
    }
    fn u(&self) -> f32 {
        self.r.width() / 16.0
    }
    fn stroke(&self) -> Stroke {
        Stroke::new(self.w, self.c)
    }
    fn line(&self, pts: &[(f32, f32)]) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.p(x, y)).collect();
        self.pa.add(Shape::line(v, self.stroke()));
        for &(x, y) in [pts[0], pts[pts.len() - 1]].iter() {
            self.pa.circle_filled(self.p(x, y), self.w / 2.0, self.c);
        }
    }
    fn outline(&self, pts: &[(f32, f32)]) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.p(x, y)).collect();
        self.pa.add(Shape::Path(PathShape { points: v, closed: true, fill: Color32::TRANSPARENT, stroke: PathStroke::from(self.stroke()) }));
    }
    fn solid(&self, pts: &[(f32, f32)]) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.p(x, y)).collect();
        self.pa.add(Shape::Path(PathShape { points: v, closed: true, fill: self.c, stroke: PathStroke::from(Stroke::new(0.6, self.c)) }));
    }
    fn rrect(&self, x0: f32, y0: f32, x1: f32, y1: f32, rad: f32, fill: bool) {
        let rr = Rect::from_min_max(self.p(x0, y0), self.p(x1, y1));
        if fill {
            self.pa.rect_filled(rr, rad * self.u(), self.c);
        } else {
            self.pa.rect_stroke(rr, rad * self.u(), self.stroke(), StrokeKind::Middle);
        }
    }
    fn dot(&self, x: f32, y: f32, rad: f32) {
        self.pa.circle_filled(self.p(x, y), rad * self.u(), self.c);
    }
    fn ring(&self, x: f32, y: f32, rad: f32) {
        self.pa.circle_stroke(self.p(x, y), rad * self.u(), self.stroke());
    }
    fn arc(&self, cx: f32, cy: f32, rad: f32, a0: f32, a1: f32) -> Vec<Pos2> {
        (0..=24)
            .map(|i| {
                let t = a0 + (a1 - a0) * i as f32 / 24.0;
                self.p(cx + rad * t.cos(), cy + rad * t.sin())
            })
            .collect()
    }
    fn dashed_rect(&self, x0: f32, y0: f32, x1: f32, y1: f32) {
        let pts = [self.p(x0, y0), self.p(x1, y0), self.p(x1, y1), self.p(x0, y1), self.p(x0, y0)];
        self.pa.extend(Shape::dashed_line(&pts, self.stroke(), 2.2 * self.u(), 1.6 * self.u()));
    }
}

pub fn draw(pa: &Painter, icon: Icon, rect: Rect, color: Color32) {
    let g = G { pa, r: rect, c: color, w: (rect.width() / 16.0 * 1.5).clamp(1.2, 2.4) };
    match icon {
        Icon::RectSelect => g.dashed_rect(2.0, 3.0, 14.0, 13.0),
        Icon::FreeSelect => {
            let pts = g.arc(8.0, 6.5, 5.5, PI * 0.75, PI * 2.6);
            pa.extend(Shape::dashed_line(&pts, g.stroke(), 2.2 * g.u(), 1.6 * g.u()));
            g.line(&[(4.1, 10.4), (3.5, 13.5), (6.0, 12.0)]);
        }
        Icon::Crop => {
            g.line(&[(4.0, 1.5), (4.0, 12.0), (14.5, 12.0)]);
            g.line(&[(1.5, 4.0), (12.0, 4.0), (12.0, 14.5)]);
        }
        Icon::Resize => {
            g.rrect(1.5, 1.5, 14.5, 14.5, 2.5, false);
            g.line(&[(5.0, 11.0), (11.0, 5.0)]);
            g.line(&[(7.5, 5.0), (11.0, 5.0), (11.0, 8.5)]);
        }
        Icon::Rotate => {
            let pts = g.arc(8.0, 8.5, 5.5, -PI * 0.25, PI * 1.45);
            pa.add(Shape::line(pts, g.stroke()));
            g.solid(&[(9.5, 1.0), (14.5, 3.5), (10.0, 6.5)]);
        }
        Icon::Flip => {
            g.solid(&[(7.0, 2.0), (7.0, 14.0), (1.5, 14.0)]);
            g.outline(&[(9.0, 2.0), (9.0, 14.0), (14.5, 14.0)]);
        }
        Icon::Pencil => {
            g.solid(&[(11.0, 1.5), (14.5, 5.0), (5.5, 14.0), (2.0, 14.0), (2.0, 10.5)]);
        }
        Icon::Fill => {
            g.solid(&[(6.5, 2.5), (12.0, 8.0), (7.5, 12.5), (2.0, 7.0)]);
            g.line(&[(6.5, 2.5), (4.5, 0.8)]);
            g.dot(13.5, 12.5, 1.6);
            g.solid(&[(13.5, 9.0), (15.1, 12.2), (11.9, 12.2)]);
        }
        Icon::Text => {
            g.line(&[(2.5, 14.0), (8.0, 2.0), (13.5, 14.0)]);
            g.line(&[(4.6, 9.5), (11.4, 9.5)]);
        }
        Icon::Eraser => {
            g.solid(&[(9.5, 1.5), (15.0, 7.0), (10.0, 12.0), (4.5, 6.5)]);
            g.outline(&[(4.5, 6.5), (10.0, 12.0), (8.0, 14.0), (4.5, 14.0), (1.5, 11.0)]);
        }
        Icon::Picker => {
            g.solid(&[(9.0, 3.5), (11.5, 1.0), (15.0, 4.5), (12.5, 7.0)]);
            g.line(&[(10.0, 6.0), (3.5, 12.5), (2.5, 13.5)]);
            g.line(&[(8.0, 4.0), (12.0, 8.0)]);
        }
        Icon::Magnifier | Icon::ZoomIn | Icon::ZoomOut => {
            g.ring(6.8, 6.8, 4.8);
            g.line(&[(10.5, 10.5), (14.5, 14.5)]);
            if icon != Icon::Magnifier {
                g.line(&[(4.6, 6.8), (9.0, 6.8)]);
            }
            if icon == Icon::ZoomIn {
                g.line(&[(6.8, 4.6), (6.8, 9.0)]);
            }
        }
        Icon::Brush => {
            g.solid(&[(13.5, 1.0), (15.0, 2.5), (8.5, 10.0), (6.0, 7.5)]);
            pa.add(Shape::Path(PathShape {
                points: vec![g.p(5.5, 8.5), g.p(7.5, 10.5), g.p(6.5, 13.0), g.p(3.0, 14.5), g.p(1.0, 14.5), g.p(2.5, 12.5), g.p(3.0, 10.0)],
                closed: true,
                fill: color,
                stroke: PathStroke::from(Stroke::new(0.6, color)),
            }));
        }
        Icon::Size => {
            for (i, w) in [1.0f32, 1.8, 2.8, 4.0].iter().enumerate() {
                let y = 3.0 + i as f32 * 3.4;
                g.rrect(1.5, y - w / 2.0, 14.5, y + w / 2.0, w / 2.0, true);
            }
        }
        Icon::Layers => {
            g.solid(&[(8.0, 1.5), (15.0, 5.0), (8.0, 8.5), (1.0, 5.0)]);
            g.line(&[(1.5, 8.5), (8.0, 11.75), (14.5, 8.5)]);
            g.line(&[(1.5, 11.5), (8.0, 14.75), (14.5, 11.5)]);
        }
        Icon::EditColors => {
            let c = rect.center();
            let n = 36;
            let rad = rect.width() * 0.47;
            for i in 0..n {
                let t0 = i as f32 / n as f32 * TAU;
                let t1 = (i + 1) as f32 / n as f32 * TAU + 0.01;
                let rgb = paint_core::Color::from_hsv(i as f32 / n as f32 * 360.0, 0.8, 0.95);
                pa.add(Shape::convex_polygon(
                    vec![c, pos2(c.x + rad * t0.cos(), c.y + rad * t0.sin()), pos2(c.x + rad * t1.cos(), c.y + rad * t1.sin())],
                    Color32::from_rgb(rgb.r, rgb.g, rgb.b),
                    Stroke::NONE,
                ));
            }
            pa.circle_filled(c, rad * 0.45, Color32::WHITE);
            let s = Stroke::new(1.4, Color32::from_gray(40));
            pa.line_segment([c - vec2(rad * 0.25, 0.0), c + vec2(rad * 0.25, 0.0)], s);
            pa.line_segment([c - vec2(0.0, rad * 0.25), c + vec2(0.0, rad * 0.25)], s);
        }
        Icon::Undo | Icon::Redo => {
            let m = |x: f32| if icon == Icon::Redo { 16.0 - x } else { x };
            let mut pts = vec![g.p(m(4.0), 6.0)];
            for i in 0..=16 {
                let t = -PI / 2.0 + i as f32 / 16.0 * PI;
                let x = 9.0 + 4.5 * t.cos();
                pts.push(g.p(m(x), 10.5 + 4.5 * t.sin()));
            }
            pts.push(g.p(m(6.0), 15.0));
            pa.add(Shape::line(pts, g.stroke()));
            g.solid(&[(m(1.0), 6.0), (m(5.5), 2.0), (m(5.5), 10.0)]);
        }
        Icon::Settings => {
            for y in [3.5, 8.0, 12.5] {
                g.line(&[(2.5, y), (13.5, y)]);
            }
        }
        Icon::ChevronDown => g.line(&[(4.0, 6.0), (8.0, 10.0), (12.0, 6.0)]),
        Icon::Fit => {
            g.line(&[(1.5, 5.0), (1.5, 1.5), (5.0, 1.5)]);
            g.line(&[(11.0, 1.5), (14.5, 1.5), (14.5, 5.0)]);
            g.line(&[(14.5, 11.0), (14.5, 14.5), (11.0, 14.5)]);
            g.line(&[(5.0, 14.5), (1.5, 14.5), (1.5, 11.0)]);
            g.rrect(5.0, 5.0, 11.0, 11.0, 1.0, true);
        }
        Icon::Cursor => g.solid(&[(3.0, 1.0), (13.0, 9.5), (8.5, 10.0), (11.0, 14.5), (9.0, 15.5), (6.5, 11.0), (3.0, 14.0)]),
        Icon::SelectionSize => g.dashed_rect(2.0, 3.0, 14.0, 13.0),
        Icon::CanvasSize => {
            g.rrect(1.5, 2.5, 14.5, 13.5, 2.0, false);
            g.line(&[(10.5, 10.5), (11.5, 10.5)]);
        }
        Icon::Save => {
            g.solid(&[(2.0, 1.5), (11.5, 1.5), (14.5, 4.5), (14.5, 14.5), (2.0, 14.5)]);
            let bg = if color.r() > 128 { Color32::from_gray(40) } else { Color32::WHITE };
            pa.rect_filled(Rect::from_min_max(g.p(4.5, 1.5), g.p(10.5, 5.5)), 0.5, bg);
            pa.rect_filled(Rect::from_min_max(g.p(4.5, 9.0), g.p(12.0, 14.5)), 0.5, bg);
        }
        Icon::Eye | Icon::EyeOff => {
            let top = g.arc(8.0, 13.0, 8.5, -PI * 0.86, -PI * 0.14);
            let bot = g.arc(8.0, 3.0, 8.5, PI * 0.14, PI * 0.86);
            pa.add(Shape::line(top, g.stroke()));
            pa.add(Shape::line(bot, g.stroke()));
            g.dot(8.0, 8.0, 2.4);
            if icon == Icon::EyeOff {
                g.line(&[(2.0, 2.0), (14.0, 14.0)]);
            }
        }
        Icon::Plus => {
            g.line(&[(8.0, 2.5), (8.0, 13.5)]);
            g.line(&[(2.5, 8.0), (13.5, 8.0)]);
        }
        Icon::Trash => {
            g.line(&[(2.0, 4.0), (14.0, 4.0)]);
            g.line(&[(6.0, 4.0), (6.5, 1.8), (9.5, 1.8), (10.0, 4.0)]);
            g.solid(&[(3.5, 5.5), (12.5, 5.5), (11.8, 14.5), (4.2, 14.5)]);
        }
        Icon::Duplicate => {
            g.rrect(5.5, 5.5, 14.5, 14.5, 2.0, true);
            g.line(&[(3.5, 10.5), (1.5, 10.5), (1.5, 1.5), (10.5, 1.5), (10.5, 3.5)]);
        }
        Icon::Merge => {
            g.line(&[(8.0, 1.5), (8.0, 10.0)]);
            g.solid(&[(4.0, 7.0), (12.0, 7.0), (8.0, 11.5)]);
            g.line(&[(2.0, 14.0), (14.0, 14.0)]);
        }
        Icon::ArrowUp => g.line(&[(3.5, 10.5), (8.0, 6.0), (12.5, 10.5)]),
        Icon::ArrowDown => g.line(&[(3.5, 6.0), (8.0, 10.5), (12.5, 6.0)]),
        Icon::ScrollUp => g.solid(&[(8.0, 5.0), (12.0, 10.5), (4.0, 10.5)]),
        Icon::ScrollDown => g.solid(&[(8.0, 11.0), (12.0, 5.5), (4.0, 5.5)]),
        Icon::More => {
            for x in [3.5, 8.0, 12.5] {
                g.dot(x, 8.0, 1.4);
            }
        }
        Icon::Outline => g.rrect(2.5, 2.5, 13.5, 13.5, 2.5, false),
        Icon::FillShape => g.rrect(2.0, 2.0, 14.0, 14.0, 2.5, true),
        Icon::AlignLeft | Icon::AlignCenter | Icon::AlignRight => {
            for (i, w) in [12.0f32, 8.0, 12.0, 8.0].iter().enumerate() {
                let y = 3.0 + i as f32 * 3.3;
                let x0 = match icon {
                    Icon::AlignLeft => 2.0,
                    Icon::AlignCenter => 8.0 - w / 2.0,
                    _ => 14.0 - w,
                };
                g.line(&[(x0, y), (x0 + w, y)]);
            }
        }
    }
}

pub fn draw_shape(pa: &Painter, kind: ShapeKind, rect: Rect, color: Color32) {
    let s = Stroke::new(1.5, color);
    let r = rect.shrink(rect.width() * 0.1);
    match kind {
        ShapeKind::Line => {
            pa.line_segment([r.left_bottom(), r.right_top()], s);
        }
        ShapeKind::Curve => {
            pa.add(CubicBezierShape::from_points_stroke(
                [r.left_bottom(), pos2(r.left(), r.top()), pos2(r.right(), r.bottom()), r.right_top()],
                false,
                Color32::TRANSPARENT,
                s,
            ));
        }
        ShapeKind::Polygon => {
            let pts = [(0.1, 0.9), (0.2, 0.3), (0.55, 0.1), (0.9, 0.4), (0.7, 0.9)];
            let v: Vec<Pos2> = pts.iter().map(|&(x, y)| r.min + vec2(r.width() * x, r.height() * y)).collect();
            pa.add(Shape::closed_line(v, s));
        }
        k => {
            for path in k.paths(r.left(), r.top(), r.right(), r.bottom()) {
                let v: Vec<Pos2> = path.iter().map(|q| pos2(q.x, q.y)).collect();
                pa.add(Shape::closed_line(v, s));
            }
        }
    }
}
