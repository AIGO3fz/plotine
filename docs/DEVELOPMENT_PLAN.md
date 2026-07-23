# 开发方案：一个 LLM 时代的 Rust 原生绘图库

<!-- English summary for international readers:
  This document is the top-level architecture & roadmap for plotine, written in Chinese.
  It covers: project positioning, LLM-era design constraints, layered architecture
  (recipes → render → backends), milestone breakdown (M0–M17), and open decisions.
  For an English overview of features and design choices, see the README and docs/MPL_GAP.md.
-->

> 状态：M0–M17 完成；首次 crates.io 发布版本 = **0.5.0**。  
> 本文档是项目的顶层设计文档。  
> **M9+ 功能对齐**：~~M9–M13~~；**M14–M17** = 输出格式 / 交互深化 / 生态薄层 / 文档社区（见 [`MPL_GAP.md`](MPL_GAP.md)）。

## 1. 项目定位

**一个高层 API、持续维护、质量优先的 Rust 原生静态绘图库。**

一句话定位：让 Rust 用户用 5 行代码画出可以直接放进论文/报告的图，并且让 AI 代理写这 5 行代码时不可能写错。

**2026-07-19 优先级决策**：在 WASM / 更多渲染后端 / §1.2 五项之前，**先对齐 Matplotlib 的静态出图**（视觉默认值 + 剩余 Axes 能力）。五项为 **M8 之后**的按序对齐里程碑，不抢当前主线。

### 1.1 Rust 绘图生态背景（2026-07）

| 库 | 定位 |
|---|---|
| egui_plot | 绑定 egui 运行时，交互为主；无独立静态导出 |
| 其他新库 | 多为特定领域工具或实验性项目 |

**plotine 的定位**：填补「高层科学出图 API + 静态导出 + 持续维护」这一空间——在 Rust 中提供类似 matplotlib 的开箱即用体验，同时利用类型系统保证 API 正确性。

### 1.2 后续功能对齐（M9+，按序）

文档与对标讨论一律按此顺序。主 API 仍是 Rust `Figure` builder；后两项以 **opt-in 旁路**对齐，不替换默认路径。

| 里程碑 | 项 | MVP 目标 |
|---|---|---|
| **M9** ✅ | 交互 GUI | `feature = "gui"`：`Figure::show` + pan/zoom + 3D elev/azim + 导出 PNG/SVG/PDF |
| **M10** ✅ | 动画 | `Figure::animate` / `Animation::map` → PNG 序列 / GIF（`feature = "gif"`；MP4 后置） |
| **M11** ✅ | 地理投影 | `GeoProjection::{PlateCarree,Mercator}` + NE 110m coastline（cartopy 薄层） |
| **M12** ✅ | pyplot facade | 独立 crate `plotine-pyplot`：`plot`/`subplots`/`savefig`…；TLS 状态；主 API 仍是 builder |
| **M13** ✅ | 外挂 LaTeX | `feature = "latex"`：`Figure::usetex` → 系统 `latex`+`dvipng` 位图嵌入；默认仍 `mathtext` |
| **M14** ✅ | 输出格式 | PGF 后端；EPS（`eps` + Ghostscript）；MP4（`mp4` + ffmpeg） |
| **M15** ✅ | 交互深化 | `show_nonblocking` + `show_with`（Slider/Button）；仍仅 egui |
| **M16** ✅ | 生态薄层 | `plotine::stats` + `ax.geojson`（非 seaborn/geopandas 全栈） |
| **M17** ✅ | 文档/社区基建 | `MPL_GAP.md`、mdBook 教程、`CONTRIBUTING`、Issue 模板 |

相对 matplotlib 四维分与非目标：[`docs/MPL_GAP.md`](MPL_GAP.md)。

**仍保留的约束（非上述里程碑）：**

- 不做符号表达式绘图 / 自适应采样（Maple、Mathematica 的 CAS 领域；我们是数值数组绘图库）
- ~~初期不做 3D~~（M7 已实现静态 3D：line/scatter/surface/wireframe/bar3d）
- 初期不做 Grammar of Graphics DSL（Charton 的路线；我们用 builder + recipes）
- **不做「无测试的图型堆砌」**：可以补齐 matplotlib 静态 2D 广度（§5 M6），但每图必须过 recipe 单测 + 视觉快照 + gallery；宁可不合并，也不合并半成品

### 1.3 与 Maple / Mathematica / matplotlib 的差距

Maple 与 Mathematica 是**计算机代数系统**，绘图输入是符号表达式（自适应采样、奇点检测由 CAS 承担），可视化只是符号计算的出口。matplotlib 与 plotine 是**数据绘图库**，输入是数值数组。因此与 Maple / Mathematica 的差距大部分是**物种差异**，不是「功能落后」。**当前主线对标的是 matplotlib 的静态 2D 出图能力**；交互 / 动画 / 投影 / pyplot / 外挂 LaTeX 见 §1.2 M9+。

#### 能力矩阵（量级对比）

