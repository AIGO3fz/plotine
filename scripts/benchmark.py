#!/usr/bin/env python3
"""Product L1 benchmark harness (docs/BENCHMARK.md).

Runs:
  cargo run -p plotine --example bench_suite --release

Then matplotlib counterparts (savefig to BytesIO, same figsize/DPI/seed),
prints a comparison table, and writes compare/bench/results.csv.

Usage (repo root):
  python scripts/benchmark.py
  python scripts/benchmark.py --tier smoke
  python scripts/benchmark.py --tier stress
  python scripts/benchmark.py --filter series.line
  # Windows PowerShell: also write sample PNGs under compare/bench/
  $env:BENCH_SAVE=1; python scripts/benchmark.py
"""

from __future__ import annotations

import argparse
import csv
import io
import math
import os
import re
import subprocess
import sys
import time
from pathlib import Path

import numpy as np

try:
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
except ImportError as exc:
    raise SystemExit("pip install matplotlib numpy") from exc

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "compare" / "bench"
COASTLINE_BIN = (
    ROOT / "crates" / "plotine" / "src" / "geo" / "data" / "coastline.bin"
)
FIGSIZE = (5.0, 3.5)
DPI = 150
WARMUP = 2
ITERS = 7

BENCH_RE = re.compile(
    r"^BENCH name=(?P<name>\S+) median_ms=(?P<median>[0-9.]+) "
    r"p95_ms=(?P<p95>[0-9.]+) bytes=(?P<bytes>\d+) fmt=(?P<fmt>\S+)$"
)

TIER_SMOKE = {
    "chrome.empty",
    "series.line_n1000",
    "series.scatter_n1000",
    "stat.heatmap_64",
    "layout.subplots_2x2",
    "math.mathtext",
    "fmt.svg_line_n1000",
}

TIER_STRESS = {
    "series.line_n1e6",
    "stat.heatmap_512",
    "field.streamplot_40",
    "d3.surface_80",
    "layout.subplots_4x4",
    "composite.dashboard",
}


def tier_allows(tier: str, name: str) -> bool:
    t = (tier or "default").lower()
    if t in ("smoke", "s"):
        return name in TIER_SMOKE
    if t in ("stress", "all", "b"):
        return True
    return name not in TIER_STRESS


def series_sin(n: int) -> tuple[np.ndarray, np.ndarray]:
    x = np.arange(n, dtype=np.float64) * (10.0 / max(n, 1))
    return x, np.sin(x)


def seeded_samples(n: int) -> np.ndarray:
    """Match `bench_suite.rs` LCG + Box–Muller (not numpy Generator).

    Both sides claim seed 42, but numpy `default_rng(42)` is a different
    stream — hist2d/hexbin/violin looked unrelated until this was aligned.
    """
    state = 42
    out = np.empty(n, dtype=np.float64)
    u32_max = float(np.iinfo(np.uint32).max)
    i = 0
    while i < n:
        state = (state * 6364136223846793005 + 1) & 0xFFFFFFFFFFFFFFFF
        u1 = min(1.0, max(1e-12, (state >> 33) / u32_max))
        state = (state * 6364136223846793005 + 1) & 0xFFFFFFFFFFFFFFFF
        u2 = min(1.0, max(1e-12, (state >> 33) / u32_max))
        r = math.sqrt(-2.0 * math.log(u1))
        out[i] = r * math.cos(2.0 * math.pi * u2)
        i += 1
    return out


def cloud_xy(n: int) -> tuple[np.ndarray, np.ndarray]:
    s = seeded_samples(n * 2)
    x = s[:n]
    y = 0.6 * s[n:] + 0.4 * s[:n]
    return x, y


def grid_gauss(n: int) -> np.ndarray:
    scale = 8.0 / max(n - 1, 1)
    ys = np.arange(n) * scale - 4.0
    xs = np.arange(n) * scale - 4.0
    xx, yy = np.meshgrid(xs, ys)
    return np.exp(-xx * xx - yy * yy) * 2.0 + 0.3 * np.sin(xx * 0.8)


def vortex_uv(n: int) -> tuple[np.ndarray, np.ndarray]:
    mid = (n - 1) * 0.5
    yy, xx = np.mgrid[0:n, 0:n]
    u = -(yy - mid)
    v = xx - mid
    return u.astype(float), v.astype(float)


