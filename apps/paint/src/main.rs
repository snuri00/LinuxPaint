mod app;
mod canvas;
mod clipboard;
mod dialogs;
mod fonts;
mod system;
mod icons;
mod theme;
mod ui;

use eframe::egui::{self, Key, KeyboardShortcut, Modifiers, ViewportCommand};

use app::{PaintApp, Tool};
use canvas::zoom_step;

impl eframe::App for PaintApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let title = self.title();
        if ctx.data(|d| d.get_temp::<String>(egui::Id::unique("win-title"))) != Some(title.clone()) {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            ctx.data_mut(|d| d.insert_temp(egui::Id::unique("win-title"), title));
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.dialogs.about_unsaved = Some(dialogs::PendingAction::Exit);
        }
        if !self.size_from_env && self.file.is_none() && !self.history.can_undo() && !self.dirty {
            if let Some(m) = ctx.input(|i| i.viewport().monitor_size) {
                let (w, h) = ((m.x * ctx.pixels_per_point() * 0.6) as u32, (m.y * ctx.pixels_per_point() * 0.6) as u32);
                if w > 0 && (w != self.doc.w || h != self.doc.h) && self.stroke.is_none() {
                    self.doc = paint_core::Document::new(w, h);
                    self.mark_all();
                }
                self.size_from_env = true;
            }
        }
        resize_edges(&ctx);
        shortcuts(self, &ctx);
        let pal = self.palette;
        egui::Panel::top("titlebar")
            .exact_size(ui::TITLE_H)
            .frame(egui::Frame::new().fill(pal.titlebar))
            .show(ui, |ui| ui::title_bar(self, ui));
        if self.show_ribbon {
            egui::Panel::top("ribbon")
                .exact_size(ui::RIBBON_H)
                .frame(egui::Frame::new().fill(pal.titlebar).inner_margin(egui::Margin { left: 0, right: 0, top: 0, bottom: 6 }))
                .show(ui, |ui| ui::ribbon(self, ui));
        }
        if self.show_statusbar {
            egui::Panel::bottom("status")
                .exact_size(ui::STATUS_H)
                .frame(egui::Frame::new().fill(pal.statusbar))
                .show(ui, |ui| ui::status_bar(self, ui));
        }
        if self.show_layers {
            egui::Panel::right("layers")
                .exact_size(260.0)
                .resizable(false)
                .frame(egui::Frame::new().fill(pal.window).inner_margin(egui::Margin::symmetric(10, 0)))
                .show(ui, |ui| ui::layers_panel(self, ui));
        }
        egui::CentralPanel::default().frame(egui::Frame::new().fill(pal.workspace)).show(ui, |ui| {
            let top = ui.max_rect().left_top();
            canvas::workspace(self, ui);
            if self.text.as_ref().is_some_and(|t| !t.creating) {
                ui::text_toolbar(self, ui, top + egui::vec2(16.0, 8.0));
            }
        });
        dialogs::show(self, &ctx);
        if self.stroke.as_ref().is_some_and(|s| s.stroke.params.kind == paint_core::stroke::BrushKind::Airbrush) {
            ctx.request_repaint();
        }
    }
}

fn resize_edges(ctx: &egui::Context) {
    use egui::{CursorIcon, ResizeDirection as D};
    let (max, full) = ctx.input(|i| (i.viewport().maximized.unwrap_or(false), i.viewport().fullscreen.unwrap_or(false)));
    if max || full {
        return;
    }
    let screen = ctx.content_rect();
    let Some(p) = ctx.input(|i| i.pointer.hover_pos()) else { return };
    let m = 6.0;
    let (l, r, t, b) = (p.x < screen.left() + m, p.x > screen.right() - m, p.y < screen.top() + m, p.y > screen.bottom() - m);
    let dir = match (l, r, t, b) {
        (true, _, true, _) => D::NorthWest,
        (_, true, true, _) => D::NorthEast,
        (true, _, _, true) => D::SouthWest,
        (_, true, _, true) => D::SouthEast,
        (true, _, _, _) => D::West,
        (_, true, _, _) => D::East,
        (_, _, true, _) => D::North,
        (_, _, _, true) => D::South,
        _ => return,
    };
    ctx.set_cursor_icon(match dir {
        D::North | D::South => CursorIcon::ResizeVertical,
        D::East | D::West => CursorIcon::ResizeHorizontal,
        D::NorthWest | D::SouthEast => CursorIcon::ResizeNwSe,
        _ => CursorIcon::ResizeNeSw,
    });
    if ctx.input(|i| i.pointer.primary_pressed()) {
        ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
    }
}

