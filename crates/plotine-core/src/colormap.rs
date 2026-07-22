//! Continuous colormaps (perceptually uniform defaults) and value norms.

use crate::{Color, PlotError, Result};

/// How scalar data is normalized into `[0, 1]` before sampling a colormap.
///
/// Independent of axis [`ScaleType`](crate::ScaleType).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Norm {
    /// Linear mapping (default; matches historical `Colormap::map`).
    #[default]
    Linear,
    /// Logarithmic mapping; domain must be strictly positive.
    Log,
    /// Diverging norm (matplotlib `TwoSlopeNorm`): `vcenter` maps to `0.5`.
    ///
    /// Requires `vmin < vcenter < vmax` for a well-defined piecewise map;
    /// otherwise falls back to linear behaviour over `[vmin, vmax]`.
    TwoSlope {
        /// Data value mapped to the colormap midpoint.
        vcenter: f64,
    },
}

impl Norm {
    /// Normalize `value` into `[0, 1]` given data limits.
    ///
    /// Non-finite values yield `0.0`. For [`Norm::Log`], non-positive inputs
    /// clamp to the low end (`0.0`).
    pub fn normalize(self, value: f64, vmin: f64, vmax: f64) -> f64 {
        if !value.is_finite() {
            return 0.0;
        }
        match self {
            Self::Linear => {
                let span = (vmax - vmin).abs().max(1e-12);
                ((value - vmin) / span).clamp(0.0, 1.0)
            }
            Self::Log => {
                if !(vmin.is_finite() && vmax.is_finite()) || vmin <= 0.0 || vmax <= 0.0 {
                    return 0.0;
                }
                if value <= 0.0 {
                    return 0.0;
                }
                let a = vmin.ln();
                let b = vmax.ln();
                let span = (b - a).abs().max(1e-12);
                ((value.ln() - a) / span).clamp(0.0, 1.0)
            }
            Self::TwoSlope { vcenter } => {
                if !(vmin.is_finite()
                    && vmax.is_finite()
                    && vcenter.is_finite()
                    && vmin < vcenter
                    && vcenter < vmax)
                {
                    let span = (vmax - vmin).abs().max(1e-12);
                    return ((value - vmin) / span).clamp(0.0, 1.0);
                }
                if value <= vcenter {
                    let span = (vcenter - vmin).max(1e-12);
                    (0.5 * (value - vmin) / span).clamp(0.0, 0.5)
                } else {
                    let span = (vmax - vcenter).max(1e-12);
                    (0.5 + 0.5 * (value - vcenter) / span).clamp(0.5, 1.0)
                }
            }
        }
    }

    /// Inverse of [`normalize`](Self::normalize): `t ∈ [0,1]` → data value.
    pub fn denormalize(self, t: f64, vmin: f64, vmax: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => vmin + t * (vmax - vmin),
            Self::Log => {
                if vmin <= 0.0 || vmax <= 0.0 {
                    return vmin;
                }
                let a = vmin.ln();
                let b = vmax.ln();
                (a + t * (b - a)).exp()
            }
            Self::TwoSlope { vcenter } => {
                if !(vmin < vcenter && vcenter < vmax) {
                    return vmin + t * (vmax - vmin);
                }
                if t <= 0.5 {
                    vmin + (t / 0.5) * (vcenter - vmin)
                } else {
                    vcenter + ((t - 0.5) / 0.5) * (vmax - vcenter)
                }
            }
        }
    }
}

/// User-defined colormap from color stops (matplotlib `LinearSegmentedColormap` subset).
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentedColormap {
    /// Sorted `(t, color)` with `t` in `[0, 1]`.
    stops: Vec<(f64, Color)>,
}

impl SegmentedColormap {
    /// Evenly spaced colors from `t = 0` to `t = 1` (at least two colors).
    pub fn from_colors(colors: impl IntoIterator<Item = Color>) -> Result<Self> {
        let colors: Vec<Color> = colors.into_iter().collect();
        if colors.len() < 2 {
            return Err(PlotError::render(
                "SegmentedColormap::from_colors needs at least 2 colors",
            ));
        }
        let n = (colors.len() - 1) as f64;
        let stops = colors
            .into_iter()
            .enumerate()
            .map(|(i, c)| (i as f64 / n, c))
            .collect();
        Ok(Self { stops })
    }

    /// Explicit `(position, color)` stops; positions are clamped to `[0, 1]` and sorted.
    pub fn from_stops(stops: impl IntoIterator<Item = (f64, Color)>) -> Result<Self> {
        let mut stops: Vec<(f64, Color)> = stops
            .into_iter()
            .map(|(t, c)| (t.clamp(0.0, 1.0), c))
            .collect();
        if stops.len() < 2 {
            return Err(PlotError::render(
                "SegmentedColormap::from_stops needs at least 2 stops",
            ));
        }
        stops.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        // Ensure endpoints cover 0 and 1 for stable sampling.
        if stops[0].0 > 0.0 {
            stops.insert(0, (0.0, stops[0].1));
        }
        if stops.last().map(|s| s.0).unwrap_or(0.0) < 1.0 {
            let last = *stops.last().unwrap();
            stops.push((1.0, last.1));
        }
        Ok(Self { stops })
    }

    /// Sample at `t ∈ [0, 1]` (clamped), linearly interpolating RGB.
    pub fn sample(&self, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        let stops = &self.stops;
        if stops.is_empty() {
            return Color::BLACK;
        }
        if t <= stops[0].0 {
            return stops[0].1;
        }
        if t >= stops[stops.len() - 1].0 {
            return stops[stops.len() - 1].1;
        }
        for w in stops.windows(2) {
            let (t0, c0) = w[0];
            let (t1, c1) = w[1];
            if t >= t0 && t <= t1 {
                let span = (t1 - t0).max(1e-12);
                let f = ((t - t0) / span).clamp(0.0, 1.0);
                return lerp_color(c0, c1, f);
            }
        }
        stops[stops.len() - 1].1
    }

    /// Sample reversed (`1 - t`).
    pub fn sample_reversed(&self, t: f64) -> Color {
        self.sample(1.0 - t)
    }

    /// Map a data value with `norm`.
    pub fn map_norm(&self, value: f64, vmin: f64, vmax: f64, norm: Norm) -> Color {
        if !value.is_finite() {
            return Color::rgba(0, 0, 0, 0);
        }
        self.sample(norm.normalize(value, vmin, vmax))
    }
}

/// Named or custom colormap handle used by heatmap / hist2d / … artists.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmap {
    /// Built-in named map ([`Colormap`]).
    Named(Colormap),
    /// User stops ([`SegmentedColormap`]).
    Segmented(SegmentedColormap),
}

impl From<Colormap> for Cmap {
    fn from(c: Colormap) -> Self {
        Self::Named(c)
    }
}

impl From<SegmentedColormap> for Cmap {
    fn from(c: SegmentedColormap) -> Self {
        Self::Segmented(c)
    }
}

impl Default for Cmap {
    fn default() -> Self {
        Self::Named(Colormap::Viridis)
    }
}

impl Cmap {
    /// Listed palette length, if this is a qualitative named map.
    pub fn listed_len(&self) -> Option<usize> {
        match self {
            Self::Named(c) => c.listed_len(),
            Self::Segmented(_) => None,
        }
    }

    /// Sample at `t ∈ [0, 1]`.
    pub fn sample(&self, t: f64) -> Color {
        match self {
            Self::Named(c) => c.sample(t),
            Self::Segmented(c) => c.sample(t),
        }
    }

    /// Sample reversed.
    pub fn sample_reversed(&self, t: f64) -> Color {
        match self {
            Self::Named(c) => c.sample_reversed(t),
            Self::Segmented(c) => c.sample_reversed(t),
        }
    }

    /// Map data → color with linear norm.
    pub fn map(&self, value: f64, vmin: f64, vmax: f64) -> Color {
        self.map_norm(value, vmin, vmax, Norm::Linear)
    }

    /// Map data → color with `norm`.
    pub fn map_norm(&self, value: f64, vmin: f64, vmax: f64, norm: Norm) -> Color {
        match self {
            Self::Named(c) => c.map_norm(value, vmin, vmax, norm),
            Self::Segmented(c) => c.map_norm(value, vmin, vmax, norm),
        }
    }

    /// Reversed mapping with linear norm.
    pub fn map_reversed(&self, value: f64, vmin: f64, vmax: f64) -> Color {
        self.map_reversed_norm(value, vmin, vmax, Norm::Linear)
    }

    /// Reversed mapping with `norm`.
    pub fn map_reversed_norm(&self, value: f64, vmin: f64, vmax: f64, norm: Norm) -> Color {
        match self {
            Self::Named(c) => c.map_reversed_norm(value, vmin, vmax, norm),
            Self::Segmented(c) => {
                if !value.is_finite() {
                    return Color::rgba(0, 0, 0, 0);
                }
                c.sample_reversed(norm.normalize(value, vmin, vmax))
            }
        }
    }
}

/// Named continuous colormap.
///
/// Covers the most-used matplotlib colormaps organized by family:
/// perceptually uniform, sequential single-hue, sequential multi-hue,
/// diverging, cyclic, miscellaneous, and qualitative/listed.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Colormap {
    // — Perceptually uniform sequential ——————————————————————————
    #[default]
    Viridis,
    Plasma,
    Inferno,
    Magma,
    Cividis,

    // — Sequential single-hue ————————————————————————————————————
    Greys,
    Blues,
    Greens,
    Reds,
    Oranges,
    Purples,

    // — Sequential multi-hue —————————————————————————————————————
    YlOrRd,
    YlOrBr,
    YlGnBu,
    BuGn,
    BuPu,
    GnBu,
    OrRd,
    PuBu,
    PuRd,
    RdPu,
    YlGn,
    Hot,
    Copper,
    Bone,
    Gray,
    PuBuGn,
    Binary,
    GistYarg,
    GistGray,
    Pink,
    Spring,
    Summer,
    Autumn,
    Winter,
    Cool,
    Wistia,
    Afmhot,
    GistHeat,

    // — Diverging —————————————————————————————————————————————————
    /// Diverging blue–white–red (`coolwarm`).
    Coolwarm,
    /// Diverging blue–white–red (matplotlib `RdBu_r`).
    RdBuR,
    Seismic,
    Bwr,
    PiYG,
    PRGn,
    BrBG,
    PuOr,
    RdYlBu,
    RdYlGn,
    Spectral,
    RdGy,
    RdBu,

    // — Cyclic ————————————————————————————————————————————————————
    Twilight,
    TwilightShifted,

    // — Other —————————————————————————————————————————————————————
    Jet,
    Turbo,
    Hsv,
    Rainbow,
    Ocean,
    GistEarth,
    Terrain,
    GistStern,
    Gnuplot,
    Gnuplot2,
    CMRmap,
    Cubehelix,
    Brg,
    GistRainbow,
    NipySpectral,
    GistNcar,

    // — Qualitative / listed (no interpolation) ——————————————————
    /// Matplotlib `tab10` listed / categorical palette (10 colors).
    Tab10,
    Set1,
    Set2,
    Set3,
    Paired,
    Pastel1,
    Pastel2,
    Dark2,
    Accent,
    Tab20,
    Tab20b,
    Tab20c,
}

