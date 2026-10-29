use std::sync::Arc;

use eframe::egui::{self, Align2, Color32, Rect, Sense, Stroke, Ui, ViewportCommand, pos2, vec2};
use paint_core::{Color, transform};

use crate::app::PaintApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingAction {
    New,
    Open,
    Exit,
}

pub fn guard(app: &mut PaintApp, action: PendingAction) {
    if app.dirty {
        app.dialogs.about_unsaved = Some(action);
    } else {
        run(app, action, None);
    }
}

fn run(app: &mut PaintApp, action: PendingAction, ctx: Option<&egui::Context>) {
    match action {
        PendingAction::New => app.new_document(),
        PendingAction::Open => app.open_dialog(),
        PendingAction::Exit => {
            app.dirty = false;
            if let Some(ctx) = ctx {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    }
}

impl PaintApp {
    pub fn request_close(&mut self, ctx: &egui::Context) {
        self.commit_all();
        if self.dirty {
            self.dialogs.about_unsaved = Some(PendingAction::Exit);
        } else {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

pub struct ResizeState {
    pub by_percent: bool,
    pub w: f32,
    pub h: f32,
    pub keep: bool,
    pub skew_x: f32,
    pub skew_y: f32,
}

impl Default for ResizeState {
    fn default() -> Self {
        ResizeState { by_percent: true, w: 100.0, h: 100.0, keep: true, skew_x: 0.0, skew_y: 0.0 }
    }
}

pub struct PropertiesState {
    pub w: f32,
    pub h: f32,
    pub unit: u8,
    pub dpi: f64,
}

impl PropertiesState {
    pub fn from(app: &PaintApp) -> Self {
        PropertiesState { w: app.doc.w as f32, h: app.doc.h as f32, unit: 0, dpi: app.doc.dpi }
    }
}

pub struct EditColorsState {
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub hex: String,
    pub background: bool,
    pub picking: Option<std::sync::Arc<std::sync::Mutex<Option<std::sync::Arc<egui::ColorImage>>>>>,
}

impl EditColorsState {
    pub fn new(c: Color) -> Self {
        let (h, s, v) = c.to_hsv();
        EditColorsState { h, s, v, hex: format!("{:06X}", c.to_hex()), background: false, picking: None }
    }
    pub fn for_background(c: Color) -> Self {
        EditColorsState { background: true, ..Self::new(c) }
    }
    fn color(&self) -> Color {
        Color::from_hsv(self.h, self.s, self.v)
    }
    fn set(&mut self, c: Color) {
        let (h, s, v) = c.to_hsv();
        self.h = h;
        self.s = s;
        self.v = v;
        self.hex = format!("{:06X}", c.to_hex());
    }
}

fn modal(ctx: &egui::Context, title: &str, add: impl FnOnce(&mut Ui) -> Option<bool>) -> Option<bool> {
    let screen = ctx.content_rect();
    let dark = ctx.global_style().visuals.dark_mode;
    egui::Area::new(egui::Id::unique(("modal-bg", title))).order(egui::Order::Middle).fixed_pos(screen.min).show(ctx, |ui| {
        let (r, _) = ui.allocate_exact_size(screen.size(), Sense::click());
        ui.painter().rect_filled(r, 0.0, Color32::from_black_alpha(if dark { 110 } else { 70 }));
    });
    let mut result = None;
    let fill = if dark { Color32::from_rgb(0x38, 0x38, 0x38) } else { Color32::WHITE };
    egui::Window::new(title)
        .id(egui::Id::unique(("modal", title)))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .min_width(300.0)
        .frame(
            egui::Frame::new()
                .fill(fill)
                .corner_radius(14.0)
                .stroke(Stroke::new(1.0, Color32::from_black_alpha(if dark { 120 } else { 30 })))
                .shadow(egui::Shadow { offset: [0, 6], blur: 28, spread: 0, color: Color32::from_black_alpha(90) })
                .inner_margin(egui::Margin::same(24)),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(title).size(17.0).family(egui::FontFamily::Name("bold".into())));
            });
            ui.add_space(14.0);
            result = add(ui);
        });
    result
}

pub fn adw_button(ui: &mut Ui, label: &str, suggested: bool, w: f32) -> bool {
    let v = ui.visuals().clone();
    let (fill, text) = if suggested {
        (v.selection.bg_fill, Color32::WHITE)
    } else if v.dark_mode {
        (Color32::from_white_alpha(26), Color32::WHITE)
    } else {
        (Color32::from_black_alpha(20), Color32::from_black_alpha(204))
    };
    ui.add(egui::Button::new(egui::RichText::new(label).color(text).family(egui::FontFamily::Name("bold".into()))).fill(fill).stroke(Stroke::NONE).corner_radius(8.0).min_size(vec2(w, 36.0)))
        .clicked()
}

fn ok_cancel(ui: &mut Ui, ok: &str) -> Option<bool> {
    ui.add_space(18.0);
    let mut r = None;
    ui.horizontal(|ui| {
        let w = (ui.available_width() - 10.0) / 2.0;
        ui.spacing_mut().item_spacing.x = 10.0;
        if adw_button(ui, "Cancel", false, w) {
            r = Some(false);
        }
        if adw_button(ui, ok, true, w) {
            r = Some(true);
        }
    });
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        r = Some(false);
    }
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && r.is_none() {
        r = Some(true);
    }
    r
}

