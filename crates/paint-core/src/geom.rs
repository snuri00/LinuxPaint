use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
    pub fn len(self) -> f32 {
        self.x.hypot(self.y)
    }
    pub fn dot(self, o: Point) -> f32 {
        self.x * o.x + self.y * o.y
    }
    pub fn normalized(self) -> Point {
        let l = self.len();
        if l < 1e-6 { Point::new(1.0, 0.0) } else { self * (1.0 / l) }
    }
    pub fn perp(self) -> Point {
        Point::new(-self.y, self.x)
    }
    pub fn lerp(self, o: Point, t: f32) -> Point {
        self + (o - self) * t
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, o: Point) -> Point {
        Point::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for Point {
    type Output = Point;
    fn sub(self, o: Point) -> Point {
        Point::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, s: f32) -> Point {
        Point::new(self.x * s, self.y * s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl Rect {
    pub const EMPTY: Rect = Rect { x0: 0, y0: 0, x1: 0, y1: 0 };

    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Rect { x0, y0, x1, y1 }
    }
    pub fn from_points(a: Point, b: Point) -> Self {
        Rect::new(
            a.x.min(b.x).floor() as i32,
            a.y.min(b.y).floor() as i32,
            a.x.max(b.x).ceil() as i32,
            a.y.max(b.y).ceil() as i32,
        )
    }
    pub fn width(&self) -> i32 {
        (self.x1 - self.x0).max(0)
    }
    pub fn height(&self) -> i32 {
        (self.y1 - self.y0).max(0)
    }
    pub fn is_empty(&self) -> bool {
        self.width() == 0 || self.height() == 0
    }
    pub fn union(&self, o: &Rect) -> Rect {
        if self.is_empty() {
            return *o;
        }
        if o.is_empty() {
            return *self;
        }
        Rect::new(self.x0.min(o.x0), self.y0.min(o.y0), self.x1.max(o.x1), self.y1.max(o.y1))
    }
    pub fn intersect(&self, o: &Rect) -> Rect {
        let r = Rect::new(self.x0.max(o.x0), self.y0.max(o.y0), self.x1.min(o.x1), self.y1.min(o.y1));
        if r.is_empty() { Rect::EMPTY } else { r }
    }
    pub fn inflate(&self, d: i32) -> Rect {
        Rect::new(self.x0 - d, self.y0 - d, self.x1 + d, self.y1 + d)
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x0 && x < self.x1 && y >= self.y0 && y < self.y1
    }
}

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
