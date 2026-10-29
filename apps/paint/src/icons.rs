use eframe::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};
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
    AppLogo,
    AlignLeft,
    AlignCenter,
    AlignRight,
}

fn p(r: Rect, x: f32, y: f32) -> Pos2 {
    pos2(r.left() + r.width() * x / 20.0, r.top() + r.height() * y / 20.0)
}

fn line(pa: &Painter, s: Stroke, r: Rect, pts: &[(f32, f32)]) {
    let v: Vec<Pos2> = pts.iter().map(|&(x, y)| p(r, x, y)).collect();
    pa.add(Shape::line(v, s));
}

fn closed(pa: &Painter, s: Stroke, fill: Color32, r: Rect, pts: &[(f32, f32)]) {
    let v: Vec<Pos2> = pts.iter().map(|&(x, y)| p(r, x, y)).collect();
    pa.add(Shape::Path(PathShape { points: v, closed: true, fill, stroke: PathStroke::from(s) }));
}

fn dashed_rect(pa: &Painter, s: Stroke, r: Rect, inset: f32) {
    let rr = r.shrink(inset);
    let pts = [rr.left_top(), rr.right_top(), rr.right_bottom(), rr.left_bottom(), rr.left_top()];
    pa.extend(Shape::dashed_line(&pts, s, 2.5, 2.0));
}