pub fn show(app: &mut PaintApp, ctx: &egui::Context) {
    if let Some(mut st) = app.dialogs.resize.take() {
        let (bw, bh) = match app.selection_image() {
            Some(i) => (i.w as f32, i.h as f32),
            None => (app.doc.w as f32, app.doc.h as f32),
        };
        let res = modal(ctx, "Resize and skew", |ui| {
            ui.label(egui::RichText::new("Resize").strong());
            ui.horizontal(|ui| {
                if ui.radio(st.by_percent, "Percentage").clicked() && !st.by_percent {
                    st.by_percent = true;
                    st.w = 100.0;
                    st.h = 100.0;
                }
                if ui.radio(!st.by_percent, "Pixels").clicked() && st.by_percent {
                    st.by_percent = false;
                    st.w = bw;
                    st.h = bh;
                }
            });
            let ratio = bh / bw;
            egui::Grid::new("resize-grid").num_columns(2).spacing(vec2(12.0, 8.0)).show(ui, |ui| {
                ui.label("Horizontal");
                if ui.add(egui::DragValue::new(&mut st.w).range(1.0..=10000.0)).changed() && st.keep {
                    st.h = if st.by_percent { st.w } else { (st.w * ratio).round() };
                }
                ui.end_row();
                ui.label("Vertical");
                if ui.add(egui::DragValue::new(&mut st.h).range(1.0..=10000.0)).changed() && st.keep {
                    st.w = if st.by_percent { st.h } else { (st.h / ratio).round() };
                }
                ui.end_row();
            });
            ui.checkbox(&mut st.keep, "Maintain aspect ratio");
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Skew (Degrees)").strong());
            egui::Grid::new("skew-grid").num_columns(2).spacing(vec2(12.0, 8.0)).show(ui, |ui| {
                ui.label("Horizontal");
                ui.add(egui::DragValue::new(&mut st.skew_x).range(-89.0..=89.0));
                ui.end_row();
                ui.label("Vertical");
                ui.add(egui::DragValue::new(&mut st.skew_y).range(-89.0..=89.0));
                ui.end_row();
            });
            ok_cancel(ui, "OK")
        });
        match res {
            Some(true) => {
                let (nw, nh) = if st.by_percent { ((bw * st.w / 100.0).round(), (bh * st.h / 100.0).round()) } else { (st.w, st.h) };
                let (sx, sy) = (st.skew_x, st.skew_y);
                app.image_op(move |i| {
                    let r = if nw as u32 != i.w || nh as u32 != i.h { transform::resize(i, nw as u32, nh as u32) } else { i.clone() };
                    if sx != 0.0 || sy != 0.0 { transform::skew(&r, sx, sy) } else { r }
                });
            }
            Some(false) => {}
            None => app.dialogs.resize = Some(st),
        }
    }

    if let Some(mut angle) = app.dialogs.rotate.take() {
        let res = modal(ctx, "Custom rotate", |ui| {
            ui.horizontal(|ui| {
                ui.label("Angle in degrees");
                ui.add(egui::DragValue::new(&mut angle).range(-360.0..=360.0).speed(1.0));
            });
            ok_cancel(ui, "OK")
        });
        match res {
            Some(true) => app.image_op(move |i| transform::rotate_any(i, angle)),
            Some(false) => {}
            None => app.dialogs.rotate = Some(angle),
        }
    }

    if let Some(mut st) = app.dialogs.properties.take() {
        let res = modal(ctx, "Image properties", |ui| {
            ui.label(egui::RichText::new("File attributes").strong());
            let saved = app
                .file
                .as_ref()
                .and_then(|p| std::fs::metadata(p).ok())
                .and_then(|m| m.modified().ok())
                .map(|t| format!("{:?}", chrono_like(t)))
                .unwrap_or_else(|| "Not available".into());
            egui::Grid::new("props").num_columns(2).spacing(vec2(16.0, 6.0)).show(ui, |ui| {
                ui.label("Last saved");
                ui.label(saved);
                ui.end_row();
                ui.label("Size on disk");
                ui.label(app.last_saved_size.map(|b| format!("{b} bytes")).unwrap_or_else(|| "Not available".into()));
                ui.end_row();
                ui.label("Resolution");
                ui.label(format!("{} DPI", st.dpi.round()));
                ui.end_row();
            });
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Units").strong());
            ui.horizontal(|ui| {
                ui.radio_value(&mut st.unit, 1, "Inches");
                ui.radio_value(&mut st.unit, 2, "Centimeters");
                ui.radio_value(&mut st.unit, 0, "Pixels");
            });
            let factor = match st.unit {
                1 => 1.0 / st.dpi as f32,
                2 => 2.54 / st.dpi as f32,
                _ => 1.0,
            };
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Image size").strong());
            let (mut w, mut h) = (st.w * factor, st.h * factor);
            egui::Grid::new("props-size").num_columns(2).spacing(vec2(16.0, 6.0)).show(ui, |ui| {
                ui.label("Width");
                if ui.add(egui::DragValue::new(&mut w).range(0.01..=100000.0).max_decimals(2)).changed() {
                    st.w = (w / factor).round().max(1.0);
                }
                ui.end_row();
                ui.label("Height");
                if ui.add(egui::DragValue::new(&mut h).range(0.01..=100000.0).max_decimals(2)).changed() {
                    st.h = (h / factor).round().max(1.0);
                }
                ui.end_row();
            });
            if ui.button("Default").clicked() {
                st.w = app.doc.w as f32;
                st.h = app.doc.h as f32;
            }
            ok_cancel(ui, "OK")
        });
        match res {
            Some(true) => {
                if st.w as u32 != app.doc.w || st.h as u32 != app.doc.h {
                    app.resize_canvas(st.w as u32, st.h as u32);
                }
            }
            Some(false) => {}
            None => app.dialogs.properties = Some(st),
        }
    }

    if let Some(mut st) = app.dialogs.edit_colors.take() {
        if st.picking.is_some() {
            if !eyedropper(ctx, &mut st) {
                app.dialogs.edit_colors = Some(st);
                return;
            }
        }
        let mut start_pick = false;
        let res = modal(ctx, "Edit colors", |ui| {
            color_wheel(ui, &app.palette, &mut st);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let (er, eresp) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::click());
                let ev = ui.visuals().clone();
                ui.painter().rect_filled(er, 8.0, if eresp.hovered() { ev.widgets.hovered.bg_fill } else { ev.widgets.inactive.bg_fill });
                crate::icons::draw(ui.painter(), crate::icons::Icon::Picker, er.shrink(8.0), ev.text_color());
                if eresp.on_hover_text("Eyedropper: pick a color from anywhere in the window").clicked() {
                    start_pick = true;
                }
                let c = st.color();
                let (r, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
                ui.painter().circle_filled(r.center(), 18.0, Color32::from_rgb(c.r, c.g, c.b));
                ui.label("#");
                if ui.add(egui::TextEdit::singleline(&mut st.hex).desired_width(80.0)).changed() {
                    if let Ok(v) = u32::from_str_radix(st.hex.trim_start_matches('#'), 16) {
                        let (h, s, vv) = Color::from_hex(v).to_hsv();
                        st.h = h;
                        st.s = s;
                        st.v = vv;
                    }
                }
            });
            let mut c = st.color();
            let (mut r, mut g, mut b) = (c.r, c.g, c.b);
            egui::Grid::new("rgb").num_columns(2).show(ui, |ui| {
                ui.label("Red");
                let a = ui.add(egui::DragValue::new(&mut r)).changed();
                ui.end_row();
                ui.label("Green");
                let bb = ui.add(egui::DragValue::new(&mut g)).changed();
                ui.end_row();
                ui.label("Blue");
                let cc = ui.add(egui::DragValue::new(&mut b)).changed();
                ui.end_row();
                if a || bb || cc {
                    c = Color::rgb(r, g, b);
                    let (h, s, v) = c.to_hsv();
                    st.h = h;
                    st.s = s;
                    st.v = v;
                    st.hex = format!("{:06X}", c.to_hex());
                }
            });
            ok_cancel(ui, "OK")
        });
        if start_pick {
            st.picking = Some(Default::default());
            app.dialogs.edit_colors = Some(st);
            ctx.request_repaint();
            return;
        }
        match res {
            Some(true) => {
                let c = st.color();
                if st.background {
                    app.layer_op(|d| d.background = c);
                } else {
                    if app.editing_color2 {
                        app.color2 = c;
                    } else {
                        app.color1 = c;
                    }
                    app.custom_colors.retain(|x| *x != c);
                    app.custom_colors.insert(0, c);
                    app.custom_colors.truncate(10);
                }
            }
            Some(false) => {}
            None => app.dialogs.edit_colors = Some(st),
        }
    }

    if let Some(action) = app.dialogs.about_unsaved {
        let name = app.title().trim_start_matches('*').trim_end_matches(" - LinuxPaint").to_string();
        let mut choice = None;
        modal(ctx, "LinuxPaint", |ui| {
            ui.label(format!("Do you want to save changes to {name}?"));
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                if adw_button(ui, "Cancel", false, 100.0) {
                    choice = Some(2);
                }
                if adw_button(ui, "Don't save", false, 110.0) {
                    choice = Some(1);
                }
                if adw_button(ui, "Save", true, 100.0) {
                    choice = Some(0);
                }
            });
            None
        });
        match choice {
            Some(0) => {
                app.dialogs.about_unsaved = None;
                if app.save() {
                    run(app, action, Some(ctx));
                }
            }
            Some(1) => {
                app.dialogs.about_unsaved = None;
                app.dirty = false;
                run(app, action, Some(ctx));
            }
            Some(2) => app.dialogs.about_unsaved = None,
            _ => {}
        }
    }

    if let Some(msg) = app.dialogs.message.clone() {
        if modal(ctx, "LinuxPaint", |ui| {
            ui.label(msg);
            ui.add_space(12.0);
            adw_button(ui, "OK", true, ui.available_width()).then_some(true)
        })
        .is_some()
        {
            app.dialogs.message = None;
        }
    }
}