impl Colormap {
    /// Number of discrete colors for listed colormaps, else `None`.
    ///
    /// Listed maps use nearest-bin indexing (no interpolation) like
    /// matplotlib `ListedColormap`.
    pub fn listed_len(self) -> Option<usize> {
        match self {
            Self::Tab10 => Some(TAB10.len()),
            Self::Set1 => Some(SET1.len()),
            Self::Set2 => Some(SET2.len()),
            Self::Set3 => Some(SET3.len()),
            Self::Paired => Some(PAIRED.len()),
            Self::Pastel1 => Some(PASTEL1.len()),
            Self::Pastel2 => Some(PASTEL2.len()),
            Self::Dark2 => Some(DARK2.len()),
            Self::Accent => Some(ACCENT.len()),
            Self::Tab20 => Some(TAB20.len()),
            Self::Tab20b => Some(TAB20B.len()),
            Self::Tab20c => Some(TAB20C.len()),
            _ => None,
        }
    }

    /// Sample the colormap at `t` in `[0, 1]` (clamped).
    ///
    /// Continuous maps interpolate between stops; listed maps (`Tab10` etc.)
    /// use nearest-bin indexing like matplotlib `ListedColormap`.
    pub fn sample(self, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        let stops = self.stops();
        if stops.is_empty() {
            return Color::BLACK;
        }
        if let Some(n) = self.listed_len() {
            if t >= 1.0 {
                return *stops.last().unwrap();
            }
            let idx = ((t * n as f64).floor() as usize).min(n - 1);
            return stops[idx];
        }
        if t <= 0.0 {
            return stops[0];
        }
        if t >= 1.0 {
            return *stops.last().unwrap();
        }
        let n = stops.len() - 1;
        let x = t * n as f64;
        let i = (x.floor() as usize).min(n - 1);
        let f = x - i as f64;
        lerp_color(stops[i], stops[i + 1], f)
    }

    /// Sample the colormap at `1 - t` (reversed direction).
    ///
    /// Equivalent to matplotlib's `*_r` suffix colormaps.
    pub fn sample_reversed(self, t: f64) -> Color {
        self.sample(1.0 - t)
    }

    /// Map a data value through `[vmin, vmax]` with linear normalization.
    pub fn map(self, value: f64, vmin: f64, vmax: f64) -> Color {
        self.map_norm(value, vmin, vmax, Norm::Linear)
    }

    /// Map a data value with reversed colormap direction.
    ///
    /// Equivalent to `self.sample_reversed(norm.normalize(value, vmin, vmax))`.
    pub fn map_reversed(self, value: f64, vmin: f64, vmax: f64) -> Color {
        self.map_reversed_norm(value, vmin, vmax, Norm::Linear)
    }

    /// Map a data value through `[vmin, vmax]` using `norm`.
    pub fn map_norm(self, value: f64, vmin: f64, vmax: f64, norm: Norm) -> Color {
        if !value.is_finite() {
            return Color::rgba(0, 0, 0, 0);
        }
        self.sample(norm.normalize(value, vmin, vmax))
    }

    /// Map a data value with reversed colormap direction and custom norm.
    pub fn map_reversed_norm(self, value: f64, vmin: f64, vmax: f64, norm: Norm) -> Color {
        if !value.is_finite() {
            return Color::rgba(0, 0, 0, 0);
        }
        self.sample_reversed(norm.normalize(value, vmin, vmax))
    }

    fn stops(self) -> &'static [Color] {
        match self {
            // Perceptually uniform
            Self::Viridis => &VIRIDIS,
            Self::Plasma => &PLASMA,
            Self::Inferno => &INFERNO,
            Self::Magma => &MAGMA,
            Self::Cividis => &CIVIDIS,
            // Sequential single-hue
            Self::Greys => &GREYS,
            Self::Blues => &BLUES,
            Self::Greens => &GREENS,
            Self::Reds => &REDS,
            Self::Oranges => &ORANGES,
            Self::Purples => &PURPLES,
            // Sequential multi-hue
            Self::YlOrRd => &YLORRD,
            Self::YlOrBr => &YLORBR,
            Self::YlGnBu => &YLGNBU,
            Self::BuGn => &BUGN,
            Self::BuPu => &BUPU,
            Self::GnBu => &GNBU,
            Self::OrRd => &ORRD,
            Self::PuBu => &PUBU,
            Self::PuRd => &PURD,
            Self::RdPu => &RDPU,
            Self::YlGn => &YLGN,
            Self::Hot => &HOT,
            Self::Copper => &COPPER,
            Self::Bone => &BONE,
            Self::Gray => &GRAY,
            Self::PuBuGn => &PUBUGN,
            Self::Binary => &BINARY,
            Self::GistYarg => &GISTYARG,
            Self::GistGray => &GISTGRAY,
            Self::Pink => &PINK,
            Self::Spring => &SPRING,
            Self::Summer => &SUMMER,
            Self::Autumn => &AUTUMN,
            Self::Winter => &WINTER,
            Self::Cool => &COOL,
            Self::Wistia => &WISTIA,
            Self::Afmhot => &AFMHOT,
            Self::GistHeat => &GISTHEAT,
            // Diverging
            Self::Coolwarm => &COOLWARM,
            Self::RdBuR => &RDBU_R,
            Self::Seismic => &SEISMIC,
            Self::Bwr => &BWR,
            Self::PiYG => &PIYG,
            Self::PRGn => &PRGN,
            Self::BrBG => &BRBG,
            Self::PuOr => &PUOR,
            Self::RdYlBu => &RDYLBU,
            Self::RdYlGn => &RDYLGN,
            Self::Spectral => &SPECTRAL,
            Self::RdGy => &RDGY,
            Self::RdBu => &RDBU,
            // Cyclic
            Self::Twilight => &TWILIGHT,
            Self::TwilightShifted => &TWILIGHT_SHIFTED,
            // Other
            Self::Jet => &JET,
            Self::Turbo => &TURBO,
            Self::Hsv => &HSV,
            Self::Rainbow => &RAINBOW,
            Self::Ocean => &OCEAN,
            Self::GistEarth => &GISTEARTH,
            Self::Terrain => &TERRAIN,
            Self::GistStern => &GISTSTERN,
            Self::Gnuplot => &GNUPLOT,
            Self::Gnuplot2 => &GNUPLOT2,
            Self::CMRmap => &CMRMAP,
            Self::Cubehelix => &CUBEHELIX,
            Self::Brg => &BRG,
            Self::GistRainbow => &GISTRAINBOW,
            Self::NipySpectral => &NIPYSPECTRAL,
            Self::GistNcar => &GISTNCAR,
            // Qualitative / listed
            Self::Tab10 => &TAB10,
            Self::Set1 => &SET1,
            Self::Set2 => &SET2,
            Self::Set3 => &SET3,
            Self::Paired => &PAIRED,
            Self::Pastel1 => &PASTEL1,
            Self::Pastel2 => &PASTEL2,
            Self::Dark2 => &DARK2,
            Self::Accent => &ACCENT,
            Self::Tab20 => &TAB20,
            Self::Tab20b => &TAB20B,
            Self::Tab20c => &TAB20C,
        }
    }
}

