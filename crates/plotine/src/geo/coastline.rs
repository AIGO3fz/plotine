//! Embedded Natural Earth 110m coastline (simplified polyline).

use crate::series::Series;

const MAGIC: u32 = 0x5047_4C43; // "CGLP"
const VERSION: u32 = 1;

static COASTLINE_BIN: &[u8] = include_bytes!("data/coastline.bin");

/// Number of lon/lat samples in the embedded coastline (including NaN breaks).
pub fn coastline_point_count() -> usize {
    if COASTLINE_BIN.len() < 12 {
        return 0;
    }
    u32::from_le_bytes(COASTLINE_BIN[8..12].try_into().unwrap()) as usize
}

/// Decode embedded coastline as parallel lon/lat series (NaN segment breaks).
pub fn coastline_lonlat() -> (Series, Series) {
    let (lon, lat) = decode_coastline(COASTLINE_BIN).expect("embedded coastline.bin is valid");
    (Series::new(lon), Series::new(lat))
}

fn decode_coastline(bytes: &[u8]) -> Result<(Vec<f64>, Vec<f64>), &'static str> {
    if bytes.len() < 12 {
        return Err("coastline.bin too short");
    }
    let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let n = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if magic != MAGIC {
        return Err("bad coastline magic");
    }
    if version != VERSION {
        return Err("unsupported coastline version");
    }
    let need = 12 + n * 8;
    if bytes.len() < need {
        return Err("coastline.bin truncated");
    }
    let mut lon = Vec::with_capacity(n);
    let mut lat = Vec::with_capacity(n);
    let mut off = 12;
    for _ in 0..n {
        let lo = f32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()) as f64;
        let la = f32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap()) as f64;
        lon.push(lo);
        lat.push(la);
        off += 8;
    }
    Ok((lon, lat))
}
