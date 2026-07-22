#!/usr/bin/env python3
"""Pixel MAE for M9–M13 feature pairs in compare/.

Usage (repo root):
  cargo run -p plotine --example matplotlib_compare
  python scripts/matplotlib_compare.py
  python scripts/pixel_align_features.py

Writes absdiff heatmaps to compare/feature_absdiff_*.png and prints MAE table.

Note: Skia vs Agg + different math fonts cannot reach MAE=0 for text-heavy
charts. Target bands (empirical, 0–255 RGB mean abs):
  excellent < 8 · good < 18 · needs work >= 18
"""

from __future__ import annotations

from pathlib import Path

import numpy as np

try:
    from PIL import Image
except ImportError as exc:
    raise SystemExit("pip install pillow numpy") from exc

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "compare"

# (stem without plotine_/mpl_ prefix, milestone label)
PAIRS = [
    ("m9_static", "M9 static render / show pixels"),
    ("m10_anim_frame", "M10 animation frame 0"),
    ("m11_geo", "M11 PlateCarree + coastline"),
    ("m12_pyplot", "M12 builder (== pyplot)"),
    ("m13_mathtext", "M13 mathtext integral"),
]


def load_rgba(path: Path) -> np.ndarray:
    im = Image.open(path).convert("RGBA")
    return np.asarray(im, dtype=np.int16)


def mae_pair(a: Path, b: Path) -> tuple[float, float, tuple[int, int], Path | None]:
    if not a.is_file() or not b.is_file():
        return float("nan"), float("nan"), (0, 0), None
    pa, pb = load_rgba(a), load_rgba(b)
    if pa.shape != pb.shape:
        # Resize mpl to plotine size for a fair-ish metric (should not happen)
        pb_im = Image.open(b).convert("RGBA").resize((pa.shape[1], pa.shape[0]), Image.Resampling.BILINEAR)
        pb = np.asarray(pb_im, dtype=np.int16)
    diff = np.abs(pa.astype(np.int16) - pb.astype(np.int16))
    mae = float(diff[:, :, :3].mean())
    mx = float(diff[:, :, :3].max())
    # Heatmap: amplify for visibility
    heat = np.clip(diff[:, :, :3].mean(axis=2) * 4.0, 0, 255).astype(np.uint8)
    heat_rgb = np.stack([heat, np.zeros_like(heat), heat], axis=2)
    out = OUT / f"feature_absdiff_{a.stem.replace('plotine_', '')}.png"
    Image.fromarray(heat_rgb).save(out)
    return mae, mx, (pa.shape[1], pa.shape[0]), out


def band(mae: float) -> str:
    if mae != mae:  # NaN
        return "MISSING"
    if mae < 8:
        return "excellent"
    if mae < 18:
        return "good"
    if mae < 35:
        return "fair"
    return "needs-work"


def main() -> None:
    print(f"{'pair':18} {'WxH':>12} {'MAE':>8} {'max':>8}  band")
    print("-" * 60)
    rows = []
    for stem, label in PAIRS:
        a = OUT / f"plotine_{stem}.png"
        b = OUT / f"mpl_{stem}.png"
        mae, mx, (w, h), heat = mae_pair(a, b)
        dim = f"{w}x{h}" if w else "—"
        print(f"{stem:18} {dim:>12} {mae:8.2f} {mx:8.1f}  {band(mae)}  # {label}")
        rows.append((stem, mae, band(mae)))

    # M12 extra: pyplot facade vs builder must be ~0 if both generated
    pyplot_facade = ROOT / "compare" / "size_bench" / "pyplot_line_150.png"
    builder = ROOT / "compare" / "size_bench" / "plotine_line_150.png"
    if pyplot_facade.is_file() and builder.is_file():
        mae, mx, _, _ = mae_pair(pyplot_facade, builder)
        print(f"{'pyplot≡builder':18} {'(size_bench)':>12} {mae:8.2f} {mx:8.1f}  {band(mae)}")

    print("\nAbsdiff heatmaps: compare/feature_absdiff_*.png")
    print(
        "Engine floor: text/AA differences (DejaVu mathtext vs mpl) prevent MAE=0; "
        "drive geometry/colors/ticks first."
    )


if __name__ == "__main__":
    main()