impl std::str::FromStr for Colormap {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace(['-', '_'], "").as_str() {
            "viridis" => Ok(Self::Viridis),
            "plasma" => Ok(Self::Plasma),
            "inferno" => Ok(Self::Inferno),
            "magma" => Ok(Self::Magma),
            "cividis" => Ok(Self::Cividis),
            "greys" | "grays" => Ok(Self::Greys),
            "blues" => Ok(Self::Blues),
            "greens" => Ok(Self::Greens),
            "reds" => Ok(Self::Reds),
            "oranges" => Ok(Self::Oranges),
            "purples" => Ok(Self::Purples),
            "ylorrd" => Ok(Self::YlOrRd),
            "ylorbr" => Ok(Self::YlOrBr),
            "ylgnbu" => Ok(Self::YlGnBu),
            "bugn" => Ok(Self::BuGn),
            "bupu" => Ok(Self::BuPu),
            "gnbu" => Ok(Self::GnBu),
            "orrd" => Ok(Self::OrRd),
            "pubu" => Ok(Self::PuBu),
            "purd" => Ok(Self::PuRd),
            "rdpu" => Ok(Self::RdPu),
            "ylgn" => Ok(Self::YlGn),
            "hot" => Ok(Self::Hot),
            "copper" => Ok(Self::Copper),
            "bone" => Ok(Self::Bone),
            "gray" | "grey" => Ok(Self::Gray),
            "pubugn" => Ok(Self::PuBuGn),
            "binary" => Ok(Self::Binary),
            "gistyarg" => Ok(Self::GistYarg),
            "gistgray" => Ok(Self::GistGray),
            "pink" => Ok(Self::Pink),
            "spring" => Ok(Self::Spring),
            "summer" => Ok(Self::Summer),
            "autumn" => Ok(Self::Autumn),
            "winter" => Ok(Self::Winter),
            "cool" => Ok(Self::Cool),
            "wistia" => Ok(Self::Wistia),
            "afmhot" => Ok(Self::Afmhot),
            "gistheat" => Ok(Self::GistHeat),
            "coolwarm" => Ok(Self::Coolwarm),
            "rdbur" => Ok(Self::RdBuR),
            "seismic" => Ok(Self::Seismic),
            "bwr" => Ok(Self::Bwr),
            "piyg" => Ok(Self::PiYG),
            "prgn" => Ok(Self::PRGn),
            "brbg" => Ok(Self::BrBG),
            "puor" => Ok(Self::PuOr),
            "rdylbu" => Ok(Self::RdYlBu),
            "rdylgn" => Ok(Self::RdYlGn),
            "spectral" => Ok(Self::Spectral),
            "rdgy" => Ok(Self::RdGy),
            "rdbu" => Ok(Self::RdBu),
            "twilight" => Ok(Self::Twilight),
            "twilightshifted" => Ok(Self::TwilightShifted),
            "jet" => Ok(Self::Jet),
            "turbo" => Ok(Self::Turbo),
            "hsv" => Ok(Self::Hsv),
            "rainbow" => Ok(Self::Rainbow),
            "ocean" => Ok(Self::Ocean),
            "gistearth" => Ok(Self::GistEarth),
            "terrain" => Ok(Self::Terrain),
            "giststern" => Ok(Self::GistStern),
            "gnuplot" => Ok(Self::Gnuplot),
            "gnuplot2" => Ok(Self::Gnuplot2),
            "cmrmap" => Ok(Self::CMRmap),
            "cubehelix" => Ok(Self::Cubehelix),
            "brg" => Ok(Self::Brg),
            "gistrainbow" => Ok(Self::GistRainbow),
            "nipyspectral" => Ok(Self::NipySpectral),
            "gistncar" => Ok(Self::GistNcar),
            "tab10" => Ok(Self::Tab10),
            "set1" => Ok(Self::Set1),
            "set2" => Ok(Self::Set2),
            "set3" => Ok(Self::Set3),
            "paired" => Ok(Self::Paired),
            "pastel1" => Ok(Self::Pastel1),
            "pastel2" => Ok(Self::Pastel2),
            "dark2" => Ok(Self::Dark2),
            "accent" => Ok(Self::Accent),
            "tab20" => Ok(Self::Tab20),
            "tab20b" => Ok(Self::Tab20b),
            "tab20c" => Ok(Self::Tab20c),
            _ => Err(format!("unknown colormap '{s}'; try one of: viridis, plasma, inferno, magma, cividis, greys, blues, greens, reds, hot, coolwarm, jet, turbo, spectral, tab10, set1, paired, …")),
        }
    }
}

fn lerp_color(a: Color, b: Color, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| -> u8 { (x as f64 + (y as f64 - x as f64) * t).round() as u8 };
    Color::rgba(
        lerp(a.r, b.r),
        lerp(a.g, b.g),
        lerp(a.b, b.b),
        lerp(a.a, b.a),
    )
}

// ═══════════════════════════════════════════════════════════════════════
//  Perceptually uniform sequential
// ═══════════════════════════════════════════════════════════════════════

const VIRIDIS: [Color; 33] = [
    Color::rgb(0x44, 0x01, 0x54),
    Color::rgb(0x47, 0x0d, 0x60),
    Color::rgb(0x48, 0x18, 0x6a),
    Color::rgb(0x48, 0x23, 0x74),
    Color::rgb(0x47, 0x2d, 0x7b),
    Color::rgb(0x45, 0x37, 0x81),
    Color::rgb(0x42, 0x40, 0x86),
    Color::rgb(0x3e, 0x49, 0x89),
    Color::rgb(0x3b, 0x52, 0x8b),
    Color::rgb(0x37, 0x5b, 0x8d),
    Color::rgb(0x33, 0x63, 0x8d),
    Color::rgb(0x2f, 0x6b, 0x8e),
    Color::rgb(0x2c, 0x72, 0x8e),
    Color::rgb(0x29, 0x7a, 0x8e),
    Color::rgb(0x26, 0x82, 0x8e),
    Color::rgb(0x23, 0x89, 0x8e),
    Color::rgb(0x21, 0x91, 0x8c),
    Color::rgb(0x1f, 0x98, 0x8b),
    Color::rgb(0x1f, 0xa0, 0x88),
    Color::rgb(0x22, 0xa7, 0x85),
    Color::rgb(0x28, 0xae, 0x80),
    Color::rgb(0x32, 0xb6, 0x7a),
    Color::rgb(0x3f, 0xbc, 0x73),
    Color::rgb(0x4e, 0xc3, 0x6b),
    Color::rgb(0x5e, 0xc9, 0x62),
    Color::rgb(0x70, 0xcf, 0x57),
    Color::rgb(0x84, 0xd4, 0x4b),
    Color::rgb(0x98, 0xd8, 0x3e),
    Color::rgb(0xad, 0xdc, 0x30),
    Color::rgb(0xc2, 0xdf, 0x23),
    Color::rgb(0xd8, 0xe2, 0x19),
    Color::rgb(0xec, 0xe5, 0x1b),
    Color::rgb(0xfd, 0xe7, 0x25),
];

const PLASMA: [Color; 9] = [
    Color::rgb(0x0d, 0x08, 0x87),
    Color::rgb(0x4b, 0x03, 0xa1),
    Color::rgb(0x7d, 0x03, 0xa8),
    Color::rgb(0xa8, 0x22, 0x96),
    Color::rgb(0xcc, 0x47, 0x78),
    Color::rgb(0xe6, 0x6c, 0x5c),
    Color::rgb(0xf8, 0x94, 0x41),
    Color::rgb(0xfd, 0xc3, 0x26),
    Color::rgb(0xf0, 0xf9, 0x21),
];

const INFERNO: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x04),
    Color::rgb(0x1b, 0x0c, 0x41),
    Color::rgb(0x4a, 0x0c, 0x6b),
    Color::rgb(0x78, 0x1c, 0x6d),
    Color::rgb(0xa5, 0x2c, 0x60),
    Color::rgb(0xcf, 0x44, 0x46),
    Color::rgb(0xed, 0x69, 0x25),
    Color::rgb(0xfb, 0x9b, 0x06),
    Color::rgb(0xfc, 0xff, 0xa4),
];

const MAGMA: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x04),
    Color::rgb(0x18, 0x0f, 0x3e),
    Color::rgb(0x45, 0x15, 0x6b),
    Color::rgb(0x75, 0x1a, 0x6d),
    Color::rgb(0xa3, 0x1e, 0x64),
    Color::rgb(0xd1, 0x3d, 0x4c),
    Color::rgb(0xed, 0x69, 0x25),
    Color::rgb(0xfb, 0xb3, 0x75),
    Color::rgb(0xfc, 0xf9, 0xf0),
];

const CIVIDIS: [Color; 9] = [
    Color::rgb(0x00, 0x22, 0x4e),
    Color::rgb(0x12, 0x39, 0x5a),
    Color::rgb(0x3b, 0x4f, 0x6e),
    Color::rgb(0x5a, 0x65, 0x7e),
    Color::rgb(0x78, 0x7c, 0x8a),
    Color::rgb(0x97, 0x94, 0x90),
    Color::rgb(0xb8, 0xae, 0x8c),
    Color::rgb(0xdb, 0xca, 0x7b),
    Color::rgb(0xfe, 0xe8, 0x38),
];

// ═══════════════════════════════════════════════════════════════════════
//  Sequential single-hue
// ═══════════════════════════════════════════════════════════════════════

const GREYS: [Color; 9] = [
    Color::rgb(0xff, 0xff, 0xff),
    Color::rgb(0xef, 0xef, 0xef),
    Color::rgb(0xd8, 0xd8, 0xd8),
    Color::rgb(0xbc, 0xbc, 0xbc),
    Color::rgb(0x95, 0x95, 0x95),
    Color::rgb(0x72, 0x72, 0x72),
    Color::rgb(0x50, 0x50, 0x50),
    Color::rgb(0x23, 0x23, 0x23),
    Color::rgb(0x00, 0x00, 0x00),
];

const BLUES: [Color; 9] = [
    Color::rgb(0xf7, 0xfb, 0xff),
    Color::rgb(0xdd, 0xea, 0xf6),
    Color::rgb(0xc5, 0xda, 0xee),
    Color::rgb(0x9d, 0xc9, 0xe0),
    Color::rgb(0x6a, 0xad, 0xd5),
    Color::rgb(0x41, 0x91, 0xc5),
    Color::rgb(0x20, 0x70, 0xb4),
    Color::rgb(0x08, 0x50, 0x9a),
    Color::rgb(0x08, 0x30, 0x6b),
];

const GREENS: [Color; 9] = [
    Color::rgb(0xf7, 0xfc, 0xf5),
    Color::rgb(0xe4, 0xf4, 0xdf),
    Color::rgb(0xc6, 0xe8, 0xbf),
    Color::rgb(0xa0, 0xd8, 0x9a),
    Color::rgb(0x73, 0xc3, 0x75),
    Color::rgb(0x40, 0xaa, 0x5c),
    Color::rgb(0x22, 0x8a, 0x44),
    Color::rgb(0x00, 0x6b, 0x2b),
    Color::rgb(0x00, 0x44, 0x1b),
];

const REDS: [Color; 9] = [
    Color::rgb(0xff, 0xf5, 0xf0),
    Color::rgb(0xfd, 0xdf, 0xd1),
    Color::rgb(0xfc, 0xba, 0xa0),
    Color::rgb(0xfb, 0x91, 0x71),
    Color::rgb(0xfa, 0x69, 0x49),
    Color::rgb(0xee, 0x3a, 0x2b),
    Color::rgb(0xca, 0x17, 0x1c),
    Color::rgb(0xa3, 0x0e, 0x14),
    Color::rgb(0x67, 0x00, 0x0c),
];

const ORANGES: [Color; 9] = [
    Color::rgb(0xff, 0xf5, 0xeb),
    Color::rgb(0xfd, 0xe5, 0xcd),
    Color::rgb(0xfd, 0xcf, 0xa1),
    Color::rgb(0xfd, 0xad, 0x6a),
    Color::rgb(0xfc, 0x8c, 0x3b),
    Color::rgb(0xf0, 0x68, 0x12),
    Color::rgb(0xd7, 0x47, 0x01),
    Color::rgb(0xa4, 0x35, 0x03),
    Color::rgb(0x7f, 0x27, 0x04),
];

