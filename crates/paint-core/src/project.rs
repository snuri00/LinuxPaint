use std::io::{Read, Seek, Write};
use std::sync::Arc;

use tiff::decoder::{Decoder, DecodingResult};
use tiff::encoder::{Compression, DeflateLevel, Rational, TiffEncoder, colortype};
use tiff::tags::{ExtraSamples, ResolutionUnit, Tag};

use crate::color::Color;
use crate::document::{Document, Layer};
use crate::image::Image;

const XMP_TAG: u16 = 700;
const NS: &str = "http://ns.linuxpaint.org/mspaint/1.0/";

fn xmp(visible: bool, background: Option<(Color, bool)>) -> Vec<u8> {
    let mut props = format!("<MSPaint:Visible>{visible}</MSPaint:Visible>");
    if let Some((c, v)) = background {
        props.push_str(&format!(
            "<MSPaint:BackgroundColor>{}</MSPaint:BackgroundColor><MSPaint:BackgroundVisible>{v}</MSPaint:BackgroundVisible>",
            c.to_argb()
        ));
    }
    format!(
        "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?><x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"><rdf:Description rdf:about=\"\" xmlns:MSPaint=\"{NS}\">{props}</rdf:Description></rdf:RDF></x:xmpmeta><?xpacket end=\"w\"?>"
    )
    .into_bytes()
}

fn xmp_field(xmp: &str, name: &str) -> Option<String> {
    let open = format!("MSPaint:{name}");
    let i = xmp.find(&open)?;
    let rest = &xmp[i + open.len()..];
    if let Some(stripped) = rest.strip_prefix("=\"") {
        return stripped.split('"').next().map(str::to_string);
    }
    let rest = rest.strip_prefix('>')?;
    rest.split('<').next().map(|s| s.trim().to_string())
}

pub fn save<W: Write + Seek>(doc: &Document, w: W) -> Result<(), String> {
    let mut enc = TiffEncoder::new(w)
        .map_err(|e| e.to_string())?
        .with_compression(Compression::Deflate(DeflateLevel::Balanced));
    for (i, layer) in doc.layers.iter().enumerate() {
        let mut img = enc.new_image::<colortype::RGB8>(doc.w, doc.h).map_err(|e| e.to_string())?;
        img.extra_samples(&[ExtraSamples::AssociatedAlpha]).map_err(|e| e.to_string())?;
        let bg = if i == 0 { Some((doc.background, doc.background_visible)) } else { None };
        img.encoder().write_tag(Tag::Unknown(XMP_TAG), &xmp(layer.visible, bg)[..]).map_err(|e| e.to_string())?;
        img.resolution(ResolutionUnit::Inch, Rational { n: doc.dpi.round() as u32, d: 1 });
        let data: Vec<u8> = layer.image.px.iter().flat_map(|p| p.iter().copied()).collect();
        img.write_data(&data).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn load<R: Read + Seek>(r: R) -> Result<Document, String> {
    let mut dec = Decoder::new(r).map_err(|e| e.to_string())?;
    let mut layers = Vec::new();
    let mut background = (Color::WHITE, true);
    let mut dpi = 96.0;
    let mut index = 0;
    loop {
        let (w, h) = dec.dimensions().map_err(|e| e.to_string())?;
        let assoc = dec
            .find_tag_unsigned_vec::<u16>(Tag::ExtraSamples)
            .ok()
            .flatten()
            .map(|v| v.first() == Some(&1))
            .unwrap_or(false);
        let xmp_text = dec
            .find_tag(Tag::Unknown(XMP_TAG))
            .ok()
            .flatten()
            .and_then(|v| v.into_u8_vec().ok())
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        if index == 0 {
            if let Some(v) = dec.find_tag(Tag::XResolution).ok().flatten() {
                if let Ok(f) = v.into_f64() {
                    dpi = f;
                }
            }
            if let Some(c) = xmp_field(&xmp_text, "BackgroundColor").and_then(|s| s.parse::<u32>().ok()) {
                background.0 = Color::from_argb(c);
            }
            if let Some(v) = xmp_field(&xmp_text, "BackgroundVisible") {
                background.1 = v.eq_ignore_ascii_case("true");
            }
        }
        let visible = xmp_field(&xmp_text, "Visible").map(|v| !v.eq_ignore_ascii_case("false")).unwrap_or(true);
        let channels = match dec.colortype().map_err(|e| e.to_string())? {
            tiff::ColorType::RGBA(8) => 4,
            tiff::ColorType::RGB(8) => 3,
            tiff::ColorType::Gray(8) => 1,
            other => return Err(format!("unsupported layer format {other:?}")),
        };
        let DecodingResult::U8(data) = dec.read_image().map_err(|e| e.to_string())? else {
            return Err("unsupported sample type".into());
        };
        let mut img = Image::new(w, h);
        for (i, px) in img.px.iter_mut().enumerate() {
            let s = &data[i * channels..i * channels + channels];
            *px = match channels {
                4 if assoc => [s[0], s[1], s[2], s[3]],
                4 => Color::rgba(s[0], s[1], s[2], s[3]).premul(),
                3 => [s[0], s[1], s[2], 255],
                _ => [s[0], s[0], s[0], 255],
            };
        }
        index += 1;
        layers.push(Layer { name: format!("Layer {index}"), visible, image: Arc::new(img) });
        if !dec.more_images() {
            break;
        }
        dec.next_image().map_err(|e| e.to_string())?;
    }
    let first = layers.first().ok_or("empty project")?;
    let mut doc = Document::new(first.image.w, first.image.h);
    doc.current = layers.len() - 1;
    doc.layers = layers;
    doc.background = background.0;
    doc.background_visible = background.1;
    doc.dpi = dpi;
    Ok(doc)
}