| 维度 | plotine（现状 v0.5 / M17） | matplotlib | Mathematica | Maple |
|---|---|---|---|---|
| 输入模型 | 数值数组（`IntoSeries`） | 数值数组 + 单位/类别 | 符号表达式 + 数据 | 符号表达式 + 数据 |
| 符号函数绘图 | 无 | 无（需先采样） | 自适应采样、奇点处理 | 自适应采样、间断点处理 |
| 2D 图表广度 | ~44 项（见 `MPL_2D_COVERAGE.md`） | 40+（contour、quiver、pie、hexbin…） | 极广（Region / Density / Vector / Geo…） | 广（implicitplot、fieldplot…） |
| 3D | 静态 3D（line/scatter/surface/wireframe/bar3d/contour3d/quiver3d） | mplot3d | 完整交互式 3D + 光照 | 完整交互式 3D |
| 交互 / 动画 | GUI（阻塞 + 非阻塞 + widgets）+ `animate` / gif / mp4 | 多后端 + FuncAnimation + widgets | Manipulate / Dynamic | Explore / animate |
| 数学公式排版 | `mathtext` + `math::unicode`（默认）；`feature = "latex"` / `usetex` 可选 | mathtext + 可选 LaTeX | 原生数学排版 | 原生数学排版 |
| 注释 / 标注 | `text` / `annotate` + 箭头 | text、annotate、箭头 | Epilog / Callout / Labeled | textplot / caption |
| 坐标系与轴 | 线性 / log / symlog / 时间 / 极坐标 / twin / 类别 / geo | + 完整投影生态 | 任意几何变换 | 极坐标、复平面等 |
| 输出格式 | PNG / SVG / PDF / PGF；EPS（gs）；MP4（ffmpeg）≈**90**/100 | PNG / PDF / SVG / EPS / PGF… | 全格式 + CDF | 全格式 |
| 颜色系统 | 82 colormap + Linear/Log Norm + 3 主题 | 170+ colormap + Norm 体系 | 内置感知均匀配色 | 内置配色 |
| 布局 | subplots + GridSpec + mosaic + tight-layout | constrained layout、mosaic、跨格 | GraphicsGrid / 自动 | display 数组 / 自动 |
| 生态 | polars / ndarray / evcxr + stats 薄层 + GeoJSON ≈**35**/100 | pandas / seaborn / cartopy… | Wolfram 语言一体化 | Maple 一体化 |
| 文档/社区 | 技术文档 + 教程 + Issue 模板 ≈**40**/100 | 极丰富 SO / 书籍 | 商业文档 | 商业文档 |
| 许可 | MIT 开源 | 开源（BSD 风格） | 商业 | 商业 |

相对分详表：[`MPL_GAP.md`](MPL_GAP.md)。

plotine 现有图表：line / scatter / bar / barh / hist / area / fill_between / step / stairs / stem / hlines / vlines / axhline / axvline / axhspan / axvspan / polygon / pie / stackplot / eventplot / broken_barh / errorbar / heatmap / hist2d / hexbin / contour / contourf / pcolormesh / spy / quiver / barbs / streamplot / polar_line / polar_scatter / boxplot / violin。  
完整对标清单与批次状态：[`docs/MPL_2D_COVERAGE.md`](MPL_2D_COVERAGE.md)。

#### 差距的三种性质

把「差距」按性质拆开，才知道哪些需要行动：

1. **已排入 M9+ 的对齐项（§1.2，M8 之后按序行动）**  
   交互 GUI → 动画 → 地理投影 → pyplot facade → 外挂 LaTeX。CAS / 符号表达式绘图仍属物种差异、不排入。（静态 3D 已在 M7 实现。）

2. **静态 2D 广度缺口（§5 M6，已完成核心）**  
   matplotlib Axes 上约 40 种静态 2D 图型中，plotine 已覆盖核心项（含 `clabel` / `barbs`）。M6a–M6d 已合入：`fill_between` / step / stem → pie / stackplot → contour / hexbin → quiver / barbs / 极坐标。  
   **约束不变**：每图必须 recipe 可测 + 视觉快照；§2.3「正确性稀缺」优先于赶工。

3. **与「论文级出图」定位冲突的盲区（§5 M5，已完成）**  
   - 文本注释系统（`ax.text` / annotate / 箭头）——论文图几乎必备  
   - 最小数学排版（默认 `mathtext` / Unicode；完整外挂 LaTeX 见 M13）  
   - 双 Y 轴（twin axes）、手动刻度位置与标签、类别轴  
   - colormap 归一化（如 LogNorm）、**PDF 输出**（LaTeX `\includegraphics` 首选）

#### plotine 的核心优势

- **编译期正确性**：错误调用尽量无法编译；`PlotError` 带修复建议  
- **LLM 原生设计**：`llms.txt`、`AGENTS.md`、可预测命名（统一 builder 风格，无多套等价 API）  
- **确定性输出**：SVG 字节级可复现  
- **Rust 生态定位**：高层科学出图 + 静态导出 + 持续维护  

对标策略：**静态 2D 论文/报告出图**上同时追求（a）可发表默认质量（M5）与（b）matplotlib 级图型广度（M6），但绝不牺牲测试与视觉审查。

## 2. LLM 时代的设计约束（本项目的差异化根基）

调研确认了四个根本变化，每一条都直接转化为设计决策：

### 2.1 你的主要用户是 AI 代理

2026 年大部分调用本库的代码将由 AI 编写。设计含义：

- **错误的调用必须编译失败**。能用类型系统表达的约束绝不留到运行时。例：空图不能 render（类型状态）、对数轴不接受负数区间（构造时验证并返回 Result）。
- **每个函数一个清晰意图**。不要太粗（20 个参数难以正确填写）也不要太细（agent 需要编排 5 步序列容易出错）。`axes.line(&x, &y)` 一步出图，细节走 builder 链。
- **"delegate what, not how"**：默认值、自动刻度、自动布局、颜色循环全部内置。调用者声明意图，库负责机制。
- **命名可预测**。统一 `名词.动词` 风格，同一概念全库同名（如 `x_range` 而非多个等价别名）。
- **错误信息包含修复指令**。`PlotError::LogScaleNonPositive { value: -3.0, suggestion: "use ScaleType::Symlog, or filter/clip values so the domain is > 0" }` —— agent 读到就能自动修复。
- **用 `#[deprecated]` 主动引导**。模型训练数据有滞后，废弃 API 必须在编译期把 agent 推向新路径。

