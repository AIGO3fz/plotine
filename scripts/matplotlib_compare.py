#!/usr/bin/env python3
"""Generate the matplotlib half of the plotine comparison set.

Requires: matplotlib, numpy

    cargo run -p plotine --example matplotlib_compare
    python scripts/matplotlib_compare.py

Outputs: ./compare/mpl_*.png next to ./compare/plotine_*.png

Saves the full figure canvas (no ``bbox_inches='tight'``) so pixel size and
on-page scaling match plotine’s declared ``figsize × dpi``.
"""

from __future__ import annotations

from pathlib import Path

import numpy as np

try:
    import matplotlib.pyplot as plt
    from matplotlib import dates as mdates
    from matplotlib.colors import LogNorm, Normalize
except ImportError as exc:  # pragma: no cover
    raise SystemExit("matplotlib is required: pip install matplotlib numpy") from exc

OUT = Path("compare")
OUT.mkdir(parents=True, exist_ok=True)


def save(fig, name: str) -> None:
    path = OUT / name
    # Full canvas (no tight crop) so side-by-side CSS scaling matches plotine,
    # which always exports the declared figsize × dpi rectangle.
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print(f"wrote {path}")


COASTLINE_BIN = (
    Path(__file__).resolve().parents[1]
    / "crates"
    / "plotine"
    / "src"
    / "geo"
    / "data"
    / "coastline.bin"
)


def load_coastline_bin(path: Path) -> tuple[np.ndarray, np.ndarray]:
    import struct

    data = path.read_bytes()
    magic, version, n = struct.unpack_from("<III", data, 0)
    if magic != 0x50474C43 or version != 1:
        raise ValueError(f"bad coastline.bin magic={magic:#x} ver={version}")
    lon = np.empty(n, dtype=np.float64)
    lat = np.empty(n, dtype=np.float64)
    off = 12
    for i in range(n):
        lo, la = struct.unpack_from("<ff", data, off)
        lon[i] = lo
        lat[i] = la
        off += 8
    return lon, lat


def style_like_plotine(ax) -> None:
    """Match plotine light-theme chrome for feature pixel-align pairs."""
    ax.set_facecolor("#fafbfc")  # Color::AXES_FACE
    ax.figure.patch.set_facecolor("#ffffff")
    for spine in ax.spines.values():
        spine.set_color("#495057")  # Color::SPINE
        spine.set_linewidth(0.8)
    ax.tick_params(colors="#495057", which="both")
    ax.xaxis.label.set_color("#000000")
    ax.yaxis.label.set_color("#000000")
    ax.title.set_color("#000000")
    # plotine grid ≈ Color::GRID #dee2e6
    ax.grid(True, color="#dee2e6", linewidth=0.8, alpha=1.0)


def write_mpl_m11_geo() -> None:
    """Matplotlib half of M11 geo — prefer cartopy 110m, else plotine coastline.bin."""
    cities_lon = [0.0, 116.4, -74.0]
    cities_lat = [51.5, 39.9, 40.7]
    try:
        import cartopy.crs as ccrs
        import cartopy.feature as cfeature

        # 7×3.5 ≈ plotine: world 2:1 fits stock Axes without heavy letterbox.
        fig = plt.figure(figsize=(7.0, 3.5))
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
            s=20.25,  # ≈ plotine marker size 4.5px radius → area ~πr²
            transform=ccrs.PlateCarree(),
            zorder=3,
        )
        ax.set_global()
        ax.set_title("M11 Geo PlateCarree")
        ax.set_xlabel("longitude (°)")
        ax.set_ylabel("latitude (°)")
        ax.gridlines(draw_labels=False, linestyle="-", alpha=0.3)
        save(fig, "mpl_m11_geo.png")
        return
    except Exception as exc:  # noqa: BLE001
        print(f"m11 geo: cartopy unavailable ({exc}); using coastline.bin")

    if not COASTLINE_BIN.is_file():
        print("m11 geo: SKIP — no cartopy and no coastline.bin")
        return
    lon, lat = load_coastline_bin(COASTLINE_BIN)
    # Match plotine M11: equal aspect (PlateCarree °) + figsize whose stock
    # box is ~2:1 so the map is not vertically letterboxed.
    fig, ax = plt.subplots(figsize=(7.0, 3.5))
    ax.plot(lon, lat, color="#555555", linewidth=0.7, solid_capstyle="round")
    ax.scatter(cities_lon, cities_lat, c="#dc143c", s=18, zorder=3, linewidths=0)
    ax.set_xlim(-180, 180)
    ax.set_ylim(-90, 90)
    ax.set_aspect("equal", adjustable="box")
    ax.set_xticks([-150, -100, -50, 0, 50, 100, 150])
    ax.set_yticks([-80, -60, -40, -20, 0, 20, 40, 60, 80])
    ax.set_xlabel("longitude (°)")
    ax.set_ylabel("latitude (°)")
    ax.set_title("M11 Geo PlateCarree")
    style_like_plotine(ax)
    save(fig, "mpl_m11_geo.png")


