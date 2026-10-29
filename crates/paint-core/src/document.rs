use std::sync::Arc;

use crate::color::Color;
use crate::image::{Image, over};

#[derive(Clone, Debug)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub image: Arc<Image>,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub w: u32,
    pub h: u32,
    pub layers: Vec<Layer>,
    pub current: usize,
    pub background: Color,
    pub background_visible: bool,
    pub dpi: f64,
    next_layer_no: u32,
}

impl Document {
    pub fn new(w: u32, h: u32) -> Self {
        let mut d = Document {
            w,
            h,
            layers: Vec::new(),
            current: 0,
            background: Color::WHITE,
            background_visible: true,
            dpi: 96.0,
            next_layer_no: 1,
        };
        let first = d.make_layer(Image::new(w, h));
        d.layers.push(first);
        d
    }

    pub fn from_image(img: Image) -> Self {
        let mut d = Document::new(img.w, img.h);
        d.layers[0].image = Arc::new(img);
        d
    }

    fn make_layer(&mut self, image: Image) -> Layer {
        let name = format!("Layer {}", self.next_layer_no);
        self.next_layer_no += 1;
        Layer { name, visible: true, image: Arc::new(image) }
    }

    pub fn layer(&self) -> &Image {
        &self.layers[self.current].image
    }

    pub fn layer_mut(&mut self) -> &mut Image {
        Arc::make_mut(&mut self.layers[self.current].image)
    }

    pub fn add_layer_above(&mut self) {
        let l = self.make_layer(Image::new(self.w, self.h));
        self.layers.insert(self.current + 1, l);
        self.current += 1;
    }

    pub fn duplicate_layer(&mut self) {
        let mut l = self.layers[self.current].clone();
        l.name = format!("{} copy", l.name);
        self.layers.insert(self.current + 1, l);
        self.current += 1;
    }

    pub fn delete_layer(&mut self) {
        if self.layers.len() > 1 {
            self.layers.remove(self.current);
            self.current = self.current.min(self.layers.len() - 1);
        } else {
            self.layers[0].image = Arc::new(Image::new(self.w, self.h));
        }
    }

    pub fn move_layer(&mut self, up: bool) {
        let i = self.current;
        if up && i + 1 < self.layers.len() {
            self.layers.swap(i, i + 1);
            self.current += 1;
        } else if !up && i > 0 {
            self.layers.swap(i, i - 1);
            self.current -= 1;
        }
    }

    pub fn merge_down(&mut self) {
        let i = self.current;
        if i == 0 {
            return;
        }
        let top = self.layers[i].clone();
        if top.visible {
            Arc::make_mut(&mut self.layers[i - 1].image).paste_over(&top.image, 0, 0, 255);
        }
        self.layers.remove(i);
        self.current = i - 1;
    }

    pub fn composite_into(&self, out: &mut Image, x0: i32, y0: i32, x1: i32, y1: i32, with_bg: bool) {
        let bg = if with_bg && self.background_visible { self.background.premul() } else { [0; 4] };
        for y in y0.max(0)..y1.min(self.h as i32) {
            for x in x0.max(0)..x1.min(self.w as i32) {
                let i = y as usize * self.w as usize + x as usize;
                let mut p = bg;
                for l in &self.layers {
                    if l.visible {
                        p = over(p, l.image.px[i]);
                    }
                }
                out.px[i] = p;
            }
        }
    }

    pub fn flatten(&self) -> Image {
        let mut out = Image::new(self.w, self.h);
        self.composite_into(&mut out, 0, 0, self.w as i32, self.h as i32, true);
        out
    }

    pub fn map_layers(&mut self, f: impl Fn(&Image) -> Image) {
        for l in &mut self.layers {
            l.image = Arc::new(f(&l.image));
        }
        self.w = self.layers[0].image.w;
        self.h = self.layers[0].image.h;
    }

    pub fn resize_canvas(&mut self, w: u32, h: u32) {
        for l in &mut self.layers {
            let mut img = Image::new(w, h);
            img.paste_copy(&l.image, 0, 0);
            l.image = Arc::new(img);
        }
        self.w = w;
        self.h = h;
    }
}
