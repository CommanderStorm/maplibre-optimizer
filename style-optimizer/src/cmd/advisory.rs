use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::Context;
use clap::Args;
use maplibre_style_optimizer::encode_mlt::mvt_to_mlt;
use maplibre_style_optimizer::prune::{intern_string_properties, prune_tile};
use maplibre_style_optimizer::stats::collect::{available_zoom_levels, decode_tile, open_mbtiles};
use maplibre_style_optimizer::{TilePruningAdvisory, mbtiles};
use prost::Message;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// Number of tiles to read, process in parallel, then write back per batch.
const TILE_BATCH_SIZE: usize = 1024;

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

        let (zoom_in, zoom_out) =
            process_zoom(&src_conn, &dst_conn, z, *zoom, source_advisory)?;

        total_in += zoom_in;
        total_out += zoom_out;
        tiles_written += zoom_out;
        eprintln!("  z{zoom}: {zoom_in} → {zoom_out} tiles");
    }

    eprintln!(
        "Tiles: {total_in} input → {total_out} output ({tiles_written} written to {})",
        out_path.display()
    );

    Ok(())
}

/// Process all tiles at a single zoom level in batches, returning `(input_count, output_count)`.
fn process_zoom(
    src_conn: &rusqlite::Connection,
    dst_conn: &rusqlite::Connection,
    z: i32,
    zoom: u8,
    source_advisory: &maplibre_style_optimizer::advisory::SourceAdvisory,
) -> anyhow::Result<(u64, u64)> {
    let mut stmt =
        src_conn.prepare("SELECT tile_column, tile_row, tile_data FROM tiles WHERE zoom_level = ?1")?;
    let mut rows = stmt.query([z])?;

    let mut zoom_in = 0u64;
    let mut zoom_out = 0u64;

    loop {
        // Read a batch of raw tiles from SQLite.
        let mut batch: Vec<(i32, i32, Vec<u8>)> = Vec::with_capacity(TILE_BATCH_SIZE);
        while batch.len() < TILE_BATCH_SIZE {
            let Some(row) = rows.next()? else { break };
            let col: i32 = row.get(0)?;
            let row_val: i32 = row.get(1)?;
            let data: Vec<u8> = row.get(2)?;
            batch.push((col, row_val, data));
        }
        if batch.is_empty() {
            break;
        }
        zoom_in += batch.len() as u64;

        // Process the batch in parallel: decode → prune → intern → encode → MLT.
        let results: Vec<_> = batch
            .into_par_iter()
            .filter_map(|(col, row_val, data)| {
                process_single_tile(&data, col, row_val, zoom, source_advisory)
            })
            .collect();

        // Write results to SQLite inside a transaction.
        let tx = dst_conn.unchecked_transaction()?;
        for (col, row_val, encoded) in &results {
            mbtiles::insert_tile(&tx, z, *col, *row_val, encoded)?;
        }
        tx.commit()?;

        zoom_out += results.len() as u64;
    }

    Ok((zoom_in, zoom_out))
}

/// Decode, prune, intern, and re-encode a single tile. Returns `None` if the tile
/// is empty after pruning or if encoding fails.
fn process_single_tile(
    data: &[u8],
    col: i32,
    row_val: i32,
    zoom: u8,
    source_advisory: &maplibre_style_optimizer::advisory::SourceAdvisory,
) -> Option<(i32, i32, Vec<u8>)> {
    let mut tile = match decode_tile(data) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("  warning: skipping tile z{zoom}/{col}/{row_val}: {e}");
            return None;
        }
    };

    prune_tile(&mut tile, source_advisory, zoom);

    for layer in &mut tile.layers {
        if let Some(la) = source_advisory.layers.get(&layer.name) {
            intern_string_properties(layer, &la.interned_properties);
        }
    }

    if tile.layers.is_empty() {
        return None;
    }

    let mvt_bytes = tile.encode_to_vec();
    match mvt_to_mlt(mvt_bytes) {
        Ok(encoded) => Some((col, row_val, encoded)),
        Err(e) => {
            eprintln!("  warning: MLT encode failed for z{zoom}/{col}/{row_val}: {e}");
            None
        }
    }
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

    // Rewrite filter expressions to use interned integer values.
    rewrite_style_interning(&mut style, advisory);

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

