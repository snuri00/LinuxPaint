use std::time::Instant;

use eframe::egui::{
    self, Align2, Color32, CursorIcon, FontId, PointerButton, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, pos2, vec2,
};
use paint_core::geom::{Point, Rect as IRect};
use paint_core::raster::{Mask, fill_polygons, stroke_polyline};
use paint_core::shapes::{ShapeKind, bezier};
use paint_core::stroke::{BrushKind, Stroke as PStroke, StrokeParams};
use paint_core::{Color, fill};

use crate::app::{Handle, OutlineStyle, PaintApp, Selection, ShapeSession, StrokeSession, TextSession, Tool, norm};

const MARGIN: f32 = 16.0;
const RULER: f32 = 22.0;
const HANDLE: f32 = 7.0;

pub struct View {
    pub origin: Pos2,
    pub zoom: f32,
}

impl View {
    pub fn to_canvas(&self, p: Pos2) -> Point {
        Point::new((p.x - self.origin.x) / self.zoom, (p.y - self.origin.y) / self.zoom)
    }
    pub fn to_screen(&self, p: Point) -> Pos2 {
        pos2(self.origin.x + p.x * self.zoom, self.origin.y + p.y * self.zoom)
    }
    pub fn rect(&self, r: [f32; 4]) -> Rect {
        let [x0, y0, x1, y1] = norm(r);
        Rect::from_min_max(self.to_screen(Point::new(x0, y0)), self.to_screen(Point::new(x1, y1)))
    }
}

