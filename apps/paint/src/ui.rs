use eframe::egui::{
    self, Align, Align2, Color32, CursorIcon, FontId, Layout, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};
use paint_core::Color;
use paint_core::shapes::ShapeKind;
use paint_core::stroke::BrushKind;
use paint_core::text::Align as TAlign;

use crate::app::{OutlineStyle, PaintApp, Tool};
use crate::canvas::{self, zoom_step};
use crate::dialogs;
use crate::icons::{self, Icon};
use crate::theme::{Palette, ThemeChoice};

pub const TITLE_H: f32 = 47.0;
pub const RIBBON_H: f32 = 100.0;
pub const STATUS_H: f32 = 32.0;

fn c32(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

pub fn icon_button(ui: &mut Ui, pal: &Palette, icon: Icon, size: egui::Vec2, selected: bool, enabled: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, 4.0, pal.selected);
        p.rect_stroke(rect, 4.0, Stroke::new(1.0, pal.selected_border), StrokeKind::Inside);
    } else if enabled && resp.is_pointer_button_down_on() {
        p.rect_filled(rect, 4.0, pal.pressed);
    } else if enabled && resp.hovered() {
        p.rect_filled(rect, 4.0, pal.hover);
    }
    let isz = 20.0f32.min(size.y - 8.0);
    let ir = Rect::from_center_size(rect.center(), vec2(isz, isz));
    icons::draw(p, icon, ir, if enabled { pal.icon } else { pal.text_disabled });
    resp
}

fn big_button(ui: &mut Ui, pal: &Palette, icon: Icon, selected: bool, chevron: bool, w: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 60.0), Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, 4.0, pal.selected);
        p.rect_stroke(rect, 4.0, Stroke::new(1.0, pal.selected_border), StrokeKind::Inside);
    } else if resp.hovered() {
        p.rect_filled(rect, 4.0, pal.hover);
    }
    let top = if chevron { rect.top() + 8.0 } else { rect.center().y - 12.0 };
    icons::draw(p, icon, Rect::from_min_size(pos2(rect.center().x - 12.0, top), vec2(24.0, 24.0)), pal.icon);
    if chevron {
        icons::draw(p, Icon::ChevronDown, Rect::from_center_size(pos2(rect.center().x, rect.bottom() - 13.0), vec2(14.0, 14.0)), pal.icon);
    }
    resp
}

fn group(ui: &mut Ui, pal: &Palette, caption: &str, add: impl FnOnce(&mut Ui)) {
    ui.vertical(|ui| {
        ui.set_height(RIBBON_H - 12.0);
        ui.add_space(6.0);
        ui.allocate_ui_with_layout(vec2(0.0, 64.0), Layout::left_to_right(Align::Center), add);
        let w = ui.min_rect().width();
        let (r, _) = ui.allocate_exact_size(vec2(w, 18.0), Sense::hover());
        ui.painter().text(r.center(), Align2::CENTER_CENTER, caption, FontId::proportional(12.0), pal.text_secondary);
    });
    divider(ui, pal);
}

fn divider(ui: &mut Ui, pal: &Palette) {
    let (r, _) = ui.allocate_exact_size(vec2(13.0, RIBBON_H - 24.0), Sense::hover());
    ui.painter().line_segment([pos2(r.center().x, r.top() + 10.0), pos2(r.center().x, r.bottom())], Stroke::new(1.0, pal.divider));
}

pub fn popup(app: &mut PaintApp, ui: &Ui, id: &'static str, anchor: Rect, width: f32, add: impl FnOnce(&mut PaintApp, &mut Ui) -> bool) {
    if app.open_menu != Some(id) {
        return;
    }
    let pal = app.palette;
    let area = egui::Area::new(egui::Id::unique(id))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor.left_bottom() + vec2(0.0, 4.0))
        .constrain(true);
    let resp = area.show(ui.ctx(), |ui| {
        egui::Frame::new()
            .fill(pal.flyout)
            .stroke(Stroke::new(1.0, pal.card_stroke))
            .corner_radius(8.0)
            .shadow(egui::Shadow { offset: [0, 8], blur: 16, spread: 0, color: Color32::from_black_alpha(if pal.dark { 90 } else { 40 }) })
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| {
                ui.set_width(width);
                add(app, ui)
            })
            .inner
    });
    let close_clicked = resp.inner;
    let clicked_outside = ui.ctx().input(|i| i.pointer.any_pressed())
        && ui.ctx().input(|i| i.pointer.interact_pos()).is_some_and(|p| !resp.response.rect.contains(p) && !anchor.contains(p));
    if close_clicked || clicked_outside || ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
        app.open_menu = None;
    }
}

pub fn menu_item(ui: &mut Ui, pal: &Palette, icon: Option<Icon>, label: &str, shortcut: &str, enabled: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), if enabled { Sense::click() } else { Sense::hover() });
    let p = ui.painter();
    if enabled && resp.hovered() {
        p.rect_filled(rect, 4.0, pal.hover);
    }
    let col = if enabled { pal.text } else { pal.text_disabled };
    let mut x = rect.left() + 12.0;
    if let Some(ic) = icon {
        icons::draw(p, ic, Rect::from_center_size(pos2(x + 8.0, rect.center().y), vec2(16.0, 16.0)), col);
        x += 28.0;
    }
    p.text(pos2(x, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(14.0), col);
    if !shortcut.is_empty() {
        p.text(pos2(rect.right() - 12.0, rect.center().y), Align2::RIGHT_CENTER, shortcut, FontId::proportional(12.0), pal.text_secondary);
    }
    enabled && resp.clicked()
}

fn menu_sep(ui: &mut Ui, pal: &Palette) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 9.0), Sense::hover());
    ui.painter().line_segment([pos2(r.left(), r.center().y), pos2(r.right(), r.center().y)], Stroke::new(1.0, pal.divider));
}