def load_plotine_coastline_bin(path: Path) -> tuple[np.ndarray, np.ndarray]:
    import struct

    data = path.read_bytes()
    magic, version, n = struct.unpack_from("<III", data, 0)
    if magic != 0x50474C43 or version != 1:
        raise ValueError(f"bad coastline.bin header magic={magic:#x} ver={version}")
    lon = np.empty(n, dtype=np.float64)
    lat = np.empty(n, dtype=np.float64)
    off = 12
    for i in range(n):
        lo, la = struct.unpack_from("<ff", data, off)
        lon[i] = lo
        lat[i] = la
        off += 8
    return lon, lat


def percentile(sorted_vals: list[float], p: float) -> float:
    if not sorted_vals:
        return 0.0
    idx = int(round((len(sorted_vals) - 1) * p))
    return sorted_vals[min(max(idx, 0), len(sorted_vals) - 1)]


def time_mpl(build_save) -> tuple[float, float, int]:
    """Warmup + measure; build_save() -> bytes."""
    for _ in range(WARMUP):
        build_save()
    times: list[float] = []
    last = b""
    for _ in range(ITERS):
        t0 = time.perf_counter()
        last = build_save()
        times.append((time.perf_counter() - t0) * 1000.0)
    times.sort()
    return percentile(times, 0.50), percentile(times, 0.95), len(last)


def save_png_bytes(fig) -> bytes:
    buf = io.BytesIO()
    fig.savefig(buf, format="png", dpi=DPI)
    plt.close(fig)
    return buf.getvalue()


def save_svg_bytes(fig) -> bytes:
    buf = io.BytesIO()
    fig.savefig(buf, format="svg", dpi=DPI)
    plt.close(fig)
    return buf.getvalue()


def save_pdf_bytes(fig) -> bytes:
    buf = io.BytesIO()
    fig.savefig(buf, format="pdf", dpi=DPI)
    plt.close(fig)
    return buf.getvalue()


