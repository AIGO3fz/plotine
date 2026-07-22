#!/usr/bin/env python3
"""Build compare/dashboard.html — L1 benchmark table/gallery + visual compare.

    python scripts/write_review_dashboard.py

Requires prior:
    $env:BENCH_SAVE=1; python scripts/benchmark.py --tier default
    cargo run -p plotine --example matplotlib_compare --release
    python scripts/matplotlib_compare.py
"""

from __future__ import annotations

import csv
import html
from pathlib import Path

try:
    import numpy as np
    from PIL import Image
except ImportError as exc:  # pragma: no cover
    raise SystemExit("need pillow + numpy") from exc

ROOT = Path(__file__).resolve().parents[1]
COMPARE = ROOT / "compare"
BENCH_DIR = COMPARE / "bench"
OUT = COMPARE / "dashboard.html"


def mae_rgb(a: Path, b: Path) -> float | None:
    if not a.exists() or not b.exists():
        return None
    A = np.asarray(Image.open(a).convert("RGB"), dtype=np.float64)
    B = np.asarray(Image.open(b).convert("RGB"), dtype=np.float64)
    if A.shape != B.shape:
        B = np.asarray(
            Image.open(b).convert("RGB").resize((A.shape[1], A.shape[0])),
            dtype=np.float64,
        )
    return float(np.abs(A - B).mean())


def load_bench() -> list[dict]:
    path = BENCH_DIR / "results.csv"
    if not path.exists():
        return []
    rows = []
    with path.open(encoding="utf-8", newline="") as f:
        for r in csv.DictReader(f):
            rows.append(r)
    rows.sort(key=lambda r: -float(r.get("speedup_mpl_over_plotine") or 0))
    return rows


def compare_names() -> list[str]:
    names = []
    for p in sorted(COMPARE.glob("plotine_*.png")):
        stem = p.name.removeprefix("plotine_").removesuffix(".png")
        if (COMPARE / f"mpl_{stem}.png").exists():
            names.append(stem)
    return names


def bench_png_stems() -> list[str]:
    """Scenario names that have a plotine PNG sample under compare/bench/."""
    if not BENCH_DIR.is_dir():
        return []
    stems = []
    for p in sorted(BENCH_DIR.glob("plotine_*.png")):
        stem = p.name.removeprefix("plotine_").removesuffix(".png")
        stems.append(stem)
    return stems