fn check_item(ui: &mut Ui, pal: &Palette, label: &str, shortcut: &str, on: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
    let p = ui.painter();
    if resp.hovered() {
        p.rect_filled(rect, 4.0, pal.hover);
    }
    if on {
        let c = pos2(rect.left() + 18.0, rect.center().y);
        let s = Stroke::new(1.6, pal.text);
        p.line_segment([c + vec2(-5.0, 0.0), c + vec2(-1.5, 3.5)], s);
        p.line_segment([c + vec2(-1.5, 3.5), c + vec2(5.0, -4.0)], s);
    }
    p.text(pos2(rect.left() + 36.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(14.0), pal.text);
    if !shortcut.is_empty() {
        p.text(pos2(rect.right() - 12.0, rect.center().y), Align2::RIGHT_CENTER, shortcut, FontId::proportional(12.0), pal.text_secondary);
    }
    resp.clicked()
}

pub fn adw_slider(ui: &mut Ui, pal: &Palette, value: &mut f32, lo: f32, hi: f32, log: bool, width: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click_and_drag());
    let to_t = |v: f32| if log { (v.max(lo).ln() - lo.ln()) / (hi.ln() - lo.ln()) } else { (v - lo) / (hi - lo) };
    let from_t = |t: f32| if log { (lo.ln() + t * (hi.ln() - lo.ln())).exp() } else { lo + t * (hi - lo) };
    let track = rect.shrink2(vec2(9.0, 0.0));
    let mut changed = false;
    if let Some(p) = resp.interact_pointer_pos() {
        let t = ((p.x - track.left()) / track.width()).clamp(0.0, 1.0);
        let nv = from_t(t);
        if (nv - *value).abs() > f32::EPSILON {
            *value = nv;
            changed = true;
        }
    }
    let t = to_t(*value).clamp(0.0, 1.0);
    let y = rect.center().y;
    let x = track.left() + t * track.width();
    let p = ui.painter();
    p.rect_filled(Rect::from_min_max(pos2(track.left(), y - 2.0), pos2(track.right(), y + 2.0)), 2.0, pal.control_stroke);
    p.rect_filled(Rect::from_min_max(pos2(track.left(), y - 2.0), pos2(x, y + 2.0)), 2.0, pal.accent);
    p.circle_filled(pos2(x, y + 1.0), 9.5, Color32::from_black_alpha(60));
    p.circle_filled(pos2(x, y), 9.0, Color32::WHITE);
    if resp.hovered() || resp.dragged() {
        p.circle_stroke(pos2(x, y), 9.0, Stroke::new(1.0, Color32::from_black_alpha(40)));
    }
    changed
}

fn text_button(ui: &mut Ui, pal: &Palette, label: &str, open: bool) -> Response {
    let galley = ui.painter().layout_no_wrap(label.to_string(), FontId::proportional(14.0), pal.text);
    let (rect, resp) = ui.allocate_exact_size(vec2(galley.size().x + 22.0, 32.0), Sense::click());
    if open || resp.hovered() {
        ui.painter().rect_filled(rect, 4.0, if open { pal.pressed } else { pal.hover });
    }
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, pal.text);
    resp
}

fn window_button(ui: &mut Ui, pal: &Palette, kind: &str, maximized: bool) -> Response {
    let (r, resp) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::click());
    let c = r.center();
    let base = if pal.dark { Color32::from_white_alpha(26) } else { Color32::from_black_alpha(20) };
    let fill = if resp.is_pointer_button_down_on() {
        if pal.dark { Color32::from_white_alpha(56) } else { Color32::from_black_alpha(46) }
    } else if resp.hovered() {
        if pal.dark { Color32::from_white_alpha(38) } else { Color32::from_black_alpha(31) }
    } else {
        base
    };
    let p = ui.painter();
    p.circle_filled(c, 12.0, fill);
    let s = Stroke::new(1.5, pal.icon);
    match kind {
        "minimize" => {
            p.line_segment([c + vec2(-4.0, 3.5), c + vec2(4.0, 3.5)], s);
        }
        "maximize" if maximized => {
            p.rect_stroke(Rect::from_center_size(c + vec2(-1.0, 1.0), vec2(7.0, 7.0)), 1.0, s, StrokeKind::Middle);
            p.line_segment([c + vec2(-2.0, -3.5), c + vec2(3.5, -3.5)], s);
            p.line_segment([c + vec2(3.5, -3.5), c + vec2(3.5, 2.0)], s);
        }
        "maximize" => {
            p.rect_stroke(Rect::from_center_size(c, vec2(8.0, 8.0)), 1.0, s, StrokeKind::Middle);
        }
        _ => {
            p.line_segment([c + vec2(-4.0, -4.0), c + vec2(4.0, 4.0)], s);
            p.line_segment([c + vec2(4.0, -4.0), c + vec2(-4.0, 4.0)], s);
        }
    }
    resp
}

fn window_buttons(app: &mut PaintApp, ui: &mut Ui, kinds: &[String]) {
    let pal = app.palette;
    let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
    for k in kinds {
        let resp = window_button(ui, &pal, k, max).on_hover_text(match k.as_str() {
            "minimize" => "Minimize",
            "maximize" if max => "Restore",
            "maximize" => "Maximize",
            _ => "Close",
        });
        if resp.clicked() {
            match k.as_str() {
                "minimize" => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true)),
                "maximize" => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max)),
                _ => app.request_close(ui.ctx()),
            }
        }
    }
}

pub fn title_bar(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    let rect = ui.max_rect();
    let drag = ui.interact(rect, ui.scope_id().with("headerbar-drag"), Sense::click_and_drag());
    if drag.drag_started_by(egui::PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if drag.double_clicked() {
        let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
    }
    ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, pal.ribbon_border));
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, app.title(), FontId::new(14.0, egui::FontFamily::Name("bold".into())), pal.text);
    let left = app.look.buttons_left.clone();
    let right = app.look.buttons_right.clone();
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.add_space(6.0);
        if !left.is_empty() {
            window_buttons(app, ui, &left);
            ui.add_space(8.0);
        }
        for (id, label) in [("file", "File"), ("edit", "Edit"), ("view", "View")] {
            let r = text_button(ui, &pal, label, app.open_menu == Some(id));
            if r.clicked() {
                app.open_menu = if app.open_menu == Some(id) { None } else { Some(id) };
            }
            menus(app, ui, id, r.rect);
        }
        ui.add_space(6.0);
        let can_undo = app.history.can_undo() || app.stroke.is_some() || app.shape_session.is_some() || app.text.is_some();
        if icon_button(ui, &pal, Icon::Undo, vec2(34.0, 34.0), false, can_undo).on_hover_text("Undo (Ctrl+Z)").clicked() {
            app.undo();
        }
        let can_redo = app.history.can_redo();
        if icon_button(ui, &pal, Icon::Redo, vec2(34.0, 34.0), false, can_redo).on_hover_text("Redo (Ctrl+Y)").clicked() {
            app.redo();
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.add_space(6.0);
            let mut r = right.clone();
            r.reverse();
            window_buttons(app, ui, &r);
            if !right.is_empty() {
                ui.add_space(4.0);
            }
            let open = app.open_menu == Some("settings");
            let resp = icon_button(ui, &pal, Icon::Settings, vec2(34.0, 34.0), open, true).on_hover_text("Main Menu");
            if resp.clicked() {
                app.open_menu = if open { None } else { Some("settings") };
            }
            let anchor = Rect::from_min_max(pos2(resp.rect.right() - 280.0, resp.rect.top()), resp.rect.max);
            popup(app, ui, "settings", anchor, 264.0, |app, ui| {
                ui.add_space(6.0);
                ui.spacing_mut().indent = 10.0;
                egui::Frame::new().inner_margin(egui::Margin::symmetric(10, 2)).show(ui, |ui| {
                ui.label(egui::RichText::new("Appearance").family(egui::FontFamily::Name("bold".into())));
                ui.label(egui::RichText::new("App theme").color(pal.text_secondary));
                let mut choice = app.theme_choice;
                ui.radio_value(&mut choice, ThemeChoice::System, "Use system setting");
                ui.radio_value(&mut choice, ThemeChoice::Light, "Light");
                ui.radio_value(&mut choice, ThemeChoice::Dark, "Dark");
                if choice != app.theme_choice {
                    app.set_theme(ui.ctx(), choice);
                }
                menu_sep(ui, &pal);
                ui.label(egui::RichText::new("About this app").family(egui::FontFamily::Name("bold".into())));
                ui.label("3DPaint for Linux");
                ui.add_space(4.0);
                });
                false
            });
        });
    });
}