const PURPLES: [Color; 9] = [
    Color::rgb(0xfc, 0xfb, 0xfd),
    Color::rgb(0xee, 0xec, 0xf4),
    Color::rgb(0xd9, 0xd9, 0xea),
    Color::rgb(0xbb, 0xbc, 0xdb),
    Color::rgb(0x9d, 0x99, 0xc7),
    Color::rgb(0x7f, 0x7c, 0xb9),
    Color::rgb(0x69, 0x50, 0xa2),
    Color::rgb(0x53, 0x25, 0x8e),
    Color::rgb(0x3f, 0x00, 0x7d),
];

// ═══════════════════════════════════════════════════════════════════════
//  Sequential multi-hue
// ═══════════════════════════════════════════════════════════════════════

const YLORRD: [Color; 9] = [
    Color::rgb(0xff, 0xff, 0xcc),
    Color::rgb(0xfe, 0xec, 0x9f),
    Color::rgb(0xfe, 0xd8, 0x75),
    Color::rgb(0xfd, 0xb1, 0x4b),
    Color::rgb(0xfc, 0x8c, 0x3b),
    Color::rgb(0xfb, 0x4c, 0x29),
    Color::rgb(0xe2, 0x19, 0x1c),
    Color::rgb(0xbb, 0x00, 0x26),
    Color::rgb(0x80, 0x00, 0x26),
];

const YLORBR: [Color; 9] = [
    Color::rgb(0xff, 0xff, 0xe5),
    Color::rgb(0xfe, 0xf6, 0xbb),
    Color::rgb(0xfe, 0xe2, 0x90),
    Color::rgb(0xfe, 0xc3, 0x4e),
    Color::rgb(0xfd, 0x98, 0x28),
    Color::rgb(0xeb, 0x6f, 0x13),
    Color::rgb(0xca, 0x4b, 0x02),
    Color::rgb(0x97, 0x33, 0x04),
    Color::rgb(0x66, 0x25, 0x05),
];

const YLGNBU: [Color; 9] = [
    Color::rgb(0xff, 0xff, 0xd9),
    Color::rgb(0xec, 0xf7, 0xb1),
    Color::rgb(0xc6, 0xe8, 0xb4),
    Color::rgb(0x7e, 0xcc, 0xbb),
    Color::rgb(0x40, 0xb5, 0xc3),
    Color::rgb(0x1d, 0x90, 0xbf),
    Color::rgb(0x22, 0x5d, 0xa7),
    Color::rgb(0x24, 0x33, 0x92),
    Color::rgb(0x08, 0x1d, 0x58),
];

const BUGN: [Color; 9] = [
    Color::rgb(0xf7, 0xfc, 0xfd),
    Color::rgb(0xe4, 0xf4, 0xf8),
    Color::rgb(0xcb, 0xeb, 0xe5),
    Color::rgb(0x98, 0xd7, 0xc8),
    Color::rgb(0x65, 0xc1, 0xa3),
    Color::rgb(0x40, 0xad, 0x75),
    Color::rgb(0x22, 0x8a, 0x44),
    Color::rgb(0x00, 0x6b, 0x2b),
    Color::rgb(0x00, 0x44, 0x1b),
];

const BUPU: [Color; 9] = [
    Color::rgb(0xf7, 0xfc, 0xfd),
    Color::rgb(0xdf, 0xeb, 0xf3),
    Color::rgb(0xbe, 0xd2, 0xe5),
    Color::rgb(0x9d, 0xbb, 0xd9),
    Color::rgb(0x8c, 0x95, 0xc5),
    Color::rgb(0x8b, 0x6a, 0xb0),
    Color::rgb(0x87, 0x3f, 0x9c),
    Color::rgb(0x7f, 0x0e, 0x7a),
    Color::rgb(0x4d, 0x00, 0x4b),
];

const GNBU: [Color; 9] = [
    Color::rgb(0xf7, 0xfc, 0xf0),
    Color::rgb(0xdf, 0xf2, 0xda),
    Color::rgb(0xcb, 0xea, 0xc4),
    Color::rgb(0xa7, 0xdc, 0xb5),
    Color::rgb(0x7a, 0xcb, 0xc4),
    Color::rgb(0x4d, 0xb2, 0xd2),
    Color::rgb(0x2a, 0x8b, 0xbd),
    Color::rgb(0x08, 0x66, 0xaa),
    Color::rgb(0x08, 0x40, 0x81),
];

const ORRD: [Color; 9] = [
    Color::rgb(0xff, 0xf7, 0xec),
    Color::rgb(0xfd, 0xe7, 0xc7),
    Color::rgb(0xfd, 0xd3, 0x9d),
    Color::rgb(0xfc, 0xba, 0x83),
    Color::rgb(0xfb, 0x8c, 0x58),
    Color::rgb(0xee, 0x63, 0x47),
    Color::rgb(0xd6, 0x2e, 0x1e),
    Color::rgb(0xb1, 0x00, 0x00),
    Color::rgb(0x7f, 0x00, 0x00),
];

const PUBU: [Color; 9] = [
    Color::rgb(0xff, 0xf7, 0xfb),
    Color::rgb(0xeb, 0xe6, 0xf1),
    Color::rgb(0xcf, 0xd0, 0xe5),
    Color::rgb(0xa5, 0xbc, 0xda),
    Color::rgb(0x73, 0xa8, 0xce),
    Color::rgb(0x35, 0x8f, 0xbf),
    Color::rgb(0x04, 0x6f, 0xaf),
    Color::rgb(0x03, 0x59, 0x8b),
    Color::rgb(0x02, 0x38, 0x58),
];

const PURD: [Color; 9] = [
    Color::rgb(0xf7, 0xf4, 0xf9),
    Color::rgb(0xe6, 0xe0, 0xee),
    Color::rgb(0xd3, 0xb8, 0xd9),
    Color::rgb(0xc9, 0x93, 0xc6),
    Color::rgb(0xdf, 0x64, 0xaf),
    Color::rgb(0xe6, 0x28, 0x88),
    Color::rgb(0xcc, 0x11, 0x55),
    Color::rgb(0x96, 0x00, 0x42),
    Color::rgb(0x67, 0x00, 0x1f),
];

const RDPU: [Color; 9] = [
    Color::rgb(0xff, 0xf7, 0xf3),
    Color::rgb(0xfc, 0xdf, 0xdc),
    Color::rgb(0xfb, 0xc4, 0xbf),
    Color::rgb(0xf9, 0x9e, 0xb4),
    Color::rgb(0xf6, 0x67, 0xa0),
    Color::rgb(0xdc, 0x33, 0x96),
    Color::rgb(0xac, 0x01, 0x7d),
    Color::rgb(0x78, 0x00, 0x76),
    Color::rgb(0x49, 0x00, 0x6a),
];

const YLGN: [Color; 9] = [
    Color::rgb(0xff, 0xff, 0xe5),
    Color::rgb(0xf6, 0xfb, 0xb8),
    Color::rgb(0xd8, 0xef, 0xa2),
    Color::rgb(0xac, 0xdc, 0x8d),
    Color::rgb(0x77, 0xc5, 0x78),
    Color::rgb(0x40, 0xaa, 0x5c),
    Color::rgb(0x22, 0x83, 0x42),
    Color::rgb(0x00, 0x67, 0x36),
    Color::rgb(0x00, 0x45, 0x29),
];

const HOT: [Color; 9] = [
    Color::rgb(0x0a, 0x00, 0x00),
    Color::rgb(0x5e, 0x00, 0x00),
    Color::rgb(0xb2, 0x00, 0x00),
    Color::rgb(0xff, 0x07, 0x00),
    Color::rgb(0xff, 0x5b, 0x00),
    Color::rgb(0xff, 0xaf, 0x00),
    Color::rgb(0xff, 0xff, 0x06),
    Color::rgb(0xff, 0xff, 0x84),
    Color::rgb(0xff, 0xff, 0xff),
];

const COPPER: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x27, 0x18, 0x0f),
    Color::rgb(0x4f, 0x31, 0x1f),
    Color::rgb(0x76, 0x4a, 0x2f),
    Color::rgb(0x9e, 0x63, 0x3f),
    Color::rgb(0xc5, 0x7c, 0x4f),
    Color::rgb(0xed, 0x95, 0x5f),
    Color::rgb(0xff, 0xae, 0x6f),
    Color::rgb(0xff, 0xc7, 0x7e),
];

const BONE: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x1c, 0x1b, 0x26),
    Color::rgb(0x38, 0x37, 0x4d),
    Color::rgb(0x54, 0x54, 0x73),
    Color::rgb(0x70, 0x7b, 0x8f),
    Color::rgb(0x8c, 0xa1, 0xab),
    Color::rgb(0xa8, 0xc7, 0xc7),
    Color::rgb(0xd4, 0xe3, 0xe3),
    Color::rgb(0xff, 0xff, 0xff),
];

const GRAY: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x20, 0x20, 0x20),
    Color::rgb(0x40, 0x40, 0x40),
    Color::rgb(0x60, 0x60, 0x60),
    Color::rgb(0x80, 0x80, 0x80),
    Color::rgb(0xa0, 0xa0, 0xa0),
    Color::rgb(0xc0, 0xc0, 0xc0),
    Color::rgb(0xe0, 0xe0, 0xe0),
    Color::rgb(0xff, 0xff, 0xff),
];

// ═══════════════════════════════════════════════════════════════════════
//  Diverging
// ═══════════════════════════════════════════════════════════════════════

const COOLWARM: [Color; 9] = [
    Color::rgb(0x3b, 0x4c, 0xc0),
    Color::rgb(0x5d, 0x7c, 0xe0),
    Color::rgb(0x8a, 0xad, 0xf0),
    Color::rgb(0xc0, 0xd4, 0xf5),
    Color::rgb(0xf7, 0xf7, 0xf7),
    Color::rgb(0xf5, 0xc4, 0xb0),
    Color::rgb(0xe8, 0x85, 0x6a),
    Color::rgb(0xd0, 0x4e, 0x40),
    Color::rgb(0xb4, 0x04, 0x26),
];

