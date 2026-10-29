use std::process::Command;

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily};

pub struct SystemLook {
    pub accent: Color32,
    pub buttons_left: Vec<String>,
    pub buttons_right: Vec<String>,
    pub prefer_dark: Option<bool>,
}

fn portal_value(namespace: &str, key: &str) -> Option<zbus::zvariant::OwnedValue> {
    let conn = zbus::blocking::Connection::session().ok()?;
    let reply = conn
        .call_method(
            Some("org.freedesktop.portal.Desktop"),
            "/org/freedesktop/portal/desktop",
            Some("org.freedesktop.portal.Settings"),
            "ReadOne",
            &(namespace, key),
        )
        .ok()?;
    let v: zbus::zvariant::OwnedValue = reply.body().deserialize().ok()?;
    match &*v {
        zbus::zvariant::Value::Value(inner) => zbus::zvariant::OwnedValue::try_from((**inner).clone()).ok(),
        _ => Some(v),
    }
}

fn portal_string(namespace: &str, key: &str) -> Option<String> {
    let v = portal_value(namespace, key)?;
    String::try_from(v).ok()
}

fn gsetting(schema: &str, key: &str) -> Option<String> {
    if let Some(v) = portal_string(schema, key) {
        return Some(v);
    }
    let out = Command::new("gsettings").args(["get", schema, key]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().trim_matches('\'').to_string())
}

fn accent_named(name: &str) -> Option<Color32> {
    let c = match name {
        "blue" => 0x3584E4,
        "teal" => 0x2190A4,
        "green" => 0x3A944A,
        "yellow" => 0xC88800,
        "orange" => 0xED5B00,
        "red" => 0xE62D42,
        "pink" => 0xD56199,
        "purple" => 0x9141AC,
        "slate" => 0x6F8396,
        "bark" => 0x787859,
        "sage" => 0x657B69,
        "olive" => 0x4B8501,
        "viridian" => 0x03875B,
        "prussiangreen" => 0x308280,
        "magenta" => 0xB34CB3,
        _ => return None,
    };
    Some(Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8))
}

fn portal_accent() -> Option<Color32> {
    let v = portal_value("org.freedesktop.appearance", "accent-color")?;
    let (r, g, b): (f64, f64, f64) = v.try_into().ok()?;
    if !(0.0..=1.0).contains(&r) || !(0.0..=1.0).contains(&g) || !(0.0..=1.0).contains(&b) {
        return None;
    }
    Some(Color32::from_rgb((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8))
}

fn portal_dark() -> Option<bool> {
    let v = portal_value("org.freedesktop.appearance", "color-scheme")?;
    match u32::try_from(v).ok()? {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    }
}

pub fn detect() -> SystemLook {
    let iface = "org.gnome.desktop.interface";
    let accent = portal_accent()
        .or_else(|| gsetting(iface, "accent-color").and_then(|a| accent_named(&a)))
        .or_else(|| {
            let theme = gsetting(iface, "gtk-theme")?.to_lowercase();
            let theme = theme.trim_end_matches("-dark");
            if let Some(rest) = theme.strip_prefix("yaru-") {
                accent_named(rest)
            } else if theme == "yaru" {
                Some(Color32::from_rgb(0xE9, 0x54, 0x20))
            } else {
                None
            }
        })
        .or_else(|| {
            let os = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
            os.lines().any(|l| l == "ID=ubuntu").then(|| Color32::from_rgb(0xE9, 0x54, 0x20))
        })
        .unwrap_or(Color32::from_rgb(0x35, 0x84, 0xE4));
    let layout = gsetting("org.gnome.desktop.wm.preferences", "button-layout").unwrap_or_else(|| ":minimize,maximize,close".into());
    let (l, r) = layout.split_once(':').unwrap_or(("", layout.as_str()));
    let split = |s: &str| s.split(',').map(str::trim).filter(|x| matches!(*x, "minimize" | "maximize" | "close")).map(String::from).collect();
    let prefer_dark = portal_dark()
        .or_else(|| gsetting(iface, "color-scheme").map(|s| s.contains("dark")))
        .or_else(|| gsetting(iface, "gtk-theme").map(|t| t.to_lowercase().ends_with("dark")));
    SystemLook { accent, buttons_left: split(l), buttons_right: split(r), prefer_dark }
}

pub fn install_ui_font(ctx: &egui::Context) {
    let family = gsetting("org.gnome.desktop.interface", "font-name")
        .map(|f| f.trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ' || c == '.').to_string())
        .unwrap_or_else(|| "Cantarell".into());
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    for dir in ["/run/host/fonts", "/run/host/user-fonts", "/run/host/local-fonts"] {
        db.load_fonts_dir(dir);
    }
    let mut fonts = FontDefinitions::default();
    let mut added = false;
    let regular = db.query(&fontdb::Query {
        families: &[fontdb::Family::Name(&family), fontdb::Family::Name("Cantarell"), fontdb::Family::Name("Noto Sans"), fontdb::Family::SansSerif],
        ..Default::default()
    });
    if let Some(data) = regular.and_then(|id| db.with_face_data(id, |d, i| (d.to_vec(), i))) {
        let mut fd = FontData::from_owned(data.0);
        fd.index = data.1;
        fonts.font_data.insert("ui".into(), fd.into());
        added = true;
    }
    let bold_families = [family.as_str(), "Ubuntu", "Cantarell", "Noto Sans", "DejaVu Sans"];
    let bold = bold_families.iter().find_map(|f| {
        let id = db.query(&fontdb::Query { families: &[fontdb::Family::Name(f)], weight: fontdb::Weight::BOLD, ..Default::default() })?;
        let face = db.face(id)?;
        let path_ok = match &face.source {
            fontdb::Source::File(p) => !p.to_string_lossy().contains('['),
            _ => true,
        };
        (face.weight >= fontdb::Weight::SEMIBOLD && Some(id) != regular && path_ok).then_some(id)
    });
    if let Some(data) = bold.and_then(|id| db.with_face_data(id, |d, i| (d.to_vec(), i))) {
        let mut fd = FontData::from_owned(data.0);
        fd.index = data.1;
        fonts.font_data.insert("ui-bold".into(), fd.into());
    }
    if !added {
        return;
    }
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "ui".into());
    if fonts.font_data.contains_key("ui-bold") {
        fonts.families.insert(FontFamily::Name("bold".into()), vec!["ui-bold".into(), "ui".into()]);
    } else {
        fonts.families.insert(FontFamily::Name("bold".into()), vec!["ui".into()]);
    }
    ctx.set_fonts(fonts);
}