def mpl_cases() -> dict[str, callable]:
    """Matplotlib twins for plotine bench_suite scenarios."""

    def empty():
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.set_title("Empty")
        ax.set_xlabel("x")
        ax.set_ylabel("y")
        ax.grid(True)
        return save_png_bytes(fig)

    def line(n: int):
        def go():
            x, y = series_sin(n)
            fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
            ax.plot(x, y, color="crimson", lw=1.5, label="sin")
            ax.set_title(f"series.line_n{n}")
            ax.set_xlabel("x")
            ax.set_ylabel("y")
            ax.legend(loc="upper right")
            ax.grid(True)
            return save_png_bytes(fig)

        return go

    def scatter(n: int):
        def go():
            x, y = series_sin(n)
            fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
            ax.scatter(x, y, s=9, c="steelblue")
            ax.set_title(f"scatter_n{n}")
            ax.set_xlabel("x")
            ax.set_ylabel("y")
            return save_png_bytes(fig)

        return go

    def bar():
        x = np.arange(50, dtype=float)
        h = 1.0 + np.abs(np.sin(x * 0.37)) * 4.0
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.bar(x, h, color="steelblue")
        ax.set_title("bar_n50")
        return save_png_bytes(fig)

    def hist():
        data = seeded_samples(10_000)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.hist(data, bins=30, color="mediumpurple")
        ax.set_title("hist_n1e4")
        return save_png_bytes(fig)

    def area():
        x, y = series_sin(1000)
        y = np.abs(y) + 0.15
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.fill_between(x, y, alpha=0.45, color="forestgreen", label="area")
        ax.set_title("area_n1e3")
        ax.legend(loc="upper right")
        return save_png_bytes(fig)

    def errorbar():
        x, y = series_sin(200)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.errorbar(x, y, yerr=0.15, color="steelblue", fmt="o", ms=2)
        ax.set_title("errorbar_n200")
        return save_png_bytes(fig)

    def multiline():
        x, _ = series_sin(1000)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        for k in range(5):
            ax.plot(x, np.sin(x + k * 0.4) + k * 0.15, lw=1.2, label=f"s{k}")
        ax.set_title("multiline_5x1e3")
        ax.legend(loc="upper right")
        return save_png_bytes(fig)

    def heatmap(n: int):
        def go():
            z = grid_gauss(n)
            fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
            im = ax.imshow(z, cmap="viridis", origin="upper", aspect="auto")
            fig.colorbar(im, ax=ax)
            ax.set_title(f"heatmap_{n}")
            return save_png_bytes(fig)

        return go

    def boxplot():
        g = seeded_samples(400).reshape(4, 100)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.boxplot(g.T)
        ax.set_title("boxplot")
        return save_png_bytes(fig)

    def violin():
        g = seeded_samples(400).reshape(4, 100)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        # Stock mpl + plotine Theme::light: purple fill, extrema, no median.
        parts = ax.violinplot([g[i] for i in range(4)], showmedians=False)
        for body in parts["bodies"]:
            body.set_facecolor("mediumpurple")
            body.set_alpha(0.55)
            body.set_edgecolor("none")
        ax.set_title("violin")
        ax.grid(True)
        return save_png_bytes(fig)

    def hist2d():
        x, y = cloud_xy(10_000)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        # plotine hist2d/hexbin default colorbar=true
        _, _, _, mesh = ax.hist2d(x, y, bins=30, cmap="viridis")
        fig.colorbar(mesh, ax=ax)
        ax.set_title("hist2d_1e4")
        ax.grid(True)
        return save_png_bytes(fig)

    def hexbin():
        x, y = cloud_xy(10_000)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        mesh = ax.hexbin(x, y, gridsize=20, cmap="plasma")
        fig.colorbar(mesh, ax=ax)
        ax.set_title("hexbin_1e4")
        ax.grid(True)
        return save_png_bytes(fig)

    def contourf():
        z = grid_gauss(80)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.contourf(z, levels=12, cmap="viridis")
        ax.set_title("contourf_80")
        return save_png_bytes(fig)

    def pcolormesh():
        n = 80
        xe = np.linspace(-4, 4, n + 1)
        ye = np.linspace(-4, 4, n + 1)
        z = grid_gauss(n)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.pcolormesh(xe, ye, z, cmap="viridis")
        ax.set_title("pcolormesh_80")
        return save_png_bytes(fig)

    def quiver():
        n = 20
        mid = (n - 1) * 0.5
        yy, xx = np.mgrid[0:n, 0:n]
        u = -(yy - mid) * 0.3
        v = (xx - mid) * 0.3
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.quiver(xx, yy, u, v, color="steelblue")
        ax.set_title("quiver_20")
        return save_png_bytes(fig)

    def streamplot():
        u, v = vortex_uv(20)
        y = np.arange(20)
        x = np.arange(20)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.streamplot(x, y, u, v, color="crimson", density=1.0, linewidth=0.9)
        ax.set_title("streamplot_20")
        return save_png_bytes(fig)

    def tripcolor():
        import matplotlib.tri as mtri

        n = 32
        xs = np.arange(n, dtype=float)
        ys = np.arange(n, dtype=float)
        xx, yy = np.meshgrid(xs, ys)
        zz = np.exp(-(xx * xx + yy * yy) * 0.02)
        tris = []
        for r in range(n - 1):
            for c in range(n - 1):
                i = r * n + c
                tris.append([i, i + 1, i + n])
                tris.append([i + 1, i + n + 1, i + n])
        tri = mtri.Triangulation(xx.ravel(), yy.ravel(), tris)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.tripcolor(tri, zz.ravel(), cmap="viridis")
        ax.set_title("tripcolor_1e3")
        return save_png_bytes(fig)

    def spy():
        n = 40
        m = np.eye(n)
        for i in range(n - 3):
            m[i, i + 3] = 0.5
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.spy(m, markersize=3, color="steelblue")
        ax.set_title("spy_40")
        return save_png_bytes(fig)

    def subplots():
        x, y = series_sin(500)
        fig, axs = plt.subplots(2, 2, figsize=FIGSIZE, dpi=DPI)
        for i, ax in enumerate(axs.flat):
            ax.plot(x, y, color="steelblue", lw=1.2)
            ax.set_title(f"p{i // 2}{i % 2}")
        fig.tight_layout()
        return save_png_bytes(fig)

    def mosaic():
        x, y = series_sin(400)
        fig = plt.figure(figsize=FIGSIZE, dpi=DPI)
        axs = fig.subplot_mosaic("AAB;CCD")
        for name, ax in axs.items():
            if name in ("A", "B"):
                ax.plot(x, y, color="steelblue")
            elif name == "C":
                ax.scatter(x, y, s=4)
            else:
                ax.hist(y, bins=12)
            ax.set_title(name)
        fig.tight_layout()
        return save_png_bytes(fig)

    def twin_y():
        x, left = series_sin(200)
        right = x * x * 0.05 + 1.0
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.plot(x, left, color="steelblue", lw=1.5, label="L")
        ax2 = ax.twinx()
        ax2.plot(x, right, color="crimson", lw=1.5, label="R")
        ax.set_title("twin_y")
        ax.grid(True)
        return save_png_bytes(fig)

    def inset():
        x, y = series_sin(200)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.plot(x, y, color="steelblue", lw=1.5)
        ax.set_title("inset")
        axins = ax.inset_axes([0.55, 0.55, 0.4, 0.4])
        axins.plot(x[:40], y[:40], color="crimson", lw=1.2)
        axins.set_title("zoom")
        return save_png_bytes(fig)

    def secondary_x():
        th = np.arange(80) * math.pi / 40.0
        y = np.sin(th)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.plot(th, y, color="steelblue", lw=1.5)
        ax.set_title("secondary_x")
        ax.set_xlabel("rad")
        sec = ax.secondary_xaxis(
            "top", functions=(np.rad2deg, np.deg2rad)
        )
        sec.set_xlabel("deg")
        return save_png_bytes(fig)

    def loglog():
        x = 10 ** np.linspace(-1, 2, 50)
        y = 3.0 * x**1.4
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.loglog(x, y, lw=1.5, label="power")
        ax.set_title("loglog")
        ax.legend(loc="upper left")
        return save_png_bytes(fig)

    def datetime_axis():
        import matplotlib.dates as mdates
        from datetime import datetime, timedelta, timezone

        t0 = datetime(2020, 1, 1, tzinfo=timezone.utc)
        xs = [t0 + timedelta(days=i) for i in range(60)]
        y = 8.0 + np.sin(np.arange(60) * 0.25) * 1.5
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.plot(mdates.date2num(xs), y, color="steelblue", lw=1.5)
        ax.xaxis_date()
        ax.set_title("datetime")
        return save_png_bytes(fig)

    def polar():
        th = np.linspace(0, 2 * math.pi, 120, endpoint=False)
        pr = 1.0 + 0.35 * np.cos(2.0 * th)
        fig, ax = plt.subplots(
            figsize=FIGSIZE, dpi=DPI, subplot_kw={"projection": "polar"}
        )
        ax.plot(th, pr, color="crimson", lw=1.5)
        ax.set_title("polar.line")
        return save_png_bytes(fig)

    def annotate_styles():
        x, y = series_sin(80)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.plot(x, y, color="steelblue", lw=1.5)
        styles = [
            ("-|>", (2.0, 0.9), (3.0, 1.2)),
            ("->", (4.0, -0.5), (5.0, -0.9)),
            ("-[", (6.0, 0.4), (7.0, 0.8)),
            ("<->", (1.0, -0.8), (0.2, -1.1)),
        ]
        for style, xy, xytext in styles:
            ax.annotate(
                "x",
                xy=xy,
                xytext=xytext,
                arrowprops=dict(arrowstyle=style, color="crimson"),
                color="crimson",
            )
        ax.set_title("annotate_styles")
        return save_png_bytes(fig)

    def table():
        x, y = series_sin(40)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.plot(x, y, color="steelblue")
        ax.table(
            cellText=[["A", "3"], ["B", "5"], ["C", "2"]],
            colLabels=["name", "n"],
            loc="upper right",
        )
        ax.set_title("table")
        return save_png_bytes(fig)

    def pie():
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.pie([35, 25, 20, 12, 8], labels=["A", "B", "C", "D", "E"])
        ax.set_title("pie")
        return save_png_bytes(fig)

    def stackplot():
        x = np.arange(40) * 0.25
        s0 = 1.0 + 0.3 * np.sin(x)
        s1 = 1.5 + 0.2 * np.cos(x * 0.7)
        s2 = 0.8 + 0.15 * np.sin(x * 1.3)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.stackplot(x, s0, s1, s2, labels=["a", "b", "c"], alpha=0.85)
        ax.set_title("stackplot")
        ax.legend(loc="upper left")
        return save_png_bytes(fig)

    def helix():
        from mpl_toolkits.mplot3d import Axes3D  # noqa: F401

        t = np.arange(200) * 0.1
        fig = plt.figure(figsize=FIGSIZE, dpi=DPI)
        ax = fig.add_subplot(111, projection="3d")
        ax.plot(np.cos(t), np.sin(t), t, color="crimson", lw=1.5)
        ax.view_init(elev=30, azim=-60)
        ax.set_title("helix")
        return save_png_bytes(fig)

    def scatter3d():
        from mpl_toolkits.mplot3d import Axes3D  # noqa: F401

        n = 200
        t = np.arange(n) * 0.1
        fig = plt.figure(figsize=FIGSIZE, dpi=DPI)
        ax = fig.add_subplot(111, projection="3d")
        ax.scatter(np.cos(t), np.sin(t), np.linspace(0, 10, n), c="steelblue", s=9)
        ax.set_title("scatter3d")
        return save_png_bytes(fig)

    def surface():
        from mpl_toolkits.mplot3d import Axes3D  # noqa: F401

        n = 40
        xs = np.linspace(-2, 2, n)
        ys = np.linspace(-2, 2, n)
        xx, yy = np.meshgrid(xs, ys)
        zz = np.exp(-0.5 * (xx * xx + yy * yy))
        fig = plt.figure(figsize=FIGSIZE, dpi=DPI)
        ax = fig.add_subplot(111, projection="3d")
        ax.plot_surface(xx, yy, zz, cmap="viridis", alpha=0.9, linewidth=0)
        ax.view_init(elev=35, azim=-50)
        ax.set_title("surface_40")
        return save_png_bytes(fig)

    def wireframe():
        from mpl_toolkits.mplot3d import Axes3D  # noqa: F401

        n = 40
        xs = np.linspace(-2, 2, n)
        ys = np.linspace(-2, 2, n)
        xx, yy = np.meshgrid(xs, ys)
        zz = np.exp(-0.5 * (xx * xx + yy * yy))
        fig = plt.figure(figsize=FIGSIZE, dpi=DPI)
        ax = fig.add_subplot(111, projection="3d")
        ax.plot_wireframe(xx, yy, zz, color="steelblue", linewidth=0.5)
        ax.view_init(elev=35, azim=-50)
        ax.set_title("wireframe_40")
        return save_png_bytes(fig)

    def bar3d():
        from mpl_toolkits.mplot3d import Axes3D  # noqa: F401

        x = np.arange(20) % 5
        y = np.arange(20) // 5
        z = np.zeros(20)
        dz = 1.0 + np.abs(np.sin(np.arange(20) * 0.4)) * 3.0
        fig = plt.figure(figsize=FIGSIZE, dpi=DPI)
        ax = fig.add_subplot(111, projection="3d")
        ax.bar3d(x, y, z, 0.6, 0.6, dz, color="steelblue", shade=True)
        ax.view_init(elev=30, azim=-55)
        ax.set_title("bar3d")
        return save_png_bytes(fig)

    def mathtext():
        x, y = series_sin(200)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.plot(x, y, color="steelblue", lw=1.5)
        ax.set_title(r"$\int_0^1 x^2\,dx$")
        ax.set_xlabel(r"$x$")
        ax.set_ylabel(r"$f(x)$")
        return save_png_bytes(fig)

    def svg_line():
        x, y = series_sin(1000)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.plot(x, y, color="crimson", lw=1.5)
        ax.set_title("svg_line_n1000")
        return save_svg_bytes(fig)

    def pdf_line():
        x, y = series_sin(1000)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.plot(x, y, color="crimson", lw=1.5)
        ax.set_title("pdf_line_n1000")
        return save_pdf_bytes(fig)

    def anim():
        x, y = series_sin(100)
        total = bytearray()
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        (line,) = ax.plot(x, y, color="crimson", lw=2.0)
        ax.set_ylim(-1.2, 1.2)
        ax.set_title("anim")
        ax.grid(True)
        for i in range(20):
            t = i * 0.15
            line.set_ydata(np.sin(x + t))
            buf = io.BytesIO()
            fig.savefig(buf, format="png", dpi=DPI)
            total.extend(buf.getvalue())
        plt.close(fig)
        return bytes(total)

    def geo():
        cities_lon = [0.0, 116.4, -74.0]
        cities_lat = [51.5, 39.9, 40.7]
        try:
            import cartopy.crs as ccrs
            import cartopy.feature as cfeature

            fig = plt.figure(figsize=FIGSIZE, dpi=DPI)
            ax = fig.add_subplot(1, 1, 1, projection=ccrs.PlateCarree())
            ax.add_feature(
                cfeature.COASTLINE.with_scale("110m"),
                linewidth=0.7,
                edgecolor="#555555",
            )
            ax.scatter(
                cities_lon,
                cities_lat,
                c="crimson",
                s=20,
                transform=ccrs.PlateCarree(),
                zorder=3,
            )
            ax.set_global()
            ax.set_title("geo")
            ax.gridlines(draw_labels=False, linestyle=":", alpha=0.5)
            return save_png_bytes(fig)
        except Exception:
            pass
        if COASTLINE_BIN.is_file():
            lon, lat = load_plotine_coastline_bin(COASTLINE_BIN)
            fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
            ax.plot(lon, lat, color="#555555", linewidth=0.7)
            ax.scatter(cities_lon, cities_lat, c="crimson", s=20, zorder=3)
            ax.set_xlim(-180, 180)
            ax.set_ylim(-90, 90)
            ax.set_title("geo")
            ax.grid(True, linestyle=":", alpha=0.5)
            ax.set_aspect("equal", adjustable="box")
            return save_png_bytes(fig)
        raise RuntimeError("SKIP geo: no cartopy and no coastline.bin")

    return {
        "chrome.empty": empty,
        "series.line_n1000": line(1000),
        "series.line_n10000": line(10000),
        "series.line_n100000": line(100000),
        "series.scatter_n1000": scatter(1000),
        "series.scatter_n10000": scatter(10000),
        "series.bar_n50": bar,
        "series.hist_n1e4": hist,
        "series.area_n1e3": area,
        "series.errorbar_n200": errorbar,
        "series.multiline_5x1e3": multiline,
        "stat.heatmap_64": heatmap(64),
        "stat.heatmap_128": heatmap(128),
        "stat.boxplot": boxplot,
        "stat.violin": violin,
        "stat.hist2d_1e4": hist2d,
        "stat.hexbin_1e4": hexbin,
        "field.contourf_80": contourf,
        "field.pcolormesh_80": pcolormesh,
        "field.quiver_20": quiver,
        "field.streamplot_20": streamplot,
        "field.tripcolor_1e3": tripcolor,
        "field.spy_40": spy,
        "layout.subplots_2x2": subplots,
        "layout.mosaic": mosaic,
        "layout.twin_y": twin_y,
        "layout.inset": inset,
        "layout.secondary_x": secondary_x,
        "scale.loglog": loglog,
        "scale.datetime": datetime_axis,
        "polar.line": polar,
        "anno.annotate_styles": annotate_styles,
        "anno.table": table,
        "prop.pie": pie,
        "prop.stackplot": stackplot,
        "d3.helix": helix,
        "d3.scatter": scatter3d,
        "d3.surface_40": surface,
        "d3.wireframe_40": wireframe,
        "d3.bar": bar3d,
        "math.mathtext": mathtext,
        "fmt.svg_line_n1000": svg_line,
        "fmt.pdf_line_n1000": pdf_line,
        "feat.anim_20f": anim,
        "feat.geo": geo,
        # Tier B stress
        "series.line_n1e6": line(1_000_000),
        "stat.heatmap_512": heatmap(512),
        "field.streamplot_40": _streamplot_n(40),
        "d3.surface_80": _surface_n(80),
        "layout.subplots_4x4": _subplots_nn(4),
        "composite.dashboard": _dashboard,
    }


