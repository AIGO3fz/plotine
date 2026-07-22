# matplotlib 静态 2D 图型覆盖清单

<!-- English summary:
  Coverage checklist for plotine vs matplotlib static 2D chart types.
  Tracks 44 Axes-level APIs (line, scatter, bar, contour, polar, etc.).
  All core items are marked ✅ (implemented with recipe + tests + gallery snapshot).
  For the English feature comparison, see docs/MPL_GAP.md.
-->

> 对标范围：matplotlib `Axes` 上的**静态 2D** 出图能力（不含 3D）。交互 GUI → 动画 → 地理投影 → pyplot facade → 外挂 LaTeX 见 `DEVELOPMENT_PLAN.md` §1.2 **M9–M13**。
> 状态与 `docs/DEVELOPMENT_PLAN.md` §5 M6 同步；实现完成后改本表 `状态` 列。
> 最后更新：2026-07-19

## 图例

| 状态 | 含义 |
|---|---|
| ✅ | 已实现（recipe + Axes API + gallery/快照） |
| 🔲 | 计划中（见所属批次） |
| ➖ | 非目标或由其他能力覆盖（见备注） |

## 覆盖总表

| # | matplotlib | plotine API | 状态 | 批次 | 备注 |
|---|---|---|---|---|---|
| 1 | `plot` | `ax.line` | ✅ | M1 | |
| 2 | `scatter` | `ax.scatter` | ✅ | M1 | |
| 3 | `bar` | `ax.bar` + `.hatch(Hatch::…)` | ✅ | M1 | hatch：`/` `\` `x` `-` `|` `+` `.` |
| 4 | `barh` | `ax.barh` | ✅ | M6a | 水平柱 |
| 5 | `hist` | `ax.hist` | ✅ | M1 | |
| 6 | `errorbar` | `ax.errorbar` + `.xerr` / `.yerr_asym` / `.xerr_asym` | ✅ | M1/M8b | 对称 + 非对称 `(2,N)` 误差 |
| 7 | `fill_between` | `ax.fill_between` | ✅ | M6a | 双曲线间填充 |
| 8 | `fill_betweenx` | `ax.fill_betweenx` | ✅ | M6a | |
| 9 | `fill`（area 至 baseline） | `ax.area` | ✅ | M1 | mpl 无同名；语义≈ fill_between(y2=0) |
| 10 | `fill`（任意多边形） | `ax.polygon` | ✅ | M6b | |
| — | `Rectangle` / `Circle` / `Ellipse` | `ax.rectangle` / `circle` / `ellipse` | ✅ | M8 | 数据坐标 Patch；rect 支持 hatch |
| 11 | `step` | `ax.step` | ✅ | M6a | pre/mid/post |
| 12 | `stairs` | `ax.stairs` | ✅ | M6a | edges + values |
| 13 | `stem` | `ax.stem` | ✅ | M6a | |
| 14 | `hlines` | `ax.hlines` | ✅ | M6a | |
| 15 | `vlines` | `ax.vlines` | ✅ | M6a | |
| 16 | `axhline` / `axvline` | `ax.axhline` / `ax.axvline` | ✅ | M6a | 全轴跨度辅助线 |
| 17 | `axhspan` / `axvspan` | `ax.axhspan` / `ax.axvspan` | ✅ | M6b | |
| 18 | `stackplot` | `ax.stackplot` | ✅ | M6b | |
| 19 | `pie` | `ax.pie` | ✅ | M6b | |
| 20 | `eventplot` | `ax.eventplot` | ✅ | M6b | |
| 21 | `broken_barh` | `ax.broken_barh` | ✅ | M6b | |
| 22 | `boxplot` | `ax.boxplot` | ✅ | M3 | |
| 23 | `violinplot` | `ax.violin` | ✅ | M3 | |
| 24 | `hist2d` | `ax.hist2d` | ✅ | M6c | |
| 25 | `hexbin` | `ax.hexbin` | ✅ | M6c | |
| 26 | `imshow` / `matshow` | `ax.heatmap` + `.extent` / `.alpha` | ✅ | M3/M8b | 网格色块 + colorbar；extent/alpha 对齐 imshow |
| 27 | `pcolormesh` / `pcolor` | `ax.pcolormesh` | ✅ | M6c | 显式边坐标 |
| 28 | `contour` | `ax.contour` | ✅ | M6c | Marching Squares |
| 29 | `contourf` | `ax.contourf` | ✅ | M6c | 分格着色 + levels |
| 30 | `clabel` | `.clabel(true)` on contour | ✅ | M6c+ | 每 level 一标签；inline 断线 + `%.3g` 样式 |
| 31 | `quiver` | `ax.quiver` | ✅ | M6d | |
| 32 | `quiverkey` | `.quiverkey(len, label)` | ✅ | M6d | |
| 33 | `streamplot` | `ax.streamplot` | ✅ | M6d | |
| 34 | `barbs` | `ax.barbs` | ✅ | M6d+ | flag/full/half；`.length` / `.flip` / `.increments` |
| 35 | `spy` | `ax.spy` | ✅ | M6c | 稀疏模式矩阵 |
| 36 | `tricontour` / `tripcolor` | `ax.tricontour` / `tripcolor` + `.triangles`（可选；省略时 Delaunay） | ✅ | M8d | 自动三角已实现 |
| 37 | `polar` / 极坐标 Axes | `ax.polar_line` / `polar_scatter` / `polar_frame` | ✅ | M6d | 圆形 spine + θ°/r 标签 + equal aspect |
| 38 | `loglog` / `semilogx` / `semilogy` | `x_scale`/`y_scale` | ✅ | M2 | 用 ScaleType，无独立方法 |
| 39 | `twinx` / `twiny` | `ax.twin_y` / `ax.twin_x` | ✅ | M5 | 共享 x→右 y；共享 y→顶 x |
| 40 | `text` / `annotate` | `ax.text` / `ax.annotate` | ✅ | M5 | 数据坐标；`.ha`/`.va`；标签支持 `$...$` mathtext |
| 40b | mathtext | `plotine::mathtext`（自动识别 `$...$`） | ✅ | M8e | 脚本 / `\frac` / `\sqrt` / `pmatrix` / 希腊；**无**外挂 LaTeX；`feature=cjk` 加载系统/用户字体；`math::unicode` 仍可用 |
| 41 | `arrow` / `FancyArrow` | annotate `.arrow` / `.arrow_style` | ✅ | M5/M8b | `ArrowStyle::{Triangle,Simple,Bracket,BothEnds}` |
| 42 | `table` | `ax.table` + `.col_labels` / `.row_labels` / `.loc` | ✅ | M8e | axes fraction 叠加；无自动数据域联动 |
| 43 | `inset_axes` | `ax.inset_axes([x0,y0,w,h], \|…\|)` | ✅ | M8c | axes fraction；一层嵌套；无 colorbar |
| 44 | `secondary_xaxis` / `yaxis` | `ax.secondary_x` / `secondary_y` | ✅ | M8c | 函数/仿射刻度；≠ twin |

**计数**：核心静态 2D 已实现项仍以 ✅ 为准；**M8** 将原「1.0 后」的 inset / secondary / tricontour 与 `errorbar.xerr` 拉入当前主线（见 `DEVELOPMENT_PLAN.md` §M8）。静态 3D 见 M7（gallery 42–46），不在本表统计。

## 批次验收标准（每图强制）

1. `recipes/<name>.rs`：纯几何，带单元测试  
2. `Axes::<name>` + artist builder（强类型样式）  
3. gallery 至少 1 张 + `visual_snapshots` PNG（改渲染须人工 `cargo insta review`）  
4. `AGENTS.md` / `llms.txt` 图表表增一行  
5. `PlotError` 在长度不匹配等失败路径带 `suggestion`

## 本表范围外（与 DEVELOPMENT_PLAN 一致）

**M9–M13 按序功能对齐**（非本表勾选；见 `DEVELOPMENT_PLAN.md` §1.2）：

1. 交互 GUI（M9）
2. 动画（M10）
3. 地理投影（M11）
4. pyplot facade（M12，opt-in）
5. 外挂 LaTeX（M13，opt-in；默认仍内置 mathtext）

**仍不做：** 符号表达式自适应采样（CAS）；GoG DSL。WASM / 额外渲染后端排在 M13 之后。