/// Rewrite filter expressions in the style to replace interned string literals with integers.
fn rewrite_style_interning(style: &mut serde_json::Value, advisory: &TilePruningAdvisory) {
    // Build lookup: source_name → source_layer → prop_name → interning table.
    let lookup: BTreeMap<&str, BTreeMap<&str, &BTreeMap<String, Vec<String>>>> = advisory
        .sources
        .iter()
        .map(|(src, sa)| {
            let layer_map = sa
                .layers
                .iter()
                .filter(|(_, la)| !la.interned_properties.is_empty())
                .map(|(sl, la)| (sl.as_str(), &la.interned_properties))
                .collect();
            (src.as_str(), layer_map)
        })
        .collect();

    let Some(layers) = style
        .as_object_mut()
        .and_then(|o| o.get_mut("layers"))
        .and_then(|v| v.as_array_mut())
    else {
        return;
    };

    for layer in layers {
        let Some(obj) = layer.as_object_mut() else {
            continue;
        };

        // Resolve source and source-layer for this style layer.
        let source = obj
            .get("source")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let source_layer = obj
            .get("source-layer")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        let Some(interned_props) = lookup.get(source).and_then(|m| m.get(source_layer)) else {
            continue;
        };

        // Rewrite the filter expression.
        if let Some(filter) = obj.get_mut("filter") {
            rewrite_filter_interning(filter, interned_props);
        }
    }
}

/// Recursively rewrite a filter expression, replacing interned string literals with integers.
fn rewrite_filter_interning(
    expr: &mut serde_json::Value,
    interned: &BTreeMap<String, Vec<String>>,
) {
    let serde_json::Value::Array(arr) = expr else {
        return;
    };
    if arr.is_empty() {
        return;
    }
    let Some(op) = arr[0].as_str().map(str::to_string) else {
        return;
    };

    match op.as_str() {
        "==" | "!=" if arr.len() == 3 => {
            if let Some(prop) = get_prop_from_get(&arr[1])
                && let Some(table) = interned.get(prop)
            {
                intern_value(&mut arr[2], table);
            } else if let Some(prop) = get_prop_from_get(&arr[2])
                && let Some(table) = interned.get(prop)
            {
                intern_value(&mut arr[1], table);
            }
        }
        "match" if arr.len() >= 3 => {
            if let Some(prop) = get_prop_from_get(&arr[1])
                && let Some(table) = interned.get(prop)
            {
                // Labels are at even positions in rest; last element is default if count is odd.
                let rest_start = 2;
                let rest_len = arr.len() - rest_start;
                let pairs_end = if rest_len % 2 == 1 {
                    arr.len() - 1
                } else {
                    arr.len()
                };

                for i in (rest_start..pairs_end).step_by(2) {
                    intern_match_label(&mut arr[i], table);
                }
            }
        }
        "in" if arr.len() == 3 => {
            if let Some(prop) = get_prop_from_get(&arr[1])
                && let Some(table) = interned.get(prop)
            {
                intern_literal_array(&mut arr[2], table);
            }
        }
        "all" | "any" | "none" | "!" => {
            for child in arr.iter_mut().skip(1) {
                rewrite_filter_interning(child, interned);
            }
        }
        _ => {
            // Recurse into unrecognized nodes.
            for child in arr.iter_mut() {
                rewrite_filter_interning(child, interned);
            }
        }
    }
}

/// Extract property name from `["get", "prop"]`.
fn get_prop_from_get(v: &serde_json::Value) -> Option<&str> {
    let arr = v.as_array()?;
    if arr.len() == 2 && arr[0].as_str() == Some("get") {
        arr[1].as_str()
    } else {
        None
    }
}

/// Replace a string literal with its interning index.
fn intern_value(v: &mut serde_json::Value, table: &[String]) {
    if let Some(s) = v.as_str()
        && let Some(idx) = table.iter().position(|t| t == s)
    {
        *v = serde_json::Value::Number(serde_json::Number::from(idx as u64));
    }
}

/// Replace strings in a match label (single value or array of values).
fn intern_match_label(v: &mut serde_json::Value, table: &[String]) {
    match v {
        serde_json::Value::String(_) => intern_value(v, table),
        serde_json::Value::Array(arr) => {
            // Could be ["literal", [...]] or a plain array of labels.
            if arr.len() == 2
                && arr[0].as_str() == Some("literal")
                && let serde_json::Value::Array(ref mut vals) = arr[1]
            {
                for val in vals.iter_mut() {
                    intern_value(val, table);
                }
            } else {
                for val in arr.iter_mut() {
                    intern_value(val, table);
                }
            }
        }
        _ => {}
    }
}

/// Replace strings in `["literal", [...]]` array used with `in`.
fn intern_literal_array(v: &mut serde_json::Value, table: &[String]) {
    let Some(arr) = v.as_array_mut() else { return };
    if arr.len() == 2
        && arr[0].as_str() == Some("literal")
        && let serde_json::Value::Array(ref mut vals) = arr[1]
    {
        for val in vals.iter_mut() {
            intern_value(val, table);
        }
    }
}
