"""Open an interactive matplotlib mplot3d window (drag to rotate).

Requires a GUI backend (TkAgg / Qt). Run:

    python scripts/mpl_3d_interactive.py
"""

from __future__ import annotations

import numpy as np
import matplotlib.pyplot as plt
from matplotlib import cm


def main() -> None:
    fig = plt.figure(figsize=(8, 6))
    ax = fig.add_subplot(111, projection="3d")

    x = np.arange(-5, 5, 0.25)
    y = np.arange(-5, 5, 0.25)
    x, y = np.meshgrid(x, y)
    r = np.sqrt(x**2 + y**2)
    z = np.sin(r)

    surf = ax.plot_surface(
        x, y, z, cmap=cm.coolwarm, linewidth=0, antialiased=True
    )
    fig.colorbar(surf, shrink=0.5, aspect=10)
    ax.set_title("matplotlib mplot3d — drag to rotate / scroll to zoom")
    ax.set_xlabel("X")
    ax.set_ylabel("Y")
    ax.set_zlabel("Z")

    print("Close the window to exit.")
    plt.show()


if __name__ == "__main__":
    main()
