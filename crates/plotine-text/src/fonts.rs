//! Extra font registration for CJK / custom faces.
//!
//! DejaVu Sans remains the primary face. Registered fonts are appended to the
//! shared [`cosmic_text::FontSystem`] database so shaping can fall back for
//! missing glyphs (e.g. Chinese/Japanese/Korean). PDF conversion reloads the
//! same bytes into usvg's fontdb.

use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use plotine_core::{PlotError, Result};

use crate::engine::with_font_system;

#[derive(Clone)]
struct RegisteredFont {
    family: String,
    data: Arc<[u8]>,
}

fn registry() -> &'static Mutex<Vec<RegisteredFont>> {
    static REG: OnceLock<Mutex<Vec<RegisteredFont>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(Vec::new()))
}

/// Family names of fonts registered via [`register_font_file`] / [`register_font_data`].
pub fn registered_families() -> Vec<String> {
    registry()
        .lock()
        .map(|g| g.iter().map(|f| f.family.clone()).collect())
        .unwrap_or_default()
}

/// Font bytes for PDF / other backends that need to embed registered faces.
pub fn registered_font_data() -> Vec<Arc<[u8]>> {
    registry()
        .lock()
        .map(|g| g.iter().map(|f| Arc::clone(&f.data)).collect())
        .unwrap_or_default()
}

/// CSS `font-family` list: DejaVu first, then registered faces, then generics.
pub fn svg_font_family_list() -> String {
    let mut parts = vec![crate::FONT_FAMILY.to_string()];
    for fam in registered_families() {
        if !parts.iter().any(|p| p == &fam) {
            parts.push(fam);
        }
    }
    parts.push("Arial".into());
    parts.push("sans-serif".into());
    parts.join(", ")
}

/// Load a font file (`.ttf` / `.otf` / `.ttc`) into the shared text engine.
///
/// Returns the primary family name reported by the font database.
pub fn register_font_file(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let data = std::fs::read(path).map_err(|e| {
        PlotError::text(format!(
            "failed to read font {}: {e}. suggestion: pass a readable .ttf/.otf/.ttc path, or call load_system_cjk()",
            path.display()
        ))
    })?;
    register_font_data(data)
}

/// Load font bytes into the shared text engine.
///
/// Returns the primary family name reported by the font database.
pub fn register_font_data(data: Vec<u8>) -> Result<String> {
    if data.is_empty() {
        return Err(PlotError::text(
            "empty font data. suggestion: pass non-empty .ttf/.otf/.ttc bytes",
        ));
    }
    let data: Arc<[u8]> = Arc::from(data.into_boxed_slice());
    let family = with_font_system(|fs| {
        let before: Vec<_> = fs.db().faces().map(|f| f.id).collect();
        fs.db_mut().load_font_data(data.to_vec());
        let fam = fs
            .db()
            .faces()
            .filter(|face| !before.contains(&face.id))
            .find_map(|face| face.families.first().map(|(name, _)| name.clone()))
            .unwrap_or_else(|| "Registered".into());
        Ok(fam)
    })?;

    let mut guard = registry()
        .lock()
        .map_err(|_| PlotError::text("font registry lock poisoned"))?;
    if !guard
        .iter()
        .any(|f| f.family == family && f.data.as_ref() == data.as_ref())
    {
        guard.push(RegisteredFont {
            family: family.clone(),
            data,
        });
    }
    Ok(family)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_simhei_when_present() {
        let path = std::path::Path::new(r"C:\Windows\Fonts\simhei.ttf");
        if !path.exists() {
            return;
        }
        let family = register_font_file(path).expect("register simhei");
        assert!(!family.is_empty());
        assert!(registered_families().iter().any(|f| f == &family));
        assert!(svg_font_family_list().contains(&family));
    }
}