### 2.2 文档同时服务人类和模型

- 发布 `llms.txt`（索引）+ `llms-full.txt`（全文）。
- **Gallery 即文档即测试**：每种图表类型至少 3 个完整可运行示例，每个示例是一个 doctest/example，CI 保证永不过期。AI 从示例学 API 的效率远高于从 API reference。
- 随库发布 `AGENTS.md`：告诉 coding agent 本库的惯用法、常见错误、迁移指南。
- API 稳定性对 LLM 格外重要（训练数据滞后 6-18 个月）：1.0 前谨慎设计，1.0 后严格语义化版本。

### 2.3 功能数量不再稀缺，正确性才稀缺

生成大量图表代码的门槛已经极低，因此：

- **视觉回归测试是硬性要求**：每种图表、每个主题、每个后端都有像素级快照测试（insta + insta-image）。
- 刻度算法、文本测量、布局求解这些"不起眼但决定观感"的部分投入最多精力。
- 本项目使用 AI 辅助开发，但关键约束是：严格的测试基建 + 人工视觉审查每一张基准图。

### 2.4 持续维护是核心承诺

本项目按 1-2 年持续投入规划，路线图按季度滚动。

## 3. 架构设计

### 3.1 分层（借鉴 matplotlib 三层 + Makie recipes）

```
┌─────────────────────────────────────────────┐
│ chart API 层: Figure / Axes / builder 链     │  用户和 AI 代理接触的表面
├─────────────────────────────────────────────┤
│ recipes 层: bar/hist/boxplot → 原语组合       │  高级图表 = 原语的纯函数组合
├─────────────────────────────────────────────┤
│ scene 层: 原语场景树 (5-6 种原语)              │  Polyline/Marks/Polygon/Text/Image
├─────────────────────────────────────────────┤
│ layout 层: 刻度/图例/边距求解, 坐标变换管道      │  数据坐标→axes坐标→像素坐标
├─────────────────────────────────────────────┤
│ render trait: 后端无关的绘制接口               │  draw_path/draw_text/draw_image
├─────────────────────────────────────────────┤
│ backends: tiny-skia(PNG) / SVG / 未来 vello   │  feature flags 选择
└─────────────────────────────────────────────┘
```

**关键决策 1 — 原语 + recipes（来自 Makie）**：后端只需要渲染极少数原语。直方图不是一个"Artist 类"，而是一个纯函数：`(data, bins) → Vec<Polygon>`。收益：
- 后端实现成本极低（新后端只实现 5-6 个方法）
- recipes 可独立测试（输入数据，断言几何输出，不需要渲染）
- 用户可以用同样的原语扩展自定义图表类型

**关键决策 2 — 借鉴 matplotlib 的部分**：Figure/Axes 概念模型（用户心智模型已经建立）、Renderer trait 分离、Transform 管道。**不借鉴**：pyplot 全局状态、双重 API、set_* 方法群、Axes 上帝类、运行时 kwargs。

**关键决策 3 — 立即模式渲染，不做响应式**。Makie 的 ComputeGraph 解决的是交互式更新问题，我们初期只做静态导出，`figure.save("out.png")` 时一次性求值渲染。保持简单。

### 3.2 API 草案

```rust
use plotine::prelude::*;

fn main() -> plotine::Result<()> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();

    Figure::new()                          // 默认尺寸/主题开箱即用
        .axes(|ax| {
            ax.line(&x, &y)                // 一步出图
                .color(Color::CRIMSON)     // builder 链微调
                .width(2.0)
                .label("sin(x)");
            ax.title("Demo")
                .x_label("time (s)")
                .y_label("amplitude")
                .legend(Legend::TopRight);
        })
        .save("out.png")                   // 后缀推断后端
}
```

设计规则：

- **数据输入走 trait**：`impl IntoSeries`（接受 `&[f64]`、`Vec<f64>`、`&[f32]`、迭代器；`ndarray`/`polars` 走可选 feature）。
- **样式是强类型结构体**，不是字符串魔法（`Color::CRIMSON` 优先；同时提供 `Color::from_str` / hex 便利层）。
- **所有 fallible 操作返回 `Result<_, PlotError>`**，`PlotError` 每个变体带修复建议字段。
- **无全局状态**：Figure 拥有 Axes，Axes 拥有 plot elements，所有权链清晰。
- **默认即精品**：默认主题、默认配色（perceptually uniform，参考 CET/viridis）、默认字体必须达到发表级别，零配置出好图。

### 3.3 技术栈选型

| 组件 | 选择 | 理由 |
|---|---|---|
| 光栅渲染 | `tiny-skia` | 成熟稳定，纯 Rust，resvg/rizzma/kuva 等验证过 |
| 矢量输出 | 自研 SVG emitter | SVG 结构简单，自研可控制输出质量和文件体积 |
| GPU 渲染 | 预留 Renderer trait 接口，观望 `vello` | vello 仍 alpha（2026-07），不押注但保持接口兼容 |
| 文本 | `cosmic-text` | shaping/fallback/BiDi/emoji 全栈，Zed/Lapce 生产验证 |
| 几何 | `kurbo` | linebender 生态标准，Bezier/path 运算完备 |
| 字体内嵌 | DejaVu Sans (base64 subset) | 保证无系统字体环境（CI/docker/wasm）可用 |
| 快照测试 | `insta` + `insta-image` | 语义化 PNG 对比，跨压缩级别稳定 |
| 数据集成 | feature: `ndarray`, `polars` | 核心零依赖这些库，可选启用 |

