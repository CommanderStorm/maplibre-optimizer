use std::path::PathBuf;
use std::{fs, process};

use anyhow::Context;
use clap::Args;
use maplibre_style_optimizer::TilePruningAdvisory;

/// Apply a tile pruning advisory to rewrite tiles and/or style.
///
/// **Not yet implemented.** This scaffold validates inputs and confirms the advisory
/// parses correctly, but exits with code 1 without performing transforms.
#[derive(Args, Debug)]
pub struct AdvisoryArgs {
    /// Path to the advisory JSON (output of `optimize --advisory`).
    #[arg(long)]
    advisory: PathBuf,

    /// Path to the input `.mbtiles` file to rewrite.
    #[arg(long)]
    tiles: Option<PathBuf>,

    /// Path to the input style JSON to rewrite.
    #[arg(long)]
    style: Option<PathBuf>,

    /// Output directory for rewritten tiles and/or style.
    #[arg(long)]
    output: PathBuf,
}

pub fn run(args: &AdvisoryArgs) -> anyhow::Result<()> {
    // Validate inputs exist.
    let advisory_text = fs::read_to_string(&args.advisory)
        .with_context(|| format!("read advisory {}", args.advisory.display()))?;
    let advisory: TilePruningAdvisory = serde_json::from_str(&advisory_text)
        .with_context(|| format!("parse advisory JSON {}", args.advisory.display()))?;

    if let Some(ref tiles) = args.tiles {
        anyhow::ensure!(tiles.exists(), "tiles file not found: {}", tiles.display());
    }
    if let Some(ref style) = args.style {
        anyhow::ensure!(style.exists(), "style file not found: {}", style.display());
    }
    anyhow::ensure!(
        args.tiles.is_some() || args.style.is_some(),
        "--tiles and/or --style must be specified"
    );

    eprintln!(
        "Advisory parsed successfully ({} source(s)).",
        advisory.sources.len()
    );
    eprintln!(
        "Advisory application transforms (MVT rewriting, string interning) are not yet implemented."
    );

    process::exit(1);
}
