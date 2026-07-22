#!/usr/bin/env python3
"""Convert Natural Earth 110m coastline GeoJSON → compact f32 coastline.bin.

Binary layout (little-endian):
  u32 magic = 0x50474C43  ("CGLP" / Coast Geo Line Plotine)
  u32 version = 1
  u32 n_points
  then n_points × (f32 lon, f32 lat)
  Rings/segments separated by NaN lon+lat pairs.
"""

from __future__ import annotations

import json
import math
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "target" / "ne_110m_coastline.geojson"
OUT = ROOT / "crates" / "plotine" / "src" / "geo" / "data" / "coastline.bin"
MAGIC = 0x50474C43
VERSION = 1


def main() -> int:
    if not SRC.is_file():
        print(f"missing {SRC}; download ne_110m_coastline.geojson first", file=sys.stderr)
        return 1
    data = json.loads(SRC.read_text(encoding="utf-8"))
    points: list[tuple[float, float]] = []
    for feat in data.get("features", []):
        geom = feat.get("geometry") or {}
        gtype = geom.get("type")
        coords = geom.get("coordinates") or []
        lines: list[list[list[float]]] = []
        if gtype == "LineString":
            lines = [coords]
        elif gtype == "MultiLineString":
            lines = coords
        else:
            continue
        for line in lines:
            if len(line) < 2:
                continue
            for lon, lat, *_ in line:
                if math.isfinite(lon) and math.isfinite(lat):
                    points.append((float(lon), float(lat)))
            points.append((math.nan, math.nan))  # segment break
    # drop trailing break
    while points and math.isnan(points[-1][0]):
        points.pop()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("wb") as f:
        f.write(struct.pack("<II", MAGIC, VERSION))
        f.write(struct.pack("<I", len(points)))
        for lon, lat in points:
            f.write(struct.pack("<ff", lon, lat))
    print(f"wrote {OUT} ({OUT.stat().st_size} bytes, {len(points)} points)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