依赖预算：默认 feature 下编译依赖 < 30 个 crate，干净 `cargo build` < 60s。臃肿的依赖树会劝退嵌入用户。

## 4. 质量与测试策略

1. **视觉回归快照**：每种图表 × 每个主题 × 每个后端一张基准图，insta-image 管理。基准图变更必须人工审查（PR 中渲染 diff）。
2. **Recipes 单元测试**：几何输出断言（直方图 bin 边界、箱线图分位数、刻度位置），不经过渲染，快速且精确。
3. **Doctest 全覆盖**：每个公开 API 的文档示例都可编译运行。
4. **刻度算法专项测试**：tick 选择是绘图库观感的灵魂，用 property-based testing（proptest）覆盖极端区间（1e-300、跨零、单点、逆序）。
5. **三平台 CI**：Linux/macOS/Windows + wasm32 编译检查。
6. **SVG 输出快照**：文本形式 insta 快照，保证矢量输出确定性（同输入字节级相同——可 diff、可缓存、可复现）。

## 5. 里程碑路线图

### M0 — 地基（约 4-6 周）✅
- [x] workspace 骨架：`plotine-core` / `plotine-render` / `plotine-text` / `plotine-backend-skia` / `plotine`
- [x] Renderer trait + tiny-skia 后端 + cosmic-text 集成 + 内嵌 DejaVu Sans
- [x] 坐标变换管道、线性 scale、Wilkinson 风格刻度算法
- [x] CI 三平台 + wasm32（core）编译检查
- [x] insta 视觉快照流水线（`assert_binary_snapshot` PNG；人工审查后入库）
- [x] Y 轴标签旋转（-90°）
- [x] **验收：空坐标系带刻度/标签/标题可导出 PNG**（`examples/empty_axes`）

### M1 — 核心五图（约 6-8 周）✅
- [x] line / scatter（builder + recipes + 自动坐标范围 + clip）
- [x] bar / histogram / area
- [x] legend（四角定位 + swatch）
- [x] errorbar（竖直误差棒 + cap）
- [x] grid、颜色循环、默认主题
- [x] `Figure::save` PNG 路径全通
- [x] gallery（`examples/gallery.rs`）
- [x] matplotlib 并排对比（`examples/matplotlib_compare` + `scripts/matplotlib_compare.py` → `./compare/index.html`；**84 对**）
- **验收（阶段性）：gallery 示例可一键生成**

### M2 — 矢量与布局（约 6 周）
- [x] SVG 后端（确定性输出，`plotine-backend-svg`）
- [x] subplots / grid layout（`Figure::subplots` + `GridSpec`；per-cell 边距）
- [x] tight-layout（同列共享 left/right、同行共享 top/bottom）
- [x] log / symlog scale
- [x] 日期时间轴（`DatetimeLocator`：日/月/年日历对齐刻度）
- [x] 主题系统（light / dark / paper）
- **验收（阶段性）：SVG + 多子图无重叠标签；M2 可视为功能完成**

### M3 — 生态集成（约 6 周）
- [x] heatmap + colorbar + colormap（Viridis / Plasma / Inferno / Magma / Cividis）
- [x] boxplot（Tukey）+ violin（Gaussian KDE）
- [x] `polars` / `ndarray` feature（`plotine::polars::xy` / `IntoSeries` / `heatmap_array`）
- [x] evcxr Jupyter 集成（`Figure::evcxr_display`，`feature = "evcxr"`）
- **验收：Polars 三行出图 + notebook 内联 PNG；M3 可视为功能完成**

### M4 — LLM 原生与发布（约 4 周）
- [x] llms.txt / llms-full.txt + AGENTS.md（仓库根目录）
- [x] 错误信息全面审查（每个变体带 suggestion；`PlotError::suggestion()` + 单元测试）
- [x] 视觉快照矩阵扩充（全图表类型 + dark/paper + 多份 SVG）
- [x] facade rustdoc 覆盖（`#![warn(missing_docs)]` + 公开 API 文档）
- [x] LICENSE（MIT）+ CI Clippy `-D warnings` 保持绿色
- [x] 文档站（mdBook：`book/` + CI `mdbook` job；docs.rs 随首次 publish 自动生成）
- [x] API 冻结审查（`docs/API_FREEZE.md` + guide「API stability」；空图保持运行时 `EmptyFigure`）
- [x] crates.io 发布节奏文档与脚本（`docs/RELEASING.md` + `scripts/publish.ps1`；每 4–6 周 minor）
- [ ] 首次 crates.io 上传 `0.5.0` + 验证 docs.rs + 打齐 `v0.5.0` tag（需维护者 `cargo login`；发布顺序见 `RELEASING.md` / `scripts/publish.ps1`）

### M5 — 论文级补齐（建议 0.3.x，与 M6 并行，1.0 前）

与「直接放进论文/报告」定位冲突的**非图型**能力；可与 M6 交错合入，不互相阻塞。