fn menus(app: &mut PaintApp, ui: &Ui, id: &'static str, anchor: Rect) {
    let pal = app.palette;
    match id {
        "file" => popup(app, ui, id, anchor, 300.0, |app, ui| {
            let mut close = false;
            if menu_item(ui, &pal, None, "New", "Ctrl+N", true) {
                dialogs::guard(app, dialogs::PendingAction::New);
                close = true;
            }
            if menu_item(ui, &pal, None, "Open", "Ctrl+O", true) {
                dialogs::guard(app, dialogs::PendingAction::Open);
                close = true;
            }
            if menu_item(ui, &pal, None, "Import to canvas", "", true) {
                if let Some(p) = rfd::FileDialog::new().add_filter("Pictures", &["png", "jpg", "jpeg", "bmp", "gif", "tif", "tiff", "webp"]).pick_file() {
                    if let Ok(i) = image::open(p) {
                        app.paste(Some(paint_core::Image::from_dynamic(&i)));
                    }
                }
                close = true;
            }
            menu_sep(ui, &pal);
            if menu_item(ui, &pal, None, "Save", "Ctrl+S", true) {
                app.save();
                close = true;
            }
            for (label, ext) in [("Save as PNG image", "png"), ("Save as JPEG picture", "jpg"), ("Save as BMP picture", "bmp"), ("Save as GIF picture", "gif"), ("Save as project", "paint")] {
                if menu_item(ui, &pal, None, label, "", true) {
                    app.save_as(Some(ext));
                    close = true;
                }
            }
            if menu_item(ui, &pal, None, "Save as other format", "F12", true) {
                app.save_as(None);
                close = true;
            }
            menu_sep(ui, &pal);
            if menu_item(ui, &pal, None, "Image properties", "Ctrl+E", true) {
                app.dialogs.properties = Some(dialogs::PropertiesState::from(app));
                close = true;
            }
            menu_sep(ui, &pal);
            if menu_item(ui, &pal, None, "Exit", "", true) {
                app.request_close(ui.ctx());
                close = true;
            }
            close
        }),
        "edit" => popup(app, ui, id, anchor, 260.0, |app, ui| {
            let mut close = false;
            if menu_item(ui, &pal, None, "Undo", "Ctrl+Z", app.history.can_undo()) {
                app.undo();
                close = true;
            }
            if menu_item(ui, &pal, None, "Redo", "Ctrl+Y", app.history.can_redo()) {
                app.redo();
                close = true;
            }
            menu_sep(ui, &pal);
            let has_sel = app.selection.is_some();
            if menu_item(ui, &pal, None, "Cut", "Ctrl+X", has_sel) {
                app.copy(ui.ctx(), true);
                close = true;
            }
            if menu_item(ui, &pal, None, "Copy", "Ctrl+C", has_sel) {
                app.copy(ui.ctx(), false);
                close = true;
            }
            if menu_item(ui, &pal, None, "Paste", "Ctrl+V", true) {
                app.paste(None);
                close = true;
            }
            close
        }),
        "view" => popup(app, ui, id, anchor, 280.0, |app, ui| {
            let mut close = false;
            if menu_item(ui, &pal, Some(Icon::ZoomIn), "Zoom in", "Ctrl+PgUp", true) {
                app.zoom = zoom_step(app.zoom, true);
                close = true;
            }
            if menu_item(ui, &pal, Some(Icon::ZoomOut), "Zoom out", "Ctrl+PgDn", true) {
                app.zoom = zoom_step(app.zoom, false);
                close = true;
            }
            if menu_item(ui, &pal, Some(Icon::Fit), "Fit to window", "Ctrl+1", true) {
                app.fit_pending = true;
                close = true;
            }
            if menu_item(ui, &pal, None, "Zoom to 100%", "Ctrl+0", true) {
                app.zoom = 1.0;
                close = true;
            }
            menu_sep(ui, &pal);
            if check_item(ui, &pal, "Rulers", "Ctrl+R", app.show_rulers) {
                app.show_rulers = !app.show_rulers;
                close = true;
            }
            if check_item(ui, &pal, "Gridlines", "Ctrl+G", app.show_grid) {
                app.show_grid = !app.show_grid;
                close = true;
            }
            if check_item(ui, &pal, "Status bar", "", app.show_statusbar) {
                app.show_statusbar = !app.show_statusbar;
                close = true;
            }
            menu_sep(ui, &pal);
            if menu_item(ui, &pal, None, "Full screen", "F11", true) {
                app.fullscreen = !app.fullscreen;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(app.fullscreen));
                close = true;
            }
            close
        }),
        _ => {}
    }
}

pub fn ribbon(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect.shrink2(vec2(6.0, 0.0)), 8.0, pal.ribbon);
    ui.painter().rect_stroke(rect.shrink2(vec2(6.0, 0.0)), 8.0, Stroke::new(1.0, pal.ribbon_border), StrokeKind::Inside);
    egui::ScrollArea::horizontal().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
            ui.add_space(16.0);
            selection_group(app, ui);
            image_group(app, ui);
            tools_group(app, ui);
            brushes_group(app, ui);
            shapes_group(app, ui);
            size_group(app, ui);
            colors_group(app, ui);
            group(ui, &pal, "Layers", |ui| {
                if big_button(ui, &pal, Icon::Layers, app.show_layers, false, 44.0).on_hover_text("Layers").clicked() {
                    app.show_layers = !app.show_layers;
                }
            });
        });
    });
}