const RDBU_R: [Color; 9] = [
    Color::rgb(0x05, 0x30, 0x61),
    Color::rgb(0x21, 0x66, 0xac),
    Color::rgb(0x43, 0x93, 0xc3),
    Color::rgb(0x92, 0xc5, 0xde),
    Color::rgb(0xf7, 0xf7, 0xf7),
    Color::rgb(0xf4, 0xa5, 0x82),
    Color::rgb(0xd6, 0x60, 0x4d),
    Color::rgb(0xb2, 0x18, 0x2b),
    Color::rgb(0x67, 0x00, 0x1f),
];

const SEISMIC: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x4c),
    Color::rgb(0x00, 0x00, 0xa6),
    Color::rgb(0x01, 0x01, 0xff),
    Color::rgb(0x81, 0x81, 0xff),
    Color::rgb(0xff, 0xfd, 0xfd),
    Color::rgb(0xff, 0x7d, 0x7d),
    Color::rgb(0xfd, 0x00, 0x00),
    Color::rgb(0xbd, 0x00, 0x00),
    Color::rgb(0x7f, 0x00, 0x00),
];

const BWR: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0xff),
    Color::rgb(0x40, 0x40, 0xff),
    Color::rgb(0x80, 0x80, 0xff),
    Color::rgb(0xc0, 0xc0, 0xff),
    Color::rgb(0xff, 0xfe, 0xfe),
    Color::rgb(0xff, 0xbe, 0xbe),
    Color::rgb(0xff, 0x7e, 0x7e),
    Color::rgb(0xff, 0x3e, 0x3e),
    Color::rgb(0xff, 0x00, 0x00),
];

const PIYG: [Color; 9] = [
    Color::rgb(0x8e, 0x01, 0x52),
    Color::rgb(0xcb, 0x32, 0x89),
    Color::rgb(0xe7, 0x97, 0xc4),
    Color::rgb(0xfa, 0xd6, 0xea),
    Color::rgb(0xf6, 0xf6, 0xf6),
    Color::rgb(0xd9, 0xef, 0xbb),
    Color::rgb(0x99, 0xcd, 0x61),
    Color::rgb(0x57, 0x9b, 0x27),
    Color::rgb(0x27, 0x64, 0x19),
];

const PRGN: [Color; 9] = [
    Color::rgb(0x40, 0x00, 0x4b),
    Color::rgb(0x7e, 0x3b, 0x8d),
    Color::rgb(0xad, 0x8b, 0xbd),
    Color::rgb(0xde, 0xc8, 0xe2),
    Color::rgb(0xf6, 0xf6, 0xf6),
    Color::rgb(0xcb, 0xea, 0xc5),
    Color::rgb(0x7d, 0xc3, 0x7e),
    Color::rgb(0x28, 0x83, 0x40),
    Color::rgb(0x00, 0x44, 0x1b),
];

const BRBG: [Color; 9] = [
    Color::rgb(0x54, 0x30, 0x05),
    Color::rgb(0x99, 0x5d, 0x12),
    Color::rgb(0xcf, 0xa2, 0x55),
    Color::rgb(0xf0, 0xdf, 0xb2),
    Color::rgb(0xf4, 0xf4, 0xf4),
    Color::rgb(0xb3, 0xe2, 0xdb),
    Color::rgb(0x58, 0xb0, 0xa6),
    Color::rgb(0x0c, 0x70, 0x68),
    Color::rgb(0x00, 0x3c, 0x30),
];

const PUOR: [Color; 9] = [
    Color::rgb(0x7f, 0x3b, 0x08),
    Color::rgb(0xbe, 0x62, 0x09),
    Color::rgb(0xee, 0x9d, 0x3c),
    Color::rgb(0xfd, 0xd6, 0xa2),
    Color::rgb(0xf6, 0xf6, 0xf6),
    Color::rgb(0xcd, 0xcd, 0xe4),
    Color::rgb(0x97, 0x8d, 0xbd),
    Color::rgb(0x5d, 0x37, 0x8f),
    Color::rgb(0x2d, 0x00, 0x4b),
];

const RDYLBU: [Color; 9] = [
    Color::rgb(0xa5, 0x00, 0x26),
    Color::rgb(0xde, 0x3f, 0x2e),
    Color::rgb(0xf8, 0x8e, 0x52),
    Color::rgb(0xfd, 0xd4, 0x84),
    Color::rgb(0xfe, 0xfe, 0xc0),
    Color::rgb(0xd1, 0xeb, 0xf3),
    Color::rgb(0x8d, 0xc1, 0xdc),
    Color::rgb(0x4f, 0x81, 0xba),
    Color::rgb(0x31, 0x36, 0x95),
];

const RDYLGN: [Color; 9] = [
    Color::rgb(0xa5, 0x00, 0x26),
    Color::rgb(0xde, 0x3f, 0x2e),
    Color::rgb(0xf8, 0x8e, 0x52),
    Color::rgb(0xfd, 0xd4, 0x81),
    Color::rgb(0xfe, 0xfe, 0xbd),
    Color::rgb(0xcb, 0xe8, 0x81),
    Color::rgb(0x84, 0xca, 0x66),
    Color::rgb(0x2a, 0x9f, 0x54),
    Color::rgb(0x00, 0x68, 0x37),
];

const SPECTRAL: [Color; 9] = [
    Color::rgb(0x9e, 0x01, 0x42),
    Color::rgb(0xdc, 0x49, 0x4b),
    Color::rgb(0xf8, 0x8e, 0x52),
    Color::rgb(0xfd, 0xd4, 0x81),
    Color::rgb(0xfe, 0xfe, 0xbe),
    Color::rgb(0xd5, 0xee, 0x9b),
    Color::rgb(0x86, 0xce, 0xa4),
    Color::rgb(0x3d, 0x94, 0xb7),
    Color::rgb(0x5e, 0x4f, 0xa2),
];

// ═══════════════════════════════════════════════════════════════════════
//  Cyclic
// ═══════════════════════════════════════════════════════════════════════

const TWILIGHT: [Color; 9] = [
    Color::rgb(0xe1, 0xd8, 0xe2),
    Color::rgb(0x94, 0xb4, 0xc6),
    Color::rgb(0x61, 0x75, 0xba),
    Color::rgb(0x59, 0x2a, 0x8f),
    Color::rgb(0x2f, 0x14, 0x36),
    Color::rgb(0x73, 0x1d, 0x4e),
    Color::rgb(0xb2, 0x56, 0x52),
    Color::rgb(0xcc, 0xa3, 0x89),
    Color::rgb(0xe1, 0xd8, 0xe1),
];

// ═══════════════════════════════════════════════════════════════════════
//  Other
// ═══════════════════════════════════════════════════════════════════════

const JET: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x7f),
    Color::rgb(0x00, 0x00, 0xff),
    Color::rgb(0x00, 0x80, 0xff),
    Color::rgb(0x15, 0xff, 0xe1),
    Color::rgb(0x7c, 0xff, 0x79),
    Color::rgb(0xe4, 0xff, 0x12),
    Color::rgb(0xff, 0x94, 0x00),
    Color::rgb(0xff, 0x1d, 0x00),
    Color::rgb(0x7f, 0x00, 0x00),
];

const TURBO: [Color; 9] = [
    Color::rgb(0x30, 0x12, 0x3b),
    Color::rgb(0x46, 0x6b, 0xe3),
    Color::rgb(0x28, 0xbb, 0xeb),
    Color::rgb(0x32, 0xf1, 0x97),
    Color::rgb(0xa4, 0xfc, 0x3b),
    Color::rgb(0xed, 0xcf, 0x39),
    Color::rgb(0xfa, 0x7d, 0x20),
    Color::rgb(0xd0, 0x2f, 0x04),
    Color::rgb(0x7a, 0x04, 0x02),
];

const HSV: [Color; 9] = [
    Color::rgb(0xff, 0x00, 0x00),
    Color::rgb(0xff, 0xbd, 0x00),
    Color::rgb(0x83, 0xff, 0x00),
    Color::rgb(0x00, 0xff, 0x39),
    Color::rgb(0x00, 0xff, 0xf5),
    Color::rgb(0x00, 0x4b, 0xff),
    Color::rgb(0x71, 0x00, 0xff),
    Color::rgb(0xff, 0x00, 0xcf),
    Color::rgb(0xff, 0x00, 0x17),
];

const RAINBOW: [Color; 9] = [
    Color::rgb(0x7f, 0x00, 0xff),
    Color::rgb(0x3f, 0x61, 0xfa),
    Color::rgb(0x00, 0xb4, 0xeb),
    Color::rgb(0x40, 0xec, 0xd3),
    Color::rgb(0x80, 0xfe, 0xb3),
    Color::rgb(0xc0, 0xea, 0x8c),
    Color::rgb(0xff, 0xb2, 0x60),
    Color::rgb(0xff, 0x5f, 0x30),
    Color::rgb(0xff, 0x00, 0x00),
];

// ═══════════════════════════════════════════════════════════════════════
//  Qualitative / listed
// ═══════════════════════════════════════════════════════════════════════

/// Matplotlib `tab10` (C0…C9).
const TAB10: [Color; 10] = [
    Color::rgb(0x1f, 0x77, 0xb4),
    Color::rgb(0xff, 0x7f, 0x0e),
    Color::rgb(0x2c, 0xa0, 0x2c),
    Color::rgb(0xd6, 0x27, 0x28),
    Color::rgb(0x94, 0x67, 0xbd),
    Color::rgb(0x8c, 0x56, 0x4b),
    Color::rgb(0xe3, 0x77, 0xc2),
    Color::rgb(0x7f, 0x7f, 0x7f),
    Color::rgb(0xbc, 0xbd, 0x22),
    Color::rgb(0x17, 0xbe, 0xcf),
];

const SET1: [Color; 9] = [
    Color::rgb(0xe4, 0x1a, 0x1c),
    Color::rgb(0x37, 0x7e, 0xb8),
    Color::rgb(0x4d, 0xaf, 0x4a),
    Color::rgb(0x98, 0x4e, 0xa3),
    Color::rgb(0xff, 0x7f, 0x00),
    Color::rgb(0xff, 0xff, 0x33),
    Color::rgb(0xa6, 0x56, 0x28),
    Color::rgb(0xf7, 0x81, 0xbf),
    Color::rgb(0x99, 0x99, 0x99),
];