def _streamplot_n(n: int):
    def go():
        u, v = vortex_uv(n)
        y = np.arange(n)
        x = np.arange(n)
        fig, ax = plt.subplots(figsize=FIGSIZE, dpi=DPI)
        ax.streamplot(x, y, u, v, color="crimson", density=1.0, linewidth=0.8)
        ax.set_title(f"streamplot_{n}")
        return save_png_bytes(fig)

    return go


def _surface_n(n: int):
    def go():
        from mpl_toolkits.mplot3d import Axes3D  # noqa: F401

        xs = np.linspace(-2, 2, n)
        ys = np.linspace(-2, 2, n)
        xx, yy = np.meshgrid(xs, ys)
        zz = np.exp(-0.5 * (xx * xx + yy * yy))
        fig = plt.figure(figsize=FIGSIZE, dpi=DPI)
        ax = fig.add_subplot(111, projection="3d")
        ax.plot_surface(xx, yy, zz, cmap="viridis", alpha=0.9, linewidth=0)
        ax.view_init(elev=35, azim=-50)
        ax.set_title(f"surface_{n}")
        return save_png_bytes(fig)

    return go


def _subplots_nn(n: int):
    def go():
        x, y = series_sin(200)
        fig, axs = plt.subplots(n, n, figsize=FIGSIZE, dpi=DPI)
        for i, ax in enumerate(np.asarray(axs).ravel()):
            ax.plot(x, y, color="steelblue", lw=1.0)
            ax.set_title(f"{i // n}{i % n}", fontsize=8)
        fig.tight_layout()
        return save_png_bytes(fig)

    return go