fn shortcuts(app: &mut PaintApp, ctx: &egui::Context) {
    if ctx.egui_wants_keyboard_input() {
        if app.text.is_some() {
            let toggles = ctx.input_mut(|i| {
                [
                    i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::B)),
                    i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::I)),
                    i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::U)),
                ]
            });
            if toggles.iter().any(|t| *t) {
                app.text_style.bold ^= toggles[0];
                app.text_style.italic ^= toggles[1];
                app.text_style.underline ^= toggles[2];
                canvas::render_text(app);
            }
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                app.commit_text();
            }
        }
        return;
    }
    let cmd = Modifiers::COMMAND;
    let cs = Modifiers::COMMAND | Modifiers::SHIFT;
    let sc = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k)));
    if sc(cmd, Key::Z) {
        app.undo();
    }
    if sc(cmd, Key::Y) {
        app.redo();
    }
    if sc(cmd, Key::N) {
        dialogs::guard(app, dialogs::PendingAction::New);
    }
    if sc(cmd, Key::O) {
        dialogs::guard(app, dialogs::PendingAction::Open);
    }
    if sc(cmd, Key::S) {
        app.save();
    }
    if sc(Modifiers::NONE, Key::F12) {
        app.save_as(None);
    }
    if sc(cs, Key::N) {
        app.layer_op(|d| d.add_layer_above());
    }
    if sc(cs, Key::Delete) {
        app.layer_op(|d| d.delete_layer());
    }
    if sc(cs, Key::PageUp) {
        app.layer_op(|d| d.move_layer(true));
    }
    if sc(cs, Key::PageDown) {
        app.layer_op(|d| d.move_layer(false));
    }
    if sc(cs, Key::H) {
        app.layer_op(|d| d.background_visible = !d.background_visible);
    }
    if sc(cs, Key::B) {
        app.dialogs.edit_colors = Some(dialogs::EditColorsState::for_background(app.doc.background));
    }
    if sc(cs, Key::X) {
        app.crop();
    }
    if sc(cs, Key::I) {
        app.invert_colors();
    }
    if sc(cs, Key::S) {
        app.select_tool(Tool::RectSelect);
    }
    if sc(cmd, Key::H) {
        let i = app.doc.current;
        app.layer_op(|d| d.layers[i].visible = !d.layers[i].visible);
    }
    if sc(cmd, Key::A) {
        app.select_all();
    }
    let events = ctx.input(|i| i.events.clone());
    for e in events {
        match e {
            egui::Event::Copy => app.copy(ctx, false),
            egui::Event::Cut => app.copy(ctx, true),
            egui::Event::PasteImage(img) => {
                let px = img.pixels.iter().map(|c| c.to_array()).collect();
                app.paste(Some(paint_core::Image { w: img.size[0] as u32, h: img.size[1] as u32, px }));
            }
            egui::Event::Paste(_) => app.paste(None),
            _ => {}
        }
    }
    if sc(cmd, Key::W) {
        app.dialogs.resize = Some(dialogs::ResizeState::default());
    }
    if sc(cmd, Key::E) {
        app.dialogs.properties = Some(dialogs::PropertiesState::from(app));
    }
    if sc(cmd, Key::R) {
        app.show_rulers = !app.show_rulers;
    }
    if sc(cmd, Key::G) {
        app.show_grid = !app.show_grid;
    }
    if sc(cmd, Key::T) {
        app.show_ribbon = !app.show_ribbon;
    }
    if sc(cmd, Key::PageUp) {
        app.zoom = zoom_step(app.zoom, true);
    }
    if sc(cmd, Key::PageDown) {
        app.zoom = zoom_step(app.zoom, false);
    }
    if sc(cmd, Key::Num0) {
        app.zoom = 1.0;
    }
    if sc(cmd, Key::Num1) {
        app.fit_pending = true;
    }
    if sc(cmd, Key::Plus) || sc(cmd, Key::Equals) {
        let v = app.size() + 1.0;
        app.set_size(v);
    }
    if sc(cmd, Key::Minus) {
        let v = app.size() - 1.0;
        app.set_size(v);
    }
    if sc(Modifiers::NONE, Key::F11) {
        app.fullscreen = !app.fullscreen;
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(app.fullscreen));
    }
    if sc(Modifiers::NONE, Key::Delete) {
        app.delete_selection();
    }
    if sc(Modifiers::NONE, Key::Escape) {
        if app.open_menu.is_some() {
            app.open_menu = None;
        } else if app.shape_session.as_ref().is_some_and(|s| s.kind == paint_core::shapes::ShapeKind::Polygon && s.stage == 0) {
            if let Some(s) = app.shape_session.as_mut() {
                s.stage = 1;
                s.creating = false;
            }
            app.rerender_shape();
        } else {
            app.commit_all();
        }
    }
    if sc(Modifiers::NONE, Key::Enter) {
        app.commit_all();
    }
    for (k, t) in [(Key::P, Tool::Pencil), (Key::B, Tool::Fill), (Key::T, Tool::Text), (Key::E, Tool::Eraser), (Key::I, Tool::Picker), (Key::Z, Tool::Magnifier)] {
        if sc(Modifiers::NONE, k) {
            app.select_tool(t);
        }
    }
}

fn main() -> eframe::Result {
    let path = std::env::args().nth(1).map(std::path::PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LinuxPaint")
            .with_app_id("io.github.snuri00.LinuxPaint")
            .with_inner_size([1280.0, 860.0])
            .with_min_inner_size([520.0, 420.0])
            .with_decorations(false),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native("LinuxPaint", options, Box::new(move |cc| Ok(Box::new(PaintApp::new(cc, path)))))
}
