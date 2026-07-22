"""Compute MSE for 3D compare pairs vs previous baseline."""

from pathlib import Path

from PIL import Image
import numpy as np

OUT = Path(__file__).resolve().parents[1] / "compare"
BASELINE = {
    # After segment z-sort + FIT_SHRINK=0.92 (2026-07-19)
    "helix_3d": 1464.0,
    "scatter_3d": 874.0,
    "surface_3d": 1022.0,
    "gaussian_3d": 1174.0,
    "wireframe_3d": 1526.0,
    "bar_3d": 1089.0,
}


def main() -> None:
    print(f"{'name':16} {'MSE':>8} {'delta':>8}")
    for name, base in BASELINE.items():
        ia = np.asarray(Image.open(OUT / f"plotine_{name}.png").convert("RGB"), dtype=np.float64)
        ib_img = Image.open(OUT / f"mpl_{name}.png").convert("RGB")
        if ib_img.size != (ia.shape[1], ia.shape[0]):
            ib_img = ib_img.resize((ia.shape[1], ia.shape[0]), Image.Resampling.BILINEAR)
        ib = np.asarray(ib_img, dtype=np.float64)
        mse = float(((ia - ib) ** 2).mean())
        print(f"{name:16} {mse:8.1f} {mse - base:+8.1f}")


if __name__ == "__main__":
    main()
