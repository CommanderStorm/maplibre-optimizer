use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use maplibre_style_optimizer::{
    OptPasses, TileStatistics, compute_advisory, ensure_expression_operator,
    load_intermediate_spec_from_v8_path, optimize_style_json_value_with_stats,
};
use maplibre_style_spec::validate::validate_style_value;

/// Optimize a `MapLibre` style JSON document (preserves unmodeled root keys).
#[derive(Args, Debug)]
#[expect(clippy::struct_excessive_bools)]
pub struct OptimizeArgs {
    /// Input style JSON path.
    #[arg(long)]
    input: PathBuf,

    /// Output style JSON path.
    #[arg(long)]
    output: PathBuf,

    /// Path to `v8.json` style reference (defaults to repo `upstream/src/reference/v8.json`).
    #[arg(long)]
    reference: Option<PathBuf>,

    /// Load pre-computed `TileStatistics` JSON and enable data-driven passes.
    ///
    /// The stats file must use the same source key names as the style's `"sources"` map.
    /// This does not enable any new pass flags; it enriches the behavior of existing
    /// passes that already have their flags set.
    #[arg(long)]
    stats: Option<PathBuf>,

    /// Enable all optimization passes (overrides individual flags).
    #[arg(long)]
    all: bool,

    /// Simplify unary boolean ops: `["any"|"all", e]` → `e`, `["!",["!",e]]` → `e`.
    #[arg(long)]
    simplify_unary: bool,

    /// Rewrite negated comparisons to `!=` / `==` when MIR defines those operators.
    #[arg(long)]
    expression_kind: bool,

    /// Fold constant comparisons, boolean `any`/`all`/`!`, arithmetic, strings, and colors.
    #[arg(long)]
    constant_fold: bool,

    /// Enable stats-driven constant folds (requires --stats for effect).
    #[arg(long)]
    constant_fold_stats: bool,

    /// Remove layers with always-false filters and unused sources.
    #[arg(long)]
    dead_elimination: bool,

    /// Enable stats-driven dead layer removal (empty source-layer, geometry mismatch).
    #[arg(long)]
    dead_elimination_stats: bool,

    /// Tighten `minzoom`/`maxzoom` from `["zoom"]` predicates inside filters.
    /// Also removes zoom predicates that are fully captured by the extracted bounds.
    #[arg(long)]
    metadata_refinement: bool,

    /// Derive minzoom from paint-property visibility analysis.
    #[arg(long)]
    metadata_refinement_paint: bool,

    /// Tighten zoom bounds from tile statistics (requires --stats for effect).
    #[arg(long)]
    metadata_refinement_stats: bool,

    /// Reorder `any`/`all` operands for static short-circuit hints (literals first/last).
    #[arg(long)]
    selectivity_reorder: bool,

    /// Remove `metadata` keys from style root and layers.
    #[arg(long)]
    strip_metadata: bool,

    /// Remove paint/layout properties that equal their spec-defined default values.
    #[arg(long)]
    strip_defaults: bool,

    /// Simplify `interpolate`/`step` with identical stops and deduplicate `match` arms.
    #[arg(long)]
    simplify_expressions: bool,

    /// Minify CSS color strings to their shortest hex/rgba representation.
    #[arg(long)]
    minify_colors: bool,

    /// Remove empty `paint`/`layout` objects, `visibility:none` layers, and zero-opacity layers.
    #[arg(long)]
    cleanup: bool,

    /// Merge adjacent layers with the same type/source/source-layer into fewer
    /// layers using data-driven `case`/`match` expressions and synthesised sort-keys.
    #[arg(long)]
    layer_merge: bool,

    /// Tighten source minzoom/maxzoom based on the effective zoom range of
    /// referencing layers.
    #[arg(long)]
    source_zoom_tightening: bool,

    /// Run JSON-tree validation after optimization (`maplibre_style_spec::validate`).
    #[arg(long)]
    validate: bool,

    /// Pretty-print JSON output (default: compact).
    #[arg(long)]
    pretty: bool,

    /// Write a tile pruning advisory JSON to this path (requires --stats).
    #[arg(long)]
    advisory: Option<PathBuf>,
}

fn default_reference_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json")
}

pub fn run(args: OptimizeArgs) -> anyhow::Result<()> {
    let reference_path = args.reference.unwrap_or_else(default_reference_path);

    let mir = load_intermediate_spec_from_v8_path(&reference_path)?;
    for op in ["any", "all", "!"] {
        ensure_expression_operator(&mir, op)?;
    }

    let json_text =
        fs::read_to_string(&args.input).with_context(|| args.input.display().to_string())?;
    let mut value: serde_json::Value = serde_json::from_str(&json_text)
        .with_context(|| format!("parse style JSON {}", args.input.display()))?;

    let tile_stats = args
        .stats
        .map(|path| {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("read stats {}", path.display()))?;
            let stats: TileStatistics = serde_json::from_str(&text)
                .with_context(|| format!("parse stats JSON {}", path.display()))?;
            Ok::<_, anyhow::Error>(stats)
        })
        .transpose()?;

    let passes = if args.all {
        OptPasses::all()
    } else {
        OptPasses {
            simplify_unary: args.simplify_unary,
            expression_kind: args.expression_kind,
            constant_fold: args.constant_fold,
            constant_fold_stats: args.constant_fold_stats,
            dead_elimination: args.dead_elimination,
            dead_elimination_stats: args.dead_elimination_stats,
            metadata_refinement: args.metadata_refinement,
            metadata_refinement_paint: args.metadata_refinement_paint,
            metadata_refinement_stats: args.metadata_refinement_stats,
            selectivity_reorder: args.selectivity_reorder,
            strip_metadata: args.strip_metadata,
            strip_defaults: args.strip_defaults,
            simplify_expressions: args.simplify_expressions,
            minify_colors: args.minify_colors,
            cleanup: args.cleanup,
            layer_merge: args.layer_merge,
            source_zoom_tightening: args.source_zoom_tightening,
        }
    };
    optimize_style_json_value_with_stats(&mut value, &mir, &passes, tile_stats.as_ref());

    if let Some(advisory_path) = &args.advisory {
        let stats = tile_stats
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--advisory requires --stats"))?;
        let advisory = compute_advisory(&value, stats);
        let advisory_json = serde_json::to_string_pretty(&advisory)?;
        fs::write(advisory_path, advisory_json)
            .with_context(|| advisory_path.display().to_string())?;
    }

    if args.validate {
        validate_style_value(&value).map_err(anyhow::Error::msg)?;
    }

    let out = if args.pretty {
        serde_json::to_string_pretty(&value)?
    } else {
        serde_json::to_string(&value)?
    };
    fs::write(&args.output, out).with_context(|| args.output.display().to_string())?;

    Ok(())
}