const SET2: [Color; 8] = [
    Color::rgb(0x66, 0xc2, 0xa5),
    Color::rgb(0xfc, 0x8d, 0x62),
    Color::rgb(0x8d, 0xa0, 0xcb),
    Color::rgb(0xe7, 0x8a, 0xc3),
    Color::rgb(0xa6, 0xd8, 0x54),
    Color::rgb(0xff, 0xd9, 0x2f),
    Color::rgb(0xe5, 0xc4, 0x94),
    Color::rgb(0xb3, 0xb3, 0xb3),
];

const SET3: [Color; 12] = [
    Color::rgb(0x8d, 0xd3, 0xc7),
    Color::rgb(0xff, 0xff, 0xb3),
    Color::rgb(0xbe, 0xba, 0xda),
    Color::rgb(0xfb, 0x80, 0x72),
    Color::rgb(0x80, 0xb1, 0xd3),
    Color::rgb(0xfd, 0xb4, 0x62),
    Color::rgb(0xb3, 0xde, 0x69),
    Color::rgb(0xfc, 0xcd, 0xe5),
    Color::rgb(0xd9, 0xd9, 0xd9),
    Color::rgb(0xbc, 0x80, 0xbd),
    Color::rgb(0xcc, 0xeb, 0xc5),
    Color::rgb(0xff, 0xed, 0x6f),
];

const PAIRED: [Color; 12] = [
    Color::rgb(0xa6, 0xce, 0xe3),
    Color::rgb(0x1f, 0x78, 0xb4),
    Color::rgb(0xb2, 0xdf, 0x8a),
    Color::rgb(0x33, 0xa0, 0x2c),
    Color::rgb(0xfb, 0x9a, 0x99),
    Color::rgb(0xe3, 0x1a, 0x1c),
    Color::rgb(0xfd, 0xbf, 0x6f),
    Color::rgb(0xff, 0x7f, 0x00),
    Color::rgb(0xca, 0xb2, 0xd6),
    Color::rgb(0x6a, 0x3d, 0x9a),
    Color::rgb(0xff, 0xff, 0x99),
    Color::rgb(0xb1, 0x59, 0x28),
];

const PASTEL1: [Color; 9] = [
    Color::rgb(0xfb, 0xb4, 0xae),
    Color::rgb(0xb3, 0xcd, 0xe3),
    Color::rgb(0xcc, 0xeb, 0xc5),
    Color::rgb(0xde, 0xcb, 0xe4),
    Color::rgb(0xfe, 0xd9, 0xa6),
    Color::rgb(0xff, 0xff, 0xcc),
    Color::rgb(0xe5, 0xd8, 0xbd),
    Color::rgb(0xfd, 0xda, 0xec),
    Color::rgb(0xf2, 0xf2, 0xf2),
];

const PASTEL2: [Color; 8] = [
    Color::rgb(0xb3, 0xe2, 0xcd),
    Color::rgb(0xfd, 0xcd, 0xac),
    Color::rgb(0xcb, 0xd5, 0xe8),
    Color::rgb(0xf4, 0xca, 0xe4),
    Color::rgb(0xe6, 0xf5, 0xc9),
    Color::rgb(0xff, 0xf2, 0xae),
    Color::rgb(0xf1, 0xe2, 0xcc),
    Color::rgb(0xcc, 0xcc, 0xcc),
];

const DARK2: [Color; 8] = [
    Color::rgb(0x1b, 0x9e, 0x77),
    Color::rgb(0xd9, 0x5f, 0x02),
    Color::rgb(0x75, 0x70, 0xb3),
    Color::rgb(0xe7, 0x29, 0x8a),
    Color::rgb(0x66, 0xa6, 0x1e),
    Color::rgb(0xe6, 0xab, 0x02),
    Color::rgb(0xa6, 0x76, 0x1d),
    Color::rgb(0x66, 0x66, 0x66),
];

const ACCENT: [Color; 8] = [
    Color::rgb(0x7f, 0xc9, 0x7f),
    Color::rgb(0xbe, 0xae, 0xd4),
    Color::rgb(0xfd, 0xc0, 0x86),
    Color::rgb(0xff, 0xff, 0x99),
    Color::rgb(0x38, 0x6c, 0xb0),
    Color::rgb(0xf0, 0x02, 0x7f),
    Color::rgb(0xbf, 0x5b, 0x16),
    Color::rgb(0x66, 0x66, 0x66),
];

const TAB20: [Color; 20] = [
    Color::rgb(0x1f, 0x77, 0xb4),
    Color::rgb(0xae, 0xc7, 0xe8),
    Color::rgb(0xff, 0x7f, 0x0e),
    Color::rgb(0xff, 0xbb, 0x78),
    Color::rgb(0x2c, 0xa0, 0x2c),
    Color::rgb(0x98, 0xdf, 0x8a),
    Color::rgb(0xd6, 0x27, 0x28),
    Color::rgb(0xff, 0x98, 0x96),
    Color::rgb(0x94, 0x67, 0xbd),
    Color::rgb(0xc5, 0xb0, 0xd5),
    Color::rgb(0x8c, 0x56, 0x4b),
    Color::rgb(0xc4, 0x9c, 0x94),
    Color::rgb(0xe3, 0x77, 0xc2),
    Color::rgb(0xf7, 0xb6, 0xd2),
    Color::rgb(0x7f, 0x7f, 0x7f),
    Color::rgb(0xc7, 0xc7, 0xc7),
    Color::rgb(0xbc, 0xbd, 0x22),
    Color::rgb(0xdb, 0xdb, 0x8d),
    Color::rgb(0x17, 0xbe, 0xcf),
    Color::rgb(0x9e, 0xda, 0xe5),
];

const TAB20B: [Color; 20] = [
    Color::rgb(0x39, 0x3b, 0x79),
    Color::rgb(0x52, 0x54, 0xa3),
    Color::rgb(0x6b, 0x6e, 0xcf),
    Color::rgb(0x9c, 0x9e, 0xde),
    Color::rgb(0x63, 0x79, 0x39),
    Color::rgb(0x8c, 0xa2, 0x52),
    Color::rgb(0xb5, 0xcf, 0x6b),
    Color::rgb(0xce, 0xdb, 0x9c),
    Color::rgb(0x8c, 0x6d, 0x31),
    Color::rgb(0xbd, 0x9e, 0x39),
    Color::rgb(0xe7, 0xba, 0x52),
    Color::rgb(0xe7, 0xcb, 0x94),
    Color::rgb(0x84, 0x3c, 0x39),
    Color::rgb(0xad, 0x49, 0x4a),
    Color::rgb(0xd6, 0x61, 0x6b),
    Color::rgb(0xe7, 0x96, 0x9c),
    Color::rgb(0x7b, 0x41, 0x73),
    Color::rgb(0xa5, 0x51, 0x94),
    Color::rgb(0xce, 0x6d, 0xbd),
    Color::rgb(0xde, 0x9e, 0xd6),
];

const TAB20C: [Color; 20] = [
    Color::rgb(0x31, 0x82, 0xbd),
    Color::rgb(0x6b, 0xae, 0xd6),
    Color::rgb(0x9e, 0xca, 0xe1),
    Color::rgb(0xc6, 0xdb, 0xef),
    Color::rgb(0xe6, 0x55, 0x0d),
    Color::rgb(0xfd, 0x8d, 0x3c),
    Color::rgb(0xfd, 0xae, 0x6b),
    Color::rgb(0xfd, 0xd0, 0xa2),
    Color::rgb(0x31, 0xa3, 0x54),
    Color::rgb(0x74, 0xc4, 0x76),
    Color::rgb(0xa1, 0xd9, 0x9b),
    Color::rgb(0xc7, 0xe9, 0xc0),
    Color::rgb(0x75, 0x6b, 0xb1),
    Color::rgb(0x9e, 0x9a, 0xc8),
    Color::rgb(0xbc, 0xbd, 0xdc),
    Color::rgb(0xda, 0xda, 0xeb),
    Color::rgb(0x63, 0x63, 0x63),
    Color::rgb(0x96, 0x96, 0x96),
    Color::rgb(0xbd, 0xbd, 0xbd),
    Color::rgb(0xd9, 0xd9, 0xd9),
];

const PUBUGN: [Color; 9] = [
    Color::rgb(0xff, 0xf7, 0xfb),
    Color::rgb(0xeb, 0xe1, 0xef),
    Color::rgb(0xcf, 0xd0, 0xe5),
    Color::rgb(0xa5, 0xbc, 0xda),
    Color::rgb(0x66, 0xa8, 0xce),
    Color::rgb(0x34, 0x8f, 0xbe),
    Color::rgb(0x01, 0x80, 0x88),
    Color::rgb(0x01, 0x6a, 0x58),
    Color::rgb(0x01, 0x46, 0x36),
];

const BINARY: [Color; 9] = [
    Color::rgb(0xff, 0xff, 0xff),
    Color::rgb(0xdf, 0xdf, 0xdf),
    Color::rgb(0xbf, 0xbf, 0xbf),
    Color::rgb(0x9f, 0x9f, 0x9f),
    Color::rgb(0x7f, 0x7f, 0x7f),
    Color::rgb(0x5f, 0x5f, 0x5f),
    Color::rgb(0x3f, 0x3f, 0x3f),
    Color::rgb(0x1f, 0x1f, 0x1f),
    Color::rgb(0x00, 0x00, 0x00),
];

const GISTYARG: [Color; 9] = [
    Color::rgb(0xff, 0xff, 0xff),
    Color::rgb(0xdf, 0xdf, 0xdf),
    Color::rgb(0xbf, 0xbf, 0xbf),
    Color::rgb(0x9f, 0x9f, 0x9f),
    Color::rgb(0x7f, 0x7f, 0x7f),
    Color::rgb(0x5f, 0x5f, 0x5f),
    Color::rgb(0x3f, 0x3f, 0x3f),
    Color::rgb(0x1f, 0x1f, 0x1f),
    Color::rgb(0x00, 0x00, 0x00),
];