fn eyedropper(ctx: &egui::Context, st: &mut EditColorsState) -> bool {
    let slot = st.picking.clone().unwrap();
    let shot = slot.lock().unwrap().clone();
    let Some(img) = shot else {
        if Arc::strong_count(&slot) == 2 {
            let s2 = slot.clone();
            let c2 = ctx.clone();
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::ScreenshotCallback::new(move |img| {
                *s2.lock().unwrap() = Some(img);
                c2.request_repaint();
            })));
        }
        ctx.request_repaint();
        return false;
    };
    let ppp = ctx.pixels_per_point();
    let screen = ctx.content_rect();
    let sample = |p: egui::Pos2| -> Option<Color32> {
        let x = (p.x * ppp) as i64;
        let y = (p.y * ppp) as i64;
        let [w, h] = img.size;
        (x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h).then(|| img.pixels[y as usize * w + x as usize])
    };
    let mut done = false;
    egui::Area::new(egui::Id::unique("eyedropper")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
        let (r, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
        let key = egui::Id::unique("eyedropper-tex");
        let cached: Option<(usize, egui::TextureHandle)> = ctx.data(|d| d.get_temp(key));
        let tex = match cached {
            Some((ptr, t)) if ptr == Arc::as_ptr(&img) as usize => t,
            _ => {
                let t = ctx.load_texture("eyedropper-shot", (*img).clone(), egui::TextureOptions::NEAREST);
                ctx.data_mut(|d| d.insert_temp(key, (Arc::as_ptr(&img) as usize, t.clone())));
                t
            }
        };
        ui.painter().image(tex.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
        if let Some(p) = resp.hover_pos() {
            let cells = 11;
            let cell = 9.0;
            let size = cells as f32 * cell;
            let mut origin = p + vec2(18.0, 18.0);
            if origin.x + size + 8.0 > screen.right() {
                origin.x = p.x - size - 18.0;
            }
            if origin.y + size + 40.0 > screen.bottom() {
                origin.y = p.y - size - 50.0;
            }
            let pa = ui.painter();
            let frame = Rect::from_min_size(origin, vec2(size, size + 26.0)).expand(4.0);
            pa.rect_filled(frame, 8.0, Color32::from_gray(30));
            for j in 0..cells {
                for i in 0..cells {
                    let q = p + vec2((i as f32 - 5.0) / ppp, (j as f32 - 5.0) / ppp);
                    if let Some(c) = sample(q) {
                        pa.rect_filled(Rect::from_min_size(origin + vec2(i as f32 * cell, j as f32 * cell), vec2(cell, cell)), 0.0, c);
                    }
                }
            }
            let mid = Rect::from_min_size(origin + vec2(5.0 * cell, 5.0 * cell), vec2(cell, cell));
            pa.rect_stroke(mid, 0.0, Stroke::new(2.0, Color32::WHITE), egui::StrokeKind::Outside);
            if let Some(c) = sample(p) {
                pa.text(
                    pos2(origin.x + size / 2.0, origin.y + size + 13.0),
                    Align2::CENTER_CENTER,
                    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b()),
                    egui::FontId::monospace(13.0),
                    Color32::WHITE,
                );
                if resp.clicked() {
                    st.set(Color::rgb(c.r(), c.g(), c.b()));
                    done = true;
                }
            }
        }
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.pointer.secondary_pressed()) {
        done = true;
    }
    if done {
        st.picking = None;
    }
    done
}

