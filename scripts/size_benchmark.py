#!/usr/bin/env python3
"""Size + render-time bench: plotine (M9–M13 features) vs matplotlib.

Runs:
  cargo run -p plotine --example size_benchmark --release --features gif,latex
  cargo run -p plotine-pyplot --example pyplot_size_benchmark --release

Then emits matplotlib counterparts and a comparison table.

For the product L1 suite (warmup + median / p95), prefer:
  python scripts/benchmark.py

Usage (repo root):
  python scripts/size_benchmark.py
"""

from __future__ import annotations

import re
import subprocess
import sys
import time
from pathlib import Path

import numpy as np

try:
    import matplotlib.pyplot as plt
except ImportError as exc:
    raise SystemExit("pip install matplotlib numpy pillow") from exc

try:
    from PIL import Image
except ImportError as exc:
    raise SystemExit("pip install pillow") from exc

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "compare" / "size_bench"
OUT.mkdir(parents=True, exist_ok=True)

FIGSIZE = (5.0, 3.5)
FEATURE_DPI = 150
TIMING_RE = re.compile(
    r"^TIMING name=(?P<name>\S+) ms=(?P<ms>[0-9.]+) bytes=(?P<bytes>\d+) path=(?P<path>.+)$"
)


def run_cargo(args: list[str]) -> list[dict]:
    print("+", " ".join(args), flush=True)
    r = subprocess.run(
        args,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    # Cargo may put compiler chatter on stderr; examples print TIMING on stdout.
    combined = (r.stdout or "") + (r.stderr or "")
    sys.stdout.write(r.stdout or "")
    if r.stderr:
        # Keep noise visible but do not fail the harness.
        for line in r.stderr.splitlines():
            if line.strip().startswith("TIMING ") or "error" in line.lower():
                print(line, flush=True)
    if r.returncode != 0:
        sys.stderr.write(r.stderr or "")
        print(f"warning: command failed ({r.returncode})", flush=True)
        return []
    rows = []
    for line in combined.splitlines():
        m = TIMING_RE.match(line.strip())
        if m:
            rows.append(
                {
                    "name": m.group("name"),
                    "ms": float(m.group("ms")),
                    "bytes": int(m.group("bytes")),
                    "path": m.group("path").strip(),
                }
            )
    if not rows:
        print("warning: no TIMING lines parsed from cargo output", flush=True)
    return rows


def mpl_line(dpi: int, tight: bool) -> Path:
    x = np.arange(0, 10, 0.1)
    y = np.sin(x)
    fig, ax = plt.subplots(figsize=FIGSIZE)
    ax.plot(x, y, color="crimson", linewidth=2, label="sin(x)")
    ax.set_title("Line")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.legend(loc="upper right")
    tag = "tight" if tight else "full"
    path = OUT / f"mpl_line_{dpi}_{tag}.png"
    kw: dict = {"dpi": dpi}
    if tight:
        kw["bbox_inches"] = "tight"
    fig.savefig(path, **kw)
    plt.close(fig)
    return path


def mpl_static_render() -> tuple[Path, float]:
    """Counterpart to plotine `static_render` (pixel path only; not GUI toolbar)."""
    x = np.arange(0, 10, 0.1)
    y = np.sin(x)
    t0 = time.perf_counter()
    fig, ax = plt.subplots(figsize=FIGSIZE)
    ax.plot(x, y, color="steelblue", linewidth=2)
    ax.set_title("static render (show pixel path)")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.grid(True, alpha=0.3)
    path = OUT / "mpl_static_render_150.png"
    fig.savefig(path, dpi=FEATURE_DPI)
    plt.close(fig)
    return path, (time.perf_counter() - t0) * 1000.0


def mpl_anim(n_frames: int = 20) -> tuple[Path, Path, float, int]:
    """Save frame_0000 + GIF at the same figsize/DPI/fps as plotine (20 fps, 50 ms)."""
    from matplotlib.animation import FuncAnimation, PillowWriter

    x = np.arange(0, 10, 0.1)
    t0 = time.perf_counter()
    fig, ax = plt.subplots(figsize=FIGSIZE)
    (line,) = ax.plot(x, np.sin(x), color="crimson", linewidth=2)
    ax.set_title("Animation")
    ax.set_ylim(-1.2, 1.2)
    ax.grid(True, alpha=0.3)

    def update(i: int):
        t = i * 0.15
        line.set_ydata(np.sin(x + t))
        return (line,)

    anim = FuncAnimation(fig, update, frames=n_frames, interval=50, blit=True)
    frame_dir = OUT / "mpl_anim_frames"
    frame_dir.mkdir(parents=True, exist_ok=True)
    for p in frame_dir.glob("frame_*.png"):
        p.unlink()
    total = 0
    for i in range(n_frames):
        update(i)
        fp = frame_dir / f"frame_{i:04d}.png"
        fig.savefig(fp, dpi=FEATURE_DPI)
        total += fp.stat().st_size
    frame0 = OUT / "mpl_anim_frame_150.png"
    frame0.write_bytes((frame_dir / "frame_0000.png").read_bytes())

    gif_path = OUT / "mpl_anim_20f_150.gif"
    try:
        # Match plotine: FIGSIZE @ FEATURE_DPI, 20 fps (interval_ms=50).
        anim.save(
            str(gif_path),
            writer=PillowWriter(fps=20),
            dpi=FEATURE_DPI,
        )
    except Exception as exc:  # noqa: BLE001
        print(f"warning: mpl GIF failed: {exc}", flush=True)
        gif_path = Path()
    plt.close(fig)
    ms = (time.perf_counter() - t0) * 1000.0
    return frame0, gif_path, ms, total


COASTLINE_BIN = (
    ROOT / "crates" / "plotine" / "src" / "geo" / "data" / "coastline.bin"
)


def load_plotine_coastline_bin(path: Path) -> tuple[np.ndarray, np.ndarray]:
    """Decode plotine coastline.bin (same NE 110m data) for a fair mpl overlay."""
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


def mpl_geo() -> tuple[Path, float, str]:
    """Fair geo counterpart: cartopy 110m if available, else same coastline.bin as plotine.

    Returns (path, ms, mode) where mode is 'cartopy' | 'coastline.bin' | 'skip'.
    """
    path = OUT / "mpl_geo_150.png"
    cities_lon = [0.0, 116.4, -74.0]
    cities_lat = [51.5, 39.9, 40.7]
    t0 = time.perf_counter()

    # Prefer cartopy Natural Earth 110m (true mpl+cartopy path).
    try:
        import cartopy.crs as ccrs
        import cartopy.feature as cfeature

        fig = plt.figure(figsize=FIGSIZE)
        ax = fig.add_subplot(1, 1, 1, projection=ccrs.PlateCarree())
        ax.add_feature(cfeature.COASTLINE.with_scale("110m"), linewidth=0.7, edgecolor="#555555")
        ax.scatter(
            cities_lon,
            cities_lat,
            c="crimson",
            s=30,
            transform=ccrs.PlateCarree(),
            zorder=3,
        )
        ax.set_global()
        ax.set_title("Geo PlateCarree")
        ax.set_xlabel("longitude (°)")
        ax.set_ylabel("latitude (°)")
        ax.gridlines(draw_labels=False, linestyle=":", alpha=0.5)
        fig.savefig(path, dpi=FEATURE_DPI)
        plt.close(fig)
        return path, (time.perf_counter() - t0) * 1000.0, "cartopy"
    except Exception as exc:  # noqa: BLE001
        print(f"note: cartopy unavailable ({exc}); trying coastline.bin", flush=True)

    # Fair fallback: plot the identical embedded NE polyline plotine uses.
    if COASTLINE_BIN.is_file():
        lon, lat = load_plotine_coastline_bin(COASTLINE_BIN)
        fig, ax = plt.subplots(figsize=FIGSIZE)
        # NaN breaks → separate segments
        ax.plot(lon, lat, color="#555555", linewidth=0.7, solid_capstyle="round")
        ax.scatter(cities_lon, cities_lat, c="crimson", s=30, zorder=3)
        ax.set_xlim(-180, 180)
        ax.set_ylim(-90, 90)
        ax.set_xlabel("longitude (°)")
        ax.set_ylabel("latitude (°)")
        ax.set_title("Geo PlateCarree")
        ax.grid(True, linestyle=":", alpha=0.5)
        ax.set_aspect("equal", adjustable="box")
        fig.savefig(path, dpi=FEATURE_DPI)
        plt.close(fig)
        return path, (time.perf_counter() - t0) * 1000.0, "coastline.bin"

    print("SKIP mpl geo: no cartopy and no coastline.bin — not comparing", flush=True)
    if path.exists():
        path.unlink()
    return path, (time.perf_counter() - t0) * 1000.0, "skip"


def mpl_mathtext() -> tuple[Path, float]:
    t0 = time.perf_counter()
    x = np.arange(0, 10, 0.1)
    fig, ax = plt.subplots(figsize=FIGSIZE)
    ax.plot(x, np.sin(x), color="steelblue", linewidth=2)
    ax.set_title(r"mathtext $\int_0^1 x^2\,dx$")
    ax.set_xlabel(r"$x$")
    ax.set_ylabel(r"$y$")
    path = OUT / "mpl_mathtext_150.png"
    fig.savefig(path, dpi=FEATURE_DPI)
    plt.close(fig)
    return path, (time.perf_counter() - t0) * 1000.0


def mpl_usetex() -> tuple[Path | None, float | None]:
    """Only if TeX is available (matplotlib usetex)."""
    try:
        plt.rcParams["text.usetex"] = True
        t0 = time.perf_counter()
        x = np.arange(0, 10, 0.1)
        fig, ax = plt.subplots(figsize=FIGSIZE)
        ax.plot(x, np.sin(x), color="steelblue", linewidth=2)
        ax.set_title(r"usetex $\int_0^1 x^2\,dx$")
        ax.set_xlabel(r"$x$")
        ax.set_ylabel(r"$y$")
        path = OUT / "mpl_usetex_150.png"
        fig.savefig(path, dpi=FEATURE_DPI)
        plt.close(fig)
        ms = (time.perf_counter() - t0) * 1000.0
        return path, ms
    except Exception as exc:  # noqa: BLE001
        print(f"SKIP mpl usetex: {exc}", flush=True)
        return None, None
    finally:
        plt.rcParams["text.usetex"] = False


def report_file(engine: str, name: str, path: Path, ms: float | None = None) -> None:
    if not path.exists():
        print(f"{engine:12} {name:22}  MISSING {path}")
        return
    # GIF / multi-frame: report file size without forcing PIL decode dims if needed
    try:
        im = Image.open(path)
        w, h = im.size
        dim = f"{w:>4}x{h:<5}"
    except Exception:  # noqa: BLE001
        dim = f"{'n/a':>10}"
    kb = path.stat().st_size / 1024
    ms_s = f"{ms:8.1f}" if ms is not None else f"{'n/a':>8}"
    print(f"{engine:12} {name:22} {dim} {kb:8.1f}KB  {ms_s} ms")


def pair_ratio(label: str, a: Path, b: Path) -> None:
    if not a.exists() or not b.exists():
        return
    ak, bk = a.stat().st_size / 1024, b.stat().st_size / 1024
    ia, ib = Image.open(a), Image.open(b)
    print(
        f"{label:28} plotine {ia.size[0]}x{ia.size[1]} {ak:7.1f}KB  |  "
        f"mpl {ib.size[0]}x{ib.size[1]} {bk:7.1f}KB  |  "
        f"bytes {ak/bk:.2f}x"
    )


def main() -> None:
    timings = run_cargo(
        [
            "cargo",
            "--quiet",
            "run",
            "-p",
            "plotine",
            "--example",
            "size_benchmark",
            "--release",
            "--features",
            "gif,latex",
        ]
    )
    timings += run_cargo(
        [
            "cargo",
            "--quiet",
            "run",
            "-p",
            "plotine-pyplot",
            "--example",
            "pyplot_size_benchmark",
            "--release",
        ]
    )
    by_name = {t["name"]: t for t in timings}

    # Fallback: discover plotine artifacts on disk if TIMING parse missed them.
    # Avoid double-counting the representative anim frame when TIMING used anim_20f_*.
    skip_fallback = set()
    if "anim_20f_150" in by_name:
        skip_fallback.add("anim_frame_150")

    for path in sorted(OUT.glob("plotine_*.*")):
        if path.suffix.lower() not in {".png", ".gif"}:
            continue
        stem = path.stem  # plotine_line_150
        name = stem.removeprefix("plotine_")
        if name in skip_fallback:
            continue
        if name not in by_name:
            by_name[name] = {
                "name": name,
                "ms": None,
                "bytes": path.stat().st_size,
                "path": str(path),
            }
    pyplot_path = OUT / "pyplot_line_150.png"
    if pyplot_path.exists() and "pyplot_line_150" not in by_name:
        by_name["pyplot_line_150"] = {
            "name": "pyplot_line_150",
            "ms": None,
            "bytes": pyplot_path.stat().st_size,
            "path": str(pyplot_path),
        }

    print("\n=== matplotlib counterparts ===", flush=True)
    mpl_timings: dict[str, float] = {}
    for dpi in (100, 150, 200, 300):
        t0 = time.perf_counter()
        mpl_line(dpi, tight=False)
        mpl_timings[f"line_{dpi}"] = (time.perf_counter() - t0) * 1000.0
        mpl_line(dpi, tight=True)

    p, ms = mpl_static_render()
    mpl_timings["static_render_150"] = ms
    _frame, gif, ms, _total = mpl_anim(20)
    mpl_timings["anim_20f_150"] = ms
    _geo_path, ms, geo_mode = mpl_geo()
    if geo_mode != "skip":
        mpl_timings["geo_150"] = ms
    print(f"mpl geo mode: {geo_mode}", flush=True)
    p, ms = mpl_mathtext()
    mpl_timings["mathtext_150"] = ms
    up, ums = mpl_usetex()
    if up is not None and ums is not None:
        mpl_timings["usetex_150"] = ums

    print(f"\n{'engine':12} {'name':22} {'WxH':>12} {'size':>10} {'time':>10}")
    print("-" * 72)

    # plotine / pyplot rows
    for name in sorted(by_name):
        t = by_name[name]
        path = Path(t["path"])
        if not path.is_absolute():
            path = ROOT / path
        report_file("plotine", name, path, t.get("ms"))

    # matplotlib rows for feature set
    for name, path in [
        ("static_render_150", OUT / "mpl_static_render_150.png"),
        ("anim_frame_150", OUT / "mpl_anim_frame_150.png"),
        ("anim_gif_20f_150", OUT / "mpl_anim_20f_150.gif"),
        ("geo_150", OUT / "mpl_geo_150.png"),
        ("mathtext_150", OUT / "mpl_mathtext_150.png"),
        ("usetex_150", OUT / "mpl_usetex_150.png"),
        ("line_150", OUT / "mpl_line_150_full.png"),
    ]:
        key = name.replace("anim_frame", "anim_20f") if name.startswith("anim_frame") else name
        report_file("matplotlib", name, path, mpl_timings.get(key))

    print("\n--- head-to-head (feature @150 DPI) ---")
    print(
        "NOTE M9: static_render = pixel path only; toolbar UX → docs/GUI_TOOLBAR.md",
        flush=True,
    )
    pairs = [
        ("line", OUT / "plotine_line_150.png", OUT / "mpl_line_150_full.png"),
        (
            "static_render (M9 pixels)",
            OUT / "plotine_static_render_150.png",
            OUT / "mpl_static_render_150.png",
        ),
        ("anim frame (M10)", OUT / "plotine_anim_frame_150.png", OUT / "mpl_anim_frame_150.png"),
        ("anim GIF (M10 fair dpi)", OUT / "plotine_anim_20f_150.gif", OUT / "mpl_anim_20f_150.gif"),
        ("pyplot vs builder (M12)", OUT / "pyplot_line_150.png", OUT / "plotine_line_150.png"),
        ("mathtext (M13 default)", OUT / "plotine_mathtext_150.png", OUT / "mpl_mathtext_150.png"),
        ("usetex (M13 opt)", OUT / "plotine_usetex_150.png", OUT / "mpl_usetex_150.png"),
    ]
    if geo_mode != "skip":
        pairs.insert(
            4,
            (f"geo (M11 via {geo_mode})", OUT / "plotine_geo_150.png", OUT / "mpl_geo_150.png"),
        )
    else:
        print("geo (M11): SKIP unfair compare (no cartopy / coastline.bin)", flush=True)

    for label, a, b in pairs:
        # M12: both plotine; label still useful
        if "pyplot" in label:
            if a.exists() and b.exists():
                ak, bk = a.stat().st_size / 1024, b.stat().st_size / 1024
                print(
                    f"{label:28} pyplot {ak:7.1f}KB  |  builder {bk:7.1f}KB  |  "
                    f"bytes {ak/bk:.3f}x (expect ~1.0)"
                )
            continue
        pair_ratio(label, a, b)

    print("\n--- line DPI ladder (no tight) ---")
    for dpi in (100, 150, 200, 300):
        pair_ratio(
            f"line@{dpi}",
            OUT / f"plotine_line_{dpi}.png",
            OUT / f"mpl_line_{dpi}_full.png",
        )

    print("\nDone. Artifacts under", OUT)


if __name__ == "__main__":
    main()
