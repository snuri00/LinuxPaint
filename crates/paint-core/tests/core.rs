use std::io::Cursor;
use std::sync::Arc;

use paint_core::fill::flood_fill;
use paint_core::geom::Point;
use paint_core::stroke::{BrushKind, Stroke, StrokeParams};
use paint_core::{Color, Document, Image, project};

#[test]
fn fill_stops_at_border() {
    let mut img = Image::filled(10, 10, Color::WHITE);
    for y in 0..10 {
        img.put(5, y, Color::BLACK.premul());
    }
    flood_fill(&mut img, 1, 1, Color::rgb(255, 0, 0), 0.0);
    assert_eq!(Color::from_premul(img.get(0, 0)), Color::rgb(255, 0, 0));
    assert_eq!(Color::from_premul(img.get(4, 9)), Color::rgb(255, 0, 0));
    assert_eq!(Color::from_premul(img.get(5, 3)), Color::BLACK);
    assert_eq!(Color::from_premul(img.get(7, 7)), Color::WHITE);
}

#[test]
fn fill_tolerance_uses_510_scale() {
    let mut img = Image::filled(4, 1, Color::rgb(100, 100, 100));
    img.put(3, 0, Color::rgb(110, 100, 100).premul());
    flood_fill(&mut img, 0, 0, Color::BLACK, 0.01);
    assert_eq!(Color::from_premul(img.get(3, 0)), Color::rgb(110, 100, 100));
    flood_fill(&mut img, 0, 0, Color::WHITE, 0.0);
    let mut img2 = Image::filled(4, 1, Color::rgb(100, 100, 100));
    img2.put(3, 0, Color::rgb(110, 100, 100).premul());
    flood_fill(&mut img2, 0, 0, Color::BLACK, 0.02);
    assert_eq!(Color::from_premul(img2.get(3, 0)), Color::BLACK);
}

#[test]
fn pencil_draws_hard_line() {
    let params = StrokeParams { kind: BrushKind::Pencil, size: 1.0, color: Color::BLACK, opacity: 1.0, recolor: None };
    let mut s = Stroke::new(params, 20, 20, 1);
    s.add_point(Point::new(2.5, 10.5), 1.0);
    let r = s.add_point(Point::new(17.5, 10.5), 1.0);
    let base = Image::filled(20, 20, Color::WHITE);
    let mut out = base.clone();
    s.composite(&base, &mut out, r);
    assert_eq!(Color::from_premul(out.get(10, 10)), Color::BLACK);
    assert_eq!(Color::from_premul(out.get(10, 12)), Color::WHITE);
}

#[test]
fn project_roundtrip() {
    let mut doc = Document::new(8, 6);
    doc.background = Color::rgb(1, 2, 3);
    doc.add_layer_above();
    Arc::make_mut(&mut doc.layers[1].image).put(2, 2, Color::rgba(200, 10, 10, 128).premul());
    doc.layers[0].visible = false;
    let mut buf = Cursor::new(Vec::new());
    project::save(&doc, &mut buf).unwrap();
    buf.set_position(0);
    let back = project::load(buf).unwrap();
    assert_eq!(back.layers.len(), 2);
    assert_eq!(back.background, Color::rgb(1, 2, 3));
    assert!(!back.layers[0].visible);
    assert_eq!(back.layers[1].image.get(2, 2), doc.layers[1].image.get(2, 2));
}

#[test]
fn pencil_one_pixel_line_has_one_pixel_per_column() {
    let params = StrokeParams { kind: BrushKind::Pencil, size: 1.0, color: Color::BLACK, opacity: 1.0, recolor: None };
    let mut s = Stroke::new(params, 64, 64, 1);
    s.add_point(Point::new(2.3, 5.7), 1.0);
    let r = s.add_point(Point::new(60.8, 31.2), 1.0);
    let base = Image::filled(64, 64, Color::WHITE);
    let mut out = base.clone();
    s.composite(&base, &mut out, r);
    for x in 2..=60 {
        let n = (0..64).filter(|&y| Color::from_premul(out.get(x, y)) == Color::BLACK).count();
        assert_eq!(n, 1, "column {x}");
    }
}