fn selection_group(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    group(ui, &pal, "Selection", |ui| {
        let sel = matches!(app.tool, Tool::RectSelect | Tool::FreeSelect);
        let ic = if app.tool == Tool::FreeSelect { Icon::FreeSelect } else { Icon::RectSelect };
        let r = big_button(ui, &pal, ic, sel, true, 44.0).on_hover_text("Select (Ctrl+Shift+S)");
        if r.clicked() {
            if !sel {
                app.select_tool(Tool::RectSelect);
            }
            app.open_menu = if app.open_menu == Some("select") { None } else { Some("select") };
        }
        popup(app, ui, "select", r.rect, 240.0, |app, ui| {
            let mut c = false;
            if menu_item(ui, &pal, Some(Icon::RectSelect), "Rectangular selection", "", true) {
                app.select_tool(Tool::RectSelect);
                c = true;
            }
            if menu_item(ui, &pal, Some(Icon::FreeSelect), "Free-form selection", "", true) {
                app.select_tool(Tool::FreeSelect);
                c = true;
            }
            menu_sep(ui, &pal);
            if menu_item(ui, &pal, None, "Select all", "Ctrl+A", true) {
                app.select_all();
                c = true;
            }
            if menu_item(ui, &pal, None, "Invert selection", "", app.selection.is_some()) {
                app.invert_selection();
                c = true;
            }
            if menu_item(ui, &pal, None, "Delete", "Delete", app.selection.is_some()) {
                app.delete_selection();
                c = true;
            }
            menu_sep(ui, &pal);
            if check_item(ui, &pal, "Transparent selection", "", app.transparent_selection) {
                app.transparent_selection = !app.transparent_selection;
                app.render_selection();
                c = true;
            }
            c
        });
    });
}

fn image_group(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    group(ui, &pal, "Image", |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if icon_button(ui, &pal, Icon::Crop, vec2(32.0, 30.0), false, app.selection.is_some()).on_hover_text("Crop (Ctrl+Shift+X)").clicked() {
                    app.crop();
                }
                if icon_button(ui, &pal, Icon::Resize, vec2(32.0, 30.0), false, true).on_hover_text("Resize and skew (Ctrl+W)").clicked() {
                    app.dialogs.resize = Some(dialogs::ResizeState::default());
                }
            });
            ui.horizontal(|ui| {
                let r = icon_button(ui, &pal, Icon::Rotate, vec2(32.0, 30.0), app.open_menu == Some("rotate"), true).on_hover_text("Rotate");
                if r.clicked() {
                    app.open_menu = Some("rotate");
                }
                popup(app, ui, "rotate", r.rect, 220.0, |app, ui| {
                    let mut c = false;
                    if menu_item(ui, &pal, None, "Rotate right 90°", "", true) {
                        app.image_op(paint_core::transform::rotate_cw);
                        c = true;
                    }
                    if menu_item(ui, &pal, None, "Rotate left 90°", "", true) {
                        app.image_op(paint_core::transform::rotate_ccw);
                        c = true;
                    }
                    if menu_item(ui, &pal, None, "Rotate 180°", "", true) {
                        app.image_op(paint_core::transform::rotate_180);
                        c = true;
                    }
                    if menu_item(ui, &pal, None, "Custom rotate", "", true) {
                        app.dialogs.rotate = Some(0.0);
                        c = true;
                    }
                    c
                });
                let r = icon_button(ui, &pal, Icon::Flip, vec2(32.0, 30.0), app.open_menu == Some("flip"), true).on_hover_text("Flip");
                if r.clicked() {
                    app.open_menu = Some("flip");
                }
                popup(app, ui, "flip", r.rect, 200.0, |app, ui| {
                    let mut c = false;
                    if menu_item(ui, &pal, None, "Flip vertical", "", true) {
                        app.image_op(paint_core::transform::flip_v);
                        c = true;
                    }
                    if menu_item(ui, &pal, None, "Flip horizontal", "", true) {
                        app.image_op(paint_core::transform::flip_h);
                        c = true;
                    }
                    c
                });
            });
        });
    });
}

fn tools_group(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    group(ui, &pal, "Tools", |ui| {
        ui.vertical(|ui| {
            for row in [[(Tool::Pencil, Icon::Pencil, "Pencil"), (Tool::Fill, Icon::Fill, "Fill"), (Tool::Text, Icon::Text, "Text")], [
                (Tool::Eraser, Icon::Eraser, "Eraser"),
                (Tool::Picker, Icon::Picker, "Color picker"),
                (Tool::Magnifier, Icon::Magnifier, "Magnifier"),
            ]] {
                ui.horizontal(|ui| {
                    for (t, ic, tip) in row {
                        if icon_button(ui, &pal, ic, vec2(32.0, 30.0), app.tool == t, true).on_hover_text(tip).clicked() {
                            app.select_tool(t);
                        }
                    }
                });
            }
        });
    });
}

pub const BRUSHES: [(BrushKind, &str); 9] = [
    (BrushKind::Brush, "Brush"),
    (BrushKind::Calligraphy45, "Calligraphy brush"),
    (BrushKind::Calligraphy135, "Calligraphy pen"),
    (BrushKind::Airbrush, "Airbrush"),
    (BrushKind::Oil, "Oil brush"),
    (BrushKind::Crayon, "Crayon"),
    (BrushKind::Marker, "Marker"),
    (BrushKind::NaturalPencil, "Natural pencil"),
    (BrushKind::Watercolor, "Watercolor brush"),
];

fn brush_preview(p: &egui::Painter, kind: BrushKind, r: Rect, col: Color32) {
    let pts: Vec<Pos2> = (0..=20)
        .map(|i| {
            let t = i as f32 / 20.0;
            pos2(r.left() + r.width() * t, r.center().y + (t * std::f32::consts::TAU).sin() * r.height() * 0.25)
        })
        .collect();
    let w = match kind {
        BrushKind::Calligraphy45 | BrushKind::Calligraphy135 => 2.0,
        BrushKind::Brush => 3.0,
        BrushKind::NaturalPencil => 1.5,
        _ => 5.0,
    };
    let a = match kind {
        BrushKind::Marker | BrushKind::Watercolor => 0.55,
        BrushKind::Airbrush => 0.35,
        _ => 1.0,
    };
    p.add(egui::Shape::line(pts, Stroke::new(w, col.gamma_multiply(a))));
}