def _dashboard():
    x, left = series_sin(400)
    right = x * x * 0.02 + 0.5
    z = grid_gauss(48)
    fig, (ax0, ax1) = plt.subplots(1, 2, figsize=FIGSIZE, dpi=DPI)
    ax0.plot(x, left, color="steelblue", lw=1.5, label="amp")
    ax0b = ax0.twinx()
    ax0b.plot(x, right, color="crimson", lw=1.2, label="energy")
    ax0.set_title("twin")
    ax0.grid(True)
    ax0.legend(loc="upper left")
    im = ax1.imshow(z, cmap="viridis", origin="upper", aspect="auto")
    fig.colorbar(im, ax=ax1)
    ax1.set_title("heat")
    fig.tight_layout()
    return save_png_bytes(fig)


def run_plotine(filter_substr: str, tier: str = "default") -> list[dict]:
    env = os.environ.copy()
    env["BENCH_TIER"] = tier or "default"
    if filter_substr:
        env["BENCH_FILTER"] = filter_substr
    cmd = [
        "cargo",
        "--quiet",
        "run",
        "-p",
        "plotine",
        "--example",
        "bench_suite",
        "--release",
    ]
    print("+", " ".join(cmd), flush=True)
    r = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    sys.stdout.write(r.stdout or "")
    if r.returncode != 0:
        sys.stderr.write(r.stderr or "")
        raise SystemExit(f"bench_suite failed ({r.returncode})")
    rows = []
    for line in (r.stdout or "").splitlines():
        m = BENCH_RE.match(line.strip())
        if m:
            rows.append(
                {
                    "impl": "plotine",
                    "name": m.group("name"),
                    "median_ms": float(m.group("median")),
                    "p95_ms": float(m.group("p95")),
                    "bytes": int(m.group("bytes")),
                    "fmt": m.group("fmt"),
                }
            )
    return rows


