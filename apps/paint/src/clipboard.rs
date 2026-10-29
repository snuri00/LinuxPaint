use paint_core::Image;

pub fn get_image() -> Option<Image> {
    let mut cb = arboard::Clipboard::new().ok()?;
    let data = cb.get_image().ok()?;
    Some(Image::from_rgba8(data.width as u32, data.height as u32, &data.bytes))
}