const GISTGRAY: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x20, 0x20, 0x20),
    Color::rgb(0x40, 0x40, 0x40),
    Color::rgb(0x60, 0x60, 0x60),
    Color::rgb(0x80, 0x80, 0x80),
    Color::rgb(0xa0, 0xa0, 0xa0),
    Color::rgb(0xc0, 0xc0, 0xc0),
    Color::rgb(0xe0, 0xe0, 0xe0),
    Color::rgb(0xff, 0xff, 0xff),
];

const PINK: [Color; 9] = [
    Color::rgb(0x1e, 0x00, 0x00),
    Color::rgb(0x74, 0x49, 0x49),
    Color::rgb(0xa1, 0x68, 0x68),
    Color::rgb(0xc2, 0x82, 0x7f),
    Color::rgb(0xd0, 0xab, 0x93),
    Color::rgb(0xdd, 0xcd, 0xa4),
    Color::rgb(0xe9, 0xe9, 0xb6),
    Color::rgb(0xf4, 0xf4, 0xde),
    Color::rgb(0xff, 0xff, 0xff),
];

const SPRING: [Color; 9] = [
    Color::rgb(0xff, 0x00, 0xff),
    Color::rgb(0xff, 0x20, 0xdf),
    Color::rgb(0xff, 0x40, 0xbf),
    Color::rgb(0xff, 0x60, 0x9f),
    Color::rgb(0xff, 0x80, 0x7f),
    Color::rgb(0xff, 0xa0, 0x5f),
    Color::rgb(0xff, 0xc0, 0x3f),
    Color::rgb(0xff, 0xe0, 0x1f),
    Color::rgb(0xff, 0xff, 0x00),
];

const SUMMER: [Color; 9] = [
    Color::rgb(0x00, 0x7f, 0x66),
    Color::rgb(0x20, 0x8f, 0x66),
    Color::rgb(0x40, 0x9f, 0x66),
    Color::rgb(0x60, 0xaf, 0x66),
    Color::rgb(0x80, 0xbf, 0x66),
    Color::rgb(0xa0, 0xcf, 0x66),
    Color::rgb(0xc0, 0xdf, 0x66),
    Color::rgb(0xe0, 0xef, 0x66),
    Color::rgb(0xff, 0xff, 0x66),
];

const AUTUMN: [Color; 9] = [
    Color::rgb(0xff, 0x00, 0x00),
    Color::rgb(0xff, 0x20, 0x00),
    Color::rgb(0xff, 0x40, 0x00),
    Color::rgb(0xff, 0x60, 0x00),
    Color::rgb(0xff, 0x80, 0x00),
    Color::rgb(0xff, 0xa0, 0x00),
    Color::rgb(0xff, 0xc0, 0x00),
    Color::rgb(0xff, 0xe0, 0x00),
    Color::rgb(0xff, 0xff, 0x00),
];

const WINTER: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0xff),
    Color::rgb(0x00, 0x20, 0xef),
    Color::rgb(0x00, 0x40, 0xdf),
    Color::rgb(0x00, 0x60, 0xcf),
    Color::rgb(0x00, 0x80, 0xbf),
    Color::rgb(0x00, 0xa0, 0xaf),
    Color::rgb(0x00, 0xc0, 0x9f),
    Color::rgb(0x00, 0xe0, 0x8f),
    Color::rgb(0x00, 0xff, 0x7f),
];

const COOL: [Color; 9] = [
    Color::rgb(0x00, 0xff, 0xff),
    Color::rgb(0x20, 0xdf, 0xff),
    Color::rgb(0x40, 0xbf, 0xff),
    Color::rgb(0x60, 0x9f, 0xff),
    Color::rgb(0x80, 0x7f, 0xff),
    Color::rgb(0xa0, 0x5f, 0xff),
    Color::rgb(0xc0, 0x3f, 0xff),
    Color::rgb(0xe0, 0x1f, 0xff),
    Color::rgb(0xff, 0x00, 0xff),
];

const WISTIA: [Color; 9] = [
    Color::rgb(0xe4, 0xff, 0x7a),
    Color::rgb(0xf1, 0xf3, 0x49),
    Color::rgb(0xff, 0xe7, 0x19),
    Color::rgb(0xff, 0xd2, 0x0c),
    Color::rgb(0xff, 0xbc, 0x00),
    Color::rgb(0xff, 0xae, 0x00),
    Color::rgb(0xfe, 0x9f, 0x00),
    Color::rgb(0xfd, 0x8f, 0x00),
    Color::rgb(0xfc, 0x7f, 0x00),
];

const AFMHOT: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x40, 0x00, 0x00),
    Color::rgb(0x80, 0x00, 0x00),
    Color::rgb(0xc0, 0x40, 0x00),
    Color::rgb(0xff, 0x80, 0x00),
    Color::rgb(0xff, 0xc0, 0x41),
    Color::rgb(0xff, 0xff, 0x81),
    Color::rgb(0xff, 0xff, 0xc1),
    Color::rgb(0xff, 0xff, 0xff),
];

const GISTHEAT: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x30, 0x00, 0x00),
    Color::rgb(0x60, 0x00, 0x00),
    Color::rgb(0x90, 0x00, 0x00),
    Color::rgb(0xc0, 0x00, 0x00),
    Color::rgb(0xf0, 0x41, 0x00),
    Color::rgb(0xff, 0x81, 0x02),
    Color::rgb(0xff, 0xc1, 0x83),
    Color::rgb(0xff, 0xff, 0xff),
];

const RDGY: [Color; 9] = [
    Color::rgb(0x67, 0x00, 0x1f),
    Color::rgb(0xbb, 0x2a, 0x33),
    Color::rgb(0xe5, 0x83, 0x68),
    Color::rgb(0xfa, 0xce, 0xb6),
    Color::rgb(0xfe, 0xfe, 0xfe),
    Color::rgb(0xd5, 0xd5, 0xd5),
    Color::rgb(0x9f, 0x9f, 0x9f),
    Color::rgb(0x59, 0x59, 0x59),
    Color::rgb(0x1a, 0x1a, 0x1a),
];

const RDBU: [Color; 9] = [
    Color::rgb(0x67, 0x00, 0x1f),
    Color::rgb(0xbb, 0x2a, 0x33),
    Color::rgb(0xe5, 0x83, 0x68),
    Color::rgb(0xfa, 0xce, 0xb6),
    Color::rgb(0xf6, 0xf6, 0xf6),
    Color::rgb(0xbf, 0xdc, 0xeb),
    Color::rgb(0x68, 0xaa, 0xcf),
    Color::rgb(0x28, 0x6f, 0xb0),
    Color::rgb(0x05, 0x30, 0x61),
];

const TWILIGHT_SHIFTED: [Color; 9] = [
    Color::rgb(0x2f, 0x13, 0x37),
    Color::rgb(0x59, 0x2a, 0x8f),
    Color::rgb(0x61, 0x75, 0xba),
    Color::rgb(0x94, 0xb4, 0xc6),
    Color::rgb(0xe1, 0xd8, 0xe1),
    Color::rgb(0xcc, 0xa3, 0x89),
    Color::rgb(0xb2, 0x56, 0x52),
    Color::rgb(0x73, 0x1d, 0x4e),
    Color::rgb(0x2f, 0x14, 0x36),
];

const OCEAN: [Color; 9] = [
    Color::rgb(0x00, 0x7f, 0x00),
    Color::rgb(0x00, 0x4f, 0x20),
    Color::rgb(0x00, 0x1f, 0x40),
    Color::rgb(0x00, 0x10, 0x60),
    Color::rgb(0x00, 0x40, 0x80),
    Color::rgb(0x00, 0x70, 0xa0),
    Color::rgb(0x41, 0xa0, 0xc0),
    Color::rgb(0xa2, 0xd0, 0xe0),
    Color::rgb(0xff, 0xff, 0xff),
];

const GISTEARTH: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x15, 0x38, 0x78),
    Color::rgb(0x2a, 0x73, 0x7e),
    Color::rgb(0x3b, 0x8d, 0x62),
    Color::rgb(0x5d, 0xa0, 0x4b),
    Color::rgb(0x99, 0xae, 0x58),
    Color::rgb(0xbc, 0xaa, 0x62),
    Color::rgb(0xda, 0xb6, 0x9f),
    Color::rgb(0xfd, 0xfa, 0xfa),
];

const TERRAIN: [Color; 9] = [
    Color::rgb(0x33, 0x33, 0x99),
    Color::rgb(0x08, 0x88, 0xee),
    Color::rgb(0x01, 0xcc, 0x66),
    Color::rgb(0x81, 0xe5, 0x7f),
    Color::rgb(0xfe, 0xfd, 0x98),
    Color::rgb(0xbe, 0xab, 0x75),
    Color::rgb(0x81, 0x5d, 0x56),
    Color::rgb(0xc1, 0xaf, 0xab),
    Color::rgb(0xff, 0xff, 0xff),
];

const GISTSTERN: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0xa5, 0x20, 0x40),
    Color::rgb(0x40, 0x40, 0x80),
    Color::rgb(0x60, 0x60, 0xc0),
    Color::rgb(0x80, 0x80, 0xfc),
    Color::rgb(0xa0, 0xa0, 0x74),
    Color::rgb(0xc0, 0xc0, 0x11),
    Color::rgb(0xe0, 0xe0, 0x8a),
    Color::rgb(0xff, 0xff, 0xff),
];

const GNUPLOT: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x5a, 0x00, 0xb4),
    Color::rgb(0x7f, 0x04, 0xfe),
    Color::rgb(0x9c, 0x0d, 0xb2),
    Color::rgb(0xb4, 0x20, 0x00),
    Color::rgb(0xc9, 0x3e, 0x00),
    Color::rgb(0xdd, 0x6c, 0x00),
    Color::rgb(0xee, 0xac, 0x00),
    Color::rgb(0xff, 0xff, 0x00),
];

const GNUPLOT2: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x00, 0x00, 0x80),
    Color::rgb(0x00, 0x00, 0xff),
    Color::rgb(0x64, 0x00, 0xff),
    Color::rgb(0xc8, 0x29, 0xd5),
    Color::rgb(0xff, 0x69, 0x95),
    Color::rgb(0xff, 0xa9, 0x55),
    Color::rgb(0xff, 0xe9, 0x15),
    Color::rgb(0xff, 0xff, 0xff),
];