- [x] 文本注释：`ax.text` / `ax.annotate`（数据坐标）+ 可选箭头（gallery 34）
- [x] 最小数学排版：`plotine::math`（Unicode 希腊字母 + 上下标；`math::unicode` 轻量 TeX→Unicode；**无** LaTeX 布局；gallery 36）
- [x] 双轴：`ax.twin_y`（右 y；gallery 35）+ `ax.twin_x`（顶 x；gallery 39）
- [x] 手动刻度位置：`ax.x_ticks` / `ax.y_ticks`（既有）
- [x] PDF 输出：`plotine-backend-pdf`（SVG→`svg2pdf`，默认 feature；gallery 也写 `.pdf`）
- [x] 类别轴：`ax.x_categories` / `y_categories` + `category_indices`（`0..n`，对齐 mpl categorical；gallery 37）
- [x] colormap Norm：`Norm::{Linear,Log}` + `.norm(...)`（heatmap/hist2d/hexbin/contourf/pcolormesh；gallery 38）
- **验收**：gallery 34–40 + PDF；视觉快照槽位已加（Linux `insta review`）

### M6 — matplotlib 静态 2D 广度（建议 0.3.x–0.5.x，1.0 前）

清单与勾选状态以 [`docs/MPL_2D_COVERAGE.md`](MPL_2D_COVERAGE.md) 为准。目标：覆盖表中 ~40 项静态 2D 能力（含已有 9 图 + scale/heatmap 入口）。

#### M6a — 曲线与线段族（优先，复用 area/line 原语）✅

- [x] `fill_between` / `fill_betweenx`
- [x] `step`（pre / mid / post）+ `stairs`
- [x] `stem`
- [x] `hlines` / `vlines` + `axhline` / `axvline`
- [x] `barh`
- **验收**：gallery 21–24；recipe 单测 + Linux PNG 快照槽位；`llms.txt` / `AGENTS.md` / `MPL_2D_COVERAGE.md` 已更新（快照基准图需在 Linux 上 `cargo insta review` 入库）

#### M6b — 比例与事件族 ✅

- [x] `pie`
- [x] `stackplot`
- [x] `eventplot`
- [x] `broken_barh`
- [x] `polygon`（任意多边形 fill）+ `axhspan` / `axvspan`
- **验收**：gallery 25–28；recipe 单测 + Linux PNG 快照槽位；文档已更新

#### M6c — 场与密度族（计算稍重）✅

- [x] `hist2d` / `hexbin`（含 colorbar）
- [x] `contour` / `contourf`（Marching Squares）+ `.clabel(true)`（gallery 40）
- [x] `pcolormesh`
- [x] `spy`
- **验收**：gallery 29–31、40；contour 鞍点/标签单测；文档已更新

#### M6d — 矢量场与极坐标 ✅

- [x] `quiver`（+ `.quiverkey`）
- [x] `streamplot` + `barbs`（gallery 41；flag/full/half + calm 圆）
- [x] 极坐标：`polar_line` / `polar_scatter` / `polar_frame`（θ/r → 笛卡尔 + 极坐标网格；可与笛卡尔 subplot 混排）
- **验收**：gallery 32–33、41；recipe 单测；文档已更新

### M7 — 3D 绘图（0.3.x）✅

基于 mplot3d 模式（3D→2D 投影 + painter's algorithm 深度排序），复用现有 2D 渲染器。

- [x] `projection.rs`：Camera(elev/azim) + orthographic 投影 + 深度排序
- [x] `axes3d.rs`：Axes3D 结构体 + `plot3d` / `scatter3d` / `surface` / `wireframe` / `bar3d`
- [x] `draw3d.rs`：3D 渲染管线（立方体边框 + 刻度标签 + artist 绘制 + 图例）
- [x] `Figure::axes3d(|ax| { ... })` 入口
- [x] gallery 42–46：helix / scatter / surface / wireframe / bar3d
- **验收**：5 种 3D 图型可导出 PNG；painter's algorithm 正确遮挡；gallery 可一键生成

### M8 — 对齐 Matplotlib 静态出图（当前主线，建议 0.3.x）

> **原则**：视觉/行为对齐 stock matplotlib；API 保持 Rust builder（不引入 pyplot）。  
> **后置**：§1.2 M9–M13（交互 GUI → 动画 → 地理投影 → pyplot → 外挂 LaTeX）；WASM / vello 更后。

#### M8a — 视觉保真度（compare 驱动）✅/🔄

