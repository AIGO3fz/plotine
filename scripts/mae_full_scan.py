#!/usr/bin/env python3
"""Full MAE scan for all compare pairs (not just M9-M13).

Usage (repo root, after generating compare PNGs):
  python scripts/mae_full_scan.py
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


def load_rgb(path: Path) -> np.ndarray:
    return np.asarray(Image.open(path).convert("RGB"), dtype=np.int16)


def mae_pair(a: Path, b: Path) -> tuple[float, float, tuple[int, int]]:
    pa, pb = load_rgb(a), load_rgb(b)
    if pa.shape != pb.shape:
        pb_im = Image.open(b).convert("RGB").resize(
            (pa.shape[1], pa.shape[0]), Image.Resampling.BILINEAR
        )
        pb = np.asarray(pb_im, dtype=np.int16)
    diff = np.abs(pa - pb)
    return float(diff.mean()), float(diff.max()), (pa.shape[1], pa.shape[0])


def band(mae: float) -> str:
    if mae < 8:
        return "excellent"
    if mae < 18:
        return "good"
    if mae < 35:
        return "fair"
    return "needs-work"


def main() -> None:
    plotine_files = sorted(OUT.glob("plotine_*.png"))
    rows: list[tuple[str, float, float, str, str]] = []

    for pf in plotine_files:
        stem = pf.stem.replace("plotine_", "")
        mf = OUT / f"mpl_{stem}.png"
        if not mf.is_file():
            continue
        mae, mx, (w, h) = mae_pair(pf, mf)
        rows.append((stem, mae, mx, f"{w}x{h}", band(mae)))

    rows.sort(key=lambda r: -r[1])

    print(f"{'rank':>4}  {'pair':<24} {'WxH':>10} {'MAE':>8} {'max':>6}  band")
    print("-" * 72)
    for i, (stem, mae, mx, dim, b) in enumerate(rows, 1):
        print(f"{i:4}  {stem:<24} {dim:>10} {mae:8.2f} {mx:6.0f}  {b}")

    excellent = sum(1 for r in rows if r[4] == "excellent")
    good = sum(1 for r in rows if r[4] == "good")
    fair = sum(1 for r in rows if r[4] == "fair")
    needs = sum(1 for r in rows if r[4] == "needs-work")
    avg = np.mean([r[1] for r in rows])

    print(f"\nTotal: {len(rows)} pairs")
    print(f"  excellent (<8):   {excellent}")
    print(f"  good (8-18):      {good}")
    print(f"  fair (18-35):     {fair}")
    print(f"  needs-work (>=35):{needs}")
    print(f"  average MAE:      {avg:.2f}")


if __name__ == "__main__":
    main()
