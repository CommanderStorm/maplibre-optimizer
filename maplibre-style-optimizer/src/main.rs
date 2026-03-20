use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use maplibre_style_optimizer::{
    OptPasses, ensure_expression_operator, load_intermediate_spec_from_v8_path,
    optimize_style_json_value,
};
use maplibre_style_spec::validate::validate_style_value;

/// Optimize a MapLibre style JSON document (preserves unmodeled root keys).
#[derive(Parser, Debug)]
#[command(name = "maplibre-style-optimize", version)]
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

    /// Fold `["any", expr]` to `expr`.
    #[arg(long)]
    fold_unary_any: bool,

    /// Run JSON-tree validation after optimization (`maplibre_style_spec::validate`).
    #[arg(long)]
    validate: bool,

    /// Pretty-print JSON output (default: compact).
    #[arg(long)]
    pretty: bool,
}

fn default_reference_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json")
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reference_path = cli.reference.unwrap_or_else(default_reference_path);

    let mir = load_intermediate_spec_from_v8_path(&reference_path)?;
    ensure_expression_operator(&mir, "any")?;

    let json_text =
        fs::read_to_string(&cli.input).with_context(|| cli.input.display().to_string())?;
    let mut value: serde_json::Value = serde_json::from_str(&json_text)
        .with_context(|| format!("parse style JSON {}", cli.input.display()))?;

    let passes = OptPasses {
        fold_unary_any: cli.fold_unary_any,
    };
    optimize_style_json_value(&mut value, &mir, &passes);

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
