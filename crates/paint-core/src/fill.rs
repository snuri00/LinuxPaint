use std::collections::VecDeque;

use crate::color::Color;
use crate::geom::Rect;
use crate::image::Image;

fn argb(p: [u8; 4]) -> [i32; 4] {
    let c = Color::from_premul(p);
    [c.a as i32, c.r as i32, c.g as i32, c.b as i32]
}

fn matches(p: [i32; 4], seed: [i32; 4], tol2: f64) -> bool {
    if p == seed {
        return true;
    }
    if tol2 <= 0.0 {
        return false;
    }
    let d: i64 = (0..4).map(|i| ((p[i] - seed[i]) as i64).pow(2)).sum();
    (d as f64) <= tol2 * 260100.0
}

pub fn flood_fill(img: &mut Image, x: i32, y: i32, color: Color, tolerance: f64) -> Rect {
    let (w, h) = (img.w as i32, img.h as i32);
    if x < 0 || y < 0 || x >= w || y >= h {
        return Rect::EMPTY;
    }
    let seed = argb(img.get(x, y));
    let fill = color.premul();
    let fill_argb = argb(fill);
    if fill_argb == seed {
        return Rect::EMPTY;
    }
    let tol2 = tolerance * tolerance;
    let mut visited = if tolerance > 0.0 { Some(vec![false; (w * h) as usize]) } else { None };
    let mut dirty = Rect::new(x, y, x + 1, y + 1);
    let mut queue: VecDeque<(i32, i32, i32)> = VecDeque::new();

    let test = |img: &Image, visited: &mut Option<Vec<bool>>, px: i32, py: i32| -> bool {
        let i = (py * w + px) as usize;
        if let Some(v) = visited {
            if v[i] {
                return false;
            }
            v[i] = true;
        }
        matches(argb(img.px[i]), seed, tol2)
    };

    if !test(img, &mut visited, x, y) {
        return Rect::EMPTY;
    }
    let mut x0 = x;
    let mut x1 = x;
    img.put(x, y, fill);
    while x0 - 1 >= 0 && test(img, &mut visited, x0 - 1, y) {
        x0 -= 1;
        img.put(x0, y, fill);
    }
    while x1 + 1 < w && test(img, &mut visited, x1 + 1, y) {
        x1 += 1;
        img.put(x1, y, fill);
    }
    queue.push_back((x0, x1, y));

    while let Some((sx0, sx1, sy)) = queue.pop_front() {
        dirty = dirty.union(&Rect::new(sx0, sy, sx1 + 1, sy + 1));
        for ny in [sy - 1, sy + 1] {
            if ny < 0 || ny >= h {
                continue;
            }
            let mut cx = sx0;
            while cx <= sx1 {
                if !test(img, &mut visited, cx, ny) {
                    cx += 1;
                    continue;
                }
                let mut a = cx;
                img.put(cx, ny, fill);
                while a - 1 >= 0 && test(img, &mut visited, a - 1, ny) {
                    a -= 1;
                    img.put(a, ny, fill);
                }
                let mut b = cx;
                while b + 1 < w && test(img, &mut visited, b + 1, ny) {
                    b += 1;
                    img.put(b, ny, fill);
                }
                queue.push_back((a, b, ny));
                cx = b + 2;
            }
        }
    }
    dirty
}