def main() -> None:
    x = np.arange(0, 10, 0.1)
    y = np.sin(x)

    # line
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.plot(x, y, color="crimson", linewidth=2, label="sin(x)")
    ax.set_title("Line")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_line.png")

    # scatter
    fig, ax = plt.subplots(figsize=(5, 3.5))
    # s=12 ↔ plotine diameter 2·√(s/π) ≈ 3.91 pt
    ax.scatter(x, y, s=12, color="steelblue", label="samples", linewidths=0)
    ax.set_title("Scatter")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_scatter.png")

    # bar
    fig, ax = plt.subplots(figsize=(5, 3.5))
    cats = [1, 2, 3, 4]
    heights = [3, 7, 2, 5]
    ax.bar(cats, heights, color="steelblue", label="counts")
    ax.set_title("Bar")
    ax.set_xlabel("category")
    ax.set_ylabel("value")
    ax.legend(loc="upper right")
    ax.grid(True, axis="y", alpha=0.3)
    save(fig, "mpl_bar.png")

    # hist
    t = np.arange(200) / 40.0
    hist_data = np.sin(t * 0.7) + 0.15 * ((np.arange(200) % 17) / 17.0)
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.hist(hist_data, bins=12, color="forestgreen", label="n")
    ax.set_title("Histogram")
    ax.set_xlabel("value")
    ax.set_ylabel("count")
    ax.legend(loc="upper right")
    ax.grid(True, axis="y", alpha=0.3)
    save(fig, "mpl_hist.png")

    # area
    area_y = np.abs(np.sin(x * 0.8)) + 0.2
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.fill_between(x, area_y, alpha=0.45, color="steelblue", label="area")
    ax.set_title("Area")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_area.png")

    # errorbar
    ex = np.array([0.0, 1.0, 2.0, 3.0, 4.0])
    ey = np.array([1.0, 1.5, 1.2, 2.0, 1.8])
    ee = np.array([0.2, 0.25, 0.15, 0.3, 0.2])
    exerr = np.array([0.12, 0.1, 0.15, 0.1, 0.12])
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.errorbar(
        ex, ey, yerr=ee, xerr=exerr, fmt="o-", color="steelblue", capsize=4, label="data"
    )
    ax.set_title("Errorbar")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.legend(loc="upper left")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_errorbar.png")

    # loglog
    x_log = 10 ** np.linspace(-1.0, 2.2, 40)
    y_log = 2.0 * x_log**1.5
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.loglog(x_log, y_log, color="crimson", linewidth=2)
    ax.set_title("Log-log")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.grid(True, which="both", alpha=0.3)
    save(fig, "mpl_loglog.png")

    # dark + symlog
    sx = np.arange(-40, 41) * 0.25
    sy = sx + 0.3 * np.sin(sx)
    fig, ax = plt.subplots(figsize=(5, 3.5), facecolor="#1a1d21")
    ax.set_facecolor("#212529")
    ax.plot(sx, sy, color="crimson", linewidth=2)
    ax.set_yscale("symlog")
    ax.set_title("Dark + Symlog", color="#f8f9fa")
    ax.set_xlabel("x", color="#dee2e6")
    ax.set_ylabel("y", color="#dee2e6")
    ax.tick_params(colors="#adb5bd")
    for spine in ax.spines.values():
        spine.set_color("#adb5bd")
    ax.grid(True, alpha=0.25, color="#495057")
    save(fig, "mpl_dark_symlog.png")

    # paper-ish theme
    fig, ax = plt.subplots(figsize=(5, 3.5), facecolor="#f7f3ea")
    ax.set_facecolor("#fffdf8")
    ax.plot(x, y, color="crimson", linewidth=2, label="sin")
    ax.set_title("Paper Theme")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.35)
    save(fig, "mpl_paper.png")

    # subplots
    fig, axes = plt.subplots(2, 2, figsize=(7, 5))
    axes[0, 0].plot(x, y, color="crimson", linewidth=1.5)
    axes[0, 0].set_title("A: line")
    axes[0, 1].scatter(x, y, s=8, color="steelblue")
    axes[0, 1].set_title("B: scatter")
    axes[1, 0].bar([1, 2, 3], [2, 4, 3], color="forestgreen")
    axes[1, 0].set_title("C: bar")
    axes[1, 1].hist(hist_data, bins=10, color="mediumpurple")
    axes[1, 1].set_title("D: hist")
    fig.tight_layout()
    save(fig, "mpl_subplots.png")

    # datetime (ConciseDateFormatter — matches plotine)
    start = np.datetime64("2020-01-01")
    dx = start + np.arange(12)
    dy = np.sin(np.arange(12) * 0.5) + 1.0
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.plot(dx, dy, color="steelblue", linewidth=2)
    ax.set_title("Datetime")
    ax.set_xlabel("date")
    ax.set_ylabel("value")
    _loc = mdates.AutoDateLocator()
    ax.xaxis.set_major_locator(_loc)
    ax.xaxis.set_major_formatter(mdates.ConciseDateFormatter(_loc))
    fig.autofmt_xdate()
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_datetime.png")

    # heatmap
    rr, cc = np.mgrid[0:8, 0:8]
    values = np.sin(rr * 0.6) + np.cos(cc * 0.7)
    fig, ax = plt.subplots(figsize=(5, 4))
    im = ax.imshow(values, cmap="viridis", origin="upper", aspect="auto")
    ax.set_title("Heatmap")
    fig.colorbar(im, ax=ax)
    save(fig, "mpl_heatmap.png")

    # boxplot
    a = [1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 7.0]
    b = [2.0, 2.5, 3.0, 3.2, 3.8, 4.5]
    c = [0.5, 1.0, 1.5, 2.0, 2.2, 2.8, 3.0]
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.boxplot([a, b, c], patch_artist=True,
               boxprops=dict(facecolor="steelblue", alpha=0.7))
    ax.set_title("Boxplot")
    ax.grid(True, axis="y", alpha=0.3)
    save(fig, "mpl_boxplot.png")

    # violin (stock mpl: Scott KDE, extrema on, medians off, width=0.5)
    fig, ax = plt.subplots(figsize=(5, 3.5))
    parts = ax.violinplot(
        [a, b, c],
        widths=0.5,
        showmeans=False,
        showmedians=False,
        showextrema=True,
    )
    for body in parts["bodies"]:
        body.set_facecolor("mediumpurple")
        body.set_alpha(0.55)
    ax.set_title("Violin")
    ax.grid(True, axis="y", alpha=0.3)
    save(fig, "mpl_violin.png")

    # ========== M5 / M6 additions ==========

    # fill_between
    y1 = np.sin(x)
    y2 = 0.5 * np.cos(x)
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.fill_between(x, y1, y2, color="steelblue", alpha=0.4, label="band")
    ax.plot(x, y1, color="crimson", linewidth=1.5, label="y1")
    ax.plot(x, y2, color="forestgreen", linewidth=1.5, label="y2")
    ax.set_title("Fill Between")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_fill_between.png")

    # step
    stx = np.array([0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
    sty = np.array([1.0, 2.0, 1.5, 3.0, 2.2, 2.8])
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.step(stx, sty, where="mid", color="steelblue", linewidth=2, label="mid")
    ax.set_title("Step")
    ax.legend(loc="upper left")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_step.png")

    # pie
    fig, ax = plt.subplots(figsize=(5, 4))
    ax.pie(
        [35, 25, 20, 20],
        labels=["A", "B", "C", "D"],
        autopct=None,
        startangle=90,
        counterclock=False,
    )
    ax.set_title("Pie")
    ax.legend(loc="upper right")
    save(fig, "mpl_pie.png")

    # stackplot
    sx_stack = np.arange(40) * 0.25
    s0 = 1.0 + 0.3 * np.sin(sx_stack)
    s1 = 1.5 + 0.2 * np.cos(sx_stack * 0.7)
    s2 = 0.8 + 0.15 * np.sin(sx_stack * 1.3)
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.stackplot(sx_stack, s0, s1, s2, labels=["low", "mid", "high"], alpha=0.85)
    ax.set_title("Stackplot")
    ax.legend(loc="upper left")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_stackplot.png")

    # contour + clabel (same scalar field as plotine; axes are index coords)
    ZZ = np.zeros((30, 30))
    for r in range(30):
        for c in range(30):
            xx = c * 0.25 - 3.5
            yy = r * 0.25 - 3.5
            ZZ[r, c] = np.exp(-xx * xx - yy * yy) * 2.0
    xs = np.arange(30, dtype=float)
    ys = np.arange(30, dtype=float)
    XX, YY = np.meshgrid(xs, ys)
    fig, ax = plt.subplots(figsize=(5, 4))
    ax.contourf(XX, YY, ZZ, levels=8, cmap="viridis")
    cs = ax.contour(XX, YY, ZZ, levels=8, colors="0.2", linewidths=0.8)
    ax.clabel(cs, inline=True, fontsize=7, fmt="%.3g")
    ax.set_title("Contour + Clabel")
    save(fig, "mpl_contour.png")

    # hist2d
    t = np.arange(400) * 0.05
    hx = np.sin(t) + 0.15 * ((np.arange(400) * 3) % 11) / 11.0
    hy = np.cos(t) + 0.15 * ((np.arange(400) * 5) % 13) / 13.0
    fig, ax = plt.subplots(figsize=(5, 4))
    h = ax.hist2d(hx, hy, bins=20, cmap="viridis")
    fig.colorbar(h[3], ax=ax)
    ax.set_title("Hist2D")
    save(fig, "mpl_hist2d.png")

    # quiver
    nq = 8
    qq = np.arange(nq, dtype=float)
    QX, QY = np.meshgrid(qq, qq)
    QU = -(QY - 3.5) * 0.35
    QV = (QX - 3.5) * 0.35
    fig, ax = plt.subplots(figsize=(5, 4))
    q = ax.quiver(QX, QY, QU, QV, color="steelblue")
    ax.quiverkey(q, 0.85, 0.9, 1.0, "1", labelpos="E")
    ax.set_title("Quiver")
    save(fig, "mpl_quiver.png")

    # barbs
    bb_x, bb_y, bb_u, bb_v = [], [], [], []
    for r in range(5):
        for c in range(6):
            speed = 10.0 + c * 12.0 + r * 8.0
            ang = (r + c) * 0.4
            bb_x.append(float(c))
            bb_y.append(float(r))
            bb_u.append(speed * np.cos(ang))
            bb_v.append(speed * np.sin(ang))
    fig, ax = plt.subplots(figsize=(5.5, 4))
    ax.barbs(bb_x, bb_y, bb_u, bb_v, length=6, color="steelblue")
    ax.set_title("Barbs")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_barbs.png")

    # streamplot
    ns = 12
    yy, xx = np.mgrid[0:ns, 0:ns]
    su = -(yy - 5.5)
    sv = xx - 5.5
    fig, ax = plt.subplots(figsize=(5, 4))
    ax.streamplot(xx, yy, su, sv, color="crimson", density=1.2, linewidth=0.9)
    ax.set_title("Streamplot")
    save(fig, "mpl_streamplot.png")

    # polar
    th = np.linspace(0, 2 * np.pi, 120, endpoint=False)
    pr = 1.0 + 0.35 * np.cos(2.0 * th)
    fig, ax = plt.subplots(figsize=(4.5, 4.5), subplot_kw={"projection": "polar"})
    ax.plot(th, pr, color="crimson", linewidth=2)
    ax.set_title("Polar")
    save(fig, "mpl_polar.png")

    # twin_y (twinx)
    ty = np.array([0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
    t_left = np.sin(ty) + 1.5
    t_right = 20.0 + 5.0 * np.cos(ty)
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.plot(ty, t_left, color="steelblue", linewidth=2, label="left")
    ax.set_ylabel("left y")
    ax2 = ax.twinx()
    ax2.plot(ty, t_right, color="crimson", linewidth=2, label="right")
    ax2.set_ylabel("right y")
    ax.set_title("Twin Y")
    ax.set_xlabel("x")
    ax.grid(True, alpha=0.3)
    lines, labels = ax.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax.legend(lines + lines2, labels + labels2, loc="upper left")
    save(fig, "mpl_twin_y.png")

    # twin_x (twiny)
    bottom_x = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
    top_x = np.array([1.0, 4.0, 9.0, 16.0, 25.0])
    ty2 = np.array([0.0, 1.0, 2.0, 3.0, 4.0])
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.plot(bottom_x, ty2, color="steelblue", linewidth=2, label="linear x")
    ax.set_xlabel("linear")
    ax.set_ylabel("y")
    ax3 = ax.twiny()
    ax3.plot(top_x, ty2, color="crimson", linewidth=2, label="quad x")
    ax3.set_xlabel("quadratic")
    ax.set_title("Twin X")
    ax.grid(True, alpha=0.3)
    lines, labels = ax.get_legend_handles_labels()
    lines2, labels2 = ax3.get_legend_handles_labels()
    ax.legend(lines + lines2, labels + labels2, loc="lower right")
    save(fig, "mpl_twin_x.png")

    # categories
    fig, ax = plt.subplots(figsize=(5, 3.5))
    cats = ["A", "B", "C", "D"]
    heights = [3, 7, 2, 5]
    ax.bar(cats, heights, color="steelblue")
    ax.set_title("Categories")
    ax.set_ylabel("value")
    ax.grid(True, axis="y", alpha=0.3)
    save(fig, "mpl_categories.png")

    # LogNorm heatmap
    z_log = np.array(
        [1, 10, 100, 3, 30, 300, 5, 50, 500, 2, 20, 200, 4, 40, 400, 6],
        dtype=float,
    ).reshape(4, 4)
    fig, ax = plt.subplots(figsize=(5, 4))
    im = ax.imshow(z_log, cmap="viridis", norm=LogNorm(), origin="upper", aspect="auto")
    ax.set_title("LogNorm Heatmap")
    fig.colorbar(im, ax=ax)
    save(fig, "mpl_lognorm.png")

    # annotate
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.plot(x, y, color="steelblue", linewidth=2)
    ax.annotate(
        "peak",
        xy=(np.pi / 2, 1.0),
        xytext=(2.5, 1.15),
        arrowprops=dict(arrowstyle="->", color="crimson"),
        color="crimson",
    )
    ax.set_title("Annotate")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_annotate.png")

    # stem
    stem_x = np.array([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
    stem_y = np.array([0.5, 1.2, 0.8, 1.5, 1.1, 0.6, 0.9])
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.stem(stem_x, stem_y, linefmt="C0-", markerfmt="C0o", basefmt="k-")
    ax.set_title("Stem")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_stem.png")

    # stairs
    edges = np.array([0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
    vals = np.array([1.0, 2.5, 1.5, 3.0, 2.0])
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.stairs(vals, edges, color="crimson", linewidth=2, label="bins")
    ax.set_title("Stairs")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_stairs.png")

    # barh
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.barh([1, 2, 3, 4], [3, 7, 2, 5], color="steelblue")
    ax.set_title("BarH")
    ax.set_xlabel("value")
    ax.set_ylabel("category")
    ax.grid(True, axis="x", alpha=0.3)
    save(fig, "mpl_barh.png")

    # hexbin
    fig, ax = plt.subplots(figsize=(5, 4))
    hb = ax.hexbin(hx, hy, gridsize=12, cmap="viridis")
    fig.colorbar(hb, ax=ax)
    ax.set_title("Hexbin")
    save(fig, "mpl_hexbin.png")

    # spy
    sparse = np.zeros((10, 10))
    for i in range(10):
        sparse[i, i] = 1.0 + i
        if i + 2 < 10:
            sparse[i, i + 2] = 0.5
    fig, ax = plt.subplots(figsize=(4.5, 4))
    ax.spy(sparse, markersize=8, color="steelblue")
    ax.set_title("Spy")
    save(fig, "mpl_spy.png")

    # eventplot
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.eventplot(
        [[1.0, 2.0, 5.0, 7.0], [0.5, 3.0, 4.5], [2.5, 6.0]],
        colors=["C0", "C1", "C2"],
        lineoffsets=[1, 2, 3],
        linelengths=0.8,
    )
    ax.set_title("Eventplot")
    ax.legend(["r1", "r2", "r3"], loc="upper right")
    save(fig, "mpl_eventplot.png")

    # broken_barh
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.broken_barh([(10, 50), (100, 20), (150, 40)], (20, 9), facecolors="steelblue", label="jobs")
    ax.broken_barh([(40, 30), (120, 50)], (35, 9), facecolors="crimson", label="tasks")
    ax.set_title("Broken BarH")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_broken_barh.png")

    # polygon + spans
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.axvspan(1.0, 2.0, color="steelblue", alpha=0.25, label="vspan")
    ax.axhspan(-0.2, 0.2, color="crimson", alpha=0.25, label="hspan")
    poly = plt.Polygon(
        [(0.5, 0.5), (2.5, 0.5), (1.5, 1.5)],
        closed=True,
        facecolor="forestgreen",
        alpha=0.55,
        label="poly",
    )
    ax.add_patch(poly)
    ax.set_xlim(0, 3)
    ax.set_ylim(-0.5, 2)
    ax.set_title("Polygon + Spans")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_polygon.png")

    # pcolormesh
    rr, cc = np.mgrid[0:8, 0:8]
    pc = np.sin(rr * 0.5) + np.cos(cc * 0.4)
    x_edges = np.arange(9, dtype=float)
    y_edges = np.arange(9, dtype=float)
    fig, ax = plt.subplots(figsize=(5, 4))
    im = ax.pcolormesh(x_edges, y_edges, pc, cmap="plasma", shading="flat")
    ax.set_title("Pcolormesh")
    fig.colorbar(im, ax=ax)
    save(fig, "mpl_pcolormesh.png")

    # multiline
    y_a = np.sin(x)
    y_b = 0.7 * np.cos(x * 0.8)
    y_c = 0.4 * np.sin(x * 1.2) + 0.3
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.plot(x, y_a, color="crimson", linewidth=2, label="sin")
    ax.plot(x, y_b, color="steelblue", linewidth=2, label="cos")
    ax.plot(x, y_c, color="forestgreen", linewidth=2, label="mix")
    ax.set_title("Multiline")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_multiline.png")

    # hlines / vlines
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.hlines([0.5, 1.0, 1.5], 0.0, 5.0, colors="steelblue", linewidth=1.5, label="hlines")
    ax.vlines([1.0, 2.5, 4.0], 0.0, 2.0, colors="crimson", linewidth=1.5, label="vlines")
    ax.set_title("HLines / VLines")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_hlines_vlines.png")

    # fill_betweenx
    fbx_y = np.arange(40) * 0.15
    fbx_x1 = np.sin(fbx_y * 0.8)
    fbx_x2 = 0.5 * np.cos(fbx_y * 0.6)
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.fill_betweenx(fbx_y, fbx_x1, fbx_x2, color="steelblue", alpha=0.4, label="band")
    ax.plot(fbx_x1, fbx_y, color="crimson", linewidth=1.5, label="x1")
    ax.plot(fbx_x2, fbx_y, color="forestgreen", linewidth=1.5, label="x2")
    ax.set_title("Fill Between X")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_fill_betweenx.png")

    # axhline / axvline
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.plot(x, y, color="steelblue", linewidth=2)
    ax.axhline(0.0, color="crimson", linewidth=1.2, label="y=0")
    ax.axvline(np.pi, color="forestgreen", linewidth=1.2, label="x=π")
    ax.set_title("AxHLine / AxVLine")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_axhline_axvline.png")

    # asymmetric errorbar
    aex = np.array([0.0, 1.0, 2.0, 3.0, 4.0])
    aey = np.array([1.0, 1.5, 1.2, 2.0, 1.8])
    ay_lo = np.array([0.3, 0.15, 0.4, 0.2, 0.25])
    ay_hi = np.array([0.1, 0.35, 0.15, 0.4, 0.2])
    ax_lo = np.array([0.12, 0.08, 0.15, 0.1, 0.12])
    ax_hi = np.array([0.08, 0.14, 0.1, 0.16, 0.09])
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.errorbar(
        aex,
        aey,
        yerr=np.vstack([ay_lo, ay_hi]),
        xerr=np.vstack([ax_lo, ax_hi]),
        fmt="o-",
        color="steelblue",
        capsize=4,
        label="asym",
    )
    ax.set_title("Asymmetric Errorbar")
    ax.legend(loc="upper left")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_errorbar_asym.png")

    # annotate arrow styles
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.plot(x, y, color="steelblue", linewidth=1.5)
    ax.annotate(
        "tri",
        xy=(1.5, 1.0),
        xytext=(0.4, 0.3),
        arrowprops=dict(arrowstyle="-|>", color="crimson"),
        color="crimson",
    )
    ax.annotate(
        "simple",
        xy=(3.0, 0.2),
        xytext=(4.2, 0.9),
        arrowprops=dict(arrowstyle="->", color="forestgreen"),
        color="forestgreen",
    )
    ax.annotate(
        "bracket",
        xy=(5.0, -0.8),
        xytext=(6.5, -0.2),
        arrowprops=dict(arrowstyle="-[", color="mediumpurple"),
        color="mediumpurple",
    )
    ax.annotate(
        "both",
        xy=(7.5, 0.9),
        xytext=(8.8, 0.2),
        arrowprops=dict(arrowstyle="<->", color="crimson"),
        color="crimson",
    )
    ax.set_title("Annotate Styles")
    # Match plotine stock grid (Color::GRID / 0.8 pt), not the translucent default.
    ax.grid(True, color="#dee2e6", linewidth=0.8, alpha=1.0)
    save(fig, "mpl_annotate_styles.png")

    # heatmap extent + alpha
    rr, cc = np.mgrid[0:4, 0:4]
    hext = np.sin(rr * 0.7) + np.cos(cc * 0.9)
    fig, ax = plt.subplots(figsize=(5, 4))
    im = ax.imshow(
        hext,
        cmap="viridis",
        origin="upper",
        aspect="auto",
        extent=[0.0, 10.0, 0.0, 4.0],
        alpha=0.75,
    )
    ax.set_title("Heatmap Extent")
    fig.colorbar(im, ax=ax)
    save(fig, "mpl_heatmap_extent.png")

    # inset_axes
    ix = np.arange(80) * 0.15
    iy = np.sin(ix * 0.9) + 0.15 * np.cos(ix * 2.3)
    fig, ax = plt.subplots(figsize=(6.0, 4.0))
    ax.plot(ix, iy, color="steelblue", linewidth=2)
    ax.set_title("Inset Axes")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    axins = ax.inset_axes([0.55, 0.55, 0.4, 0.4])
    axins.plot(ix[:20], iy[:20], color="crimson", linewidth=1.5)
    axins.set_title("zoom")
    save(fig, "mpl_inset_axes.png")

    # secondary axes
    th_sec = np.arange(60) * np.pi / 30.0
    y_sec = np.sin(th_sec)
    fig, ax = plt.subplots(figsize=(6.0, 4.0))
    ax.plot(th_sec, y_sec, color="steelblue", linewidth=2)
    ax.set_title("Secondary Axes")
    ax.set_xlabel("radians")
    ax.set_ylabel("amplitude")
    sec = ax.secondary_xaxis("top", functions=(np.rad2deg, np.deg2rad))
    sec.set_xlabel("degrees")
    save(fig, "mpl_secondary_axes.png")

    # subplot span (tall left + two right)
    sx_span = np.arange(40) * 0.15
    sy_span = np.sin(sx_span)
    fig = plt.figure(figsize=(6.5, 4.5))
    gs = fig.add_gridspec(2, 2, hspace=0.28, wspace=0.22)
    ax_tall = fig.add_subplot(gs[:, 0])
    ax_tr = fig.add_subplot(gs[0, 1])
    ax_br = fig.add_subplot(gs[1, 1])
    ax_tall.plot(sx_span, sy_span, color="steelblue", linewidth=2)
    ax_tall.set_title("Span (tall)")
    ax_tall.set_ylabel("y")
    ax_tall.grid(False)
    ax_tr.scatter(sx_span, sy_span, color="crimson", s=9)
    ax_tr.set_title("top-right")
    ax_tr.grid(False)
    ax_br.hist(sy_span, bins=10, color="forestgreen")
    ax_br.set_title("bottom-right")
    ax_br.grid(False)
    save(fig, "mpl_subplot_span.png")

    # tripcolor + tricontour
    import matplotlib.tri as mtri

    tx = np.array([0.0, 1.0, 2.0, 0.5, 1.5, 1.0])
    ty = np.array([0.0, 0.0, 0.0, 0.9, 0.9, 1.6])
    tz = np.array([0.0, 0.4, 0.1, 0.8, 1.0, 0.6])
    triangles = np.array([[0, 1, 3], [1, 2, 4], [1, 3, 4], [3, 4, 5]])
    triang = mtri.Triangulation(tx, ty, triangles)
    fig, ax = plt.subplots(figsize=(6.0, 4.5))
    tpc = ax.tripcolor(triang, tz, cmap="RdBu_r")
    ax.tricontour(triang, tz, levels=7, colors="0.13", linewidths=0.9)
    ax.set_title("Tripcolor + Tricontour")
    fig.colorbar(tpc, ax=ax)
    save(fig, "mpl_tripcolor.png")

    # nested inset_axes
    fig, ax = plt.subplots(figsize=(6.0, 4.0))
    ax.plot(ix, iy, color="steelblue", linewidth=2)
    ax.set_title("Nested Inset")
    outer = ax.inset_axes([0.48, 0.48, 0.48, 0.48])
    outer.plot(ix[:30], iy[:30], color="crimson", linewidth=1.5)
    outer.set_title("outer")
    inner = outer.inset_axes([0.5, 0.5, 0.45, 0.45])
    inner.plot(ix[:12], iy[:12], color="forestgreen", linewidth=1.2)
    inner.set_title("inner")
    save(fig, "mpl_nested_inset.png")

    # secondary_y (°C ↔ °F)
    t_sec = np.arange(40) * 0.25
    c_sec = 10.0 + 8.0 * np.sin(t_sec * 0.7)
    fig, ax = plt.subplots(figsize=(6.0, 4.0))
    ax.plot(t_sec, c_sec, color="steelblue", linewidth=2)
    ax.set_title("Secondary Y")
    ax.set_xlabel("t")
    ax.set_ylabel("°C")
    sec_y = ax.secondary_yaxis(
        "right", functions=(lambda c: c * 1.8 + 32.0, lambda f: (f - 32.0) / 1.8)
    )
    sec_y.set_ylabel("°F")
    save(fig, "mpl_secondary_y.png")

    # text + annotate callouts
    fig, ax = plt.subplots(figsize=(6.0, 4.0))
    ax.plot(x, y, color="steelblue", linewidth=2, label="sin")
    ax.text(0.4, -0.6, "trough region", color="0.2", fontsize=10, ha="left")
    imax = int(np.argmax(y))
    ax.annotate(
        "peak",
        xy=(x[imax], y[imax]),
        xytext=(x[imax] + 0.8, y[imax] + 0.35),
        arrowprops=dict(arrowstyle="->", color="0.13"),
        color="crimson",
        ha="left",
        va="bottom",
    )
    ax.set_title("Text + Annotate")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.legend(loc="lower right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_text.png")

    # mathtext (scripts + frac)
    mx = np.linspace(0.0, 2.0 * np.pi, 100)
    my = np.sin(2.0 * mx) * np.exp(-0.15 * mx)
    fig, ax = plt.subplots(figsize=(6.0, 4.0))
    ax.plot(mx, my, color="steelblue", linewidth=2, label=r"$e^{-0.15t}\sin(2t)$")
    ax.set_title(r"Damped oscillator: $\alpha$-decay")
    ax.set_xlabel(r"$t$ (s)")
    ax.set_ylabel(r"$\theta$ (rad)")
    ax.text(3.6, 0.45, r"$H_2O:\frac{1}{2}mv^2$", color="0.2", fontsize=11)
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_math_labels.png")

    # table
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.bar([1, 2, 3], [3, 5, 2], color="steelblue", label="counts")
    ax.table(
        cellText=[["A", "3"], ["B", "5"], ["C", "2"]],
        colLabels=["Item", "Value"],
        loc="upper right",
        cellLoc="center",
    )
    ax.set_title("Table")
    ax.legend(loc="lower left")
    ax.grid(True, axis="y", alpha=0.3)
    save(fig, "mpl_table.png")

    # polar_scatter
    th_sc = np.arange(36) * np.pi / 18.0
    pr_sc = 0.6 + 0.35 * np.cos(3.0 * th_sc)
    fig, ax = plt.subplots(figsize=(4.5, 4.5), subplot_kw={"projection": "polar"})
    # s=16 ↔ plotine diameter 2·√(s/π) ≈ 4.51 pt
    ax.scatter(th_sc, pr_sc, c="steelblue", s=16, linewidths=0)
    ax.set_title("Polar Scatter")
    save(fig, "mpl_polar_scatter.png")

    # Coolwarm colormap
    ZZ_cw = np.zeros((24, 24))
    for r in range(24):
        for c in range(24):
            xx = c * 0.3 - 3.5
            yy = r * 0.3 - 3.5
            ZZ_cw[r, c] = xx * np.exp(-xx * xx - yy * yy)
    xs_cw = np.arange(24, dtype=float)
    ys_cw = np.arange(24, dtype=float)
    XX_cw, YY_cw = np.meshgrid(xs_cw, ys_cw)
    fig, ax = plt.subplots(figsize=(5, 4))
    cf_cw = ax.contourf(XX_cw, YY_cw, ZZ_cw, levels=10, cmap="coolwarm")
    fig.colorbar(cf_cw, ax=ax)
    ax.set_title("Coolwarm")
    save(fig, "mpl_coolwarm.png")

    # step modes (pre / mid / post)
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.step(stx, sty, where="pre", color="crimson", linewidth=1.8, label="pre")
    ax.step(stx, sty, where="mid", color="steelblue", linewidth=1.8, label="mid")
    ax.step(stx, sty, where="post", color="forestgreen", linewidth=1.8, label="post")
    ax.set_title("Step Modes")
    ax.legend(loc="upper left")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_step_modes.png")

    # axhspan / axvspan (dedicated)
    fig, ax = plt.subplots(figsize=(5.0, 3.5))
    ax.plot(x, y, color="0.13", linewidth=1.5)
    ax.axvspan(1.0, 2.5, color="steelblue", alpha=0.25, label="vspan")
    ax.axhspan(-0.4, 0.4, color="crimson", alpha=0.25, label="hspan")
    ax.set_title("AxHSpan / AxVSpan")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_axspan.png")

    # empty axes
    fig, ax = plt.subplots(figsize=(5.0, 3.5))
    ax.set_title("Empty")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    save(fig, "mpl_empty.png")

    # bar + bottom legend
    fig, ax = plt.subplots(figsize=(5.0, 3.5))
    ax.bar([1, 2, 3], [3, 5, 4], color="darkorange", label="A")
    ax.set_title("Bar Legend")
    ax.legend(loc="lower right")
    save(fig, "mpl_bar_legend.png")

    # contour clabel (dedicated)
    ZZ_cl = np.zeros((30, 30))
    for r in range(30):
        for c in range(30):
            xx = c * 0.25 - 3.5
            yy = r * 0.25 - 3.5
            ZZ_cl[r, c] = np.exp(-xx * xx - yy * yy) * 2.0
    xs_cl = np.arange(30, dtype=float)
    ys_cl = np.arange(30, dtype=float)
    XX_cl, YY_cl = np.meshgrid(xs_cl, ys_cl)
    fig, ax = plt.subplots(figsize=(5.5, 4.5))
    ax.contourf(XX_cl, YY_cl, ZZ_cl, levels=8, cmap="viridis")
    cs_cl = ax.contour(XX_cl, YY_cl, ZZ_cl, levels=8, colors="0.2", linewidths=0.9)
    ax.clabel(cs_cl, inline=True, fontsize=8, fmt="%.3g")
    ax.set_title("Contour Labels")
    save(fig, "mpl_clabel.png")

    # scatter + line overlay
    xs_ov = np.linspace(0.0, 8.0, 35)
    ys_ov = 0.2 * xs_ov + np.sin(np.arange(35, dtype=float)) * 0.4
    fig, ax = plt.subplots(figsize=(5.0, 3.5))
    ax.scatter(xs_ov, ys_ov, s=25, label="data")
    ax.plot(xs_ov, 0.2 * xs_ov, linewidth=2, label="trend")
    ax.set_title("Scatter + Line")
    ax.legend(loc="upper left")
    save(fig, "mpl_scatter_line.png")

    # area + line overlay
    ya_ov = np.abs(np.sin(x * 0.7)) + 0.15
    fig, ax = plt.subplots(figsize=(5.0, 3.5))
    ax.fill_between(x, ya_ov, alpha=0.35, label="fill")
    ax.plot(x, ya_ov, linewidth=1.8, label="edge")
    ax.set_title("Area Overlay")
    ax.legend(loc="upper right")
    save(fig, "mpl_area_line.png")

    # barh + h/vlines combo
    fig, ax = plt.subplots(figsize=(5.5, 4.0))
    ax.barh([1, 2, 3], [4.0, 7.0, 2.5], color="steelblue", label="barh")
    ax.vlines([2.0, 5.0], 0.5, 3.5, colors="crimson", linewidths=1.5, label="vlines")
    ax.hlines([2.5], 0.0, 8.0, colors="darkorange", label="hline")
    ax.set_title("BarH / Spans")
    ax.legend(loc="upper right")
    ax.grid(True, alpha=0.3)
    save(fig, "mpl_barh_spans.png")

    # y_categories
    fig, ax = plt.subplots(figsize=(5.0, 3.5))
    ax.barh([1, 2, 3], [4.0, 7.0, 2.5], color="steelblue")
    ax.set_yticks([1, 2, 3], ["low", "mid", "high"])
    ax.set_title("Y Categories")
    ax.set_xlabel("value")
    ax.grid(True, axis="x", alpha=0.3)
    save(fig, "mpl_y_categories.png")

    # y_datetime (unix UTC seconds on y, same as plotine)
    from datetime import datetime, timezone

    yd_unix = 1_577_836_800.0 + np.arange(12) * 86_400.0
    xd_val = np.sin(np.arange(12) * 0.5) + 1.0
    yd_dates = [datetime.fromtimestamp(t, tz=timezone.utc) for t in yd_unix]
    fig, ax = plt.subplots(figsize=(5.5, 3.5))
    ax.plot(xd_val, yd_dates, color="steelblue", linewidth=2)
    ax.yaxis.set_major_formatter(mdates.ConciseDateFormatter(ax.yaxis.get_major_locator()))
    ax.set_title("Y Datetime")
    ax.set_xlabel("value")
    ax.set_ylabel("date")
    save(fig, "mpl_y_datetime.png")

    # heatmap origin=lower
    rr_o, cc_o = np.mgrid[0:4, 0:4]
    z_origin = rr_o + cc_o * 0.25
    fig, ax = plt.subplots(figsize=(5, 4))
    im_o = ax.imshow(z_origin, cmap="viridis", origin="lower", aspect="auto")
    ax.set_title("Heatmap Origin Lower")
    fig.colorbar(im_o, ax=ax)
    save(fig, "mpl_heatmap_origin.png")

    # semilogy
    sx_log = np.arange(1, 41, dtype=float)
    sy_log = np.exp(0.15 * sx_log) * 0.05
    fig, ax = plt.subplots(figsize=(5.0, 3.5))
    ax.semilogy(sx_log, sy_log, color="crimson", linewidth=2)
    ax.set_title("Semilogy")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    save(fig, "mpl_semilogy.png")

    # Tab10 colormap
    z_tab = np.arange(10, dtype=float).reshape(2, 5)
    fig, ax = plt.subplots(figsize=(5.0, 3.0))
    im_t = ax.imshow(z_tab, cmap="tab10", aspect="auto")
    ax.set_title("Tab10")
    fig.colorbar(im_t, ax=ax)
    save(fig, "mpl_tab10.png")

    # Inferno heatmap
    rr_i, cc_i = np.mgrid[0:6, 0:8]
    z_inf = np.sin(rr_i * 0.55) + np.cos(cc_i * 0.65)
    fig, ax = plt.subplots(figsize=(5, 4))
    im_i = ax.imshow(z_inf, cmap="inferno", origin="upper", aspect="auto")
    ax.set_title("Inferno")
    fig.colorbar(im_i, ax=ax)
    save(fig, "mpl_inferno.png")

    # quiver + streamplot subplot
    nqs = 12
    qq = np.arange(nqs, dtype=float)
    QXS, QYS = np.meshgrid(qq, qq)
    QUS = -(QYS - 5.5) * 0.3
    QVS = (QXS - 5.5) * 0.3
    SUS = -(QYS - 5.5)
    SVS = QXS - 5.5
    fig, axes = plt.subplots(1, 2, figsize=(7.0, 3.5))
    qmix = axes[0].quiver(QXS, QYS, QUS, QVS, color="steelblue")
    axes[0].quiverkey(qmix, 0.85, 0.9, 1.0, "1 unit", labelpos="E")
    axes[0].set_title("Quiver")
    axes[1].streamplot(qq, qq, SUS, SVS, color="crimson", density=1.2, linewidth=0.9)
    axes[1].set_title("Streamplot")
    save(fig, "mpl_quiver_stream.png")

    # polar + cartesian mix
    th_mix = np.arange(120) * np.pi / 60.0
    pr_mix = 1.0 + 0.35 * np.cos(2.0 * th_mix)
    fig = plt.figure(figsize=(7.0, 3.5))
    ax_pol = fig.add_subplot(1, 2, 1, projection="polar")
    ax_car = fig.add_subplot(1, 2, 2)
    ax_pol.plot(th_mix, pr_mix, color="mediumpurple", linewidth=2)
    ax_pol.set_title("Polar")
    ax_car.plot(th_mix, pr_mix, color="steelblue", linewidth=1.75)
    ax_car.set_title("Cartesian")
    ax_car.set_xlabel("theta")
    ax_car.set_ylabel("r")
    ax_car.grid(True, alpha=0.3)
    fig.subplots_adjust(wspace=0.3)
    save(fig, "mpl_polar_mix.png")

    # hist2d + hexbin subplot
    t_mix = np.arange(400) * 0.05
    hx_mix = np.sin(t_mix) * 2.0 + (np.arange(400) % 17) * 0.05
    hy_mix = np.cos(t_mix) * 2.0 + (np.arange(400) % 13) * 0.04
    fig, axes = plt.subplots(1, 2, figsize=(7.0, 3.5))
    _, _, _, im_h2d = axes[0].hist2d(hx_mix, hy_mix, bins=16, cmap="viridis")
    fig.colorbar(im_h2d, ax=axes[0])
    axes[0].set_title("Hist2D")
    hb = axes[1].hexbin(hx_mix, hy_mix, gridsize=12, cmap="plasma")
    fig.colorbar(hb, ax=axes[1])
    axes[1].set_title("Hexbin")
    save(fig, "mpl_hist2d_hexbin.png")

    # --- 3D (aligned with plotine matplotlib_compare / mplot3d gallery) ---
    from matplotlib import cm

    # helix — gallery: Parametric curve
    fig = plt.figure(figsize=(6.0, 5.0))
    ax = fig.add_subplot(111, projection="3d")
    t3 = np.linspace(0.0, 4.0 * np.pi, 200)
    ax.plot(np.cos(t3), np.sin(t3), t3, color="crimson", linewidth=2, label="helix")
    ax.set_title("3D Helix")
    ax.legend(loc="upper right")
    save(fig, "mpl_helix_3d.png")

    # scatter
    fig = plt.figure(figsize=(6.0, 5.0))
    ax = fig.add_subplot(111, projection="3d")
    n3 = 200
    i = np.arange(n3, dtype=float)
    sx = np.cos(i * 0.1) + np.sin(i * 0.037) * 0.3
    sy = np.sin(i * 0.1) + np.cos(i * 0.029) * 0.3
    sz = i / n3 * 10.0
    ax.scatter(sx, sy, sz, c="steelblue", s=16, depthshade=True)
    ax.set_title("3D Scatter")
    save(fig, "mpl_scatter_3d.png")

    # surface — gallery: surface3d (sombrero), same grid as plotine
    fig = plt.figure(figsize=(6.0, 5.0))
    ax = fig.add_subplot(111, projection="3d")
    xs = np.arange(-5.0, 5.0, 0.25)
    ys = np.arange(-5.0, 5.0, 0.25)
    X, Y = np.meshgrid(xs, ys)
    Z = np.sin(np.sqrt(X**2 + Y**2))
    # plasma to match plotine Colormap::Plasma (mpl coolwarm differs)
    ax.plot_surface(X, Y, Z, cmap=cm.plasma, linewidth=0.15, edgecolor="k", alpha=0.95, antialiased=True)
    ax.view_init(elev=30.0, azim=-60.0)
    ax.set_title("3D Surface")
    save(fig, "mpl_surface_3d.png")

    # gaussian surface — gallery 44 style
    fig = plt.figure(figsize=(6.0, 5.0))
    ax = fig.add_subplot(111, projection="3d")
    gn = 25
    gx = np.linspace(-2.0, 2.0, gn)
    gy = np.linspace(-2.0, 2.0, gn)
    GX, GY = np.meshgrid(gx, gy)
    GZ = np.exp(-(GX * GX + GY * GY) * 0.5)
    ax.plot_surface(
        GX, GY, GZ, cmap=cm.plasma, linewidth=0.15, edgecolor="k", alpha=0.9, antialiased=True
    )
    ax.view_init(elev=35.0, azim=-50.0)
    ax.set_title("3D Gaussian")
    save(fig, "mpl_gaussian_3d.png")

    # wireframe — gallery: wire3d style
    fig = plt.figure(figsize=(6.0, 5.0))
    ax = fig.add_subplot(111, projection="3d")
    wn = 30
    wx = np.linspace(-3.0, 3.0, wn)
    wy = np.linspace(-3.0, 3.0, wn)
    WX, WY = np.meshgrid(wx, wy)
    WZ = np.sin(WX) * np.cos(WY)
    ax.plot_wireframe(WX, WY, WZ, color="steelblue", linewidth=0.7)
    ax.view_init(elev=25.0, azim=-70.0)
    ax.set_title("3D Wireframe")
    save(fig, "mpl_wireframe_3d.png")

    # bar3d — gallery: bars3d
    fig = plt.figure(figsize=(6.0, 5.0))
    ax = fig.add_subplot(111, projection="3d")
    bx = np.array([0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0])
    by = np.array([0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0])
    bz = np.zeros(8)
    bdz = np.array([3.0, 5.0, 2.0, 4.0, 1.0, 6.0, 3.0, 2.0])
    ax.bar3d(bx, by, bz, 0.6, 0.6, bdz, color="steelblue", shade=True, alpha=0.85)
    ax.view_init(elev=30.0, azim=-55.0)
    ax.set_title("3D Bar")
    save(fig, "mpl_bar_3d.png")

    # ----- M9–M13 feature pixel-align pairs (chrome matched to plotine light) -----
    # M9 static render (show pixel path)
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.plot(x, y, color="#4682b4", linewidth=2)  # STEEL_BLUE
    ax.set_title("M9 static render")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    style_like_plotine(ax)
    save(fig, "mpl_m9_static.png")

    # M10 anim frame 0
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.plot(x, y, color="#dc143c", linewidth=2)  # CRIMSON
    ax.set_title("M10 anim frame")
    ax.set_ylim(-1.2, 1.2)
    style_like_plotine(ax)
    save(fig, "mpl_m10_anim_frame.png")

    # M11 geo — same coastline.bin as plotine (fair pixel compare)
    write_mpl_m11_geo()

    # M12 pyplot facade target (== builder line with grid)
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.plot(x, y, color="#dc143c", linewidth=2, label="sin(x)")
    ax.set_title("M12 builder (== pyplot)")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    ax.legend(loc="upper right", frameon=True)
    style_like_plotine(ax)
    save(fig, "mpl_m12_pyplot.png")

    # M13 mathtext — inline ∫ limits (textstyle), matching stock mpl titles
    fig, ax = plt.subplots(figsize=(5, 3.5))
    ax.plot(x, y, color="#4682b4", linewidth=2)
    ax.set_title(r"M13 mathtext $\int_0^1 x^2\,dx$")
    ax.set_xlabel(r"$x$")
    ax.set_ylabel(r"$y$")
    style_like_plotine(ax)
    save(fig, "mpl_m13_mathtext.png")

    plotine = sorted(OUT.glob("plotine_*.png"))
    mpl = {p.name.replace("mpl_", "plotine_"): p for p in OUT.glob("mpl_*.png")}
    missing = [p.name for p in plotine if p.name not in mpl]
    extra = [n for n in mpl if not (OUT / n).exists()]
    print(f"Done. Compare files in {OUT.resolve()}")
    print(f"Pairs: {len(plotine)} plotine / {len(mpl)} mpl")
    if missing:
        print("Missing mpl for:", ", ".join(missing))
    if extra:
        print("Orphan mpl keys:", ", ".join(extra))
    write_index([p.stem.replace("plotine_", "") for p in plotine if p.name in mpl])


def write_index(names: list[str]) -> None:
    """Write a local HTML gallery for side-by-side visual review."""
    cards = []
    for name in names:
        title = name.replace("_", " ").title()
        cards.append(
            f"""
    <section class="card" data-name="{name}">
      <h2>{title}</h2>
      <div class="pair">
        <figure>
          <figcaption>plotine</figcaption>
          <img src="plotine_{name}.png" alt="plotine {name}" loading="lazy"/>
        </figure>
        <figure>
          <figcaption>matplotlib</figcaption>
          <img src="mpl_{name}.png" alt="matplotlib {name}" loading="lazy"/>
        </figure>
      </div>
    </section>"""
        )
    html = f"""<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>plotine ↔ matplotlib · {len(names)} charts</title>
<style>
  :root {{
    --bg: #f4f1ec;
    --ink: #1c1917;
    --muted: #78716c;
    --line: #d6d3d1;
    --panel: #fffdf9;
    --accent: #0f766e;
  }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0;
    font-family: "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
    background: var(--bg);
    color: var(--ink);
  }}
  header {{
    position: sticky; top: 0; z-index: 2;
    backdrop-filter: blur(8px);
    background: color-mix(in srgb, var(--bg) 88%, transparent);
    border-bottom: 1px solid var(--line);
    padding: 1rem 1.25rem 0.85rem;
  }}
  header h1 {{
    margin: 0 0 0.35rem;
    font-size: 1.35rem;
    font-weight: 650;
    letter-spacing: -0.02em;
  }}
  header p {{ margin: 0; color: var(--muted); font-size: 0.92rem; }}
  .toolbar {{
    display: flex; flex-wrap: wrap; gap: 0.6rem; align-items: center;
    margin-top: 0.75rem;
  }}
  input[type="search"] {{
    flex: 1 1 220px;
    min-width: 180px;
    padding: 0.45rem 0.7rem;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--panel);
    color: var(--ink);
    font: inherit;
  }}
  .count {{ color: var(--muted); font-size: 0.88rem; }}
  main {{
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem 1.25rem 3rem;
    display: grid;
    gap: 1.1rem;
  }}
  .card {{
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 0.85rem 0.95rem 1rem;
  }}
  .card h2 {{
    margin: 0 0 0.65rem;
    font-size: 1rem;
    font-weight: 600;
  }}
  .pair {{
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }}
  @media (max-width: 800px) {{
    .pair {{ grid-template-columns: 1fr; }}
  }}
  figure {{ margin: 0; }}
  figcaption {{
    font-size: 0.78rem;
    color: var(--accent);
    font-weight: 600;
    margin-bottom: 0.3rem;
    text-transform: lowercase;
    letter-spacing: 0.04em;
  }}
  img {{
    width: 100%;
    height: auto;
    display: block;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: #fff;
  }}
  .hidden {{ display: none; }}
</style>
</head>
<body>
<header>
  <h1>plotine ↔ matplotlib</h1>
  <p>同数据 / 同尺寸并排对比 · 共 {len(names)} 组（含 3D）</p>
  <div class="toolbar">
    <input id="q" type="search" placeholder="筛选图型，如 contour / barbs / twin…"/>
    <span class="count"><span id="shown">{len(names)}</span> / {len(names)}</span>
  </div>
</header>
<main>
{"".join(cards)}
</main>
<script>
const q = document.getElementById('q');
const shown = document.getElementById('shown');
const cards = [...document.querySelectorAll('.card')];
q.addEventListener('input', () => {{
  const needle = q.value.trim().toLowerCase();
  let n = 0;
  for (const c of cards) {{
    const ok = !needle || c.dataset.name.includes(needle) || c.querySelector('h2').textContent.toLowerCase().includes(needle);
    c.classList.toggle('hidden', !ok);
    if (ok) n++;
  }}
  shown.textContent = String(n);
}});
</script>
</body>
</html>
"""
    path = OUT / "index.html"
    path.write_text(html, encoding="utf-8")
    print(f"Viewer: {path.resolve()}")


if __name__ == "__main__":
    main()