def _fmt_for(name: str) -> str:
    if name.startswith("fmt.svg"):
        return "svg"
    if name.startswith("fmt.pdf"):
        return "pdf"
    if "anim" in name:
        return "png_seq"
    return "png"


def run_matplotlib(filter_substr: str, tier: str = "default") -> list[dict]:
    print("\n=== matplotlib counterparts ===", flush=True)
    save = bool(os.environ.get("BENCH_SAVE"))
    if save:
        OUT.mkdir(parents=True, exist_ok=True)
    rows = []
    for name, fn in mpl_cases().items():
        if not tier_allows(tier, name):
            continue
        if filter_substr and filter_substr not in name:
            continue
        try:
            median, p95, nbytes = time_mpl(fn)
        except Exception as exc:  # noqa: BLE001
            print(f"SKIP name={name} reason={exc}", flush=True)
            continue
        fmt = _fmt_for(name)
        print(
            f"BENCH name={name} median_ms={median:.3f} p95_ms={p95:.3f} "
            f"bytes={nbytes} fmt={fmt} impl=matplotlib",
            flush=True,
        )
        if save and fmt in ("png", "svg", "pdf"):
            try:
                (OUT / f"mpl_{name}.{fmt}").write_bytes(fn())
            except Exception as exc:  # noqa: BLE001
                print(f"SAVE_SKIP name={name} reason={exc}", flush=True)
        rows.append(
            {
                "impl": "matplotlib",
                "name": name,
                "median_ms": median,
                "p95_ms": p95,
                "bytes": nbytes,
                "fmt": fmt,
            }
        )
    return rows