- [x] 文档同步：`MPL_2D_COVERAGE` / README / API_FREEZE / mdBook / `llms.txt` 与真实公开 API（含 3D）
- [x] 复杂图补视觉快照槽位：hexbin / streamplot / helix_3d / surface_3d（Linux `insta review`；Windows ignore）
- [x] 字段图默认关笛卡尔网格：`streamplot` / `quiver` / `spy`（与 heatmap/contour 一致）
- [x] `GridAxis::{Both,X,Y}` + compare bar/hist/barh 等 `axis="y"|"x"` 对齐
- [x] 线性刻度密度：`mpl_policy::ticks::LINEAR_TARGETS = 9`（更接近 MaxNLocator）
- [x] `compare/` 首轮全套扫描（45 对 MSE + 抽查）：修 hist `.bins` 迟滞 ymax、twin_x 标题叠层、subplots 去网格、polygon 显式范围
- [x] 尺寸感知刻度：`mpl_policy::ticks::auto_targets`（对齐 `AutoLocator`/`get_tick_space`）；`nice_number` round 步长贴近 `[1,2,5,10]`
- [x] hexbin extent：真正的 `nonsingular(expander=0.1)`；view 用 `ax.margins(0.05)`（mpl `tight=True` 仍加 margin；无 sticky_edges）
- [x] 刻度保真：近零格式化为 `0`；locator 用 ceiling nice（不小于 rough）；colorbar 整数跨度 step≈2
- [x] subplots 间距：`GridSpec` 改用 mpl `cell/(n+(n-1)·space)`；多 panel 内侧 chrome 并入 gap（避免双重留白）
- [x] datetime：`AUTOFMT_BOTTOM` + 加大旋转 tick 带（贴近 `autofmt_xdate`）
- [x] 3D limits：x/y = 5% + `VIEW_MARGIN(1/48)`；z 仅 `VIEW_MARGIN`；`TICK_TARGETS=11`（贴近 Axes3D AutoLocator）
- [x] twin：`chrome_expands_stock_insets` 不再为 twin 撑大 axes（对齐 mpl 保持 stock subplot box）
- [x] datetime：`AUTOFMT_BOTTOM` 为最终底边（不再叠加 xlabel）；旋转 tick 的 tip→label pad = 3.5pt
- [x] hist：默认无描边（mpl `edgecolor` 全透明）；显式 `.edgecolor(...)` 才描边
- [x] subplots：`tight_layout_for_grid` — 外缘 chrome → GridSpec margin，panel 等大 spine pad（axes≈cell）
- [x] 3D 视口：fit 进 mpl `Axes3D` 默认 subplot 分数 `(0.192,0.11,0.642×0.77)`；分轴 pane 色；刻度 `−1.00` 式
- [x] 刻度格式：`format_tick_with_step` — 分数步长保留小数；Unicode minus（U+2212）
- [x] 残余高 MSE：subplots / datetime / twin / categories → 归入 `mpl_policy` + tight_layout 内隙自动抬升 + 类别轴 `0..n`（2026-07-19：subplots ~2692→~1944、datetime ~1216→~1139、twin_x ~1110→~1106、twin_y ~1015→~977、categories ~1156→~526；`TITLE_TWIN_BASELINE_EM=0.90`、`TIGHT_PAD_TOP_FACTOR=0.85`、twin_y 专用 `Y_LABEL_INSET_TWIN_Y_PT`；subplots 残差以 panel 内容为主，datetime 以底/左 chrome/字体为主）
- [x] 样式长尾：`Legend::Best`（采样避让）+ `Hatch`（bar / barh / hist；gallery 65）
- [x] 样式长尾续：`grid_linestyle` + `title/x_label/y_label_fontsize` + `Legend::Outside*`（gallery 66）
- [x] 样式长尾续：图例 Line 手柄跟随 `linestyle` + `TickFormatter`（`fixed`/`percent`/`scientific`/`new`；gallery 67）
- [x] 样式长尾续：`rectangle` / `circle` / `ellipse` Patch（gallery 68）
- [x] 3D 一轮压 MSE（2026-07-19）：`FIT_SHRINK` 0.88→0.90；3D tick 去尾零。compare 均值 ~1373→~1333（surface −147、bar −238；scatter/gaussian 略升）

#### M8b — 误差与标注缺口

- [x] `errorbar` 水平误差：`.xerr(...)`（matplotlib `xerr=`）
- [x] 非对称误差：`.yerr_asym(lo, hi)` / `.xerr_asym(lo, hi)`（matplotlib `yerr`/`xerr` shape `(2, N)`）
- [x] annotate 箭头：点尺寸 FancyArrowPatch（`mpl_policy::annotate`），不再用 quiver 比例头
- [x] annotate `.arrow_style(ArrowStyle::{Triangle, Simple, Bracket, BothEnds})`（常用 Fancy 样式）
- [x] annotate styles 对齐 mpl：`BothEnds`=`<->` 双向开口 V；`mutation_scale=10`；`shrinkA/B=2`；`BracketB` 全宽=`2·widthB·ms`；wedge + round stroke；文字框 `patchA` 避让（2026-07-21）
- [x] Axes3D 刻度标签沿 stub 屏幕射线放置（修 z 透视漂移）；mathtext ∫ nestle/斜体 kern（2026-07-21）
- [x] heatmap `.extent([l,r,b,t])` + `.alpha(...)`（对齐 `imshow(extent=…, alpha=…)`）

#### M8c — 布局能力（论文高频）

- [x] `inset_axes([x0,y0,w,h], |inset| { ... })`：axes fraction（`transAxes`）；递归 `draw_panel`；支持一层嵌套；无 colorbar
- [ ] `inset_axes` 数据坐标定位 / figure fraction（按需）
- [x] `secondary_x` / `secondary_y`（+ `_linear`）：函数/仿射变换刻度；≠ twin；与同侧 twin 互斥
- [x] subplot 跨格：`g.at_span(row, col, rowspan, colspan, |ax| { … })`（简易 mosaic；完整 `subplot_mosaic` 另议）

#### M8d — 场/网格与排版（按需求穿插）

- [x] `tripcolor` / `tricontour`：显式 `.triangles([[i,j,k],…])`（无自动 Delaunay）
- [x] colormap：`Tab10` / `Coolwarm` / `RdBuR`（mpl `RdBu_r`）
- [x] mathtext 布局引擎（2026-07-19）：`plotine::mathtext` 解析 `$...$`，支持上下标 / `\frac` / 希腊与常用符号 / `\sin` 等；**不**依赖外挂 LaTeX。`math::unicode` 保留为纯字符串路径。
- [x] `ax.table`：单元格 + col/row labels + `TableLoc`（axes fraction）

#### M8e — 复杂排版加深（当前并行主线）

> **原则**：加深内置 mathtext；可选 CJK；外挂 LaTeX = **M13**（`feature = "latex"`；默认路径仍无外挂）。

