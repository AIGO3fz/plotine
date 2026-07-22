//! Pure geometry recipes: data → draw primitives (no rendering).
//!
//! These helpers convert raw data into paths/rects used by the figure renderer.
//! Most users never call them directly — prefer [`Axes`](crate::Axes) methods.

#![allow(missing_docs)]

pub mod annotate;
pub mod area;
pub mod bar;
pub mod barbs;
pub mod barh;
pub mod boxplot;
pub mod broken_barh;
pub mod contour;
pub mod errorbar;
pub mod eventplot;
pub mod fill_between;
pub mod heatmap;
pub mod hexbin;
pub mod hist;
pub mod hist2d;
pub mod line;
pub mod patches;
pub mod pcolormesh;
pub mod pie;
pub mod polar;
pub mod polygon;
pub mod quiver;
pub mod scatter;
pub mod spans;
pub mod spy;
pub mod stackplot;
pub mod stem;
pub mod step;
pub mod streamplot;
pub mod table;
pub mod tri;
pub mod violin;

pub use annotate::{annotation_arrow, annotation_arrow_styled, data_to_pixel, AnnotateTextBox};
pub use area::area_path;
pub use bar::{bar_rects, infer_bar_width, BarRect};
pub use barbs::{barb_components, barb_geoms, BarbGeom};
pub use barh::{barh_rects, infer_barh_height};
pub use boxplot::{boxplot_stats, BoxStats};
pub use broken_barh::broken_barh_rects;
pub use contour::{
    auto_levels, contour_labels, contour_level_segments, contour_paths, contourf_fills,
    format_contour_level, nice_levels, segment_in_label_gap, ContourFill, ContourLabel,
    ContourPath, ContourSegment,
};
pub use errorbar::{
    errorbar_geoms, errorbar_geoms_asym, errorbar_x_geoms, errorbar_x_geoms_asym, ErrorBarGeom,
    ErrorBarXGeom,
};
pub use eventplot::eventplot_segments;
pub use fill_between::{fill_between_path, fill_betweenx_path};
pub use heatmap::{heatmap_cells, heatmap_limits, HeatmapCell, HeatmapOrigin};
pub use hexbin::{hexbin_cells, hexbin_extent, HexCell};
pub use hist::{histogram, HistogramBins};
pub use hist2d::{hist2d_bins, hist2d_cells, hist2d_limits, Hist2dBins};
pub use line::line_path;
pub use patches::{circle_path, ellipse_path, rectangle_data_rect, rectangle_pixel_rect};
pub use pcolormesh::{pcolormesh_cells, pcolormesh_limits};
pub use pie::{pie_wedges, PieWedge};
pub use polar::{
    polar_angle_labels, polar_frame_paths, polar_radial_labels, polar_rings, polar_to_cartesian,
    PolarLabel, PolarLabelAlign, PolarLabelBaseline,
};
pub use polygon::polygon_path;
pub use quiver::{infer_quiver_scale, quiver_arrows, QuiverArrow};
pub use scatter::{marker_path, scatter_markers, Marker};
pub use spans::{axhspan_rect, axvspan_rect, hline_segments, vline_segments, SpanSegment};
pub use spy::spy_markers;
pub use stackplot::{stackplot_paths, stackplot_ymax};
pub use stem::{stem_geoms, StemGeom};
pub use step::{stairs_path, step_path, StepMode};
pub use streamplot::{streamlines, streamlines_count, Streamline};
pub use table::{table_cell_geoms, TableCellGeom, TableLoc};
pub use tri::{
    delaunay, resolve_tri_levels, tri_limits, tricontour_paths, tricontourf_fills,
    tripcolor_face_limits, tripcolor_fills, validate_triangles, TriFill,
};
pub use violin::{violin_geoms, violin_path, ViolinGeom};