fn brushes_group(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    group(ui, &pal, "Brushes", |ui| {
        let r = big_button(ui, &pal, Icon::Brush, app.tool == Tool::Brush, true, 44.0).on_hover_text("Brushes");
        if r.clicked() {
            app.select_tool(Tool::Brush);
            app.open_menu = if app.open_menu == Some("brushes") { None } else { Some("brushes") };
        }
        popup(app, ui, "brushes", r.rect, 230.0, |app, ui| {
            let mut c = false;
            for (k, name) in BRUSHES {
                let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
                if app.brush == k {
                    ui.painter().rect_filled(rect, 4.0, pal.selected);
                } else if resp.hovered() {
                    ui.painter().rect_filled(rect, 4.0, pal.hover);
                }
                brush_preview(ui.painter(), k, Rect::from_min_size(rect.min + vec2(10.0, 8.0), vec2(44.0, 20.0)), pal.icon);
                ui.painter().text(pos2(rect.left() + 66.0, rect.center().y), Align2::LEFT_CENTER, name, FontId::proportional(14.0), pal.text);
                if resp.clicked() {
                    app.brush = k;
                    app.select_tool(Tool::Brush);
                    c = true;
                }
            }
            c
        });
    });
}

fn shapes_group(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    group(ui, &pal, "Shapes", |ui| {
        let (gallery, _) = ui.allocate_exact_size(vec2(7.0 * 26.0 + 4.0, 3.0 * 22.0), Sense::hover());
        let p = ui.painter();
        p.rect_stroke(gallery, 4.0, Stroke::new(1.0, pal.divider), StrokeKind::Inside);
        let scroll_id = egui::Id::unique("shape-row");
        let row0: usize = ui.ctx().data(|d| d.get_temp(scroll_id)).unwrap_or(0);
        let rows = ShapeKind::ALL.len().div_ceil(6);
        for (i, k) in ShapeKind::ALL.iter().enumerate().skip(row0 * 6).take(18) {
            let idx = i - row0 * 6;
            let cell = Rect::from_min_size(gallery.min + vec2(2.0 + (idx % 6) as f32 * 26.0, 1.0 + (idx / 6) as f32 * 21.5), vec2(24.0, 20.0));
            let resp = ui.interact(cell, ui.scope_id().with(("shape", i)), Sense::click()).on_hover_text(k.name());
            let sel = app.tool == Tool::Shape && app.shape == *k;
            if sel {
                ui.painter().rect_filled(cell, 3.0, pal.selected);
                ui.painter().rect_stroke(cell, 3.0, Stroke::new(1.0, pal.selected_border), StrokeKind::Inside);
            } else if resp.hovered() {
                ui.painter().rect_filled(cell, 3.0, pal.hover);
            }
            icons::draw_shape(ui.painter(), *k, Rect::from_center_size(cell.center(), vec2(18.0, 16.0)), pal.icon);
            if resp.clicked() {
                app.shape = *k;
                app.select_tool(Tool::Shape);
            }
        }
        let arrows = Rect::from_min_max(pos2(gallery.right() - 24.0, gallery.top() + 1.0), pos2(gallery.right() - 2.0, gallery.bottom() - 1.0));
        let up = Rect::from_min_size(arrows.min, vec2(22.0, arrows.height() / 3.0));
        let down = up.translate(vec2(0.0, arrows.height() / 3.0));
        let more = down.translate(vec2(0.0, arrows.height() / 3.0));
        for (r, ic, delta) in [(up, Icon::ScrollUp, -1i32), (down, Icon::ScrollDown, 1)] {
            let resp = ui.interact(r, ui.scope_id().with(("shape-scroll", delta)), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(r, 2.0, pal.hover);
            }
            icons::draw(ui.painter(), ic, r.shrink(4.0), pal.icon);
            if resp.clicked() {
                let nv = (row0 as i32 + delta).clamp(0, rows as i32 - 3) as usize;
                ui.ctx().data_mut(|d| d.insert_temp(scroll_id, nv));
            }
        }
        let mresp = ui.interact(more, ui.scope_id().with("shape-more"), Sense::click());
        icons::draw(ui.painter(), Icon::More, more.shrink(4.0), pal.icon);
        if mresp.clicked() {
            app.open_menu = Some("shapes-all");
        }
        popup(app, ui, "shapes-all", gallery, 6.0 * 30.0, |app, ui| {
            let mut c = false;
            egui::Grid::new("all-shapes").spacing(vec2(2.0, 2.0)).show(ui, |ui| {
                for (i, k) in ShapeKind::ALL.iter().enumerate() {
                    if icon_shape_button(ui, &pal, *k, app.tool == Tool::Shape && app.shape == *k).on_hover_text(k.name()).clicked() {
                        app.shape = *k;
                        app.select_tool(Tool::Shape);
                        c = true;
                    }
                    if i % 6 == 5 {
                        ui.end_row();
                    }
                }
            });
            c
        });
        ui.add_space(6.0);
        ui.vertical(|ui| {
            for (id, ic, label, is_fill) in [("outline", Icon::Outline, "Outline", false), ("fill", Icon::FillShape, "Fill", true)] {
                let (r, resp) = ui.allocate_exact_size(vec2(52.0, 30.0), Sense::click());
                if resp.hovered() || app.open_menu == Some(id) {
                    ui.painter().rect_filled(r, 4.0, pal.hover);
                }
                icons::draw(ui.painter(), ic, Rect::from_min_size(r.min + vec2(6.0, 5.0), vec2(20.0, 20.0)), pal.icon);
                icons::draw(ui.painter(), Icon::ChevronDown, Rect::from_min_size(r.min + vec2(32.0, 9.0), vec2(12.0, 12.0)), pal.icon);
                let resp = resp.on_hover_text(if is_fill { "Shape fill" } else { "Shape outline" });
                if resp.clicked() {
                    app.open_menu = Some(id);
                }
                popup(app, ui, id, r, 220.0, |app, ui| {
                    let mut c = false;
                    let opts = [
                        (OutlineStyle::None, if is_fill { "No fill" } else { "No outline" }),
                        (OutlineStyle::Solid, if is_fill { "Solid fill" } else { "Solid color" }),
                        (OutlineStyle::Textured(BrushKind::Crayon), "Crayon"),
                        (OutlineStyle::Textured(BrushKind::Marker), "Marker"),
                        (OutlineStyle::Textured(BrushKind::Oil), "Oil"),
                        (OutlineStyle::Textured(BrushKind::NaturalPencil), "Natural pencil"),
                        (OutlineStyle::Textured(BrushKind::Watercolor), "Watercolor"),
                    ];
                    for (o, name) in opts {
                        let cur = if is_fill { app.fill } else { app.outline };
                        if check_item(ui, &pal, name, "", cur == o) {
                            if is_fill {
                                app.fill = o;
                            } else {
                                app.outline = o;
                            }
                            app.rerender_shape();
                            c = true;
                        }
                    }
                    c
                });
                let _ = label;
            }
        });
    });
}

