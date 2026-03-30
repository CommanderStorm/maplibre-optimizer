use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use maplibre_style_optimizer::complexity::complexity_report;
use maplibre_style_optimizer::load_intermediate_spec_from_v8_path;

/// Compute static complexity metrics for a style JSON document.
#[derive(Args, Debug)]
pub struct ComplexityArgs {
    /// Input style JSON path.
    #[arg(long)]
    input: PathBuf,

    /// Path to `v8.json` style reference (defaults to repo `upstream/src/reference/v8.json`).
    #[arg(long)]
    reference: Option<PathBuf>,
}

fn default_reference_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json")
}

pub fn run(args: &ComplexityArgs) -> anyhow::Result<()> {
    let reference_path = args.reference.clone().unwrap_or_else(default_reference_path);
    let mir = load_intermediate_spec_from_v8_path(&reference_path)?;

    let json_text =
        fs::read_to_string(&args.input).with_context(|| args.input.display().to_string())?;
    let mut value: serde_json::Value = serde_json::from_str(&json_text)
        .with_context(|| format!("parse style JSON {}", args.input.display()))?;

    let report = complexity_report(&mut value, &mir);
    let out = serde_json::to_string(&report)?;
    println!("{out}");

    Ok(())
}
