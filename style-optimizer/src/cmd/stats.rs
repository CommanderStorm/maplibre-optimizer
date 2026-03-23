use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::Args;

/// Collect tile statistics from an `MBTiles` file.
///
/// Reads vector tiles, decodes MVT data, and writes a `TileStatistics` JSON file
/// suitable for use with `optimize --stats`.
#[derive(Args, Debug)]
pub struct StatsArgs {
    /// Path to the input `.mbtiles` file.
    #[arg(long)]
    input: PathBuf,

    /// Source name key (must match the style's `"sources"` map key).
    #[arg(long)]
    source_name: String,

    /// Output JSON path for the generated statistics.
    #[arg(long)]
    output: PathBuf,

    /// Zoom levels to scan (e.g. `0-14` or `6,10,14`). Defaults to all available.
    #[arg(long)]
    zoom_levels: Option<String>,

    /// Fraction of tiles to sample per zoom level (0.0–1.0, default: 1.0).
    #[arg(long, default_value_t = 1.0)]
    sample_rate: f64,

    /// Pretty-print JSON output.
    #[arg(long)]
    pretty: bool,
}

pub fn run(args: &StatsArgs) -> anyhow::Result<()> {
    use maplibre_style_optimizer::stats::collect;

    let conn = collect::open_mbtiles(&args.input)?;

    let zoom_levels = match &args.zoom_levels {
        Some(spec) => parse_zoom_levels(spec)?,
        None => collect::available_zoom_levels(&conn)?,
    };

    eprintln!(
        "Collecting statistics from {} for source {:?} at zoom levels {zoom_levels:?} (sample rate {:.0}%)",
        args.input.display(),
        args.source_name,
        args.sample_rate * 100.0,
    );

    let stats =
        collect::collect_statistics(&conn, &args.source_name, &zoom_levels, args.sample_rate)?;

    let json = if args.pretty {
        serde_json::to_string_pretty(&stats)?
    } else {
        serde_json::to_string(&stats)?
    };

    fs::write(&args.output, json).with_context(|| args.output.display().to_string())?;
    eprintln!("Wrote statistics to {}", args.output.display());

    Ok(())
}

/// Parse a zoom level spec like `"0-14"` or `"6,10,14"` or `"0-5,10,14"`.
fn parse_zoom_levels(spec: &str) -> anyhow::Result<Vec<u8>> {
    let mut levels = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if let Some((start, end)) = part.split_once('-') {
            let start: u8 = start.trim().parse().context("invalid zoom start")?;
            let end: u8 = end.trim().parse().context("invalid zoom end")?;
            for z in start..=end {
                levels.push(z);
            }
        } else {
            levels.push(part.parse().context("invalid zoom level")?);
        }
    }
    levels.sort_unstable();
    levels.dedup();
    Ok(levels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_range() {
        assert_eq!(parse_zoom_levels("0-3").unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn parse_list() {
        assert_eq!(parse_zoom_levels("6,10,14").unwrap(), vec![6, 10, 14]);
    }

    #[test]
    fn parse_mixed() {
        assert_eq!(
            parse_zoom_levels("0-2,5,10-12").unwrap(),
            vec![0, 1, 2, 5, 10, 11, 12]
        );
    }

    #[test]
    fn parse_dedup() {
        assert_eq!(parse_zoom_levels("5,5,3-5").unwrap(), vec![3, 4, 5]);
    }
}