fn icon_shape_button(ui: &mut Ui, pal: &Palette, k: ShapeKind, sel: bool) -> Response {
    let (r, resp) = ui.allocate_exact_size(vec2(28.0, 26.0), Sense::click());
    if sel {
        ui.painter().rect_filled(r, 3.0, pal.selected);
    } else if resp.hovered() {
        ui.painter().rect_filled(r, 3.0, pal.hover);
    }
    icons::draw_shape(ui.painter(), k, r.shrink(4.0), pal.icon);
    resp
}

fn size_group(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    group(ui, &pal, "Size", |ui| {
        let r = big_button(ui, &pal, Icon::Size, app.open_menu == Some("size"), true, 44.0).on_hover_text("Size");
        if r.clicked() {
            app.open_menu = if app.open_menu == Some("size") { None } else { Some("size") };
        }
        popup(app, ui, "size", r.rect, 260.0, |app, ui| {
            ui.add_space(6.0);
            let enabled = !matches!(app.tool, Tool::Fill | Tool::Picker | Tool::Magnifier | Tool::Text | Tool::RectSelect | Tool::FreeSelect);
            if app.tool == Tool::Fill {
                ui.label("Tolerance");
                let mut t = app.tolerance * 100.0;
                ui.horizontal(|ui| {
                    adw_slider(ui, &pal, &mut t, 0.0, 100.0, false, 180.0);
                    ui.label(format!("{}%", t.round()));
                });
                app.tolerance = t.round() / 100.0;
            } else if enabled {
                ui.label("Size");
                let (lo, hi) = app.size_range();
                let mut v = app.size();
                ui.horizontal(|ui| {
                    if adw_slider(ui, &pal, &mut v, lo, hi, true, 180.0) {
                        app.set_size(v);
                    }
                    ui.label(format!("{}px", app.size()));
                });
                if app.tool == Tool::Eraser {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut app.eraser_square, false, "Round eraser");
                        ui.selectable_value(&mut app.eraser_square, true, "Square eraser");
                    });
                }
                if let Some(k) = app.opacity_key() {
                    ui.add_space(6.0);
                    ui.label("Opacity");
                    let mut o = app.opacity[&k] * 100.0;
                    ui.horizontal(|ui| {
                        adw_slider(ui, &pal, &mut o, 0.0, 100.0, false, 180.0);
                        ui.label(format!("{}%", o.round()));
                    });
                    app.opacity.insert(k, o.round() / 100.0);
                }
            } else {
                ui.label(egui::RichText::new("Select a drawing tool to change its size.").color(pal.text_secondary));
            }
            ui.add_space(6.0);
            false
        });
    });
}

fn swatch(ui: &mut Ui, pal: &Palette, c: Option<Color>, d: f32, selected: bool) -> Response {
    let (r, resp) = ui.allocate_exact_size(vec2(d + 4.0, d + 4.0), Sense::click());
    let p = ui.painter();
    let center = r.center();
    if selected {
        p.circle_stroke(center, d / 2.0 + 1.5, Stroke::new(1.5, pal.accent));
    } else if resp.hovered() {
        p.circle_stroke(center, d / 2.0 + 1.5, Stroke::new(1.0, pal.text_secondary));
    }
    match c {
        Some(c) => {
            p.circle_filled(center, d / 2.0, c32(c));
            p.circle_stroke(center, d / 2.0, Stroke::new(1.0, pal.control_stroke));
        }
        None => {
            p.circle_stroke(center, d / 2.0 - 0.5, Stroke::new(1.0, pal.divider));
        }
    }
    resp
}

fn colors_group(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    group(ui, &pal, "Colors", |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            if swatch(ui, &pal, Some(app.color1), 24.0, !app.editing_color2).on_hover_text("Color 1 (foreground)").clicked() {
                app.editing_color2 = false;
            }
            if swatch(ui, &pal, Some(app.color2), 24.0, app.editing_color2).on_hover_text("Color 2 (background)").clicked() {
                app.editing_color2 = true;
            }
        });
        ui.add_space(4.0);
        let colors = PaintApp::palette_colors();
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            ui.spacing_mut().interact_size.y = 19.0;
            for row in 0..3 {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    for col in 0..10 {
                        let c = if row < 2 { Some(colors[row * 10 + col]) } else { app.custom_colors.get(col).copied() };
                        let resp = swatch(ui, &pal, c, 15.0, false);
                        if let Some(c) = c {
                            let resp = resp.on_hover_text(format!("#{:06X}", c.to_hex()));
                            if resp.clicked() {
                                if app.editing_color2 {
                                    app.color2 = c;
                                } else {
                                    app.color1 = c;
                                }
                                app.rerender_shape();
                                crate::canvas::render_text(app);
                            }
                            if resp.secondary_clicked() {
                                app.color2 = c;
                            }
                        }
                    }
                });
            }
        });
        ui.add_space(4.0);
        let (r, resp) = ui.allocate_exact_size(vec2(36.0, 60.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r, 4.0, pal.hover);
        }
        icons::draw(ui.painter(), Icon::EditColors, Rect::from_center_size(r.center(), vec2(26.0, 26.0)), pal.icon);
        if resp.on_hover_text("Edit colors").clicked() {
            let c = if app.editing_color2 { app.color2 } else { app.color1 };
            app.dialogs.edit_colors = Some(dialogs::EditColorsState::new(c));
        }
    });
}

