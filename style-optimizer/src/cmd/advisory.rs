use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::Context;
use clap::Args;
use maplibre_style_optimizer::encode_mlt::mvt_to_mlt;
use maplibre_style_optimizer::prune::prune_tile;
use maplibre_style_optimizer::stats::collect::{available_zoom_levels, decode_tile, open_mbtiles};
use maplibre_style_optimizer::{TilePruningAdvisory, mbtiles};
use prost::Message;

/// Apply a tile pruning advisory to rewrite tiles and/or style.
///
/// Reads an advisory JSON produced by `optimize --advisory`, then:
/// - **`--tiles`**: reads an input `.mbtiles`, prunes MVT data per the advisory,
///   re-encodes as MLT, and writes a new `.mbtiles`.
/// - **`--style`**: rewrites the style JSON, setting `encoding: "mlt"` on
///   relevant vector sources.
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
    // Parse advisory.
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

    // Create output directory.
    fs::create_dir_all(&args.output)
        .with_context(|| format!("create output directory {}", args.output.display()))?;

    eprintln!("Advisory parsed: {} source(s).", advisory.sources.len());

    // Process tiles.
    if let Some(ref tiles_path) = args.tiles {
        process_tiles(tiles_path, &advisory, &args.output)?;
    }

    // Process style.
    if let Some(ref style_path) = args.style {
        process_style(style_path, &advisory, &args.output)?;
    }

    Ok(())
}

fn process_tiles(
    tiles_path: &Path,
    advisory: &TilePruningAdvisory,
    output_dir: &Path,
) -> anyhow::Result<()> {
    // We process each source in the advisory. For now, assume a single mbtiles input
    // corresponds to the first (or only) source in the advisory.
    let (source_name, source_advisory) = advisory
        .sources
        .iter()
        .next()
        .context("advisory has no sources")?;

    eprintln!("Processing source: {source_name}");

    let src_conn = open_mbtiles(tiles_path)?;
    let zooms = available_zoom_levels(&src_conn)?;

    let out_path = output_dir.join(
        tiles_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("tiles.mbtiles")),
    );
    let dst_conn = mbtiles::create_mbtiles(&out_path)?;

    // Copy metadata and override format.
    let has_metadata: bool = src_conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='metadata'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);
    if has_metadata {
        mbtiles::copy_metadata(&src_conn, &dst_conn)?;
    }
    mbtiles::set_metadata(&dst_conn, "format", "mlt")?;

    let mut total_in = 0u64;
    let mut total_out = 0u64;
    let mut tiles_written = 0u64;

    for zoom in &zooms {
        let z = i32::from(*zoom);

        // Check if this zoom is unused across ALL source-layers.
        let globally_unused = source_advisory
            .layers
            .values()
            .all(|la| la.unused_zoom_levels.contains(zoom))
            && source_advisory.unused_source_layers.len() + source_advisory.layers.len() > 0;

        if globally_unused {
            eprintln!("  z{zoom}: skipped (globally unused)");
            continue;
        }

        let mut stmt = src_conn
            .prepare("SELECT tile_column, tile_row, tile_data FROM tiles WHERE zoom_level = ?1")?;
        let rows = stmt.query_map([z], |row| {
            let col: i32 = row.get(0)?;
            let row_val: i32 = row.get(1)?;
            let data: Vec<u8> = row.get(2)?;
            Ok((col, row_val, data))
        })?;

        let mut zoom_in = 0u64;
        let mut zoom_out = 0u64;

        for row in rows {
            let (col, row_val, data) = row?;
            total_in += 1;
            zoom_in += 1;

            // Decode MVT.
            let mut tile = match decode_tile(&data) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("  warning: skipping tile z{zoom}/{col}/{row_val}: {e}");
                    continue;
                }
            };

            // Prune.
            prune_tile(&mut tile, source_advisory, *zoom);

            if tile.layers.is_empty() {
                continue;
            }

            // Re-encode pruned MVT to bytes.
            let mvt_bytes = tile.encode_to_vec();

            // Convert to MLT.
            match mvt_to_mlt(mvt_bytes) {
                Ok(encoded) => {
                    mbtiles::insert_tile(&dst_conn, z, col, row_val, &encoded)?;
                    zoom_out += 1;
                    total_out += 1;
                    tiles_written += 1;
                }
                Err(e) => {
                    eprintln!("  warning: MLT encode failed for z{zoom}/{col}/{row_val}: {e}");
                }
            }
        }

        eprintln!("  z{zoom}: {zoom_in} → {zoom_out} tiles");
    }

    eprintln!(
        "Tiles: {total_in} input → {total_out} output ({tiles_written} written to {})",
        out_path.display()
    );

    Ok(())
}

fn process_style(
    style_path: &Path,
    advisory: &TilePruningAdvisory,
    output_dir: &Path,
) -> anyhow::Result<()> {
    let style_text = fs::read_to_string(style_path)
        .with_context(|| format!("read style {}", style_path.display()))?;
    let mut style: serde_json::Value = serde_json::from_str(&style_text)
        .with_context(|| format!("parse style JSON {}", style_path.display()))?;

    // For each source in the advisory, set encoding="mlt" on matching vector sources.
    if let Some(sources) = style
        .as_object_mut()
        .and_then(|o| o.get_mut("sources"))
        .and_then(|s| s.as_object_mut())
    {
        for source_name in advisory.sources.keys() {
            if let Some(source) = sources.get_mut(source_name)
                && source.get("type").and_then(|t| t.as_str()) == Some("vector")
            {
                source.as_object_mut().expect("source is object").insert(
                    "encoding".to_string(),
                    serde_json::Value::String("mlt".to_string()),
                );
                eprintln!("Style: set encoding=\"mlt\" on source \"{source_name}\"");
            }
        }
    }

    let out_path = output_dir.join(
        style_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("style.json")),
    );

    let out_file = fs::File::create(&out_path)
        .with_context(|| format!("create output style {}", out_path.display()))?;
    let writer = io::BufWriter::new(out_file);
    serde_json::to_writer_pretty(writer, &style)
        .with_context(|| format!("write output style {}", out_path.display()))?;

    eprintln!("Style written to {}", out_path.display());

    Ok(())
}