- [x] `\sqrt{…}` / `\sqrt[n]{…}`：伸展根号（vinculum + radical），非仅 Unicode `√`
- [x] 基础矩阵：`\begin{matrix|pmatrix|bmatrix} … \end{…}`（`&` / `\\`）
- [x] 更多常用符号与间距微调：修复 `\,`/`\;`/`\:`/`\!` 空格解析；脚本/分数/矩阵间距收紧；parse 单测覆盖
- [x] 可选 feature `cjk`：`plotine::fonts::{load_system_cjk, register_font_file}`（不内嵌 Noto CJK；PDF 同步嵌入已注册字体）
- [x] gallery 52 + parse 单测覆盖根号/矩阵；（Linux）视觉快照待 `insta review`
- **验收**：论文常见 `$\\sqrt{…}$` / 小矩阵可排；无外挂 LaTeX；CJK 仅 feature 开启时可用

#### M8f — 静态 3D 对齐（当前并行主线）

> **原则**：先压 compare MSE，再加静态图型；交互旋转 / GUI 排入 **M9**（本里程碑仍为静态）。

- [x] 压低 3D compare MSE（第一轮）：`FIT_SHRINK=0.90` + tick 去尾零；`scripts/mse_3d.py` / `examples/refresh_3d_compare.rs`
- [x] scatter depthshade 改为 mpl `_zalpha`（调 alpha，非 RGB）；默认直径对齐 `s=16`
- [x] `FIT_SHRINK=0.92`（mean-MSE 甜点）；scatter MSE ~874（较首轮 ~1033）
- [x] helix/wireframe：线段 painter's algorithm（zsort average）+ Round cap/join；`FIT_SHRINK` 保持 0.92（2026-07-19：helix ~1464、wireframe ~1526；残余主要为 pane/刻度 chrome）
- [x] 3D smoke：scatter / wireframe / bar / contour3d / quiver3d
- [x] 补齐 3D 视觉快照测试入口（Linux）：wireframe / bar / scatter / contour / quiver（需 `cargo insta review` 入库）
- [x] 静态 `contour3d` / `quiver3d`（painter's algorithm；无交互；gallery 53/54）
- [x] 3D 刻度 chrome：mplot3d `highs` 选边 + 数据空间 `deltas=0.08` 外推 + inward/outward stub（2026-07-20）
- [ ] （按需）3D 线性以外的 scale —— 对齐 mpl 3.11 非线性 3D scales 的**静态**子集
- **验收**：3D compare 无「一眼假」；新图型有 recipe + gallery + 快照；文档写明交互旋转见 M9

**M8 验收**：compare 无「一眼假」的布局/刻度问题；M8b–M8c 每项均有 recipe 单测 + gallery +（Linux）快照；文档与 API_FREEZE 同步。

### M9–M13 — 按序功能对齐（§1.2；M8 之后）

> 顺序固定；完成前一项的 MVP 再开下一项。主 API 始终是 `Figure` builder。

#### M9 — 交互 GUI
- [x] egui/winit（`feature = "gui"`）窗口承载已渲染帧；阻塞式 `Figure::show()`
- [x] 2D pan/zoom（含框选 zoom、Home/Back/Forward、log/symlog 变换空间）；3D elev/azim 拖拽旋转 + 滚轮缩放
- [x] 从窗口再导出 PNG / SVG / PDF（工具栏 Save / `rfd`）
- [x] 示例：`cargo run -p plotine --example interactive_show --features gui`
- [x] 能力矩阵：[`docs/GUI_TOOLBAR.md`](GUI_TOOLBAR.md)（明确与 NavigationToolbar2 的差距）
- **验收**：交互示例可运行；文档说明与静态 `.save` 并存；不做 Configure Subplots / picking / `ion()` / 多 toolkit

#### M10 — 动画
- [x] 离线多帧 API（`FuncAnimation` 语义）：`Figure::animate` + `LinePlot::set_y` / `Animation::map`
- [x] PNG 序列（`save_png_sequence`）+ GIF（`feature = "gif"` / `save_gif`；MP4 后置）
- [x] 示例：`cargo run -p plotine --example animate_wave --features gif`
- [x] size_bench：与 mpl 统一 `figsize` / DPI / fps 后再比 GIF 体积
- **验收**：example + 单测（帧数 / 尺寸）；不依赖 M9 GUI 循环

#### M11 — 地理投影
- [x] `GeoProjection::{PlateCarree, Mercator}` + 嵌入 NE 110m coastline
- [x] Axes 级 `ax.projection(...)` / `ax.coastline()`（lon/lat → 投影平面后走 `line`/`scatter`）
- [x] gallery `69_geo_map` + `tests/m11_geo.rs` + `geo` 单测
- [x] size_bench：cartopy 110m 或同款 `coastline.bin` 公平对照（无数据则 SKIP）
- **验收**：gallery 地图示例 + 单测；非完整 GIS（无 shapefile / 多 CRS 混画）

#### M12 — pyplot facade
- [x] 独立 crate `plotine-pyplot`：`plot` / `scatter` / `subplots` / `xlabel` / `title` / `legend` / `savefig` / `clf` / `figure` / `sca`
- [x] 线程局部 gcf/gca；内部委托 `Figure`；`show` 可选 `features = ["gui"]`
- [x] 示例：`cargo run -p plotine-pyplot --example migrate_pyplot`
- **验收**：迁移示例 + 单测；不在 `plotine` default-members / 默认 feature 中

