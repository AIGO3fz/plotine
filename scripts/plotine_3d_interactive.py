"""Interactive viewer for plotine gallery 3D demos (42–46).

plotine itself is static-export only (PNG/SVG/PDF). This script recreates the
same datasets as `examples/gallery.rs` (helix / scatter / surface / wireframe /
bar3d) with matplotlib's mplot3d so you can drag-rotate them.

Usage (from repo root):

    python scripts/plotine_3d_interactive.py
    python scripts/plotine_3d_interactive.py surface
    python scripts/plotine_3d_interactive.py all

Keys while a window is open:
    1–5   switch demo (when started with `all` / default)
    q     quit
"""

from __future__ import annotations

import sys
from typing import Callable

import numpy as np
import matplotlib.pyplot as plt
from matplotlib import cm
from matplotlib.figure import Figure
from mpl_toolkits.mplot3d import Axes3D  # noqa: F401


def demo_helix(ax) -> None:
    """Gallery 42 — 3D helix."""
    t = np.linspace(0.0, 4.0 * np.pi, 200)
    ax.plot(np.cos(t), np.sin(t), t, color="crimson", lw=2, label="helix")
    ax.set_title("42 3D Helix (plotine gallery data)")
    ax.legend(loc="upper right")


def demo_scatter(ax) -> None:
    """Gallery 43 — 3D scatter."""
    n = 200
    i = np.arange(n, dtype=float)
    x = np.cos(i * 0.1) + np.sin(i * 0.037) * 0.3
    y = np.sin(i * 0.1) + np.cos(i * 0.029) * 0.3
    z = i / n * 10.0
    ax.scatter(x, y, z, c="steelblue", s=16, depthshade=True)
    ax.set_title("43 3D Scatter (plotine gallery data)")


def demo_surface(ax) -> None:
    """Gallery 44 — gaussian surface."""
    sn = 25
    xs = np.linspace(-2.0, 2.0, sn)
    ys = np.linspace(-2.0, 2.0, sn)
    x, y = np.meshgrid(xs, ys)
    z = np.exp(-(x * x + y * y) * 0.5)
    ax.plot_surface(
        x,
        y,
        z,
        cmap=cm.plasma,
        linewidth=0.15,
        edgecolor="k",
        alpha=0.9,
        antialiased=True,
    )
    ax.view_init(elev=35.0, azim=-50.0)
    ax.set_title("44 3D Surface (plotine gallery data)")


def demo_wireframe(ax) -> None:
    """Gallery 45 — wireframe."""
    wn = 15
    xs = np.linspace(-3.0, 3.0, wn)
    ys = np.linspace(-3.0, 3.0, wn)
    x, y = np.meshgrid(xs, ys)
    z = np.sin(x) * np.cos(y)
    ax.plot_wireframe(x, y, z, color="steelblue", linewidth=0.9)
    ax.view_init(elev=25.0, azim=-70.0)
    ax.set_title("45 3D Wireframe (plotine gallery data)")


def demo_bar3d(ax) -> None:
    """Gallery 46 — 3D bars."""
    x = np.array([0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0])
    y = np.array([0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0])
    z = np.zeros(8)
    dz = np.array([3.0, 5.0, 2.0, 4.0, 1.0, 6.0, 3.0, 2.0])
    ax.bar3d(x, y, z, 0.6, 0.6, dz, color="steelblue", shade=True, alpha=0.85)
    ax.view_init(elev=30.0, azim=-55.0)
    ax.set_title("46 3D Bar (plotine gallery data)")


DEMOS: dict[str, Callable] = {
    "helix": demo_helix,
    "scatter": demo_scatter,
    "surface": demo_surface,
    "wireframe": demo_wireframe,
    "bar3d": demo_bar3d,
}

ORDER = ["helix", "scatter", "surface", "wireframe", "bar3d"]


def draw(fig: Figure, name: str) -> None:
    fig.clear()
    ax = fig.add_subplot(111, projection="3d")
    DEMOS[name](ax)
    ax.set_xlabel("X")
    ax.set_ylabel("Y")
    ax.set_zlabel("Z")
    fig.canvas.draw_idle()
    fig.suptitle(
        "Drag to rotate · scroll to zoom · keys 1–5 switch · q quit",
        fontsize=10,
        y=0.02,
    )


def run_single(name: str) -> None:
    if name not in DEMOS:
        print(f"Unknown demo {name!r}. Choose from: {', '.join(ORDER)}")
        sys.exit(1)
    fig = plt.figure(figsize=(8, 6))
    draw(fig, name)
    print(f"Showing {name}. Close the window to exit.")
    plt.show()


def run_all() -> None:
    state = {"idx": 2}  # start on surface (most interesting)

    fig = plt.figure(figsize=(8, 6))
    draw(fig, ORDER[state["idx"]])

    def on_key(event) -> None:
        if event.key in ("q", "escape"):
            plt.close(fig)
            return
        if event.key in "12345":
            state["idx"] = int(event.key) - 1
            draw(fig, ORDER[state["idx"]])
            print(f"→ {ORDER[state['idx']]}")

    fig.canvas.mpl_connect("key_press_event", on_key)
    print("Interactive plotine-gallery 3D viewer")
    print("  1 helix  2 scatter  3 surface  4 wireframe  5 bar3d")
    print("  drag = rotate, scroll = zoom, q = quit")
    plt.show()


def main() -> None:
    arg = (sys.argv[1] if len(sys.argv) > 1 else "all").lower()
    if arg in ("all", "gallery"):
        run_all()
    else:
        run_single(arg)


if __name__ == "__main__":
    main()