pub fn status_bar(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    let rect = ui.max_rect();
    ui.painter().line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, pal.divider));
    ui.horizontal_centered(|ui| {
        ui.add_space(12.0);
        let small = FontId::proportional(12.0);
        let item = |ui: &mut Ui, ic: Icon, text: String, w: f32| {
            let (r, _) = ui.allocate_exact_size(vec2(w, 24.0), Sense::hover());
            icons::draw(ui.painter(), ic, Rect::from_min_size(r.min + vec2(0.0, 4.0), vec2(16.0, 16.0)), pal.icon);
            ui.painter().text(pos2(r.left() + 24.0, r.center().y), Align2::LEFT_CENTER, text, small.clone(), pal.text);
        };
        let pos = app.cursor.filter(|(x, y)| *x >= 0 && *y >= 0 && *x < app.doc.w as i32 && *y < app.doc.h as i32);
        item(ui, Icon::Cursor, pos.map(|(x, y)| format!("{x}, {y}px")).unwrap_or_default(), 130.0);
        let width = ui.available_width();
        if width > 600.0 {
            item(ui, Icon::SelectionSize, canvas::selection_size(app).map(|(w, h)| format!("{w} × {h}px")).unwrap_or_default(), 130.0);
        }
        if width > 480.0 {
            item(ui, Icon::CanvasSize, format!("{} × {}px", app.doc.w, app.doc.h), 140.0);
        }
        if width > 700.0 {
            let sz = app.last_saved_size.map(|b| human_size(b)).unwrap_or_default();
            item(ui, Icon::Save, if app.file.is_some() { format!("Size: {sz}") } else { String::new() }, 140.0);
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(10.0);
            if icon_button(ui, &pal, Icon::Fit, vec2(30.0, 26.0), false, true).on_hover_text("Fit to window (Ctrl+1)").clicked() {
                app.fit_pending = true;
            }
            if icon_button(ui, &pal, Icon::ZoomIn, vec2(30.0, 26.0), false, true).on_hover_text("Zoom in").clicked() {
                app.zoom = zoom_step(app.zoom, true);
            }
            let mut z = app.zoom;
            if adw_slider(ui, &pal, &mut z, 0.125, 8.0, true, 130.0) {
                app.zoom = z;
            }
            if icon_button(ui, &pal, Icon::ZoomOut, vec2(30.0, 26.0), false, true).on_hover_text("Zoom out").clicked() {
                app.zoom = zoom_step(app.zoom, false);
            }
            let label = format!("{}%", (app.zoom * 100.0).round());
            egui::ComboBox::from_id_salt("zoom-combo").width(70.0).selected_text(label).show_ui(ui, |ui| {
                for s in canvas::zoom_steps().iter().rev() {
                    if ui.selectable_label((app.zoom - s).abs() < 1e-3, format!("{}%", s * 100.0)).clicked() {
                        app.zoom = *s;
                    }
                }
                if ui.selectable_label(false, "Fit to window").clicked() {
                    app.fit_pending = true;
                }
            });
        });
    });
}

fn human_size(b: u64) -> String {
    if b < 1024 {
        format!("{b} bytes")
    } else if b < 1024 * 1024 {
        format!("{:.1}KB", b as f64 / 1024.0)
    } else {
        format!("{:.1}MB", b as f64 / 1048576.0)
    }
}

pub fn layers_panel(app: &mut PaintApp, ui: &mut Ui) {
    let pal = app.palette;
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Layers").size(16.0).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icon_button(ui, &pal, Icon::Plus, vec2(30.0, 28.0), false, true).on_hover_text("Create layer (Ctrl+Shift+N)").clicked() {
                app.layer_op(|d| d.add_layer_above());
            }
        });
    });
    ui.add_space(6.0);
    let n = app.doc.layers.len();
    egui::ScrollArea::vertical().show(ui, |ui| {
        for i in (0..n).rev() {
            let selected = i == app.doc.current;
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 76.0), Sense::click());
            let p = ui.painter();
            p.rect_filled(r.shrink(2.0), 6.0, if selected { pal.selected } else { pal.card });
            p.rect_stroke(r.shrink(2.0), 6.0, Stroke::new(1.0, if selected { pal.selected_border } else { pal.card_stroke }), StrokeKind::Inside);
            let thumb = Rect::from_min_size(r.min + vec2(10.0, 10.0), vec2(90.0, 56.0));
            draw_thumb(app, ui, i, thumb);
            let lname = app.doc.layers[i].name.clone();
            ui.painter().text(pos2(thumb.right() + 10.0, r.top() + 22.0), Align2::LEFT_CENTER, lname, FontId::proportional(13.0), pal.text);
            let vis = app.doc.layers[i].visible;
            let eye = Rect::from_min_size(pos2(r.right() - 38.0, r.bottom() - 36.0), vec2(28.0, 26.0));
            let eresp = ui.interact(eye, ui.scope_id().with(("eye", i)), Sense::click());
            if eresp.hovered() {
                ui.painter().rect_filled(eye, 4.0, pal.hover);
            }
            icons::draw(ui.painter(), if vis { Icon::Eye } else { Icon::EyeOff }, eye.shrink(5.0), pal.icon);
            if eresp.on_hover_text(if vis { "Hide layer (Ctrl+H)" } else { "Show layer" }).clicked() {
                app.layer_op(|d| d.layers[i].visible = !d.layers[i].visible);
            } else if resp.clicked() {
                app.commit_all();
                app.doc.current = i;
            }
            resp.context_menu(|ui| {
                if ui.button("Duplicate layer").clicked() {
                    app.doc.current = i;
                    app.layer_op(|d| d.duplicate_layer());
                    ui.close();
                }
                if ui.add_enabled(i > 0, egui::Button::new("Merge down")).clicked() {
                    app.doc.current = i;
                    app.layer_op(|d| d.merge_down());
                    ui.close();
                }
                if ui.add_enabled(i + 1 < n, egui::Button::new("Move layer up")).clicked() {
                    app.doc.current = i;
                    app.layer_op(|d| d.move_layer(true));
                    ui.close();
                }
                if ui.add_enabled(i > 0, egui::Button::new("Move layer down")).clicked() {
                    app.doc.current = i;
                    app.layer_op(|d| d.move_layer(false));
                    ui.close();
                }
                if ui.button("Delete layer").clicked() {
                    app.doc.current = i;
                    app.layer_op(|d| d.delete_layer());
                    ui.close();
                }
            });
        }
        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 60.0), Sense::click());
        let p = ui.painter();
        p.rect_filled(r.shrink(2.0), 6.0, pal.card);
        p.rect_stroke(r.shrink(2.0), 6.0, Stroke::new(1.0, pal.card_stroke), StrokeKind::Inside);
        let sw = Rect::from_min_size(r.min + vec2(10.0, 10.0), vec2(40.0, 40.0));
        p.rect_filled(sw, 4.0, c32(app.doc.background));
        p.rect_stroke(sw, 4.0, Stroke::new(1.0, pal.control_stroke), StrokeKind::Inside);
        p.text(pos2(sw.right() + 10.0, r.center().y), Align2::LEFT_CENTER, "Background", FontId::proportional(13.0), pal.text);
        let eye = Rect::from_min_size(pos2(r.right() - 38.0, r.center().y - 13.0), vec2(28.0, 26.0));
        let eresp = ui.interact(eye, ui.scope_id().with("bg-eye"), Sense::click());
        icons::draw(ui.painter(), if app.doc.background_visible { Icon::Eye } else { Icon::EyeOff }, eye.shrink(5.0), pal.icon);
        if eresp.on_hover_text("Show/hide background (Ctrl+Shift+H)").clicked() {
            app.layer_op(|d| d.background_visible = !d.background_visible);
        } else if resp.clicked() {
            app.dialogs.edit_colors = Some(dialogs::EditColorsState::for_background(app.doc.background));
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let cur = app.doc.current;
            if icon_button(ui, &pal, Icon::Duplicate, vec2(32.0, 28.0), false, true).on_hover_text("Duplicate layer").clicked() {
                app.layer_op(|d| d.duplicate_layer());
            }
            if icon_button(ui, &pal, Icon::Merge, vec2(32.0, 28.0), false, cur > 0).on_hover_text("Merge down").clicked() {
                app.layer_op(|d| d.merge_down());
            }
            if icon_button(ui, &pal, Icon::ArrowUp, vec2(32.0, 28.0), false, cur + 1 < n).on_hover_text("Move layer up").clicked() {
                app.layer_op(|d| d.move_layer(true));
            }
            if icon_button(ui, &pal, Icon::ArrowDown, vec2(32.0, 28.0), false, cur > 0).on_hover_text("Move layer down").clicked() {
                app.layer_op(|d| d.move_layer(false));
            }
            if icon_button(ui, &pal, Icon::Trash, vec2(32.0, 28.0), false, true).on_hover_text("Delete layer").clicked() {
                app.layer_op(|d| d.delete_layer());
            }
        });
    });
}

