//! Optional CJK / custom font loading (`feature = "cjk"`).
//!
//! plotine never embeds a CJK face (too large for the default crate). Call
//! [`register_font_file`] / [`load_system_cjk`] before rendering so PNG shaping
//! and PDF embedding can fall back for Chinese/Japanese/Korean glyphs.
//!
//! ```ignore
//! plotine::fonts::load_system_cjk()?;
//! Figure::new().axes(|ax| {
//!     ax.title("中文标题");
//!     ax.line(&x, &y);
//! }).save("cjk.png")?;
//! ```

use plotine_core::{PlotError, Result};

pub use plotine_text::{
    register_font_data, register_font_file, registered_families, svg_font_family_list,
};

/// Candidate system font paths for common CJK faces (Windows / macOS / Linux).
pub fn system_cjk_candidates() -> Vec<&'static str> {
    #[cfg(windows)]
    {
        vec![
            r"C:\Windows\Fonts\msyh.ttc",
            r"C:\Windows\Fonts\simhei.ttf",
            r"C:\Windows\Fonts\simsun.ttc",
            r"C:\Windows\Fonts\msjh.ttc",
            r"C:\Windows\Fonts\malgun.ttf",
            r"C:\Windows\Fonts\YuGothR.ttc",
        ]
    }
    #[cfg(target_os = "macos")]
    {
        vec![
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/Library/Fonts/Arial Unicode.ttf",
        ]
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        vec![
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/opentype/source-han-sans/SourceHanSansCN-Regular.otf",
        ]
    }
    #[cfg(not(any(windows, target_os = "macos", unix)))]
    {
        Vec::new()
    }
}

/// Register the first readable font from [`system_cjk_candidates`].
///
/// Returns the loaded family names (usually one). Fails if none of the
/// candidates exist — then call [`register_font_file`] with an explicit path.
pub fn load_system_cjk() -> Result<Vec<String>> {
    let mut loaded = Vec::new();
    for path in system_cjk_candidates() {
        let p = std::path::Path::new(path);
        if !p.is_file() {
            continue;
        }
        match register_font_file(p) {
            Ok(family) => {
                loaded.push(family);
                break;
            }
            Err(_) => continue,
        }
    }
    if loaded.is_empty() {
        return Err(PlotError::text(
            "no system CJK font found. suggestion: install Noto Sans CJK / Microsoft YaHei, or plotine::fonts::register_font_file(\"path/to/cjk.ttf\")",
        ));
    }
    Ok(loaded)
}
