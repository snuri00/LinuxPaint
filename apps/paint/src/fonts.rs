use std::collections::HashMap;

use ab_glyph::FontArc;
use fontdb::{Database, Family, Query, Stretch, Style, Weight};

pub struct FontCache {
    db: Database,
    pub families: Vec<String>,
    cache: HashMap<(String, bool, bool), Option<FontArc>>,
}

impl FontCache {
    pub fn new() -> Self {
        let mut db = Database::new();
        db.load_system_fonts();
        let mut families: Vec<String> = db
            .faces()
            .filter_map(|f| f.families.first().map(|(n, _)| n.clone()))
            .collect();
        families.sort_by_key(|f| f.to_lowercase());
        families.dedup();
        FontCache { db, families, cache: HashMap::new() }
    }

    pub fn default_family(&self) -> String {
        for want in ["Segoe UI", "Calibri", "Noto Sans", "DejaVu Sans", "Liberation Sans", "Cantarell", "Ubuntu"] {
            if self.families.iter().any(|f| f == want) {
                return want.to_string();
            }
        }
        self.families.first().cloned().unwrap_or_default()
    }

    pub fn get(&mut self, family: &str, bold: bool, italic: bool) -> Option<FontArc> {
        let key = (family.to_string(), bold, italic);
        if let Some(f) = self.cache.get(&key) {
            return f.clone();
        }
        let q = Query {
            families: &[Family::Name(family), Family::SansSerif],
            weight: if bold { Weight::BOLD } else { Weight::NORMAL },
            stretch: Stretch::Normal,
            style: if italic { Style::Italic } else { Style::Normal },
        };
        let font = self.db.query(&q).and_then(|id| {
            self.db
                .with_face_data(id, |data, index| FontArc::try_from_vec(data.to_vec()).ok().filter(|_| index == 0).or_else(|| {
                    ab_glyph::FontVec::try_from_vec_and_index(data.to_vec(), index).ok().map(FontArc::new)
                }))
                .flatten()
        });
        self.cache.insert(key, font.clone());
        font
    }
}