fn draw_thumb(app: &PaintApp, ui: &Ui, i: usize, r: Rect) {
    let img = &app.doc.layers[i].image;
    let scale = (r.width() / img.w as f32).min(r.height() / img.h as f32);
    let size = vec2(img.w as f32 * scale, img.h as f32 * scale);
    let tr = Rect::from_center_size(r.center(), size);
    let p = ui.painter();
    p.rect_filled(tr, 0.0, Color32::WHITE);
    let id = egui::Id::unique(("thumb", i));
    let key = (std::sync::Arc::as_ptr(&app.doc.layers[i].image) as usize, img.w, img.h);
    let cached: Option<((usize, u32, u32), egui::TextureHandle)> = ui.ctx().data(|d| d.get_temp(id));
    let tex = match cached {
        Some((k, t)) if k == key && app.stroke.is_none() => t,
        _ => {
            let tw = (size.x * 2.0).max(1.0) as u32;
            let th = (size.y * 2.0).max(1.0) as u32;
            let small = paint_core::transform::resize(img, tw, th);
            let t = ui.ctx().load_texture(format!("thumb{i}"), crate::app::to_color_image(&small, small.bounds()), egui::TextureOptions::LINEAR);
            ui.ctx().data_mut(|d| d.insert_temp(id, (key, t.clone())));
            t
        }
    };
    p.image(tex.id(), tr, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    p.rect_stroke(tr, 0.0, Stroke::new(1.0, app.palette.control_stroke), StrokeKind::Outside);
}

pub fn text_toolbar(app: &mut PaintApp, ui: &mut Ui, anchor: Pos2) {
    let pal = app.palette;
    egui::Area::new(egui::Id::unique("text-toolbar")).order(egui::Order::Foreground).fixed_pos(anchor).show(ui.ctx(), |ui| {
        egui::Frame::new()
            .fill(pal.flyout)
            .stroke(Stroke::new(1.0, pal.card_stroke))
            .corner_radius(8.0)
            .shadow(egui::Shadow { offset: [0, 4], blur: 12, spread: 0, color: Color32::from_black_alpha(40) })
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut changed = false;
                    let fam = app.font_family.clone();
                    egui::ComboBox::from_id_salt("font-family").width(170.0).selected_text(&fam).show_ui(ui, |ui| {
                        for f in app.fonts.families.clone() {
                            if ui.selectable_label(f == fam, &f).clicked() {
                                app.font_family = f;
                                changed = true;
                            }
                        }
                    });
                    egui::ComboBox::from_id_salt("font-size").width(56.0).selected_text(format!("{}", app.text_style.size_pt)).show_ui(ui, |ui| {
                        for s in [8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 36.0, 48.0, 72.0, 96.0] {
                            if ui.selectable_label(app.text_style.size_pt == s, format!("{s}")).clicked() {
                                app.text_style.size_pt = s;
                                changed = true;
                            }
                        }
                    });
                    for (label, field, tip) in [("B", 0, "Bold (Ctrl+B)"), ("I", 1, "Italic (Ctrl+I)"), ("U", 2, "Underline (Ctrl+U)"), ("S", 3, "Strikethrough")] {
                        let on = match field {
                            0 => app.text_style.bold,
                            1 => app.text_style.italic,
                            2 => app.text_style.underline,
                            _ => app.text_style.strikethrough,
                        };
                        let mut rt = egui::RichText::new(label).size(15.0);
                        rt = match field {
                            0 => rt.strong(),
                            1 => rt.italics(),
                            2 => rt.underline(),
                            _ => rt.strikethrough(),
                        };
                        if ui.add(egui::Button::selectable(on, rt).min_size(vec2(30.0, 28.0))).on_hover_text(tip).clicked() {
                            match field {
                                0 => app.text_style.bold = !on,
                                1 => app.text_style.italic = !on,
                                2 => app.text_style.underline = !on,
                                _ => app.text_style.strikethrough = !on,
                            }
                            changed = true;
                        }
                    }
                    ui.separator();
                    for (a, ic) in [(TAlign::Left, Icon::AlignLeft), (TAlign::Center, Icon::AlignCenter), (TAlign::Right, Icon::AlignRight)] {
                        if icon_button(ui, &pal, ic, vec2(30.0, 28.0), app.text_style.align == a, true).clicked() {
                            app.text_style.align = a;
                            changed = true;
                        }
                    }
                    ui.separator();
                    let mut bgf = app.text_style.background.is_some();
                    if ui.checkbox(&mut bgf, "Background fill").changed() {
                        app.text_style.background = bgf.then_some(app.color2);
                        changed = true;
                    }
                    if changed {
                        canvas::render_text(app);
                    }
                });
            });
    });
    let _ = CursorIcon::Default;
}
