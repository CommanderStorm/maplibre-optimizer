use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use maplibre_style_optimizer::{
    OptPasses, TileStatistics, compute_advisory, ensure_expression_operator,
    load_intermediate_spec_from_v8_path, optimize_style_json_value_with_stats,
};
use maplibre_style_spec::validate::validate_style_value;

/// Optimize a `MapLibre` style JSON document (preserves unmodeled root keys).
#[derive(Parser, Debug)]
#[command(name = "maplibre-style-optimize", version)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
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

    /// Remove layers with always-false filters and unused sources.
    #[arg(long)]
    dead_elimination: bool,

    /// Tighten `minzoom`/`maxzoom` from `["zoom"]` predicates inside filters.
    /// Also removes zoom predicates that are fully captured by the extracted bounds.
    #[arg(long)]
    metadata_refinement: bool,

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

    /// Remove empty `paint`/`layout` objects, `visibility:none` layers, and zero-opacity layers.
    #[arg(long)]
    cleanup: bool,

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reference_path = cli.reference.unwrap_or_else(default_reference_path);

    let mir = load_intermediate_spec_from_v8_path(&reference_path)?;
    for op in ["any", "all", "!"] {
        ensure_expression_operator(&mir, op)?;
    }

    let json_text =
        fs::read_to_string(&cli.input).with_context(|| cli.input.display().to_string())?;
    let mut value: serde_json::Value = serde_json::from_str(&json_text)
        .with_context(|| format!("parse style JSON {}", cli.input.display()))?;

    let tile_stats = cli
        .stats
        .map(|path| {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("read stats {}", path.display()))?;
            let stats: TileStatistics = serde_json::from_str(&text)
                .with_context(|| format!("parse stats JSON {}", path.display()))?;
            Ok::<_, anyhow::Error>(stats)
        })
        .transpose()?;

    let passes = if cli.all {
        OptPasses::all()
    } else {
        OptPasses {
            simplify_unary: cli.simplify_unary,
            expression_kind: cli.expression_kind,
            constant_fold: cli.constant_fold,
            dead_elimination: cli.dead_elimination,
            metadata_refinement: cli.metadata_refinement,
            selectivity_reorder: cli.selectivity_reorder,
            strip_metadata: cli.strip_metadata,
            strip_defaults: cli.strip_defaults,
            simplify_expressions: cli.simplify_expressions,
            cleanup: cli.cleanup,
        }
    };
    optimize_style_json_value_with_stats(&mut value, &mir, &passes, tile_stats.as_ref());

    if let Some(advisory_path) = &cli.advisory {
        let stats = tile_stats
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--advisory requires --stats"))?;
        let advisory = compute_advisory(&value, stats);
        let advisory_json = serde_json::to_string_pretty(&advisory)?;
        fs::write(advisory_path, advisory_json)
            .with_context(|| advisory_path.display().to_string())?;
    }

    if cli.validate {
        validate_style_value(&value).map_err(anyhow::Error::msg)?;
    }

    let out = if cli.pretty {
        serde_json::to_string_pretty(&value)?
    } else {
        serde_json::to_string(&value)?
    };
    fs::write(&cli.output, out).with_context(|| cli.output.display().to_string())?;

    Ok(())
}
