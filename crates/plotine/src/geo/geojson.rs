//! Minimal GeoJSON parser (FeatureCollection / Feature / geometry subset).
//!
//! Supports `Point`, `LineString`, `Polygon`, `MultiLineString`, `MultiPolygon`.
//! No CRS / shapefile stack — lon/lat degrees only.

use plotine_core::{PlotError, Result};

/// One drawable geometry in lon/lat degrees.
#[derive(Debug, Clone)]
pub enum GeoGeom {
    /// Single point.
    Point(f64, f64),
    /// Open polyline (NaN breaks not used; one continuous ring).
    LineString(Vec<(f64, f64)>),
    /// Exterior ring only (holes ignored in MVP).
    Polygon(Vec<(f64, f64)>),
}

/// Parse a GeoJSON document into drawable geometries.
pub fn parse_geojson(data: &[u8]) -> Result<Vec<GeoGeom>> {
    let text = std::str::from_utf8(data)
        .map_err(|e| PlotError::render(format!("GeoJSON is not UTF-8: {e}")))?;
    let mut p = Parser::new(text);
    p.skip_ws();
    let v = p.parse_value()?;
    collect_geoms(&v)
}

#[derive(Debug, Clone)]
enum Json {
    Null,
    Bool(#[allow(dead_code)] bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }

    fn err(&self, msg: impl Into<String>) -> PlotError {
        PlotError::render(format!("GeoJSON parse at {}: {}", self.i, msg.into()))
    }

    fn peek(&self) -> Option<char> {
        self.s[self.i..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.i += ch.len_utf8();
        Some(ch)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.bump();
        }
    }

    fn expect(&mut self, c: char) -> Result<()> {
        self.skip_ws();
        match self.bump() {
            Some(ch) if ch == c => Ok(()),
            other => Err(self.err(format!("expected '{c}', got {other:?}"))),
        }
    }

    fn parse_value(&mut self) -> Result<Json> {
        self.skip_ws();
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => Ok(Json::String(self.parse_string()?)),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c == '-' || c.is_ascii_digit() => Ok(Json::Number(self.parse_number()?)),
            other => Err(self.err(format!("unexpected token {other:?}"))),
        }
    }

    fn parse_null(&mut self) -> Result<Json> {
        if self.s[self.i..].starts_with("null") {
            self.i += 4;
            Ok(Json::Null)
        } else {
            Err(self.err("expected null"))
        }
    }

    fn parse_bool(&mut self) -> Result<Json> {
        if self.s[self.i..].starts_with("true") {
            self.i += 4;
            Ok(Json::Bool(true))
        } else if self.s[self.i..].starts_with("false") {
            self.i += 5;
            Ok(Json::Bool(false))
        } else {
            Err(self.err("expected true/false"))
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.bump() {
                Some('"') => return Ok(out),
                Some('\\') => match self.bump() {
                    Some(c) => out.push(match c {
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'b' => '\u{0008}',
                        'f' => '\u{000c}',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'u' => {
                            let mut hex = String::new();
                            for _ in 0..4 {
                                hex.push(self.bump().ok_or_else(|| self.err("bad \\u"))?);
                            }
                            let cp = u32::from_str_radix(&hex, 16)
                                .map_err(|_| self.err("bad \\u hex"))?;
                            char::from_u32(cp).ok_or_else(|| self.err("bad unicode"))?
                        }
                        other => other,
                    }),
                    None => return Err(self.err("unterminated escape")),
                },
                Some(c) => out.push(c),
                None => return Err(self.err("unterminated string")),
            }
        }
    }

    fn parse_number(&mut self) -> Result<f64> {
        let start = self.i;
        if self.peek() == Some('-') {
            self.bump();
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        if self.peek() == Some('.') {
            self.bump();
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.bump();
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
            }
        }
        self.s[start..self.i]
            .parse()
            .map_err(|_| self.err("invalid number"))
    }

    fn parse_array(&mut self) -> Result<Json> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.bump();
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some(']') => {
                    self.bump();
                    return Ok(Json::Array(items));
                }
                other => return Err(self.err(format!("expected , or ], got {other:?}"))),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json> {
        self.expect('{')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.bump();
            return Ok(Json::Object(items));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.expect(':')?;
            let val = self.parse_value()?;
            items.push((key, val));
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some('}') => {
                    self.bump();
                    return Ok(Json::Object(items));
                }
                other => return Err(self.err(format!("expected , or }}, got {other:?}"))),
            }
        }
    }
}

