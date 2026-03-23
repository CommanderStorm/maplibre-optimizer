//! Typed optimizations for `MapLibre` style documents.
//!
//! The primary entry point is [`optimize_style`], which operates on the typed
//! [`maplibre_style_spec::spec::MaplibreStyleSpecification`].
//!
//! JSON wrappers [`optimize_style_json_value`] / [`optimize_style_json_value_with_stats`]
//! are provided for backward compatibility.

pub mod advisory;
pub mod encode_mlt;
pub mod mbtiles;
#[expect(
    clippy::doc_markdown,
    clippy::trivially_copy_pass_by_ref,
    clippy::must_use_candidate
)]
pub mod mvt;
mod optimize;
pub mod prune;
pub mod stats;

use std::fs;
use std::path::Path;

pub use advisory::{TilePruningAdvisory, compute_advisory};
use anyhow::Context;
use maplibre_style_spec::decoder::StyleReference;
use maplibre_style_spec::mir::MirSpec;
pub use optimize::{
    OptPasses, optimize_style, optimize_style_json_value, optimize_style_json_value_with_stats,
};
pub use stats::TileStatistics;
pub use stats::collect::collect_statistics;

/// Load MIR from a `MapLibre` style reference `v8.json` on disk.
pub fn load_intermediate_spec_from_v8_path(path: &Path) -> anyhow::Result<MirSpec> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read reference {}", path.display()))?;
    let reference: StyleReference = serde_json::from_str(&text)
        .with_context(|| format!("parse reference {}", path.display()))?;
    Ok(MirSpec::from(reference))
}

/// Ensure the reference defines an expression operator (sanity check against wrong `v8.json`).
pub fn ensure_expression_operator(mir: &MirSpec, name: &str) -> anyhow::Result<()> {
    if mir.expressions.operators.contains_key(name) {
        Ok(())
    } else {
        anyhow::bail!("reference MIR missing expression operator {name:?}");
    }
}