def main() -> None:
    bench = load_bench()
    by_bench = {r["name"]: r for r in bench}
    names = compare_names()
    bench_stems = bench_png_stems()
    mae_rows = []
    for stem in names:
        m = mae_rgb(COMPARE / f"plotine_{stem}.png", COMPARE / f"mpl_{stem}.png")
        if m is not None:
            mae_rows.append((stem, m))
    mae_rows.sort(key=lambda t: -t[1])

    # Visual status: not fully aligned — surface worst MAEs.
    worst = mae_rows[:12]
    high = sum(1 for _, m in mae_rows if m >= 8.0)
    mid = sum(1 for _, m in mae_rows if 3.0 <= m < 8.0)
    low = sum(1 for _, m in mae_rows if m < 3.0)

    bench_rows_html = []
    for r in bench:
        sp = float(r.get("speedup_mpl_over_plotine") or 0)
        cls = "fast" if sp >= 2 else ("ok" if sp >= 1 else "slow")
        bench_rows_html.append(
            "<tr class='{cls}'><td><code>{name}</code></td>"
            "<td class='num'>{p:.1f}</td><td class='num'>{m:.1f}</td>"
            "<td class='num'>{sp:.2f}×</td></tr>".format(
                cls=cls,
                name=html.escape(r["name"]),
                p=float(r["plotine_median_ms"] or 0),
                m=float(r["mpl_median_ms"] or 0),
                sp=sp,
            )
        )

    mae_table = []
    for stem, m in worst:
        cls = "bad" if m >= 8 else ("warn" if m >= 3 else "good")
        mae_table.append(
            f"<tr class='{cls}'><td><code>{html.escape(stem)}</code></td>"
            f"<td class='num'>{m:.2f}</td></tr>"
        )

    cards = []
    for stem in names:
        m = next((v for s, v in mae_rows if s == stem), None)
        mae_s = f"{m:.1f}" if m is not None else "—"
        badge = (
            "bad"
            if m is not None and m >= 8
            else ("warn" if m is not None and m >= 3 else "good")
        )
        cards.append(
            f"""
<article class="card" data-name="{html.escape(stem)}">
  <h2>{html.escape(stem)} <span class="mae {badge}">MAE {mae_s}</span></h2>
  <div class="pair">
    <figure>
      <figcaption>plotine</figcaption>
      <img src="plotine_{html.escape(stem)}.png" alt="plotine {html.escape(stem)}" loading="lazy"/>
    </figure>
    <figure>
      <figcaption>matplotlib</figcaption>
      <img src="mpl_{html.escape(stem)}.png" alt="mpl {html.escape(stem)}" loading="lazy"/>
    </figure>
  </div>
</article>"""
        )

    bench_cards = []
    for stem in bench_stems:
        r = by_bench.get(stem)
        sp = float(r["speedup_mpl_over_plotine"]) if r and r.get("speedup_mpl_over_plotine") else None
        if sp is None:
            badge_html = '<span class="mae">sample</span>'
        else:
            cls = "good" if sp >= 2 else ("warn" if sp >= 1 else "bad")
            badge_html = f'<span class="mae {cls}">{sp:.2f}×</span>'
        mpl_path = BENCH_DIR / f"mpl_{stem}.png"
        mpl_fig = ""
        if mpl_path.exists():
            mpl_fig = f"""
    <figure>
      <figcaption>matplotlib</figcaption>
      <img src="bench/mpl_{html.escape(stem)}.png" alt="mpl {html.escape(stem)}" loading="lazy"/>
    </figure>"""
        bench_cards.append(
            f"""
<article class="card" data-name="{html.escape(stem)}">
  <h2><code>{html.escape(stem)}</code> {badge_html}</h2>
  <div class="pair">
    <figure>
      <figcaption>plotine</figcaption>
      <img src="bench/plotine_{html.escape(stem)}.png" alt="plotine {html.escape(stem)}" loading="lazy"/>
    </figure>{mpl_fig}
  </div>
</article>"""
        )

    n_bench = len(bench)
    n_bench_png = len(bench_stems)
    med_sp = (
        float(np.median([float(r["speedup_mpl_over_plotine"]) for r in bench]))
        if bench
        else 0.0
    )

    page = f"""<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>plotine review — bench + compare</title>
<style>
:root {{
  --bg: #f4f1ec; --ink: #1c1917; --muted: #78716c; --line: #d6d3d1;
  --panel: #fffdf9; --accent: #0f766e; --bad: #b91c1c; --warn: #b45309; --good: #047857;
}}
* {{ box-sizing: border-box; }}
body {{
  margin: 0; font-family: "Segoe UI", "PingFang SC", sans-serif;
  background: var(--bg); color: var(--ink); line-height: 1.45;
}}
header {{
  padding: 1.25rem 1.5rem 1rem; border-bottom: 1px solid var(--line);
  background: #f4f1ecee; position: sticky; top: 0; z-index: 5;
  backdrop-filter: blur(8px);
}}
h1 {{ margin: 0 0 0.35rem; font-size: 1.35rem; }}
.lead {{ margin: 0 0 0.75rem; color: var(--muted); max-width: 70rem; }}
nav {{ display: flex; flex-wrap: wrap; gap: 0.6rem; align-items: center; }}
nav a {{
  color: var(--accent); text-decoration: none; font-weight: 600; font-size: 0.9rem;
  border: 1px solid var(--line); background: var(--panel); padding: 0.25rem 0.6rem; border-radius: 6px;
}}
.banner {{
  margin: 1rem 1.5rem 0; padding: 0.85rem 1rem; border-radius: 8px;
  border: 1px solid #fcd34d; background: #fffbeb; color: #78350f;
}}
.banner strong {{ color: #92400e; }}
section {{ max-width: 1200px; margin: 0 auto; padding: 1.25rem 1.25rem 0.5rem; }}
h2.sec {{ margin: 0 0 0.75rem; font-size: 1.15rem; }}
.stats {{ display: flex; flex-wrap: wrap; gap: 0.75rem; margin-bottom: 1rem; }}
.stat {{
  background: var(--panel); border: 1px solid var(--line); border-radius: 8px;
  padding: 0.55rem 0.8rem; min-width: 8rem;
}}
.stat b {{ display: block; font-size: 1.2rem; }}
.stat span {{ color: var(--muted); font-size: 0.8rem; }}
table {{
  width: 100%; border-collapse: collapse; background: var(--panel);
  border: 1px solid var(--line); border-radius: 8px; overflow: hidden;
  font-size: 0.88rem;
}}
th, td {{ padding: 0.4rem 0.55rem; border-bottom: 1px solid var(--line); text-align: left; }}
th {{ background: #f5f5f4; font-weight: 600; }}
td.num, th.num {{ text-align: right; font-variant-numeric: tabular-nums; }}
tr.fast td.num:last-child {{ color: var(--good); font-weight: 600; }}
tr.slow td.num:last-child {{ color: var(--bad); font-weight: 600; }}
tr.bad td {{ color: var(--bad); }}
tr.warn td {{ color: var(--warn); }}
.toolbar {{ display: flex; gap: 0.75rem; align-items: center; margin: 0.5rem 0 1rem; }}
#q {{
  flex: 1; max-width: 28rem; padding: 0.45rem 0.65rem; border: 1px solid var(--line);
  border-radius: 6px; background: var(--panel); color: var(--ink); font: inherit;
}}
.cards {{ display: grid; gap: 1.1rem; padding-bottom: 3rem; }}
.card {{
  background: var(--panel); border: 1px solid var(--line); border-radius: 10px;
  padding: 0.85rem 0.95rem 1rem;
}}
.card h2 {{ margin: 0 0 0.65rem; font-size: 1rem; display: flex; gap: 0.6rem; align-items: baseline; flex-wrap: wrap; }}
.mae {{
  font-size: 0.75rem; font-weight: 700; padding: 0.1rem 0.45rem; border-radius: 999px;
  border: 1px solid var(--line);
}}
.mae.bad {{ background: #fef2f2; color: var(--bad); border-color: #fecaca; }}
.mae.warn {{ background: #fffbeb; color: var(--warn); border-color: #fde68a; }}
.mae.good {{ background: #ecfdf5; color: var(--good); border-color: #a7f3d0; }}
.pair {{ display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; }}
@media (max-width: 800px) {{ .pair {{ grid-template-columns: 1fr; }} }}
figure {{ margin: 0; }}
figcaption {{
  font-size: 0.78rem; color: var(--accent); font-weight: 600; margin-bottom: 0.3rem;
  text-transform: lowercase; letter-spacing: 0.04em;
}}
img {{
  width: 100%; height: auto; display: block; border: 1px solid var(--line);
  border-radius: 6px; background: #fff;
}}
.hidden {{ display: none; }}
.two-col {{ display: grid; grid-template-columns: 1.4fr 1fr; gap: 1rem; }}
@media (max-width: 900px) {{ .two-col {{ grid-template-columns: 1fr; }} }}
.scroll {{ max-height: 28rem; overflow: auto; border-radius: 8px; }}
</style>
</head>
<body>
<header>
  <h1>plotine review — performance + visual</h1>
  <p class="lead">
    一次生成的汇总页：L1 benchmark（计时）与 matplotlib compare（视觉）。
    性能目标是明显快于 mpl；视觉目标是对齐 stock mpl，但<strong>尚未全部对齐</strong>。
  </p>
  <nav>
    <a href="#bench">Benchmark</a>
    <a href="#bench-gallery">Bench gallery</a>
    <a href="#visual-gaps">Visual gaps</a>
    <a href="#compare">Compare pairs</a>
    <a href="index.html">Compare-only</a>
    <a href="bench/results.md">results.md</a>
  </nav>
</header>

<div class="banner">
  <strong>视觉对齐状态：未完成。</strong>
  当前 {len(names)} 对 compare 中，MAE≥8 约 {high} 对、3–8 约 {mid} 对、&lt;3 约 {low} 对。
  高 MAE 常见于 layout chrome、mathtext、3D、geo 等——属于通用引擎差距，不是「只修了 benchmark」。
</div>

<section id="bench">
  <h2 class="sec">L1 benchmark（tier=default，{n_bench} cases）</h2>
  <div class="stats">
    <div class="stat"><b>{med_sp:.2f}×</b><span>median speedup (mpl/plotine)</span></div>
    <div class="stat"><b>{n_bench}</b><span>timed scenarios</span></div>
    <div class="stat"><b>{sum(1 for r in bench if float(r.get('speedup_mpl_over_plotine') or 0)>=1)}</b><span>faster than mpl</span></div>
    <div class="stat"><b>{n_bench_png}</b><span>PNG samples on disk</span></div>
  </div>
  <div class="scroll">
  <table>
    <thead><tr><th>name</th><th class="num">plotine ms</th><th class="num">mpl ms</th><th class="num">speedup</th></tr></thead>
    <tbody>
{"".join(bench_rows_html)}
    </tbody>
  </table>
  </div>
  <p class="lead">speedup = mpl_median / plotine_median（&gt;1 表示 plotine 更快）。样本图见下方 Bench gallery（<code>BENCH_SAVE=1</code>）。</p>
</section>

<section id="bench-gallery">
  <h2 class="sec">Bench gallery（{n_bench_png} samples）</h2>
  <p class="lead">独立 L1 场景落盘图（<code>compare/bench/</code>），与 compare 视觉语料分开。角标为 speedup。</p>
  <div class="toolbar">
    <input id="bq" type="search" placeholder="筛选 bench：line / heatmap / contour…"/>
    <span class="count"><span id="bshown">{n_bench_png}</span> / {n_bench_png}</span>
  </div>
  <div class="cards" id="bench-cards">
{"".join(bench_cards) if bench_cards else "<p class='lead'>尚无 PNG。请先 <code>$env:BENCH_SAVE=1; python scripts/benchmark.py</code></p>"}
  </div>
</section>

<section id="visual-gaps">
  <h2 class="sec">Visual gaps（按 MAE 最高）</h2>
  <div class="two-col">
    <div class="scroll">
    <table>
      <thead><tr><th>pair</th><th class="num">MAE</th></tr></thead>
      <tbody>
{"".join(mae_table)}
      </tbody>
    </table>
    </div>
    <p class="lead">
      MAE 是粗指标（Skia vs Agg 本身有底噪）。重点看 ≥8 的对：肉眼通常仍有明显差异。
      下面 Compare 区可并排核对。
    </p>
  </div>
</section>

<section id="compare">
  <h2 class="sec">Compare pairs（{len(names)}）</h2>
  <div class="toolbar">
    <input id="q" type="search" placeholder="筛选：contour / math / twin / 3d…"/>
    <span class="count"><span id="shown">{len(names)}</span> / {len(names)}</span>
  </div>
  <div class="cards">
{"".join(cards)}
  </div>
</section>

<script>
function wireFilter(inputId, shownId, rootSel) {{
  const q = document.getElementById(inputId);
  const shown = document.getElementById(shownId);
  if (!q || !shown) return;
  const cards = [...document.querySelectorAll(rootSel + ' .card')];
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
}}
wireFilter('q', 'shown', '#compare');
wireFilter('bq', 'bshown', '#bench-gallery');
</script>
</body>
</html>
"""
    OUT.write_text(page, encoding="utf-8")
    print(f"Wrote {OUT}")
    print(
        f"Pairs={len(names)} bench={n_bench} bench_png={n_bench_png} "
        f"MAE>=8={high} median_speedup={med_sp:.2f}x"
    )


if __name__ == "__main__":
    main()