pub fn zoom_steps() -> [f32; 12] {
    [0.125, 0.25, 0.5, 0.75, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
}

pub fn zoom_step(z: f32, up: bool) -> f32 {
    let steps = zoom_steps();
    if up {
        steps.iter().copied().find(|&s| s > z + 1e-4).unwrap_or(8.0)
    } else {
        steps.iter().rev().copied().find(|&s| s < z - 1e-4).unwrap_or(0.125)
    }
}

pub fn workspace(app: &mut PaintApp, ui: &mut Ui) {
    let full = ui.available_rect_before_wrap();
    let pal = app.palette;
    ui.painter().rect_filled(full, 0.0, pal.workspace);
    let mut area = full;
    if app.show_rulers {
        area.min += vec2(RULER, RULER);
    }
    app.upload(ui.ctx());
    let (cw, ch) = (app.doc.w as f32 * app.zoom, app.doc.h as f32 * app.zoom);
    if app.fit_pending && area.width() > 50.0 {
        let z = ((area.width() - 2.0 * MARGIN) / app.doc.w as f32).min((area.height() - 2.0 * MARGIN) / app.doc.h as f32);
        app.zoom = z.clamp(0.125, 1.0);
        app.scroll = egui::Vec2::ZERO;
        app.fit_pending = false;
    }
    let max_sx = (cw + 2.0 * MARGIN + 14.0 - area.width()).max(0.0);
    let max_sy = (ch + 2.0 * MARGIN + 14.0 - area.height()).max(0.0);
    app.scroll.x = app.scroll.x.clamp(0.0, max_sx);
    app.scroll.y = app.scroll.y.clamp(0.0, max_sy);
    let ox = if max_sx <= 0.0 { area.left() + ((area.width() - cw) / 2.0).max(MARGIN) } else { area.left() + MARGIN - app.scroll.x };
    let oy = if max_sy <= 0.0 { area.top() + ((area.height() - ch) / 2.0).max(MARGIN) } else { area.top() + MARGIN - app.scroll.y };
    let view = View { origin: pos2(ox.round(), oy.round()), zoom: app.zoom };
    let canvas_rect = Rect::from_min_size(view.origin, vec2(cw, ch));

    let id = ui.scope_id().with("workspace");
    let resp = ui.interact(area, id, Sense::click_and_drag());
    handle_wheel(app, ui, &view, area, &resp);

    let painter = ui.painter_at(area);
    painter.rect_filled(canvas_rect.translate(vec2(0.0, 2.0)).expand(1.0), 2.0, Color32::from_black_alpha(if pal.dark { 60 } else { 18 }));
    if !app.doc.background_visible {
        checkerboard(&painter, canvas_rect.intersect(area));
    }
    if let Some(t) = &app.texture {
        painter.image(t.id(), canvas_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
    painter.rect_stroke(canvas_rect, 0.0, Stroke::new(1.0, pal.canvas_border), StrokeKind::Outside);
    if app.show_grid && app.zoom >= 2.0 {
        grid(&painter, &view, app.doc.w, app.doc.h, canvas_rect.intersect(area), pal.dark);
    }

    let hover = resp.hover_pos().filter(|p| area.contains(*p));
    app.cursor = hover.map(|p| {
        let c = view.to_canvas(p);
        (c.x.floor() as i32, c.y.floor() as i32)
    });

    let resize_hit = canvas_handles(&painter, canvas_rect, &pal, app.canvas_resize.is_none());
    tool_input(app, ui, &view, &resp, hover, canvas_rect, &resize_hit);
    overlays(app, ui, &painter, &view);
    if let Some((h, w, hh)) = app.canvas_resize {
        let r = Rect::from_min_size(view.origin, vec2(w as f32 * app.zoom, hh as f32 * app.zoom));
        dashed(&painter, r, pal.text);
        let _ = h;
    }
    if app.show_rulers {
        rulers(app, ui, full, area, &view);
    }
    scrollbars(app, ui, area, max_sx, max_sy, cw, ch);
    if hover.is_some() {
        brush_cursor(app, ui, &painter, &view, hover.unwrap());
    }
}

fn handle_wheel(app: &mut PaintApp, ui: &Ui, view: &View, area: Rect, resp: &egui::Response) {
    if !resp.hovered() {
        return;
    }
    let (delta, ctrl, shift, pos) = ui.ctx().input(|i| (i.smooth_scroll_delta, i.modifiers.command, i.modifiers.shift, i.pointer.hover_pos()));
    if delta == egui::Vec2::ZERO {
        return;
    }
    if ctrl {
        let before = pos.map(|p| view.to_canvas(p));
        let f = if delta.y > 0.0 { 1.1 } else { 1.0 / 1.1 };
        app.zoom = (app.zoom * f).clamp(0.125, 8.0);
        if let (Some(c), Some(p)) = (before, pos) {
            app.scroll.x = MARGIN + c.x * app.zoom - (p.x - area.left());
            app.scroll.y = MARGIN + c.y * app.zoom - (p.y - area.top());
        }
    } else if shift {
        app.scroll.x -= delta.y + delta.x;
    } else {
        app.scroll -= delta;
    }
}

fn checkerboard(p: &egui::Painter, r: Rect) {
    let s = 8.0;
    p.rect_filled(r, 0.0, Color32::WHITE);
    let mut y = r.top();
    let mut row = 0;
    while y < r.bottom() {
        let mut x = r.left() + if row % 2 == 0 { 0.0 } else { s };
        while x < r.right() {
            p.rect_filled(Rect::from_min_max(pos2(x, y), pos2((x + s).min(r.right()), (y + s).min(r.bottom()))), 0.0, Color32::from_gray(222));
            x += 2.0 * s;
        }
        y += s;
        row += 1;
    }
}

fn grid(p: &egui::Painter, view: &View, w: u32, h: u32, clip: Rect, dark: bool) {
    let col = if dark { Color32::from_white_alpha(30) } else { Color32::from_black_alpha(40) };
    let step = if view.zoom >= 4.0 { 1 } else { 4 };
    let x0 = (((clip.left() - view.origin.x) / view.zoom).floor().max(0.0)) as u32;
    let x1 = (((clip.right() - view.origin.x) / view.zoom).ceil() as u32).min(w);
    let y0 = (((clip.top() - view.origin.y) / view.zoom).floor().max(0.0)) as u32;
    let y1 = (((clip.bottom() - view.origin.y) / view.zoom).ceil() as u32).min(h);
    for x in (x0..=x1).filter(|x| x % step == 0) {
        let sx = view.origin.x + x as f32 * view.zoom;
        p.line_segment([pos2(sx, clip.top()), pos2(sx, clip.bottom())], Stroke::new(1.0, col));
    }
    for y in (y0..=y1).filter(|y| y % step == 0) {
        let sy = view.origin.y + y as f32 * view.zoom;
        p.line_segment([pos2(clip.left(), sy), pos2(clip.right(), sy)], Stroke::new(1.0, col));
    }
}

fn handle_rect(c: Pos2) -> Rect {
    Rect::from_center_size(c, vec2(HANDLE, HANDLE))
}

fn draw_handle(p: &egui::Painter, c: Pos2, pal: &crate::theme::Palette) {
    let r = handle_rect(c);
    p.rect_filled(r, 1.0, if pal.dark { Color32::from_gray(40) } else { Color32::WHITE });
    p.rect_stroke(r, 1.0, Stroke::new(1.0, pal.text_secondary), StrokeKind::Inside);
}

fn canvas_handles(p: &egui::Painter, r: Rect, pal: &crate::theme::Palette, draw: bool) -> Vec<(Handle, Rect)> {
    let hs = [
        (Handle::E, pos2(r.right() + 5.0, r.center().y)),
        (Handle::S, pos2(r.center().x, r.bottom() + 5.0)),
        (Handle::SE, pos2(r.right() + 5.0, r.bottom() + 5.0)),
    ];
    if draw {
        for (_, c) in hs {
            draw_handle(p, c, pal);
        }
    }
    hs.iter().map(|(h, c)| (*h, handle_rect(*c).expand(3.0))).collect()
}

fn box_handles(r: Rect) -> Vec<(Handle, Pos2)> {
    vec![
        (Handle::NW, r.left_top()),
        (Handle::N, r.center_top()),
        (Handle::NE, r.right_top()),
        (Handle::E, r.right_center()),
        (Handle::SE, r.right_bottom()),
        (Handle::S, r.center_bottom()),
        (Handle::SW, r.left_bottom()),
        (Handle::W, r.left_center()),
    ]
}

fn hit_box(r: Rect, p: Pos2) -> Option<Handle> {
    for (h, c) in box_handles(r) {
        if handle_rect(c).expand(3.0).contains(p) {
            return Some(h);
        }
    }
    if r.expand(2.0).contains(p) { Some(Handle::Move) } else { None }
}

fn apply_handle(h: Handle, orig: [f32; 4], d: Point) -> [f32; 4] {
    let [mut x0, mut y0, mut x1, mut y1] = norm(orig);
    match h {
        Handle::Move => {
            x0 += d.x;
            x1 += d.x;
            y0 += d.y;
            y1 += d.y;
        }
        Handle::N => y0 += d.y,
        Handle::S => y1 += d.y,
        Handle::E => x1 += d.x,
        Handle::W => x0 += d.x,
        Handle::NE => {
            y0 += d.y;
            x1 += d.x
        }
        Handle::NW => {
            y0 += d.y;
            x0 += d.x
        }
        Handle::SE => {
            y1 += d.y;
            x1 += d.x
        }
        Handle::SW => {
            y1 += d.y;
            x0 += d.x
        }
        Handle::Point(_) => {}
    }
    [x0, y0, x1, y1]
}

fn cursor_for(h: Handle) -> CursorIcon {
    match h {
        Handle::Move => CursorIcon::Move,
        Handle::N | Handle::S => CursorIcon::ResizeVertical,
        Handle::E | Handle::W => CursorIcon::ResizeHorizontal,
        Handle::NE | Handle::SW => CursorIcon::ResizeNeSw,
        Handle::NW | Handle::SE => CursorIcon::ResizeNwSe,
        Handle::Point(_) => CursorIcon::Grab,
    }
}

struct Input {
    pressed: Option<PointerButton>,
    down: bool,
    pos: Option<Pos2>,
    shift: bool,
    double: bool,
}

fn tool_input(app: &mut PaintApp, ui: &mut Ui, view: &View, resp: &egui::Response, hover: Option<Pos2>, canvas: Rect, resize_hit: &[(Handle, Rect)]) {
    let inp = ui.ctx().input(|i| Input {
        pressed: if i.pointer.button_pressed(PointerButton::Primary) {
            Some(PointerButton::Primary)
        } else if i.pointer.button_pressed(PointerButton::Secondary) {
            Some(PointerButton::Secondary)
        } else {
            None
        },
        down: i.pointer.any_down(),
        pos: i.pointer.interact_pos(),
        shift: i.modifiers.shift,
        double: i.pointer.button_double_clicked(PointerButton::Primary),
    });
    let Some(pos) = inp.pos else { return };
    let cp = view.to_canvas(pos);
    let secondary = inp.pressed == Some(PointerButton::Secondary);

    if let Some((h, _, _)) = app.canvas_resize {
        if inp.down {
            let w = ((cp.x).round().max(1.0)) as u32;
            let hh = ((cp.y).round().max(1.0)) as u32;
            let (w, hh) = match h {
                Handle::E => (w, app.doc.h),
                Handle::S => (app.doc.w, hh),
                _ => (w, hh),
            };
            app.canvas_resize = Some((h, w, hh));
            ui.ctx().set_cursor_icon(cursor_for(h));
        } else {
            let (_, w, hh) = app.canvas_resize.take().unwrap();
            if w != app.doc.w || hh != app.doc.h {
                app.resize_canvas(w, hh);
            }
        }
        return;
    }
    if let Some(hp) = hover {
        if let Some((h, _)) = resize_hit.iter().find(|(_, r)| r.contains(hp)) {
            ui.ctx().set_cursor_icon(cursor_for(*h));
            if inp.pressed == Some(PointerButton::Primary) {
                app.commit_all();
                app.canvas_resize = Some((*h, app.doc.w, app.doc.h));
                return;
            }
        }
    }
    let _ = canvas;

    match app.tool {
        Tool::Pencil | Tool::Brush | Tool::Eraser => stroke_tool(app, ui, &inp, cp, resp.hovered(), secondary),
        Tool::Fill => {
            if inp.pressed.is_some() && resp.hovered() {
                let c = if secondary { app.color2 } else { app.color1 };
                let before = app.snapshot();
                let tol = app.tolerance as f64;
                let r = fill::flood_fill(app.doc.layer_mut(), cp.x.floor() as i32, cp.y.floor() as i32, c, tol);
                if !r.is_empty() {
                    app.push_history(before);
                    app.mark(r);
                }
            }
        }
        Tool::Picker => {
            if inp.pressed.is_some() && resp.hovered() {
                let p = app.display.get(cp.x.floor() as i32, cp.y.floor() as i32);
                let c = Color::from_premul(p).with_alpha(255);
                if secondary {
                    app.color2 = c;
                } else {
                    app.color1 = c;
                }
                app.tool = app.prev_tool;
            }
        }
        Tool::Magnifier => {
            if inp.pressed.is_some() && resp.hovered() {
                let z = zoom_step(app.zoom, !secondary);
                app.zoom = z;
                app.scroll = egui::vec2(MARGIN + cp.x * z - 200.0, MARGIN + cp.y * z - 150.0);
            }
        }
        Tool::RectSelect | Tool::FreeSelect => select_tool(app, ui, &inp, view, pos, cp, resp.hovered()),
        Tool::Shape => shape_tool(app, ui, &inp, view, pos, cp, resp.hovered(), secondary),
        Tool::Text => text_tool(app, ui, &inp, view, pos, cp, resp.hovered()),
    }
}

fn stroke_tool(app: &mut PaintApp, ui: &Ui, inp: &Input, cp: Point, hovered: bool, secondary: bool) {
    if inp.pressed.is_some() && hovered && app.stroke.is_none() {
        let kind = match app.tool {
            Tool::Pencil => BrushKind::Pencil,
            Tool::Eraser => {
                if app.eraser_square {
                    BrushKind::EraserSquare
                } else {
                    BrushKind::EraserRound
                }
            }
            _ => app.brush,
        };
        let size_key = if kind.is_eraser() { BrushKind::EraserRound } else { kind };
        let params = StrokeParams {
            kind,
            size: app.sizes[&size_key],
            color: if secondary { app.color2 } else { app.color1 },
            opacity: app.opacity[&size_key],
            recolor: (kind.is_eraser() && secondary).then_some((app.color1, app.color2)),
        };
        app.seed = app.seed.wrapping_add(1);
        let before = app.snapshot();
        let mut stroke = PStroke::new(params, app.doc.w, app.doc.h, app.seed);
        let r = stroke.add_point(cp, 1.0);
        let base = app.doc.layer().clone();
        let sess = StrokeSession { stroke, base, before, last_tick: Instant::now() };
        sess.stroke.composite(&sess.base, app.doc.layer_mut(), r);
        app.mark(r);
        app.stroke = Some(sess);
        return;
    }
    let Some(mut sess) = app.stroke.take() else { return };
    if inp.down {
        let mut r = sess.stroke.add_point(cp, 1.0);
        if sess.stroke.params.kind == BrushKind::Airbrush {
            let now = Instant::now();
            while now.duration_since(sess.last_tick).as_millis() >= 10 {
                r = r.union(&sess.stroke.tick());
                sess.last_tick += std::time::Duration::from_millis(10);
            }
            ui.ctx().request_repaint();
        }
        sess.stroke.composite(&sess.base, app.doc.layer_mut(), r);
        app.mark(r);
        app.stroke = Some(sess);
    } else {
        app.stroke = Some(sess);
        app.commit_stroke();
    }
}

fn select_tool(app: &mut PaintApp, ui: &Ui, inp: &Input, view: &View, pos: Pos2, cp: Point, hovered: bool) {
    if let Some(sel) = &app.selection {
        if !sel.creating && sel.drag.is_none() {
            if let Some(h) = hit_box(view.rect(sel.rect), pos) {
                ui.ctx().set_cursor_icon(cursor_for(h));
            }
        }
    }
    if inp.pressed == Some(PointerButton::Primary) && hovered {
        let hit = app.selection.as_ref().and_then(|s| hit_box(view.rect(s.rect), pos));
        if let Some(h) = hit {
            app.lift_selection();
            let sel = app.selection.as_mut().unwrap();
            sel.drag = Some((h, cp, sel.rect));
            return;
        }
        app.commit_selection();
        app.selection = Some(Selection {
            rect: [cp.x.round(), cp.y.round(), cp.x.round(), cp.y.round()],
            image: None,
            lasso: if app.tool == Tool::FreeSelect { vec![cp] } else { Vec::new() },
            creating: true,
            drag: None,
            base: None,
            before: None,
        });
        return;
    }
    let (w, h) = (app.doc.w as f32, app.doc.h as f32);
    let shift = inp.shift;
    let mut rerender = false;
    let mut remove = false;
    if let Some(sel) = app.selection.as_mut() {
        if sel.creating {
            if inp.down {
                let c = Point::new(cp.x.clamp(0.0, w), cp.y.clamp(0.0, h));
                if sel.lasso.is_empty() {
                    let mut x1 = c.x.round();
                    let mut y1 = c.y.round();
                    if shift {
                        let d = (x1 - sel.rect[0]).abs().max((y1 - sel.rect[1]).abs());
                        x1 = sel.rect[0] + d * (x1 - sel.rect[0]).signum();
                        y1 = sel.rect[1] + d * (y1 - sel.rect[1]).signum();
                    }
                    sel.rect[2] = x1;
                    sel.rect[3] = y1;
                } else {
                    sel.lasso.push(c);
                }
            } else {
                sel.creating = false;
                if !sel.lasso.is_empty() {
                    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                    for p in &sel.lasso {
                        x0 = x0.min(p.x);
                        y0 = y0.min(p.y);
                        x1 = x1.max(p.x);
                        y1 = y1.max(p.y);
                    }
                    sel.rect = [x0.floor(), y0.floor(), x1.ceil(), y1.ceil()];
                    if sel.lasso.len() < 3 {
                        remove = true;
                    }
                }
                let [a, b, c, d] = norm(sel.rect);
                if c - a < 1.0 || d - b < 1.0 {
                    remove = true;
                }
            }
        } else if let Some((h, start, orig)) = sel.drag {
            if inp.down {
                let d = Point::new((cp.x - start.x).round(), (cp.y - start.y).round());
                sel.rect = apply_handle(h, orig, d);
                rerender = true;
            } else {
                sel.drag = None;
                sel.rect = norm(sel.rect);
            }
        }
    }
    if remove {
        app.selection = None;
    }
    if rerender {
        app.render_selection();
    }
}

fn constrain(p0: Point, p1: Point, line: bool) -> Point {
    let d = p1 - p0;
    if line {
        let ang = d.y.atan2(d.x);
        let snap = (ang / (std::f32::consts::PI / 4.0)).round() * (std::f32::consts::PI / 4.0);
        let l = d.len();
        p0 + Point::new(snap.cos() * l, snap.sin() * l)
    } else {
        let m = d.x.abs().max(d.y.abs());
        p0 + Point::new(m * d.x.signum(), m * d.y.signum())
    }
}

fn shape_tool(app: &mut PaintApp, ui: &Ui, inp: &Input, view: &View, pos: Pos2, cp: Point, hovered: bool, secondary: bool) {
    let kind = app.shape;
    if inp.pressed.is_some() && hovered {
        if let Some(s) = app.shape_session.as_mut() {
            if s.kind == ShapeKind::Curve && s.stage < 3 && !s.creating {
                let idx = if s.stage == 1 { 2 } else { 3 };
                s.pts[idx] = cp;
                if s.stage == 1 {
                    s.pts[3] = cp;
                }
                s.drag = Some((Handle::Point(idx), cp, s.pts.clone()));
                s.stage += 1;
                render_shape(app);
                return;
            }
            if s.kind == ShapeKind::Polygon && s.stage == 0 {
                let close = s.pts.len() >= 3 && (view.to_screen(s.pts[0]) - pos).length() < 8.0;
                if inp.double || close {
                    s.stage = 1;
                    if close {
                        s.pts.pop();
                    }
                } else {
                    s.pts.push(cp);
                    s.creating = true;
                }
                render_shape(app);
                return;
            }
            if let Some(h) = shape_hit(s, view, pos) {
                s.drag = Some((h, cp, s.pts.clone()));
                return;
            }
        }
        app.commit_shape();
        let before = app.snapshot();
        let base = app.doc.layer().clone();
        let pts = match kind {
            ShapeKind::Curve => vec![cp, cp, cp, cp],
            _ => vec![cp, cp],
        };
        app.shape_session = Some(ShapeSession { kind, pts, stage: 0, creating: true, swap_colors: secondary, drag: None, base, before });
        return;
    }
    let Some(s) = app.shape_session.as_mut() else { return };
    if let Some((h, start, orig)) = s.drag.clone() {
        if inp.down {
            let d = cp - start;
            match h {
                Handle::Point(i) => s.pts[i] = orig[i] + d,
                _ if s.kind.is_box_shape() => {
                    let r = apply_handle(h, [orig[0].x, orig[0].y, orig[1].x, orig[1].y], d);
                    s.pts[0] = Point::new(r[0], r[1]);
                    s.pts[1] = Point::new(r[2], r[3]);
                }
                _ => {
                    for (i, p) in s.pts.iter_mut().enumerate() {
                        *p = orig[i] + d;
                    }
                }
            }
            render_shape(app);
        } else {
            s.drag = None;
        }
        return;
    }
    if s.creating {
        let n = s.pts.len();
        let line = matches!(s.kind, ShapeKind::Line | ShapeKind::Curve | ShapeKind::Polygon);
        let target = if inp.shift { constrain(s.pts[if s.kind == ShapeKind::Polygon { n - 2 } else { 0 }], cp, line) } else { cp };
        if s.kind == ShapeKind::Polygon {
            if n >= 2 {
                s.pts[n - 1] = target;
            }
            if !inp.down {
                s.creating = false;
            }
        } else if s.kind == ShapeKind::Curve {
            s.pts[1] = target;
            s.pts[3] = target;
            if !inp.down {
                s.creating = false;
                s.stage = 1;
            }
        } else {
            s.pts[1] = target;
            if !inp.down {
                s.creating = false;
            }
        }
        if s.kind == ShapeKind::Polygon && !inp.down && n == 2 {
            s.pts.push(target);
            s.creating = true;
            if (s.pts[0] - s.pts[1]).len() < 1.0 {
                s.pts.truncate(1);
            }
        }
        render_shape(app);
        if s_is_degenerate(app) && !inp.down {
            app.cancel_shape_if_empty();
        }
    } else if hovered {
        if let Some(h) = shape_hit(s, view, pos) {
            ui.ctx().set_cursor_icon(cursor_for(h));
        }
    }
}

fn s_is_degenerate(app: &PaintApp) -> bool {
    app.shape_session.as_ref().is_some_and(|s| {
        s.kind != ShapeKind::Polygon && s.kind != ShapeKind::Curve && (s.pts[0] - s.pts[1]).len() < 1.0
    })
}

impl PaintApp {
    fn cancel_shape_if_empty(&mut self) {
        if let Some(s) = self.shape_session.take() {
            *self.doc.layer_mut() = s.base;
            self.mark_all();
        }
    }
}

fn shape_hit(s: &ShapeSession, view: &View, pos: Pos2) -> Option<Handle> {
    if s.kind.is_box_shape() {
        hit_box(view.rect([s.pts[0].x, s.pts[0].y, s.pts[1].x, s.pts[1].y]), pos)
    } else {
        for (i, p) in s.pts.iter().enumerate() {
            if handle_rect(view.to_screen(*p)).expand(3.0).contains(pos) {
                return Some(Handle::Point(i));
            }
        }
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for p in &s.pts {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
        view.rect([x0, y0, x1, y1]).expand(4.0).contains(pos).then_some(Handle::Move)
    }
}

pub fn render_shape(app: &mut PaintApp) {
    let Some(s) = &app.shape_session else { return };
    let (outline_c, fill_c) = if s.swap_colors { (app.color2, app.color1) } else { (app.color1, app.color2) };
    let mut layer = s.base.clone();
    let bounds = layer.bounds();
    let width = app.shape_width;
    let (paths, closed): (Vec<Vec<Point>>, bool) = match s.kind {
        ShapeKind::Line => (vec![vec![s.pts[0], s.pts[1]]], false),
        ShapeKind::Curve => (vec![bezier(s.pts[0], s.pts[2], s.pts[3], s.pts[1], 64)], false),
        ShapeKind::Polygon => (vec![s.pts.clone()], s.stage == 1),
        k => (k.paths(s.pts[0].x, s.pts[0].y, s.pts[1].x, s.pts[1].y), true),
    };
    if closed {
        match app.fill {
            OutlineStyle::None => {}
            OutlineStyle::Solid | OutlineStyle::Textured(_) => {
                let src = fill_c.premul();
                let alpha = if let OutlineStyle::Textured(k) = app.fill { textured_alpha(k) } else { 1.0 };
                fill_polygons(&paths, bounds, |x, y, c| layer.blend(x, y, src, (c * alpha * 255.0) as u8));
            }
        }
    }
    let outline = if matches!(s.kind, ShapeKind::Line | ShapeKind::Curve) && app.outline == OutlineStyle::None {
        OutlineStyle::Solid
    } else {
        app.outline
    };
    match outline {
        OutlineStyle::None => {}
        OutlineStyle::Solid => {
            let mut mask = Mask::new(layer.w, layer.h);
            let mut dirty = IRect::EMPTY;
            for p in &paths {
                dirty = dirty.union(&stroke_polyline(&mut mask, p, closed, width, true));
            }
            let src = outline_c.premul();
            for y in dirty.y0..dirty.y1 {
                for x in dirty.x0..dirty.x1 {
                    let m = mask.get(x, y);
                    if m > 0.0 {
                        layer.blend(x, y, src, (m * 255.0) as u8);
                    }
                }
            }
        }
        OutlineStyle::Textured(kind) => {
            let params = StrokeParams { kind, size: width.max(4.0), color: outline_c, opacity: 1.0, recolor: None };
            let mut st = PStroke::new(params, layer.w, layer.h, 7);
            let mut dirty = IRect::EMPTY;
            for p in &paths {
                for q in p {
                    dirty = dirty.union(&st.add_point(*q, 1.0));
                }
                if closed {
                    dirty = dirty.union(&st.add_point(p[0], 1.0));
                }
            }
            let b = layer.clone();
            st.composite(&b, &mut layer, dirty);
        }
    }
    *app.doc.layer_mut() = layer;
    app.mark_all();
}

fn textured_alpha(k: BrushKind) -> f32 {
    match k {
        BrushKind::Marker => 0.55,
        BrushKind::Watercolor => 0.6,
        BrushKind::NaturalPencil => 0.85,
        _ => 1.0,
    }
}

fn text_tool(app: &mut PaintApp, ui: &Ui, inp: &Input, view: &View, pos: Pos2, cp: Point, hovered: bool) {
    if inp.pressed == Some(PointerButton::Primary) && hovered {
        if let Some(t) = app.text.as_mut() {
            let r = view.rect(t.rect);
            if let Some(h) = hit_box(r, pos) {
                if h != Handle::Move || !r.shrink(4.0).contains(pos) {
                    t.drag = Some((h, cp, t.rect));
                    return;
                }
                return;
            }
        }
        app.commit_text();
        let before = app.snapshot();
        let base = app.doc.layer().clone();
        app.text = Some(TextSession { rect: [cp.x, cp.y, cp.x, cp.y], text: String::new(), base, before, creating: true, drag: None });
        return;
    }
    let Some(t) = app.text.as_mut() else { return };
    if t.creating {
        if inp.down {
            t.rect[2] = cp.x;
            t.rect[3] = cp.y;
        } else {
            t.creating = false;
            let [x0, y0, x1, y1] = norm(t.rect);
            let min_w = 120.0;
            let min_h = app.text_style.size_pt * 96.0 / 72.0 * 1.6;
            t.rect = [x0, y0, x0 + (x1 - x0).max(min_w), y0 + (y1 - y0).max(min_h)];
            render_text(app);
            ui.ctx().memory_mut(|m| m.request_focus(egui::Id::unique("text-tool-edit")));
        }
    } else if let Some((h, start, orig)) = t.drag {
        if inp.down {
            t.rect = apply_handle(h, orig, cp - start);
            render_text(app);
        } else {
            t.drag = None;
        }
    } else if let Some(h) = hit_box(view.rect(t.rect), pos) {
        if h != Handle::Move || !view.rect(t.rect).shrink(4.0).contains(pos) {
            ui.ctx().set_cursor_icon(cursor_for(h));
        } else {
            ui.ctx().set_cursor_icon(CursorIcon::Text);
        }
    }
}

pub fn render_text(app: &mut PaintApp) {
    let Some(t) = &app.text else { return };
    let mut style = app.text_style.clone();
    style.color = app.color1;
    style.background = style.background.map(|_| app.color2);
    let family = app.font_family.clone();
    let Some(font) = app.fonts.get(&family, style.bold, style.italic) else { return };
    let [x0, y0, x1, y1] = norm(t.rect);
    let img = paint_core::text::render(&font, &t.text, &style, (x1 - x0).max(1.0) as u32, (y1 - y0).max(1.0) as u32);
    let mut layer = t.base.clone();
    layer.paste_over(&img, x0.round() as i32, y0.round() as i32, 255);
    *app.doc.layer_mut() = layer;
    app.mark_all();
}

fn dashed(p: &egui::Painter, r: Rect, col: Color32) {
    let pts = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
    p.extend(Shape::dashed_line(&pts, Stroke::new(1.0, Color32::WHITE), 4.0, 0.0));
    p.extend(Shape::dashed_line(&pts, Stroke::new(1.0, col), 4.0, 4.0));
}

fn overlays(app: &mut PaintApp, ui: &mut Ui, painter: &egui::Painter, view: &View) {
    let pal = app.palette;
    let ink = if pal.dark { Color32::from_gray(220) } else { Color32::from_gray(30) };
    if let Some(sel) = &app.selection {
        if sel.creating && !sel.lasso.is_empty() {
            let pts: Vec<Pos2> = sel.lasso.iter().map(|p| view.to_screen(*p)).collect();
            painter.extend(Shape::dashed_line(&pts, Stroke::new(1.0, ink), 4.0, 4.0));
        } else {
            let r = view.rect(sel.rect);
            dashed(painter, r, ink);
            if !sel.creating {
                for (_, c) in box_handles(r) {
                    draw_handle(painter, c, &pal);
                }
            }
        }
    }
    if let Some(s) = &app.shape_session {
        if s.kind.is_box_shape() {
            let r = view.rect([s.pts[0].x, s.pts[0].y, s.pts[1].x, s.pts[1].y]);
            dashed(painter, r, ink);
            if !s.creating {
                for (_, c) in box_handles(r) {
                    draw_handle(painter, c, &pal);
                }
            }
        } else if !s.creating || s.kind == ShapeKind::Polygon {
            let idx: Vec<usize> = match s.kind {
                ShapeKind::Curve => vec![0, 1],
                _ => (0..s.pts.len()).collect(),
            };
            for i in idx {
                draw_handle(painter, view.to_screen(s.pts[i]), &pal);
            }
        }
    }
    let mut text_changed = false;
    if let Some(t) = &mut app.text {
        let r = view.rect(t.rect);
        dashed(painter, r, ink);
        if !t.creating {
            for (_, c) in box_handles(r) {
                draw_handle(painter, c, &pal);
            }
            let size = app.text_style.size_pt * 96.0 / 72.0 * view.zoom;
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(r.shrink2(vec2(2.0, 0.0))));
            let edit = egui::TextEdit::multiline(&mut t.text)
                .id(egui::Id::unique("text-tool-edit"))
                .frame(egui::Frame::NONE)
                .font(FontId::proportional(size.max(4.0)))
                .text_color(Color32::TRANSPARENT)
                .desired_width(r.width() - 4.0)
                .desired_rows(1)
                .margin(egui::Margin::ZERO);
            let out = child.add(edit);
            text_changed = out.changed();
        }
    }
    if text_changed {
        render_text(app);
    }
}

fn brush_cursor(app: &PaintApp, ui: &Ui, painter: &egui::Painter, view: &View, p: Pos2) {
    let icon = match app.tool {
        Tool::Pencil | Tool::Brush | Tool::Eraser => {
            let size = if app.tool == Tool::Eraser { app.sizes[&BrushKind::EraserRound] } else { app.size() };
            let d = size * view.zoom;
            if d >= 5.0 && app.canvas_resize.is_none() {
                for (w, c) in [(3.0, Color32::WHITE), (1.0, Color32::BLACK)] {
                    if app.tool == Tool::Eraser && app.eraser_square {
                        painter.rect_stroke(Rect::from_center_size(p, vec2(d, d)), 0.0, Stroke::new(w, c), StrokeKind::Middle);
                    } else {
                        painter.circle_stroke(p, d / 2.0, Stroke::new(w, c));
                    }
                }
                CursorIcon::None
            } else {
                CursorIcon::Crosshair
            }
        }
        Tool::Text => CursorIcon::Text,
        Tool::Picker | Tool::Fill | Tool::Shape | Tool::RectSelect | Tool::FreeSelect => CursorIcon::Crosshair,
        Tool::Magnifier => CursorIcon::ZoomIn,
    };
    if ui.ctx().output(|o| o.cursor_icon) == CursorIcon::Default {
        ui.ctx().set_cursor_icon(icon);
    }
}

fn rulers(app: &PaintApp, ui: &Ui, full: Rect, area: Rect, view: &View) {
    let pal = app.palette;
    let p = ui.painter_at(full);
    let top = Rect::from_min_max(pos2(area.left(), full.top()), pos2(full.right(), area.top()));
    let left = Rect::from_min_max(pos2(full.left(), area.top()), pos2(area.left(), full.bottom()));
    p.rect_filled(top, 0.0, pal.ruler_bg);
    p.rect_filled(left, 0.0, pal.ruler_bg);
    p.rect_filled(Rect::from_min_max(full.min, area.min), 0.0, pal.ruler_bg);
    p.line_segment([top.left_bottom(), top.right_bottom()], Stroke::new(1.0, pal.divider));
    p.line_segment([left.right_top(), left.right_bottom()], Stroke::new(1.0, pal.divider));
    let target_px = 80.0;
    let mut step = 1.0;
    for s in [1.0, 2.0, 5.0, 10.0, 20.0, 25.0, 50.0, 100.0, 200.0, 250.0, 500.0, 1000.0, 2000.0] {
        step = s;
        if s * view.zoom >= target_px {
            break;
        }
    }
    let font = FontId::proportional(10.0);
    let minor = step / 10.0;
    let mut v = ((top.left() - view.origin.x) / view.zoom / minor).floor() * minor;
    while view.origin.x + v * view.zoom < top.right() {
        let x = view.origin.x + v * view.zoom;
        let major = (v / step).round() * step == v || ((v / step) - (v / step).round()).abs() < 1e-3;
        let h = if major { 10.0 } else if ((v / (step / 2.0)) - (v / (step / 2.0)).round()).abs() < 1e-3 { 6.0 } else { 3.0 };
        if x >= top.left() {
            p.line_segment([pos2(x, top.bottom() - h), pos2(x, top.bottom())], Stroke::new(1.0, pal.ruler_fg));
            if major {
                p.text(pos2(x + 2.0, top.top() + 1.0), Align2::LEFT_TOP, format!("{}", v as i32), font.clone(), pal.ruler_fg);
            }
        }
        v += minor;
    }
    let mut v = ((left.top() - view.origin.y) / view.zoom / minor).floor() * minor;
    while view.origin.y + v * view.zoom < left.bottom() {
        let y = view.origin.y + v * view.zoom;
        let major = ((v / step) - (v / step).round()).abs() < 1e-3;
        let w = if major { 10.0 } else if ((v / (step / 2.0)) - (v / (step / 2.0)).round()).abs() < 1e-3 { 6.0 } else { 3.0 };
        if y >= left.top() {
            p.line_segment([pos2(left.right() - w, y), pos2(left.right(), y)], Stroke::new(1.0, pal.ruler_fg));
            if major {
                p.text(pos2(left.left() + 2.0, y + 2.0), Align2::LEFT_TOP, format!("{}", v as i32), font.clone(), pal.ruler_fg);
            }
        }
        v += minor;
    }
    if let Some((cx, cy)) = app.cursor {
        let x = view.origin.x + (cx as f32 + 0.5) * view.zoom;
        let y = view.origin.y + (cy as f32 + 0.5) * view.zoom;
        p.line_segment([pos2(x, top.top()), pos2(x, top.bottom())], Stroke::new(1.0, pal.accent));
        p.line_segment([pos2(left.left(), y), pos2(left.right(), y)], Stroke::new(1.0, pal.accent));
    }
}

fn scrollbars(app: &mut PaintApp, ui: &mut Ui, area: Rect, max_sx: f32, max_sy: f32, cw: f32, ch: f32) {
    let pal = app.palette;
    let gutter = 12.0;
    let thick = 6.0;
    let thumb_col = if pal.dark { Color32::from_white_alpha(140) } else { Color32::from_black_alpha(130) };
    let thumb_hot = if pal.dark { Color32::from_white_alpha(200) } else { Color32::from_black_alpha(190) };
    if max_sx > 0.0 {
        let strip = Rect::from_min_max(pos2(area.left(), area.bottom() - gutter), pos2(area.right() - if max_sy > 0.0 { gutter } else { 0.0 }, area.bottom()));
        ui.painter().rect_filled(strip, 0.0, pal.workspace);
        let track = strip.shrink2(vec2(4.0, (gutter - thick) / 2.0));
        let total = cw + 2.0 * MARGIN;
        let len = (track.width() * area.width() / total).max(30.0);
        let x = track.left() + (track.width() - len) * (app.scroll.x / max_sx);
        let thumb = Rect::from_min_size(pos2(x, track.top()), vec2(len, thick));
        let r = ui.interact(strip, ui.scope_id().with("hscroll"), Sense::click_and_drag());
        if r.dragged() {
            app.scroll.x += r.drag_delta().x * max_sx / (track.width() - len).max(1.0);
        }
        ui.painter().rect_filled(thumb, 3.0, if r.hovered() || r.dragged() { thumb_hot } else { thumb_col });
    }
    if max_sy > 0.0 {
        let strip = Rect::from_min_max(pos2(area.right() - gutter, area.top()), pos2(area.right(), area.bottom() - if max_sx > 0.0 { gutter } else { 0.0 }));
        ui.painter().rect_filled(strip, 0.0, pal.workspace);
        let track = strip.shrink2(vec2((gutter - thick) / 2.0, 4.0));
        let total = ch + 2.0 * MARGIN;
        let len = (track.height() * area.height() / total).max(30.0);
        let y = track.top() + (track.height() - len) * (app.scroll.y / max_sy);
        let thumb = Rect::from_min_size(pos2(track.left(), y), vec2(thick, len));
        let r = ui.interact(strip, ui.scope_id().with("vscroll"), Sense::click_and_drag());
        if r.dragged() {
            app.scroll.y += r.drag_delta().y * max_sy / (track.height() - len).max(1.0);
        }
        ui.painter().rect_filled(thumb, 3.0, if r.hovered() || r.dragged() { thumb_hot } else { thumb_col });
    }
    if max_sx > 0.0 && max_sy > 0.0 {
        ui.painter().rect_filled(Rect::from_min_max(area.right_bottom() - vec2(gutter, gutter), area.right_bottom()), 0.0, pal.workspace);
    }
}

pub fn selection_size(app: &PaintApp) -> Option<(i32, i32)> {
    let r = app
        .selection
        .as_ref()
        .map(|s| s.rect)
        .or_else(|| app.shape_session.as_ref().filter(|s| s.kind.is_box_shape()).map(|s| [s.pts[0].x, s.pts[0].y, s.pts[1].x, s.pts[1].y]))
        .or_else(|| app.text.as_ref().map(|t| t.rect))?;
    let [x0, y0, x1, y1] = norm(r);
    Some(((x1 - x0).round() as i32, (y1 - y0).round() as i32))
}
