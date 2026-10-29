use std::borrow::Cow;

use paint_core::Image;

pub fn set_image(img: &Image) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let data = arboard::ImageData { width: img.w as usize, height: img.h as usize, bytes: Cow::Owned(img.to_rgba8()) };
        let _ = cb.set_image(data);
    }
}

pub fn get_image() -> Option<Image> {
    let mut cb = arboard::Clipboard::new().ok()?;
    let data = cb.get_image().ok()?;
    Some(Image::from_rgba8(data.width as u32, data.height as u32, &data.bytes))
}

pub fn get_text() -> Option<String> {
    let mut cb = arboard::Clipboard::new().ok()?;
    cb.get_text().ok().filter(|t| !t.is_empty()).map(|t| t.replace("\r\n", "\n"))
}
