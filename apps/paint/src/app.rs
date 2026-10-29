use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions, Vec2};
use paint_core::color::BASIC_PALETTE;
use paint_core::geom::{Point, Rect};
use paint_core::history::History;
use paint_core::shapes::ShapeKind;
use paint_core::stroke::{BrushKind, Stroke};
use paint_core::text::{Align, TextStyle};
use paint_core::{Color, Document, Image, project, transform};

use crate::fonts::FontCache;
use crate::theme::{self, Palette, ThemeChoice};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    RectSelect,
    FreeSelect,
    Pencil,
    Fill,
    Text,
    Eraser,
    Picker,
    Magnifier,
    Brush,
    Shape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutlineStyle {
    None,
    Solid,
    Textured(BrushKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Move,
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
    Point(usize),
}

pub struct StrokeSession {
    pub stroke: Stroke,
    pub base: Image,
    pub before: Document,
    pub last_tick: Instant,
}

pub struct ShapeSession {
    pub kind: ShapeKind,
    pub pts: Vec<Point>,
    pub stage: u8,
    pub creating: bool,
    pub swap_colors: bool,
    pub drag: Option<(Handle, Point, Vec<Point>)>,
    pub base: Image,
    pub before: Document,
}

pub struct Selection {
    pub rect: [f32; 4],
    pub image: Option<Image>,
    pub lasso: Vec<Point>,
    pub creating: bool,
    pub drag: Option<(Handle, Point, [f32; 4])>,
    pub base: Option<Image>,
    pub before: Option<Document>,
}

pub struct TextSession {
    pub rect: [f32; 4],
    pub text: String,
    pub base: Image,
    pub before: Document,
    pub creating: bool,
    pub drag: Option<(Handle, Point, [f32; 4])>,
}

#[derive(Default)]
pub struct Dialogs {
    pub resize: Option<crate::dialogs::ResizeState>,
    pub properties: Option<crate::dialogs::PropertiesState>,
    pub rotate: Option<f32>,
    pub edit_colors: Option<crate::dialogs::EditColorsState>,
    pub about_unsaved: Option<crate::dialogs::PendingAction>,
    pub message: Option<String>,
}

pub struct PaintApp {
    pub doc: Document,
    pub history: History,
    pub file: Option<PathBuf>,
    pub dirty: bool,
    pub tool: Tool,
    pub prev_tool: Tool,
    pub brush: BrushKind,
    pub shape: ShapeKind,
    pub eraser_square: bool,
    pub sizes: HashMap<BrushKind, f32>,
    pub shape_width: f32,
    pub opacity: HashMap<BrushKind, f32>,
    pub tolerance: f32,
    pub outline: OutlineStyle,
    pub fill: OutlineStyle,
    pub color1: Color,
    pub color2: Color,
    pub editing_color2: bool,
    pub custom_colors: Vec<Color>,
    pub transparent_selection: bool,
    pub text_style: TextStyle,
    pub font_family: String,
    pub fonts: FontCache,
    pub zoom: f32,
    pub scroll: Vec2,
    pub fit_pending: bool,
    pub show_rulers: bool,
    pub show_grid: bool,
    pub show_statusbar: bool,
    pub show_layers: bool,
    pub show_ribbon: bool,
    pub fullscreen: bool,
    pub theme_choice: ThemeChoice,
    pub palette: Palette,
    pub cursor: Option<(i32, i32)>,
    pub stroke: Option<StrokeSession>,
    pub shape_session: Option<ShapeSession>,
    pub selection: Option<Selection>,
    pub text: Option<TextSession>,
    pub canvas_resize: Option<(Handle, u32, u32)>,
    pub display: Image,
    pub texture: Option<TextureHandle>,
    pub full_refresh: bool,
    pub dirty_rect: Rect,
    pub dialogs: Dialogs,
    pub open_menu: Option<&'static str>,
    pub seed: u32,
    pub last_saved_size: Option<u64>,
    pub size_from_env: bool,
    pub look: crate::system::SystemLook,
}

impl PaintApp {
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        let (sw, sh) = screen_size(cc);
        let doc = Document::new((sw * 0.6) as u32, (sh * 0.6) as u32);
        let mut sizes = HashMap::new();
        let mut opacity = HashMap::new();
        for k in [
            BrushKind::Pencil,
            BrushKind::Brush,
            BrushKind::Calligraphy45,
            BrushKind::Calligraphy135,
            BrushKind::Airbrush,
            BrushKind::Oil,
            BrushKind::Crayon,
            BrushKind::Marker,
            BrushKind::NaturalPencil,
            BrushKind::Watercolor,
            BrushKind::EraserRound,
        ] {
            sizes.insert(k, k.size_range().0 as f32);
            opacity.insert(k, 1.0);
        }
        let look = crate::system::detect();
        crate::system::install_ui_font(&cc.egui_ctx);
        let dark = look.prefer_dark.unwrap_or(cc.egui_ctx.global_style().visuals.dark_mode);
        let palette = theme::palette(dark, look.accent);
        theme::apply(&cc.egui_ctx, &palette);
        let fonts = FontCache::new();
        let font_family = fonts.default_family();
        let display = doc.flatten();
        let mut app = PaintApp {
            doc,
            history: History::default(),
            file: None,
            dirty: false,
            tool: Tool::Pencil,
            prev_tool: Tool::Pencil,
            brush: BrushKind::Brush,
            shape: ShapeKind::Rectangle,
            eraser_square: false,
            sizes,
            shape_width: 5.0,
            opacity,
            tolerance: 0.0,
            outline: OutlineStyle::Solid,
            fill: OutlineStyle::None,
            color1: Color::BLACK,
            color2: Color::WHITE,
            editing_color2: false,
            custom_colors: Vec::new(),
            transparent_selection: false,
            text_style: TextStyle {
                size_pt: 11.0,
                bold: false,
                italic: false,
                underline: false,
                strikethrough: false,
                align: Align::Left,
                color: Color::BLACK,
                background: None,
            },
            font_family,
            fonts,
            zoom: 1.0,
            scroll: Vec2::ZERO,
            fit_pending: false,
            show_rulers: false,
            show_grid: false,
            show_statusbar: true,
            show_layers: false,
            show_ribbon: true,
            fullscreen: false,
            theme_choice: ThemeChoice::System,
            palette,
            cursor: None,
            stroke: None,
            shape_session: None,
            selection: None,
            text: None,
            canvas_resize: None,
            display,
            texture: None,
            full_refresh: true,
            dirty_rect: Rect::EMPTY,
            dialogs: Dialogs::default(),
            open_menu: None,
            seed: 1,
            last_saved_size: None,
            size_from_env: std::env::var("PAINT_SCREEN").is_ok(),
            look,
        };
        if let Some(p) = path {
            app.open_path(p);
        }
        app
    }

    pub fn palette_colors() -> Vec<Color> {
        BASIC_PALETTE.iter().map(|&c| Color::from_hex(c)).collect()
    }

    pub fn size(&self) -> f32 {
        match self.tool {
            Tool::Pencil => self.sizes[&BrushKind::Pencil],
            Tool::Eraser => self.sizes[&BrushKind::EraserRound],
            Tool::Brush => self.sizes[&self.brush],
            Tool::Shape => self.shape_width,
            _ => self.sizes[&self.brush],
        }
    }

    pub fn size_range(&self) -> (f32, f32) {
        match self.tool {
            Tool::Pencil => (1.0, 248.0),
            Tool::Eraser => (1.0, 248.0),
            Tool::Shape => (1.0, 50.0),
            _ => {
                let r = self.brush.size_range();
                (r.1 as f32, r.2 as f32)
            }
        }
    }

    pub fn set_size(&mut self, v: f32) {
        let (lo, hi) = self.size_range();
        let v = v.round().clamp(lo, hi);
        match self.tool {
            Tool::Pencil => {
                self.sizes.insert(BrushKind::Pencil, v);
            }
            Tool::Eraser => {
                self.sizes.insert(BrushKind::EraserRound, v);
            }
            Tool::Shape => {
                self.shape_width = v;
                self.rerender_shape();
            }
            _ => {
                self.sizes.insert(self.brush, v);
            }
        }
    }

    pub fn opacity_key(&self) -> Option<BrushKind> {
        match self.tool {
            Tool::Pencil => Some(BrushKind::Pencil),
            Tool::Eraser => Some(BrushKind::EraserRound),
            Tool::Brush => Some(self.brush),
            _ => None,
        }
    }

    pub fn select_tool(&mut self, t: Tool) {
        self.commit_all();
        if t != self.tool {
            if !matches!(self.tool, Tool::Picker | Tool::Magnifier) {
                self.prev_tool = self.tool;
            }
            self.tool = t;
        }
    }

    pub fn commit_all(&mut self) {
        self.commit_stroke();
        self.commit_shape();
        self.commit_selection();
        self.commit_text();
    }

    pub fn snapshot(&self) -> Document {
        self.doc.clone()
    }

    pub fn push_history(&mut self, before: Document) {
        self.history.push(before);
        self.dirty = true;
    }

    pub fn mark_all(&mut self) {
        self.full_refresh = true;
    }

    pub fn mark(&mut self, r: Rect) {
        self.dirty_rect = self.dirty_rect.union(&r);
    }

    pub fn undo(&mut self) {
        if self.stroke.is_some() || self.shape_session.is_some() || self.text.is_some() {
            self.cancel_sessions();
            return;
        }
        if self.selection.as_ref().is_some_and(|s| s.image.is_some()) {
            self.cancel_sessions();
            return;
        }
        self.selection = None;
        if self.history.undo(&mut self.doc) {
            self.doc.current = self.doc.current.min(self.doc.layers.len() - 1);
            self.mark_all();
            self.dirty = true;
        }
    }

    pub fn redo(&mut self) {
        self.commit_all();
        if self.history.redo(&mut self.doc) {
            self.doc.current = self.doc.current.min(self.doc.layers.len() - 1);
            self.mark_all();
            self.dirty = true;
        }
    }

    pub fn cancel_sessions(&mut self) {
        if let Some(s) = self.stroke.take() {
            *self.doc.layer_mut() = s.base;
        }
        if let Some(s) = self.shape_session.take() {
            *self.doc.layer_mut() = s.base;
        }
        if let Some(t) = self.text.take() {
            *self.doc.layer_mut() = t.base;
        }
        if let Some(sel) = self.selection.take() {
            if let Some(before) = sel.before {
                self.doc = before;
            }
        }
        self.mark_all();
    }

    pub fn commit_stroke(&mut self) {
        if let Some(s) = self.stroke.take() {
            self.push_history(s.before);
        }
    }

    pub fn commit_shape(&mut self) {
        if let Some(s) = self.shape_session.take() {
            if s.pts.len() >= 2 {
                self.push_history(s.before);
            } else {
                *self.doc.layer_mut() = s.base;
                self.mark_all();
            }
        }
    }

    pub fn commit_text(&mut self) {
        if let Some(t) = self.text.take() {
            if t.text.trim().is_empty() {
                *self.doc.layer_mut() = t.base;
                self.mark_all();
            } else {
                self.push_history(t.before);
            }
        }
    }

    pub fn commit_selection(&mut self) {
        if let Some(sel) = self.selection.take() {
            if let Some(before) = sel.before {
                if sel.image.is_some() {
                    self.push_history(before);
                }
            }
            self.mark_all();
        }
    }

    pub fn render_selection(&mut self) {
        let transparent = self.transparent_selection.then_some(self.color2);
        let Some(sel) = &self.selection else { return };
        let (Some(img), Some(base)) = (&sel.image, &sel.base) else { return };
        let [x0, y0, x1, y1] = norm(sel.rect);
        let w = (x1 - x0).round().max(1.0) as u32;
        let h = (y1 - y0).round().max(1.0) as u32;
        let scaled = if w == img.w && h == img.h { img.clone() } else { transform::resize(img, w, h) };
        let mut layer = base.clone();
        for y in 0..scaled.h as i32 {
            for x in 0..scaled.w as i32 {
                let mut p = scaled.get(x, y);
                if let Some(tc) = transparent {
                    if Color::from_premul(p) == tc {
                        p = [0; 4];
                    }
                }
                layer.blend(x0.round() as i32 + x, y0.round() as i32 + y, p, 255);
            }
        }
        *self.doc.layer_mut() = layer;
        self.mark_all();
    }

    pub fn lift_selection(&mut self) {
        let Some(sel) = &self.selection else { return };
        if sel.image.is_some() {
            return;
        }
        let before = self.snapshot();
        let [x0, y0, x1, y1] = norm(sel.rect);
        let r = Rect::new(x0.round() as i32, y0.round() as i32, x1.round() as i32, y1.round() as i32);
        let layer = self.doc.layer().clone();
        let mut img = layer.crop(r);
        let mut base = layer;
        let lasso = sel.lasso.clone();
        if lasso.len() >= 3 {
            let mut inside = Image::new(img.w, img.h);
            let poly: Vec<Point> = lasso.iter().map(|p| Point::new(p.x - r.x0 as f32, p.y - r.y0 as f32)).collect();
            paint_core::raster::fill_polygons(&[poly], inside.bounds(), |x, y, c| {
                if c >= 0.5 {
                    inside.put(x, y, [255; 4]);
                }
            });
            for y in 0..img.h as i32 {
                for x in 0..img.w as i32 {
                    if inside.get(x, y)[3] == 0 {
                        img.put(x, y, [0; 4]);
                    } else {
                        base.put(r.x0 + x, r.y0 + y, [0; 4]);
                    }
                }
            }
        } else {
            for y in r.y0..r.y1 {
                for x in r.x0..r.x1 {
                    base.put(x, y, [0; 4]);
                }
            }
        }
        let sel = self.selection.as_mut().unwrap();
        sel.image = Some(img);
        sel.base = Some(base);
        sel.before = Some(before);
    }

    pub fn delete_selection(&mut self) {
        if self.selection.is_none() {
            return;
        }
        self.lift_selection();
        if let Some(sel) = self.selection.take() {
            if let (Some(base), Some(before)) = (sel.base, sel.before) {
                *self.doc.layer_mut() = base;
                self.push_history(before);
                self.mark_all();
            }
        }
    }

    pub fn selection_image(&self) -> Option<Image> {
        let sel = self.selection.as_ref()?;
        if let Some(img) = &sel.image {
            return Some(img.clone());
        }
        let [x0, y0, x1, y1] = norm(sel.rect);
        let r = Rect::new(x0.round() as i32, y0.round() as i32, x1.round() as i32, y1.round() as i32);
        Some(self.doc.layer().crop(r))
    }

    pub fn select_all(&mut self) {
        self.commit_all();
        self.tool = Tool::RectSelect;
        self.selection = Some(Selection {
            rect: [0.0, 0.0, self.doc.w as f32, self.doc.h as f32],
            image: None,
            lasso: Vec::new(),
            creating: false,
            drag: None,
            base: None,
            before: None,
        });
    }

    pub fn invert_selection(&mut self) {
        let Some(sel) = &self.selection else { return };
        let [x0, y0, x1, y1] = norm(sel.rect);
        let (w, h) = (self.doc.w as f32, self.doc.h as f32);
        let lasso = vec![
            Point::new(0.0, 0.0),
            Point::new(w, 0.0),
            Point::new(w, h),
            Point::new(0.0, h),
            Point::new(0.0, y0),
            Point::new(x0, y0),
            Point::new(x0, y1),
            Point::new(x1, y1),
            Point::new(x1, y0),
            Point::new(0.0, y0),
        ];
        self.commit_all();
        self.selection = Some(Selection {
            rect: [0.0, 0.0, w, h],
            image: None,
            lasso,
            creating: false,
            drag: None,
            base: None,
            before: None,
        });
    }

    pub fn copy(&mut self, cut: bool) {
        let Some(img) = self.selection_image() else { return };
        crate::clipboard::set_image(&img);
        if cut {
            self.delete_selection();
        }
    }

    pub fn paste(&mut self, img: Option<Image>) {
        let Some(img) = img.or_else(crate::clipboard::get_image) else { return };
        self.commit_all();
        if img.w > self.doc.w || img.h > self.doc.h {
            let before = self.snapshot();
            self.doc.resize_canvas(self.doc.w.max(img.w), self.doc.h.max(img.h));
            self.history.push(before);
        }
        self.tool = Tool::RectSelect;
        let before = self.snapshot();
        self.selection = Some(Selection {
            rect: [0.0, 0.0, img.w as f32, img.h as f32],
            base: Some(self.doc.layer().clone()),
            image: Some(img),
            lasso: Vec::new(),
            creating: false,
            drag: None,
            before: Some(before),
        });
        self.render_selection();
    }

    pub fn crop(&mut self) {
        let Some(sel) = &self.selection else { return };
        let [x0, y0, x1, y1] = norm(sel.rect);
        let r = Rect::new(x0.round() as i32, y0.round() as i32, x1.round() as i32, y1.round() as i32);
        let img = sel.image.clone();
        self.commit_all();
        let before = self.snapshot();
        let r = r.intersect(&self.doc.layer().bounds()).union(&if img.is_some() { r } else { Rect::EMPTY });
        if r.is_empty() {
            return;
        }
        self.doc.map_layers(|l| l.crop(r));
        self.push_history(before);
        self.mark_all();
    }

    pub fn image_op(&mut self, f: impl Fn(&Image) -> Image) {
        if self.selection.as_ref().is_some_and(|s| !s.creating) {
            self.lift_selection();
            if let Some(sel) = self.selection.as_mut() {
                if let Some(img) = &sel.image {
                    let out = f(img);
                    let [x0, y0, _, _] = norm(sel.rect);
                    sel.rect = [x0, y0, x0 + out.w as f32, y0 + out.h as f32];
                    sel.image = Some(out);
                }
            }
            self.render_selection();
            return;
        }
        self.commit_all();
        let before = self.snapshot();
        self.doc.map_layers(f);
        self.push_history(before);
        self.mark_all();
    }

    pub fn invert_colors(&mut self) {
        self.image_op(|i| {
            let mut o = i.clone();
            transform::invert(&mut o);
            o
        });
    }

    pub fn resize_canvas(&mut self, w: u32, h: u32) {
        self.commit_all();
        let before = self.snapshot();
        self.doc.resize_canvas(w.max(1), h.max(1));
        self.push_history(before);
        self.mark_all();
    }

    pub fn layer_op(&mut self, f: impl FnOnce(&mut Document)) {
        self.commit_all();
        let before = self.snapshot();
        f(&mut self.doc);
        self.push_history(before);
        self.mark_all();
    }

    pub fn new_document(&mut self) {
        self.commit_all();
        let (w, h) = (self.doc.w, self.doc.h);
        self.doc = Document::new(w, h);
        self.history.clear();
        self.file = None;
        self.dirty = false;
        self.selection = None;
        self.zoom = 1.0;
        self.scroll = Vec2::ZERO;
        self.mark_all();
    }

    pub fn open_dialog(&mut self) {
        let f = rfd::FileDialog::new()
            .add_filter("All Picture Files", &["png", "jpg", "jpeg", "bmp", "dib", "gif", "tif", "tiff", "ico", "webp", "paint"])
            .add_filter("Paint project", &["paint"])
            .add_filter("All Files", &["*"])
            .pick_file();
        if let Some(p) = f {
            self.open_path(p);
        }
    }

    pub fn open_path(&mut self, p: PathBuf) {
        let res = if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("paint")) {
            std::fs::File::open(&p).map_err(|e| e.to_string()).and_then(|f| project::load(std::io::BufReader::new(f)))
        } else {
            image::open(&p).map(|i| Document::from_image(Image::from_dynamic(&i))).map_err(|e| e.to_string())
        };
        match res {
            Ok(doc) => {
                self.commit_all();
                self.doc = doc;
                self.history.clear();
                self.file = Some(p.clone());
                self.dirty = false;
                self.selection = None;
                self.fit_pending = true;
                self.last_saved_size = std::fs::metadata(&p).ok().map(|m| m.len());
                self.mark_all();
            }
            Err(e) => self.dialogs.message = Some(format!("LinuxPaint cannot read this file.\n{e}")),
        }
    }

    pub fn save(&mut self) -> bool {
        match self.file.clone() {
            Some(p) => self.save_to(p),
            None => self.save_as(None),
        }
    }

    pub fn save_as(&mut self, ext: Option<&str>) -> bool {
        let name = self
            .file
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into());
        let mut dlg = rfd::FileDialog::new().set_file_name(format!("{name}.{}", ext.unwrap_or("png")));
        let filters: [(&str, &[&str]); 7] = [
            ("PNG", &["png"]),
            ("JPEG", &["jpg", "jpeg"]),
            ("BMP", &["bmp"]),
            ("GIF", &["gif"]),
            ("TIFF", &["tif", "tiff"]),
            ("WEBP", &["webp"]),
            ("Paint project", &["paint"]),
        ];
        for (n, e) in filters {
            if ext.is_none() || e.contains(&ext.unwrap()) {
                dlg = dlg.add_filter(n, e);
            }
        }
        match dlg.save_file() {
            Some(p) => self.save_to(p),
            None => false,
        }
    }

    pub fn save_to(&mut self, p: PathBuf) -> bool {
        self.commit_all();
        let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        let res = if ext == "paint" {
            std::fs::File::create(&p).map_err(|e| e.to_string()).and_then(|f| project::save(&self.doc, std::io::BufWriter::new(f)))
        } else {
            let flat = self.doc.flatten();
            let dynimg = flat.to_dynamic();
            let out = match ext.as_str() {
                "jpg" | "jpeg" | "bmp" => image::DynamicImage::ImageRgb8(dynimg.to_rgb8()),
                _ => dynimg,
            };
            out.save(&p).map_err(|e| e.to_string())
        };
        match res {
            Ok(()) => {
                self.file = Some(p.clone());
                self.dirty = false;
                self.last_saved_size = std::fs::metadata(&p).ok().map(|m| m.len());
                true
            }
            Err(e) => {
                self.dialogs.message = Some(format!("LinuxPaint cannot save this file.\n{e}"));
                false
            }
        }
    }

    pub fn title(&self) -> String {
        let name = self
            .file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into());
        format!("{}{name} - LinuxPaint", if self.dirty { "*" } else { "" })
    }

    pub fn set_theme(&mut self, ctx: &egui::Context, choice: ThemeChoice) {
        self.theme_choice = choice;
        let dark = match choice {
            ThemeChoice::Light => false,
            ThemeChoice::Dark => true,
            ThemeChoice::System => crate::system::detect().prefer_dark.or_else(|| ctx.system_theme().map(|t| t == egui::Theme::Dark)).unwrap_or(false),
        };
        self.palette = theme::palette(dark, self.look.accent);
        theme::apply(ctx, &self.palette);
    }

    pub fn rerender_shape(&mut self) {
        crate::canvas::render_shape(self);
    }

    pub fn upload(&mut self, ctx: &egui::Context) {
        let (w, h) = (self.doc.w, self.doc.h);
        let size_changed = self.display.w != w || self.display.h != h;
        if size_changed {
            self.display = Image::new(w, h);
            self.full_refresh = true;
        }
        let r = if self.full_refresh { self.doc.layer().bounds() } else { self.dirty_rect.intersect(&self.doc.layer().bounds()) };
        if r.is_empty() && self.texture.is_some() && !size_changed {
            return;
        }
        self.doc.composite_into(&mut self.display, r.x0, r.y0, r.x1, r.y1, true);
        let opts = TextureOptions::NEAREST;
        if self.texture.is_none() || size_changed || self.full_refresh {
            let ci = to_color_image(&self.display, self.display.bounds());
            match &mut self.texture {
                Some(t) if !size_changed => t.set(ci, opts),
                _ => self.texture = Some(ctx.load_texture("canvas", ci, opts)),
            }
        } else if let Some(t) = &mut self.texture {
            t.set_partial([r.x0 as usize, r.y0 as usize], to_color_image(&self.display, r), opts);
        }
        self.full_refresh = false;
        self.dirty_rect = Rect::EMPTY;
    }
}

pub fn to_color_image(img: &Image, r: Rect) -> ColorImage {
    let mut bytes = Vec::with_capacity((r.width() * r.height() * 4) as usize);
    for y in r.y0..r.y1 {
        let row = img.idx(r.x0, y);
        for p in &img.px[row..row + r.width() as usize] {
            bytes.extend_from_slice(p);
        }
    }
    ColorImage::from_rgba_premultiplied([r.width() as usize, r.height() as usize], &bytes)
}

pub fn norm(r: [f32; 4]) -> [f32; 4] {
    [r[0].min(r[2]), r[1].min(r[3]), r[0].max(r[2]), r[1].max(r[3])]
}

fn screen_size(cc: &eframe::CreationContext<'_>) -> (f32, f32) {
    let _ = cc;
    match std::env::var("PAINT_SCREEN").ok().and_then(|s| {
        let (w, h) = s.split_once('x')?;
        Some((w.parse().ok()?, h.parse().ok()?))
    }) {
        Some(v) => v,
        None => (1920.0, 1080.0),
    }
}