#### M13 — 外挂 LaTeX
- [x] 可选 `feature = "latex"`：`Figure::usetex(true)` → 系统 `latex` + `dvipng` → RGBA 嵌入（PNG/SVG/PDF）
- [x] 默认路径不变：内置 `mathtext` + `math::unicode`（无需 TeX）
- [x] 大算子 limits：默认 textstyle 侧标（对齐 mpl title）；`\displaystyle` / `\limits` 上下限
- [x] 错误：`PlotError::LatexUnavailable` / `LatexFailed` + `suggestion`
- [x] 示例：`cargo run -p plotine --example usetex_demo --features latex`；gallery `70_usetex`（有 TeX 时）
- **验收**：无 LaTeX 时清晰错误；有 TeX 时公式渲染；CI 不依赖系统 TeX

#### M14 — 输出格式（EPS / PGF / MP4）
- [x] `plotine-backend-pgf` + `Figure::save_pgf` / `.pgf`
- [x] `feature = "eps"`：PDF → Ghostscript `eps2write`
- [x] `feature = "mp4"`：`Animation::save_mp4` via `ffmpeg`
- [x] 示例 `export_formats` / `animate_wave`；测试 `tests/m14_formats.rs`
- **验收**：无 ffmpeg/gs 时 `ExternalToolUnavailable` + suggestion

#### M15 — 交互深化
- [x] `Figure::show_nonblocking` → `ShowHandle`
- [x] `Figure::show_with` + egui Slider/Button 侧栏
- [x] 示例 `interactive_widgets`；[`GUI_TOOLBAR.md`](GUI_TOOLBAR.md) 更新
- **验收**：不做 Qt/Tk/WebAgg 多后端

#### M16 — 生态薄层
- [x] `plotine::stats`：`corr_heatmap` / `pair_scatter` / `regline`
- [x] `Axes::geojson` / `geojson_path`（FeatureCollection 子集）
- [x] gallery 71–72；`tests/m16_eco.rs`
- **验收**：非完整 seaborn / geopandas

#### M17 — 文档 / 社区基建
- [x] [`MPL_GAP.md`](MPL_GAP.md) 四维相对分
- [x] mdBook tutorials（migrate / export / interactive）
- [x] `CONTRIBUTING.md` + `.github/ISSUE_TEMPLATE/`
- **验收**：不声称已有 SO 社区体量

#### 补充功能 — Norm / 自定义 colormap / stats 加深
- [x] `Norm::TwoSlope { vcenter }`（matplotlib `TwoSlopeNorm`）
- [x] `SegmentedColormap` + `Cmap`（named 或自定义 stops）；heatmap / hist2d / … 经 `impl Into<Cmap>`
- [x] `corr_heatmap` 单元格数值标注 + Coolwarm/`TwoSlope`；`regline` 95% CI `fill_between`
- [x] 差距分类写入 [`MPL_GAP.md`](MPL_GAP.md)（功能对比 + 设计选择 + 相对评分）

### 更后方向（按需求）

- wasm/浏览器 canvas 后端、vello GPU 后端（等它 beta）
- 加深 usetex（旋转标签、pdflatex/SVG 路径、预编译 preamble）
- `constrained_layout`（比 tight-layout 更稳的自动布局）
- M9–M13 像素对齐：`compare/plotine_m{9..13}_*.png` + `scripts/pixel_align_features.py`（MAE；Skia/Agg+字体决定无法 MAE=0）
- **超出范围**：多 GUI toolkit、完整 seaborn/geopandas 对等、大型社区体量（见 `MPL_GAP.md`）

## 6. 开发工作流约定

- 本项目大量使用 AI 辅助开发，但**每张基准快照图必须人工过目**后才能进版本库。
- 每个 PR 必须附带：受影响图表的渲染 diff、doctest、CHANGELOG 条目。
- 设计变更先改本文档，再写代码。
- 提交信息和代码注释用英文，设计文档用中文。

## 7. 已决问题与开放问题

### 已决

1. **库名：`plotine`**（2026-07-17）
   - 仓库 / crate / 模块统一为 `plotine`
   - 读音：plo-teen（plot + 轻快后缀）
   - 不暗示 matplotlib 兼容，避开商标风险；crates.io 可用（已确认）
   - 子 crate：`plotine-core` / `plotine-render` / `plotine-text` / `plotine-backend-skia`

### 开放

2. **cosmic-text vs parley**：Bevy 正在从 cosmic-text 迁往 parley（性能原因）。M0 用 cosmic-text（更成熟），但把文本测量封装在自己的 trait 后面，保留切换可能。
3. **MSRV 策略**：建议跟随 stable - 4 个版本（当前目标 MSRV = 1.85）。
4. **是否提供 MCP server**（让 agent 直接调用绘图）：暂缓，1.0 后评估。
5. **数学排版深度（已决）**：默认继续加深内置 `mathtext`（`\\sqrt` / 矩阵 / 符号）+ `math::unicode`；**外挂 LaTeX = M13 可选对齐项**；CJK 仅可选 feature。
6. **PDF 实现路径（已决）**：`plotine-backend-pdf` = 确定性 SVG + Typst `svg2pdf`（嵌入 DejaVu）；字节级稳定已有单测。
7. **3D 范围（已决）**：静态 mplot3d 子集可扩展（contour/quiver + 视觉对齐）；**交互旋转 = M9**（GUI 里程碑）。
8. **§1.2 五项（已完成）**：M9–M13 均已完成。
9. **M14–M17（已完成）**：输出格式 / 交互深化 / 生态薄层 / 文档社区已落地；下一优先 = crates.io 首次上传 → 其后 `constrained_layout` / mathtext·usetex 加深 / 视觉保真；WASM/vello 仍后置。