fn chrono_like(t: std::time::SystemTime) -> String {
    let secs = t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86400;
    let (y, m, d) = civil(days as i64);
    format!("{d:02}.{m:02}.{y} {:02}:{:02}", (secs % 86400) / 3600, (secs % 3600) / 60)
}

fn civil(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn color_wheel(ui: &mut Ui, pal: &crate::theme::Palette, st: &mut EditColorsState) {
    let size = 200.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());
    let c = rect.center();
    let rad = size / 2.0;
    let key = egui::Id::unique("color-wheel-tex");
    let tex: egui::TextureHandle = match ui.ctx().data(|d| d.get_temp(key)) {
        Some(t) => t,
        None => {
            let n = 400usize;
            let mut px = Vec::with_capacity(n * n);
            for y in 0..n {
                for x in 0..n {
                    let dx = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                    let dy = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                    let r = (dx * dx + dy * dy).sqrt();
                    let a = (1.0 - (r - 1.0) * n as f32 / 2.0).clamp(0.0, 1.0);
                    let col = Color::from_hsv(dy.atan2(dx).to_degrees().rem_euclid(360.0), r.min(1.0), 1.0);
                    px.push(Color32::from_rgba_unmultiplied(col.r, col.g, col.b, (a * 255.0) as u8));
                }
            }
            let img = egui::ColorImage { size: [n, n], source_size: vec2(n as f32, n as f32), pixels: px };
            let t = ui.ctx().load_texture("color-wheel", img, egui::TextureOptions::LINEAR);
            ui.ctx().data_mut(|d| d.insert_temp(key, t.clone()));
            t
        }
    };
    let p = ui.painter();
    p.image(tex.id(), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    let a = st.h.to_radians();
    let marker = pos2(c.x + rad * st.s * a.cos(), c.y + rad * st.s * a.sin());
    p.circle_stroke(marker, 6.0, Stroke::new(2.0, Color32::WHITE));
    p.circle_stroke(marker, 7.0, Stroke::new(1.0, Color32::BLACK));
    if let Some(pos) = resp.interact_pointer_pos() {
        let d = pos - c;
        st.h = d.y.atan2(d.x).to_degrees().rem_euclid(360.0);
        st.s = (d.length() / rad).clamp(0.0, 1.0);
        if st.v < 0.05 {
            st.v = 1.0;
        }
        st.hex = format!("{:06X}", st.color().to_hex());
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let mut v = st.v;
        if crate::ui::adw_slider(ui, pal, &mut v, 0.0, 1.0, false, size - 50.0) {
            st.v = v;
            st.hex = format!("{:06X}", st.color().to_hex());
        }
        ui.label("Value");
    });
}
