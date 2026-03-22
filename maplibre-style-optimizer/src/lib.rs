//! Typed optimizations for `MapLibre` style documents.
//!
//! The primary entry point is [`optimize_style`], which operates on the typed
//! [`maplibre_style_spec::spec::MaplibreStyleSpecification`].
//!
//! JSON wrappers [`optimize_style_json_value`] / [`optimize_style_json_value_with_stats`]
//! are provided for backward compatibility.

pub mod advisory;
mod eval;
mod optimize;
pub mod stats;

use std::fs;
use std::path::Path;

use anyhow::Context;
use maplibre_style_spec::decoder::StyleReference;
use maplibre_style_spec::mir::IntermediateSpec;
pub use optimize::{
    OptPasses, optimize_style, optimize_style_json_value, optimize_style_json_value_with_stats,
};
pub use stats::TileStatistics;

use crate::advisory::DataRewriteAdvisory;
use crate::optimize::field_analysis::analyze_fields;
use crate::optimize::source_util::precompute_vector_layer_info;

/// Load MIR from a `MapLibre` style reference `v8.json` on disk.
pub fn load_intermediate_spec_from_v8_path(path: &Path) -> anyhow::Result<IntermediateSpec> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read reference {}", path.display()))?;
    let reference: StyleReference = serde_json::from_str(&text)
        .with_context(|| format!("parse reference {}", path.display()))?;
    Ok(IntermediateSpec::from(reference))
}

/// Ensure the reference defines an expression operator (sanity check against wrong `v8.json`).
pub fn ensure_expression_operator(mir: &IntermediateSpec, name: &str) -> anyhow::Result<()> {
    if mir.expressions.operators.contains_key(name) {
        Ok(())
    } else {
        anyhow::bail!("reference MIR missing expression operator {name:?}");
    }
}

/// Compute a data rewrite advisory from a style and tile statistics.
///
/// The advisory describes data transformations (column drops, string→integer encodings,
/// row filtering) that pair with a rewritten style to produce equivalent rendering.
pub fn compute_data_advisory(
    style: &mut serde_json::Value,
    mir: &IntermediateSpec,
    stats: &TileStatistics,
) -> DataRewriteAdvisory {
    let layer_info = precompute_vector_layer_info(style);
    let field_analysis = analyze_fields(style, mir, &layer_info);
    advisory::compute_advisory(&field_analysis, stats)
}

/// Apply a data advisory's expression rewrites to a style.
///
/// After this, the style assumes the tile data has been transformed per the advisory.
pub fn apply_data_advisory(
    style: &mut serde_json::Value,
    mir: &IntermediateSpec,
    advisory: &DataRewriteAdvisory,
) {
    let layer_info = precompute_vector_layer_info(style);
    optimize::advisory_rewrite::apply_advisory(style, mir, &layer_info, advisory);
}
