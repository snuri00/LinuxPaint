use crate::image::Image;

pub fn flip_h(img: &Image) -> Image {
    let mut out = Image::new(img.w, img.h);
    for y in 0..img.h as i32 {
        for x in 0..img.w as i32 {
            out.put(img.w as i32 - 1 - x, y, img.get(x, y));
        }
    }
    out
}

pub fn flip_v(img: &Image) -> Image {
    let mut out = Image::new(img.w, img.h);
    for y in 0..img.h as i32 {
        for x in 0..img.w as i32 {
            out.put(x, img.h as i32 - 1 - y, img.get(x, y));
        }
    }
    out
}

pub fn rotate_cw(img: &Image) -> Image {
    let mut out = Image::new(img.h, img.w);
    for y in 0..img.h as i32 {
        for x in 0..img.w as i32 {
            out.put(img.h as i32 - 1 - y, x, img.get(x, y));
        }
    }
    out
}

pub fn rotate_ccw(img: &Image) -> Image {
    let mut out = Image::new(img.h, img.w);
    for y in 0..img.h as i32 {
        for x in 0..img.w as i32 {
            out.put(y, img.w as i32 - 1 - x, img.get(x, y));
        }
    }
    out
}

pub fn rotate_180(img: &Image) -> Image {
    let mut out = Image::new(img.w, img.h);
    for y in 0..img.h as i32 {
        for x in 0..img.w as i32 {
            out.put(img.w as i32 - 1 - x, img.h as i32 - 1 - y, img.get(x, y));
        }
    }
    out
}

fn sample(img: &Image, x: f32, y: f32) -> [u8; 4] {
    let x = x - 0.5;
    let y = y - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (xi, yi) = (x0 as i32, y0 as i32);
    let p = [img.get(xi, yi), img.get(xi + 1, yi), img.get(xi, yi + 1), img.get(xi + 1, yi + 1)];
    let w = [(1.0 - fx) * (1.0 - fy), fx * (1.0 - fy), (1.0 - fx) * fy, fx * fy];
    let mut o = [0u8; 4];
    for c in 0..4 {
        let v: f32 = (0..4).map(|k| p[k][c] as f32 * w[k]).sum();
        o[c] = v.round().clamp(0.0, 255.0) as u8;
    }
    o
}

pub fn resize(img: &Image, w: u32, h: u32) -> Image {
    let w = w.max(1);
    let h = h.max(1);
    let mut out = Image::new(w, h);
    let sx = img.w as f32 / w as f32;
    let sy = img.h as f32 / h as f32;
    if sx > 1.0 || sy > 1.0 {
        for y in 0..h {
            for x in 0..w {
                let (fx0, fx1) = (x as f32 * sx, (x + 1) as f32 * sx);
                let (fy0, fy1) = (y as f32 * sy, (y + 1) as f32 * sy);
                let mut acc = [0f32; 4];
                let mut wsum = 0.0;
                for yy in fy0.floor() as i32..fy1.ceil() as i32 {
                    let wy = (fy1.min(yy as f32 + 1.0) - fy0.max(yy as f32)).max(0.0);
                    for xx in fx0.floor() as i32..fx1.ceil() as i32 {
                        let wx = (fx1.min(xx as f32 + 1.0) - fx0.max(xx as f32)).max(0.0);
                        let p = img.get(xx, yy);
                        for c in 0..4 {
                            acc[c] += p[c] as f32 * wx * wy;
                        }
                        wsum += wx * wy;
                    }
                }
                let px = acc.map(|v| (v / wsum.max(1e-6)).round().clamp(0.0, 255.0) as u8);
                out.put(x as i32, y as i32, px);
            }
        }
    } else {
        for y in 0..h {
            for x in 0..w {
                out.put(x as i32, y as i32, sample(img, (x as f32 + 0.5) * sx, (y as f32 + 0.5) * sy));
            }
        }
    }
    out
}

pub fn skew(img: &Image, deg_x: f32, deg_y: f32) -> Image {
    let tx = deg_x.to_radians().tan();
    let ty = deg_y.to_radians().tan();
    let (w, h) = (img.w as f32, img.h as f32);
    let new_w = (w + (h * tx).abs()).round().max(1.0) as u32;
    let new_h = (h + (w * ty).abs()).round().max(1.0) as u32;
    let ox = if tx < 0.0 { -h * tx } else { 0.0 };
    let oy = if ty < 0.0 { -w * ty } else { 0.0 };
    let det = 1.0 - tx * ty;
    let mut out = Image::new(new_w, new_h);
    for y in 0..new_h {
        for x in 0..new_w {
            let u = x as f32 + 0.5 - ox;
            let v = y as f32 + 0.5 - oy;
            let sx = (u - tx * v) / det;
            let sy = (v - ty * u) / det;
            if sx >= 0.0 && sy >= 0.0 && sx < w && sy < h {
                out.put(x as i32, y as i32, sample(img, sx, sy));
            }
        }
    }
    out
}

pub fn rotate_any(img: &Image, deg: f32) -> Image {
    let a = deg.to_radians();
    let (s, c) = a.sin_cos();
    let (w, h) = (img.w as f32, img.h as f32);
    let nw = (w * c.abs() + h * s.abs()).ceil().max(1.0) as u32;
    let nh = (w * s.abs() + h * c.abs()).ceil().max(1.0) as u32;
    let mut out = Image::new(nw, nh);
    let (cx, cy) = (w / 2.0, h / 2.0);
    let (ncx, ncy) = (nw as f32 / 2.0, nh as f32 / 2.0);
    for y in 0..nh {
        for x in 0..nw {
            let dx = x as f32 + 0.5 - ncx;
            let dy = y as f32 + 0.5 - ncy;
            let sx = c * dx + s * dy + cx;
            let sy = -s * dx + c * dy + cy;
            if sx >= -0.5 && sy >= -0.5 && sx < w + 0.5 && sy < h + 0.5 {
                out.put(x as i32, y as i32, sample(img, sx, sy));
            }
        }
    }
    out
}

pub fn invert(img: &mut Image) {
    for p in &mut img.px {
        let a = p[3];
        *p = [a - p[0].min(a), a - p[1].min(a), a - p[2].min(a), a];
    }
}