const CMRMAP: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x26, 0x26, 0x7f),
    Color::rgb(0x4d, 0x26, 0xbe),
    Color::rgb(0x9a, 0x33, 0x7e),
    Color::rgb(0xfe, 0x40, 0x25),
    Color::rgb(0xe5, 0x80, 0x00),
    Color::rgb(0xe5, 0xc0, 0x1b),
    Color::rgb(0xe6, 0xe6, 0x83),
    Color::rgb(0xff, 0xff, 0xff),
];

const CUBEHELIX: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x1a, 0x1d, 0x3b),
    Color::rgb(0x15, 0x53, 0x4b),
    Color::rgb(0x43, 0x77, 0x30),
    Color::rgb(0xa1, 0x79, 0x4a),
    Color::rgb(0xd3, 0x83, 0xa9),
    Color::rgb(0xc6, 0xb4, 0xed),
    Color::rgb(0xcb, 0xe7, 0xef),
    Color::rgb(0xff, 0xff, 0xff),
];

const BRG: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0xff),
    Color::rgb(0x40, 0x00, 0xbf),
    Color::rgb(0x80, 0x00, 0x7f),
    Color::rgb(0xc0, 0x00, 0x3f),
    Color::rgb(0xfe, 0x01, 0x00),
    Color::rgb(0xbe, 0x41, 0x00),
    Color::rgb(0x7e, 0x81, 0x00),
    Color::rgb(0x3e, 0xc1, 0x00),
    Color::rgb(0x00, 0xff, 0x00),
];

const GISTRAINBOW: [Color; 9] = [
    Color::rgb(0xff, 0x00, 0x28),
    Color::rgb(0xff, 0x83, 0x00),
    Color::rgb(0xcd, 0xff, 0x00),
    Color::rgb(0x20, 0xff, 0x00),
    Color::rgb(0x00, 0xff, 0x8b),
    Color::rgb(0x00, 0xc5, 0xff),
    Color::rgb(0x00, 0x17, 0xff),
    Color::rgb(0x96, 0x00, 0xff),
    Color::rgb(0xff, 0x00, 0xbf),
];

const NIPYSPECTRAL: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x00),
    Color::rgb(0x42, 0x00, 0xa1),
    Color::rgb(0x00, 0x77, 0xdd),
    Color::rgb(0x00, 0xaa, 0x97),
    Color::rgb(0x00, 0xbc, 0x00),
    Color::rgb(0x66, 0xff, 0x00),
    Color::rgb(0xff, 0xc9, 0x00),
    Color::rgb(0xeb, 0x00, 0x00),
    Color::rgb(0xcc, 0xcc, 0xcc),
];

const GISTNCAR: [Color; 9] = [
    Color::rgb(0x00, 0x00, 0x80),
    Color::rgb(0x00, 0x46, 0xff),
    Color::rgb(0x00, 0xfa, 0xb0),
    Color::rgb(0x67, 0xd4, 0x00),
    Color::rgb(0xda, 0xff, 0x1f),
    Color::rgb(0xff, 0xbc, 0x0c),
    Color::rgb(0xff, 0x00, 0x46),
    Color::rgb(0xcd, 0x61, 0xf4),
    Color::rgb(0xfe, 0xf7, 0xfe),
];

// ═══════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_differ() {
        let a = Colormap::Viridis.sample(0.0);
        let b = Colormap::Viridis.sample(1.0);
        assert_ne!(a, b);
    }

    #[test]
    fn coolwarm_is_diverging() {
        let lo = Colormap::Coolwarm.sample(0.0);
        let mid = Colormap::Coolwarm.sample(0.5);
        let hi = Colormap::Coolwarm.sample(1.0);
        assert_ne!(lo, hi);
        assert!(mid.r > 200 && mid.g > 200 && mid.b > 200);
    }

    #[test]
    fn tab10_first_is_steelish_blue() {
        let c0 = Colormap::Tab10.sample(0.0);
        assert_eq!(c0, Color::rgb(0x1f, 0x77, 0xb4));
    }

    #[test]
    fn tab10_is_listed_no_lerp() {
        let mid = Colormap::Tab10.sample(0.05);
        assert_eq!(mid, TAB10[0]);
        let next = Colormap::Tab10.sample(0.15);
        assert_eq!(next, TAB10[1]);
        assert_eq!(Colormap::Tab10.listed_len(), Some(10));
    }

    #[test]
    fn map_clamps() {
        let c = Colormap::Plasma.map(100.0, 0.0, 1.0);
        assert_eq!(c, Colormap::Plasma.sample(1.0));
    }

    #[test]
    fn log_norm_midpoint() {
        let t = Norm::Log.normalize(10.0, 1.0, 100.0);
        assert!((t - 0.5).abs() < 1e-9);
        let v = Norm::Log.denormalize(0.5, 1.0, 100.0);
        assert!((v - 10.0).abs() < 1e-9);
    }

    #[test]
    fn two_slope_centers_vcenter() {
        let n = Norm::TwoSlope { vcenter: 0.0 };
        assert!((n.normalize(0.0, -2.0, 4.0) - 0.5).abs() < 1e-12);
        assert!((n.normalize(-2.0, -2.0, 4.0) - 0.0).abs() < 1e-12);
        assert!((n.normalize(4.0, -2.0, 4.0) - 1.0).abs() < 1e-12);
        assert!((n.denormalize(0.5, -2.0, 4.0) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn segmented_from_colors_endpoints() {
        let cmap = SegmentedColormap::from_colors([Color::BLACK, Color::WHITE]).unwrap();
        assert_eq!(cmap.sample(0.0), Color::BLACK);
        assert_eq!(cmap.sample(1.0), Color::WHITE);
        let mid = cmap.sample(0.5);
        assert!(mid.r > 100 && mid.g > 100 && mid.b > 100);
    }

    // ── New colormap tests ─────────────────────────────────────────

    #[test]
    fn sequential_single_hue_endpoints() {
        for cmap in [
            Colormap::Greys,
            Colormap::Blues,
            Colormap::Greens,
            Colormap::Reds,
            Colormap::Oranges,
            Colormap::Purples,
        ] {
            let lo = cmap.sample(0.0);
            let hi = cmap.sample(1.0);
            assert_ne!(lo, hi, "{cmap:?} endpoints should differ");
        }
    }

    #[test]
    fn diverging_midpoint_is_light() {
        for cmap in [
            Colormap::Seismic,
            Colormap::Bwr,
            Colormap::PiYG,
            Colormap::PRGn,
            Colormap::BrBG,
            Colormap::PuOr,
        ] {
            let mid = cmap.sample(0.5);
            assert!(
                mid.r > 200 && mid.g > 200 && mid.b > 200,
                "{cmap:?} midpoint should be near-white, got ({}, {}, {})",
                mid.r,
                mid.g,
                mid.b,
            );
        }
    }

    #[test]
    fn listed_colormaps_have_correct_len() {
        assert_eq!(Colormap::Set1.listed_len(), Some(9));
        assert_eq!(Colormap::Set2.listed_len(), Some(8));
        assert_eq!(Colormap::Set3.listed_len(), Some(12));
        assert_eq!(Colormap::Paired.listed_len(), Some(12));
        assert_eq!(Colormap::Pastel1.listed_len(), Some(9));
        assert_eq!(Colormap::Pastel2.listed_len(), Some(8));
        assert_eq!(Colormap::Dark2.listed_len(), Some(8));
        assert_eq!(Colormap::Accent.listed_len(), Some(8));
        assert_eq!(Colormap::Tab20.listed_len(), Some(20));
        assert_eq!(Colormap::Tab20b.listed_len(), Some(20));
        assert_eq!(Colormap::Tab20c.listed_len(), Some(20));
    }

    #[test]
    fn listed_no_interpolation() {
        let c0 = Colormap::Set1.sample(0.0);
        assert_eq!(c0, SET1[0]);
        let c1 = Colormap::Paired.sample(0.0);
        assert_eq!(c1, PAIRED[0]);
    }

    #[test]
    fn continuous_not_listed() {
        assert_eq!(Colormap::Hot.listed_len(), None);
        assert_eq!(Colormap::Jet.listed_len(), None);
        assert_eq!(Colormap::Turbo.listed_len(), None);
        assert_eq!(Colormap::Seismic.listed_len(), None);
        assert_eq!(Colormap::Greys.listed_len(), None);
    }

    #[test]
    fn sample_reversed_mirrors() {
        let fwd = Colormap::Viridis.sample(0.0);
        let rev = Colormap::Viridis.sample_reversed(0.0);
        assert_eq!(rev, Colormap::Viridis.sample(1.0));
        assert_eq!(fwd, Colormap::Viridis.sample_reversed(1.0));
    }

    #[test]
    fn map_reversed_swaps_ends() {
        let lo = Colormap::Blues.map(0.0, 0.0, 1.0);
        let hi = Colormap::Blues.map(1.0, 0.0, 1.0);
        let lo_r = Colormap::Blues.map_reversed(0.0, 0.0, 1.0);
        let hi_r = Colormap::Blues.map_reversed(1.0, 0.0, 1.0);
        assert_eq!(lo, hi_r);
        assert_eq!(hi, lo_r);
    }

    #[test]
    fn hot_goes_black_to_white() {
        let lo = Colormap::Hot.sample(0.0);
        let hi = Colormap::Hot.sample(1.0);
        assert!(lo.r < 30 && lo.g == 0 && lo.b == 0);
        assert!(hi.r == 255 && hi.g == 255 && hi.b == 255);
    }

    #[test]
    fn jet_midpoint_is_bright() {
        let mid = Colormap::Jet.sample(0.5);
        assert!(mid.r > 100 || mid.g > 200 || mid.b > 100);
    }

    #[test]
    fn from_str_case_insensitive() {
        assert_eq!("Viridis".parse::<Colormap>().unwrap(), Colormap::Viridis);
        assert_eq!("coolwarm".parse::<Colormap>().unwrap(), Colormap::Coolwarm);
        assert_eq!("RdBu_r".parse::<Colormap>().unwrap(), Colormap::RdBuR);
        assert_eq!("grey".parse::<Colormap>().unwrap(), Colormap::Gray);
        assert!("nonexistent".parse::<Colormap>().is_err());
    }
}