def write_csv(plotine: list[dict], mpl: list[dict]) -> Path:
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / "results.csv"
    by_p = {r["name"]: r for r in plotine}
    by_m = {r["name"]: r for r in mpl}
    names = sorted(set(by_p) | set(by_m))
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "name",
                "plotine_median_ms",
                "plotine_p95_ms",
                "plotine_bytes",
                "mpl_median_ms",
                "mpl_p95_ms",
                "mpl_bytes",
                "speedup_mpl_over_plotine",
                "bytes_ratio_plotine_over_mpl",
            ]
        )
        for name in names:
            p, m = by_p.get(name), by_m.get(name)
            pm = p["median_ms"] if p else ""
            pp = p["p95_ms"] if p else ""
            pb = p["bytes"] if p else ""
            mm = m["median_ms"] if m else ""
            mp = m["p95_ms"] if m else ""
            mb = m["bytes"] if m else ""
            speed = (m["median_ms"] / p["median_ms"]) if p and m and p["median_ms"] else ""
            br = (p["bytes"] / m["bytes"]) if p and m and m["bytes"] else ""
            w.writerow([name, pm, pp, pb, mm, mp, mb, speed, br])
    return path


def print_table(plotine: list[dict], mpl: list[dict]) -> None:
    by_p = {r["name"]: r for r in plotine}
    by_m = {r["name"]: r for r in mpl}
    names = sorted(set(by_p) | set(by_m))
    print(
        f"\n{'name':28} {'plotine ms':>10} {'mpl ms':>10} {'speedup':>8} {'bytes_x':>8}"
    )
    print("-" * 70)
    for name in names:
        p, m = by_p.get(name), by_m.get(name)
        ps = f"{p['median_ms']:10.1f}" if p else f"{'n/a':>10}"
        ms = f"{m['median_ms']:10.1f}" if m else f"{'n/a':>10}"
        if p and m and p["median_ms"] > 0:
            sp = f"{m['median_ms'] / p['median_ms']:8.2f}x"
        else:
            sp = f"{'n/a':>8}"
        if p and m and m["bytes"] > 0:
            br = f"{p['bytes'] / m['bytes']:8.2f}x"
        else:
            br = f"{'n/a':>8}"
        print(f"{name:28} {ps} {ms} {sp} {br}")
    print("\nspeedup = mpl_median / plotine_median (>1 => plotine faster)")
    print("bytes_x = plotine_bytes / mpl_bytes")
    print("NOTE: feat.anim_20f bytes = sum of 20 PNG frames on both sides.")
    print("NOTE: feat.geo uses cartopy 110m or plotine coastline.bin when available.")


