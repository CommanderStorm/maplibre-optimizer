//! JSON-tree optimizations for MapLibre style documents.
//!
//! Operates on [`serde_json::Value`] so root keys not yet in generated `spec.rs` are preserved.

mod optimize;

use std::fs;
use std::path::Path;

use anyhow::Context;
use maplibre_style_spec::decoder::StyleReference;
use maplibre_style_spec::mir::IntermediateSpec;

pub use optimize::{OptPasses, optimize_style_json_value};

/// Load MIR from a MapLibre style reference `v8.json` on disk.
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