pub fn draw(pa: &Painter, icon: Icon, rect: Rect, color: Color32) {
    let s = Stroke::new((rect.width() / 16.0).clamp(1.0, 1.6), color);
    let r = rect;
    let c = r.center();
    let u = r.width() / 20.0;
    match icon {
        Icon::RectSelect => dashed_rect(pa, s, r, 3.0 * u),
        Icon::FreeSelect => {
            let pts: Vec<Pos2> = (0..=24)
                .map(|i| {
                    let t = i as f32 / 24.0 * std::f32::consts::TAU;
                    let rad = 6.5 + 1.5 * (3.0 * t).sin();
                    pos2(c.x + rad * u * t.cos(), c.y + rad * u * 0.8 * t.sin())
                })
                .collect();
            pa.extend(Shape::dashed_line(&pts, s, 2.5, 2.0));
        }
        Icon::Crop => {
            line(pa, s, r, &[(5.0, 2.0), (5.0, 15.0), (18.0, 15.0)]);
            line(pa, s, r, &[(2.0, 5.0), (15.0, 5.0), (15.0, 18.0)]);
        }
        Icon::Resize => {
            pa.rect_stroke(Rect::from_min_max(p(r, 3.0, 3.0), p(r, 17.0, 17.0)), 2.0 * u, s, eframe::egui::StrokeKind::Middle);
            pa.rect_stroke(Rect::from_min_max(p(r, 3.0, 10.0), p(r, 10.0, 17.0)), 1.0 * u, s, eframe::egui::StrokeKind::Middle);
            line(pa, s, r, &[(11.0, 9.0), (15.0, 5.0)]);
            line(pa, s, r, &[(12.0, 5.0), (15.0, 5.0), (15.0, 8.0)]);
        }
        Icon::Rotate => {
            let pts: Vec<Pos2> = (0..=20)
                .map(|i| {
                    let t = -0.6 + i as f32 / 20.0 * 4.6;
                    pos2(c.x + 6.5 * u * t.cos(), c.y + 6.5 * u * t.sin())
                })
                .collect();
            pa.add(Shape::line(pts, s));
            line(pa, s, r, &[(13.5, 2.5), (15.5, 6.0), (12.0, 7.0)]);
        }
        Icon::Flip => {
            closed(pa, s, Color32::TRANSPARENT, r, &[(9.0, 4.0), (9.0, 16.0), (3.0, 16.0)]);
            closed(pa, s, color.gamma_multiply(0.35), r, &[(11.0, 4.0), (11.0, 16.0), (17.0, 16.0)]);
        }
        Icon::Pencil => {
            closed(pa, s, Color32::TRANSPARENT, r, &[(13.0, 3.0), (17.0, 7.0), (7.0, 17.0), (3.0, 17.0), (3.0, 13.0)]);
            line(pa, s, r, &[(11.0, 5.0), (15.0, 9.0)]);
        }
        Icon::Fill => {
            closed(pa, s, Color32::TRANSPARENT, r, &[(8.0, 3.0), (15.0, 10.0), (9.0, 16.0), (2.0, 9.0)]);
            line(pa, s, r, &[(2.0, 9.0), (15.0, 10.0)]);
            line(pa, s, r, &[(8.0, 3.0), (6.0, 1.5)]);
            pa.circle_filled(p(r, 17.0, 15.0), 1.8 * u, color);
            closed(pa, Stroke::NONE, color, r, &[(17.0, 11.0), (18.6, 14.6), (15.4, 14.6)]);
        }
        Icon::Text => {
            line(pa, s, r, &[(4.0, 17.0), (10.0, 3.0), (16.0, 17.0)]);
            line(pa, s, r, &[(6.5, 12.0), (13.5, 12.0)]);
        }
        Icon::Eraser => {
            closed(pa, s, Color32::TRANSPARENT, r, &[(11.0, 3.0), (18.0, 10.0), (11.0, 17.0), (6.0, 17.0), (2.0, 13.0)]);
            line(pa, s, r, &[(6.5, 7.5), (13.5, 14.5)]);
            line(pa, s, r, &[(11.0, 17.0), (18.0, 17.0)]);
        }
        Icon::Picker => {
            line(pa, s, r, &[(3.0, 17.0), (4.0, 13.0), (11.0, 6.0)]);
            line(pa, s, r, &[(3.0, 17.0), (7.0, 16.0), (14.0, 9.0)]);
            closed(pa, s, color, r, &[(10.0, 5.0), (13.0, 2.0), (18.0, 7.0), (15.0, 10.0)]);
        }
        Icon::Magnifier => {
            pa.circle_stroke(p(r, 8.5, 8.5), 5.5 * u, s);
            line(pa, s, r, &[(12.5, 12.5), (17.5, 17.5)]);
        }
        Icon::Brush => {
            closed(pa, s, Color32::TRANSPARENT, r, &[(17.0, 2.0), (18.0, 3.0), (10.0, 13.0), (7.0, 10.0)]);
            let b = CubicBezierShape::from_points_stroke(
                [p(r, 7.0, 11.0), p(r, 3.0, 11.0), p(r, 4.0, 16.0), p(r, 1.5, 18.0)],
                false,
                Color32::TRANSPARENT,
                s,
            );
            pa.add(b);
            line(pa, s, r, &[(1.5, 18.0), (6.0, 18.0), (9.0, 13.0)]);
        }
        Icon::Size => {
            for (i, w) in [1.0f32, 2.0, 3.5, 5.0].iter().enumerate() {
                let y = 4.0 + i as f32 * 4.0;
                pa.line_segment([p(r, 3.0, y), p(r, 17.0, y)], Stroke::new(w * u, color));
            }
        }
        Icon::Layers => {
            closed(pa, s, Color32::TRANSPARENT, r, &[(10.0, 2.5), (18.0, 6.5), (10.0, 10.5), (2.0, 6.5)]);
            line(pa, s, r, &[(2.0, 10.0), (10.0, 14.0), (18.0, 10.0)]);
            line(pa, s, r, &[(2.0, 13.5), (10.0, 17.5), (18.0, 13.5)]);
        }
        Icon::EditColors => {
            let n = 36;
            for i in 0..n {
                let t0 = i as f32 / n as f32 * std::f32::consts::TAU;
                let t1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
                let rgb = paint_core::Color::from_hsv(i as f32 / n as f32 * 360.0, 0.85, 1.0);
                let col = Color32::from_rgb(rgb.r, rgb.g, rgb.b);
                let rad = 8.5 * u;
                pa.add(Shape::convex_polygon(
                    vec![c, pos2(c.x + rad * t0.cos(), c.y + rad * t0.sin()), pos2(c.x + rad * t1.cos(), c.y + rad * t1.sin())],
                    col,
                    Stroke::NONE,
                ));
            }
            pa.circle_filled(c, 4.5 * u, Color32::WHITE);
            pa.line_segment([pos2(c.x - 2.5 * u, c.y), pos2(c.x + 2.5 * u, c.y)], Stroke::new(1.3, Color32::BLACK));
            pa.line_segment([pos2(c.x, c.y - 2.5 * u), pos2(c.x, c.y + 2.5 * u)], Stroke::new(1.3, Color32::BLACK));
        }
        Icon::Undo | Icon::Redo => {
            let flip = if icon == Icon::Redo { -1.0 } else { 1.0 };
            let fx = |x: f32| if flip < 0.0 { 20.0 - x } else { x };
            let pts: Vec<Pos2> = (0..=16)
                .map(|i| {
                    let t = -std::f32::consts::FRAC_PI_2 + i as f32 / 16.0 * std::f32::consts::PI;
                    p(r, fx(11.0 + 5.0 * t.cos()), 11.0 + 5.0 * t.sin())
                })
                .collect();
            pa.add(Shape::line(pts, s));
            line(pa, s, r, &[(fx(11.0), 16.0), (fx(6.0), 16.0)]);
            line(pa, s, r, &[(fx(7.5), 2.5), (fx(4.0), 6.0), (fx(7.5), 9.5)]);
            line(pa, s, r, &[(fx(4.0), 6.0), (fx(11.0), 6.0)]);
        }
        Icon::Settings => {
            let pts: Vec<Pos2> = (0..48)
                .map(|i| {
                    let t = i as f32 / 48.0 * std::f32::consts::TAU;
                    let rad = if (i / 3) % 2 == 0 { 7.5 } else { 6.0 };
                    pos2(c.x + rad * u * t.cos(), c.y + rad * u * t.sin())
                })
                .collect();
            pa.add(Shape::closed_line(pts, s));
            pa.circle_stroke(c, 2.5 * u, s);
        }
        Icon::ChevronDown => line(pa, s, r, &[(5.0, 8.0), (10.0, 13.0), (15.0, 8.0)]),
        Icon::ZoomIn | Icon::ZoomOut => {
            pa.circle_stroke(p(r, 8.5, 8.5), 5.5 * u, s);
            line(pa, s, r, &[(12.5, 12.5), (17.5, 17.5)]);
            line(pa, s, r, &[(6.0, 8.5), (11.0, 8.5)]);
            if icon == Icon::ZoomIn {
                line(pa, s, r, &[(8.5, 6.0), (8.5, 11.0)]);
            }
        }
        Icon::Fit => {
            line(pa, s, r, &[(3.0, 7.0), (3.0, 3.0), (7.0, 3.0)]);
            line(pa, s, r, &[(13.0, 3.0), (17.0, 3.0), (17.0, 7.0)]);
            line(pa, s, r, &[(17.0, 13.0), (17.0, 17.0), (13.0, 17.0)]);
            line(pa, s, r, &[(7.0, 17.0), (3.0, 17.0), (3.0, 13.0)]);
            pa.rect_stroke(Rect::from_min_max(p(r, 6.5, 6.5), p(r, 13.5, 13.5)), 1.0, s, eframe::egui::StrokeKind::Middle);
        }
        Icon::Cursor => {
            closed(pa, s, Color32::TRANSPARENT, r, &[(5.0, 3.0), (15.0, 11.0), (10.5, 11.5), (13.0, 17.0), (11.0, 18.0), (8.5, 12.5), (5.0, 15.5)]);
        }
        Icon::SelectionSize => dashed_rect(pa, s, r, 3.5 * u),
        Icon::CanvasSize => {
            pa.rect_stroke(Rect::from_min_max(p(r, 3.0, 4.0), p(r, 17.0, 16.0)), 1.5 * u, s, eframe::egui::StrokeKind::Middle);
            line(pa, s, r, &[(13.0, 13.0), (15.0, 13.0), (15.0, 11.0)]);
        }
        Icon::Save => {
            closed(pa, s, Color32::TRANSPARENT, r, &[(3.0, 3.0), (14.0, 3.0), (17.0, 6.0), (17.0, 17.0), (3.0, 17.0)]);
            line(pa, s, r, &[(6.0, 3.0), (6.0, 7.0), (13.0, 7.0), (13.0, 3.0)]);
            line(pa, s, r, &[(6.0, 17.0), (6.0, 11.0), (14.0, 11.0), (14.0, 17.0)]);
        }
        Icon::Eye | Icon::EyeOff => {
            let top: Vec<Pos2> = (0..=16)
                .map(|i| {
                    let t = i as f32 / 16.0;
                    pos2(r.left() + r.width() * (0.1 + 0.8 * t), c.y - 5.0 * u * (t * std::f32::consts::PI).sin())
                })
                .collect();
            let bot: Vec<Pos2> = top.iter().map(|q| pos2(q.x, 2.0 * c.y - q.y)).collect();
            pa.add(Shape::line(top, s));
            pa.add(Shape::line(bot, s));
            pa.circle_stroke(c, 2.5 * u, s);
            if icon == Icon::EyeOff {
                line(pa, s, r, &[(3.0, 3.0), (17.0, 17.0)]);
            }
        }
        Icon::Plus => {
            line(pa, s, r, &[(10.0, 4.0), (10.0, 16.0)]);
            line(pa, s, r, &[(4.0, 10.0), (16.0, 10.0)]);
        }
        Icon::Trash => {
            line(pa, s, r, &[(3.0, 5.0), (17.0, 5.0)]);
            line(pa, s, r, &[(8.0, 5.0), (8.0, 3.0), (12.0, 3.0), (12.0, 5.0)]);
            closed(pa, s, Color32::TRANSPARENT, r, &[(5.0, 5.0), (15.0, 5.0), (14.0, 17.0), (6.0, 17.0)]);
        }
        Icon::Duplicate => {
            pa.rect_stroke(Rect::from_min_max(p(r, 7.0, 7.0), p(r, 17.0, 17.0)), 1.5 * u, s, eframe::egui::StrokeKind::Middle);
            line(pa, s, r, &[(4.0, 13.0), (3.0, 13.0), (3.0, 3.0), (13.0, 3.0), (13.0, 4.0)]);
        }
        Icon::Merge => {
            line(pa, s, r, &[(10.0, 3.0), (10.0, 12.0)]);
            line(pa, s, r, &[(6.0, 8.0), (10.0, 12.0), (14.0, 8.0)]);
            line(pa, s, r, &[(3.0, 16.0), (17.0, 16.0)]);
        }
        Icon::ArrowUp => line(pa, s, r, &[(5.0, 12.0), (10.0, 7.0), (15.0, 12.0)]),
        Icon::ArrowDown => line(pa, s, r, &[(5.0, 8.0), (10.0, 13.0), (15.0, 8.0)]),
        Icon::ScrollUp => closed(pa, Stroke::NONE, color, r, &[(10.0, 6.0), (15.0, 13.0), (5.0, 13.0)]),
        Icon::ScrollDown => closed(pa, Stroke::NONE, color, r, &[(10.0, 14.0), (15.0, 7.0), (5.0, 7.0)]),
        Icon::More => {
            for x in [5.0, 10.0, 15.0] {
                pa.circle_filled(p(r, x, 10.0), 1.2 * u, color);
            }
        }
        Icon::Outline => {
            pa.rect_stroke(Rect::from_min_max(p(r, 4.0, 4.0), p(r, 16.0, 16.0)), 1.5 * u, Stroke::new(2.0 * u, color), eframe::egui::StrokeKind::Middle);
        }
        Icon::FillShape => {
            pa.rect_filled(Rect::from_min_max(p(r, 4.0, 4.0), p(r, 16.0, 16.0)), 1.5 * u, color);
        }
        Icon::AlignLeft | Icon::AlignCenter | Icon::AlignRight => {
            for (i, w) in [14.0f32, 9.0, 14.0, 9.0].iter().enumerate() {
                let y = 4.0 + i as f32 * 4.0;
                let x0 = match icon {
                    Icon::AlignLeft => 3.0,
                    Icon::AlignCenter => 10.0 - w / 2.0,
                    _ => 17.0 - w,
                };
                line(pa, s, r, &[(x0, y), (x0 + w, y)]);
            }
        }
        Icon::AppLogo => {
            let pal = [
                Color32::from_rgb(0xF2, 0x50, 0x22),
                Color32::from_rgb(0x7F, 0xBA, 0x00),
                Color32::from_rgb(0x00, 0xA4, 0xEF),
                Color32::from_rgb(0xFF, 0xB9, 0x00),
            ];
            pa.circle_filled(c, 8.0 * u, Color32::from_rgb(0xF5, 0xF0, 0xE6));
            pa.circle_stroke(c, 8.0 * u, Stroke::new(1.0, Color32::from_rgb(0x9A, 0x8A, 0x70)));
            for (i, col) in pal.iter().enumerate() {
                let t = -2.2 + i as f32 * 0.9;
                pa.circle_filled(pos2(c.x + 4.5 * u * t.cos(), c.y + 4.5 * u * t.sin()), 1.8 * u, *col);
            }
            pa.circle_filled(pos2(c.x + 3.0 * u, c.y + 3.5 * u), 2.0 * u, Color32::from_rgb(0x20, 0x20, 0x20).gamma_multiply(0.15));
        }
    }
}

pub fn draw_shape(pa: &Painter, kind: ShapeKind, rect: Rect, color: Color32) {
    let s = Stroke::new(1.2, color);
    let r = rect.shrink(rect.width() * 0.12);
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