def write_markdown(plotine: list[dict], mpl: list[dict]) -> Path:
    """Compact markdown table for CI artifacts / release notes."""
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / "results.md"
    by_p = {r["name"]: r for r in plotine}
    by_m = {r["name"]: r for r in mpl}
    names = sorted(set(by_p) | set(by_m))
    lines = [
        "# plotine benchmark",
        "",
        "| name | plotine ms | mpl ms | speedup |",
        "|---|---:|---:|---:|",
    ]
    for name in names:
        p, m = by_p.get(name), by_m.get(name)
        ps = f"{p['median_ms']:.1f}" if p else "n/a"
        ms = f"{m['median_ms']:.1f}" if m else "n/a"
        if p and m and p["median_ms"] > 0:
            sp = f"{m['median_ms'] / p['median_ms']:.2f}x"
        else:
            sp = "n/a"
        lines.append(f"| `{name}` | {ps} | {ms} | {sp} |")
    lines.append("")
    lines.append("speedup = mpl_median / plotine_median (>1 => plotine faster)")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--filter", default="", help="substring filter for scenario names")
    ap.add_argument(
        "--tier",
        default="default",
        choices=["smoke", "default", "stress", "all"],
        help="smoke=Tier S; default=A (no stress); stress/all=include Tier B",
    )
    ap.add_argument("--plotine-only", action="store_true")
    ap.add_argument("--mpl-only", action="store_true")
    args = ap.parse_args()

    plotine: list[dict] = []
    mpl: list[dict] = []
    if not args.mpl_only:
        plotine = run_plotine(args.filter, args.tier)
    if not args.plotine_only:
        mpl = run_matplotlib(args.filter, args.tier)
    print_table(plotine, mpl)
    csv_path = write_csv(plotine, mpl)
    md_path = write_markdown(plotine, mpl)
    print(f"\nWrote {csv_path}")
    print(f"Wrote {md_path}")


if __name__ == "__main__":
    main()