fn obj_get<'a>(obj: &'a [(String, Json)], key: &str) -> Option<&'a Json> {
    obj.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn collect_geoms(v: &Json) -> Result<Vec<GeoGeom>> {
    let mut out = Vec::new();
    match v {
        Json::Object(obj) => {
            if let Some(Json::String(t)) = obj_get(obj, "type") {
                match t.as_str() {
                    "FeatureCollection" => {
                        if let Some(Json::Array(feats)) = obj_get(obj, "features") {
                            for f in feats {
                                out.extend(collect_geoms(f)?);
                            }
                        }
                    }
                    "Feature" => {
                        if let Some(geom) = obj_get(obj, "geometry") {
                            out.extend(collect_geoms(geom)?);
                        }
                    }
                    "GeometryCollection" => {
                        if let Some(Json::Array(geoms)) = obj_get(obj, "geometries") {
                            for g in geoms {
                                out.extend(collect_geoms(g)?);
                            }
                        }
                    }
                    "Point" | "LineString" | "Polygon" | "MultiLineString" | "MultiPolygon" => {
                        out.extend(geom_from_typed(t, obj_get(obj, "coordinates"))?);
                    }
                    _ => {}
                }
            }
        }
        _ => {
            return Err(PlotError::render(
                "GeoJSON root must be an object (FeatureCollection / Feature / geometry)",
            ));
        }
    }
    Ok(out)
}

fn geom_from_typed(ty: &str, coords: Option<&Json>) -> Result<Vec<GeoGeom>> {
    let coords =
        coords.ok_or_else(|| PlotError::render(format!("GeoJSON {ty} missing coordinates")))?;
    match ty {
        "Point" => {
            let (x, y) = pair(coords)?;
            Ok(vec![GeoGeom::Point(x, y)])
        }
        "LineString" => Ok(vec![GeoGeom::LineString(line_string(coords)?)]),
        "Polygon" => {
            let rings = polygon_rings(coords)?;
            Ok(rings.into_iter().take(1).map(GeoGeom::Polygon).collect())
        }
        "MultiLineString" => {
            let Json::Array(lines) = coords else {
                return Err(PlotError::render(
                    "MultiLineString coordinates must be array",
                ));
            };
            let mut out = Vec::new();
            for line in lines {
                out.push(GeoGeom::LineString(line_string(line)?));
            }
            Ok(out)
        }
        "MultiPolygon" => {
            let Json::Array(polys) = coords else {
                return Err(PlotError::render("MultiPolygon coordinates must be array"));
            };
            let mut out = Vec::new();
            for poly in polys {
                let rings = polygon_rings(poly)?;
                if let Some(ext) = rings.into_iter().next() {
                    out.push(GeoGeom::Polygon(ext));
                }
            }
            Ok(out)
        }
        _ => Ok(vec![]),
    }
}

fn pair(v: &Json) -> Result<(f64, f64)> {
    let Json::Array(a) = v else {
        return Err(PlotError::render("coordinate pair must be array"));
    };
    if a.len() < 2 {
        return Err(PlotError::render("coordinate pair needs [lon, lat]"));
    }
    Ok((as_f64(&a[0])?, as_f64(&a[1])?))
}

fn as_f64(v: &Json) -> Result<f64> {
    match v {
        Json::Number(n) => Ok(*n),
        _ => Err(PlotError::render("expected number in coordinates")),
    }
}

fn line_string(v: &Json) -> Result<Vec<(f64, f64)>> {
    let Json::Array(pts) = v else {
        return Err(PlotError::render("LineString coordinates must be array"));
    };
    pts.iter().map(pair).collect()
}

fn polygon_rings(v: &Json) -> Result<Vec<Vec<(f64, f64)>>> {
    let Json::Array(rings) = v else {
        return Err(PlotError::render(
            "Polygon coordinates must be array of rings",
        ));
    };
    rings.iter().map(line_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_feature_collection() {
        let js = br#"{
          "type":"FeatureCollection",
          "features":[{
            "type":"Feature",
            "geometry":{"type":"LineString","coordinates":[[0,0],[10,5],[20,0]]},
            "properties":{}
          },{
            "type":"Feature",
            "geometry":{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]},
            "properties":{}
          }]
        }"#;
        let g = parse_geojson(js).unwrap();
        assert_eq!(g.len(), 2);
        match &g[0] {
            GeoGeom::LineString(pts) => assert_eq!(pts.len(), 3),
            other => panic!("expected LineString, got {other:?}"),
        }
        match &g[1] {
            GeoGeom::Polygon(pts) => assert_eq!(pts.len(), 5),
            other => panic!("expected Polygon, got {other:?}"),
        }
    }
}
