#!/usr/bin/env -S uv run
"""
Plot tile size comparisons between original MVT, MLT-Java, and MLT-Rust mbtiles databases.

Usage:
    uv run plot_sizes.py <mvt.mbtiles> <mlt-java.mbtiles> <mlt-rust.mbtiles> [--out figures/] [--format png]
"""

import argparse
import gzip
import itertools
import sqlite3
from pathlib import Path

import brotli
import plotly.graph_objects as go
import zstandard

IMG_WIDTH = 1400
IMG_HEIGHT = 700
IMG_SCALE = 2

FETCH_SIZE = 5000


def query_sizes(db_path: Path) -> dict[int, dict]:
    """Return {zoom: {count, total_bytes, avg_bytes}} for each zoom level."""
    conn = sqlite3.connect(str(db_path))
    rows = conn.execute(
        "SELECT zoom_level, COUNT(*), SUM(LENGTH(tile_data)), AVG(LENGTH(tile_data)) "
        "FROM tiles GROUP BY zoom_level ORDER BY zoom_level"
    ).fetchall()
    conn.close()
    return {
        int(z): {"count": int(c), "total_bytes": int(t), "avg_bytes": float(a)}
        for z, c, t, a in rows
    }


def query_all_compression_sizes(
    db_path: Path, stored_compressed: bool
) -> dict[int, dict[str, int]]:
    """Compute per-zoom total bytes for plain/gzip/zstd/brotli variants.

    If stored_compressed=True (MVT), blobs are gzip-compressed on disk:
      plain = decompress each blob, measure raw size
      gzip  = LENGTH(tile_data) (already gzipped)
      zstd  = decompress gzip → re-compress with zstd
      brotli = decompress gzip → re-compress with brotli

    If stored_compressed=False (MLT), blobs are raw on disk:
      plain = LENGTH(tile_data)
      gzip  = compress each blob with gzip
      zstd  = compress each blob with zstd
      brotli = compress each blob with brotli

    Streams tiles with fetchmany() to avoid loading everything into memory.
    Returns {zoom: {plain, gzip, zstd, brotli}}.
    """
    zstd_compressor = zstandard.ZstdCompressor()
    result: dict[int, dict[str, int]] = {}

    conn = sqlite3.connect(str(db_path))
    cur = conn.execute(
        "SELECT zoom_level, tile_data FROM tiles ORDER BY zoom_level"
    )

    current_zoom = None
    totals: dict[str, int] = {}

    while True:
        rows = cur.fetchmany(FETCH_SIZE)
        if not rows:
            break
        for zoom_level, tile_data in rows:
            z = int(zoom_level)
            if z != current_zoom:
                if current_zoom is not None:
                    result[current_zoom] = totals
                    print(f"  z{current_zoom}: {totals['plain'] / 1e6:.1f} MB plain", flush=True)
                current_zoom = z
                totals = {"plain": 0, "gzip": 0, "zstd": 0, "brotli": 0}

            blob = bytes(tile_data)
            if stored_compressed:
                raw = gzip.decompress(blob)
                totals["plain"] += len(raw)
                totals["gzip"] += len(blob)
                totals["zstd"] += len(zstd_compressor.compress(raw))
                totals["brotli"] += len(brotli.compress(raw, quality=6))
            else:
                raw = blob
                totals["plain"] += len(raw)
                totals["gzip"] += len(gzip.compress(raw))
                totals["zstd"] += len(zstd_compressor.compress(raw))
                totals["brotli"] += len(brotli.compress(raw, quality=6))

    if current_zoom is not None:
        result[current_zoom] = totals
        print(f"  z{current_zoom}: {totals['plain'] / 1e6:.1f} MB plain", flush=True)

    conn.close()
    return result


def write_fig(fig: go.Figure, out: Path, name: str, fmt: str) -> None:
    if fmt == "html":
        path = out / f"{name}.html"
        fig.write_html(path)
    else:
        path_png = out / f"{name}.png"
        path_pdf = out / f"{name}.pdf"
        fig.write_image(path_png, width=IMG_WIDTH, height=IMG_HEIGHT, scale=IMG_SCALE)
        fig.write_image(path_pdf, width=IMG_WIDTH, height=IMG_HEIGHT, scale=IMG_SCALE)
        print(f"  {path_png}")
        print(f"  {path_pdf}")


def fmt_bytes(b: float) -> str:
    if b >= 1e9:
        return f"{b / 1e9:.2f} GB"
    if b >= 1e6:
        return f"{b / 1e6:.1f} MB"
    if b >= 1e3:
        return f"{b / 1e3:.1f} KB"
    return f"{b:.0f} B"


def plot_compression_ratio(
    mvt_sizes: dict[int, dict[str, int]],
    mlt_java_sizes: dict[int, dict[str, int]],
    mlt_rust_sizes: dict[int, dict[str, int]],
    out: Path,
    fmt: str,
) -> None:
    """Plot #6: per-zoom compression ratio anchored on plain MVT = 1.0."""
    all_zooms = sorted(set(mvt_sizes) | set(mlt_java_sizes) | set(mlt_rust_sizes))
    zoom_labels = [f"z{z}" for z in all_zooms]

    # Anchor: plain MVT total bytes per zoom
    mvt_plain = {z: mvt_sizes[z]["plain"] for z in all_zooms if z in mvt_sizes}

    # Series config: (label, data_dict, compression, color, dash)
    compressions = ["plain", "gzip", "zstd", "brotli"]
    dash_styles = {"plain": "solid", "gzip": "dash", "zstd": "dot", "brotli": "dashdot"}
    comp_labels = {"plain": "Plain", "gzip": "GZip", "zstd": "ZSTD", "brotli": "Brotli"}

    format_configs = [
        ("MVT", mvt_sizes, ["#1f77b4", "#4a90d9", "#7bafd4", "#aec7e8"]),
        ("MLT-Java", mlt_java_sizes, ["#e67e22", "#f0a04b", "#f5be7b", "#f9d4a0"]),
        ("MLT-Rust", mlt_rust_sizes, ["#27ae60", "#52c77e", "#7fd9a0", "#a9e8c0"]),
    ]

    fig = go.Figure()

    for format_name, sizes, colors in format_configs:
        for i, comp in enumerate(compressions):
            ratios = []
            for z in all_zooms:
                anchor = mvt_plain.get(z, 0)
                comp_bytes = sizes.get(z, {}).get(comp, 0)
                if anchor > 0:
                    ratios.append(comp_bytes / anchor)
                else:
                    ratios.append(None)

            fig.add_trace(go.Scatter(
                name=f"{format_name} + {comp_labels[comp]}",
                x=zoom_labels,
                y=ratios,
                mode="lines+markers",
                line=dict(color=colors[i], width=2, dash=dash_styles[comp]),
                marker=dict(size=6),
                legendgroup=format_name,
                legendgrouptitle_text=format_name,
            ))

    fig.update_layout(
        title="Per-Zoom Compression Ratio (vs Plain MVT)",
        xaxis_title="Zoom Level",
        yaxis_title="Compression Ratio (lower is better)",
        template="plotly_white",
        legend=dict(groupclick="togglegroup"),
    )
    fig.add_hline(y=1.0, line_dash="dot", line_color="gray", opacity=0.5,
                  annotation_text="MVT Plain baseline")

    write_fig(fig, out, "compression_ratio_per_zoom", fmt)


def main():
    parser = argparse.ArgumentParser(description="Plot tile size comparisons")
    parser.add_argument("mvt", type=Path, help="MVT mbtiles file (gzip-compressed blobs)")
    parser.add_argument("mlt_java", type=Path, help="MLT-Java mbtiles file (raw blobs)")
    parser.add_argument("mlt_rust", type=Path, help="MLT-Rust mbtiles file (raw blobs)")
    parser.add_argument("--out", type=Path, default=Path("figures"), help="Output directory")
    parser.add_argument("--format", choices=["png", "html"], default="png")
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)

    # Existing plots use MVT vs MLT-Rust (the "rewritten" format from this project)
    orig = query_sizes(args.mvt)
    rewr = query_sizes(args.mlt_rust)

    all_zooms = sorted(set(orig) | set(rewr))
    zoom_labels = [f"z{z}" for z in all_zooms]

    orig_total = [orig.get(z, {}).get("total_bytes", 0) for z in all_zooms]
    rewr_total = [rewr.get(z, {}).get("total_bytes", 0) for z in all_zooms]
    orig_avg = [orig.get(z, {}).get("avg_bytes", 0) for z in all_zooms]
    rewr_avg = [rewr.get(z, {}).get("avg_bytes", 0) for z in all_zooms]
    orig_count = [orig.get(z, {}).get("count", 0) for z in all_zooms]
    rewr_count = [rewr.get(z, {}).get("count", 0) for z in all_zooms]

    # Reduction percentages
    reduction_pct = []
    for o, r in zip(orig_total, rewr_total):
        if o > 0:
            reduction_pct.append((1 - r / o) * 100)
        else:
            reduction_pct.append(0)

    grand_orig = sum(orig_total)
    grand_rewr = sum(rewr_total)
    grand_pct = (1 - grand_rewr / grand_orig) * 100 if grand_orig > 0 else 0

    print(f"Original: {fmt_bytes(grand_orig)}")
    print(f"Rewritten: {fmt_bytes(grand_rewr)}")
    print(f"Reduction: {grand_pct:.1f}%\n")

    # ── 1. Total size per zoom (stacked bar) ─────────────────────────────────
    fig = go.Figure()
    fig.add_trace(go.Bar(
        name="Original (MVT)",
        x=zoom_labels, y=[b / 1e6 for b in orig_total],
        text=[fmt_bytes(b) for b in orig_total],
        textposition="auto",
        marker_color="#636EFA",
    ))
    fig.add_trace(go.Bar(
        name="Rewritten (MLT)",
        x=zoom_labels, y=[b / 1e6 for b in rewr_total],
        text=[fmt_bytes(b) for b in rewr_total],
        textposition="auto",
        marker_color="#EF553B",
    ))
    fig.update_layout(
        title=f"Total Tile Size per Zoom Level (overall: {fmt_bytes(grand_orig)} → {fmt_bytes(grand_rewr)}, {grand_pct:.1f}% reduction)",
        xaxis_title="Zoom Level",
        yaxis_title="Total Size (MB)",
        barmode="group",
        template="plotly_white",
    )
    write_fig(fig, args.out, "size_total_per_zoom", args.format)

    # ── 2. Average tile size per zoom ─────────────────────────────────────────
    fig = go.Figure()
    fig.add_trace(go.Bar(
        name="Original (MVT)",
        x=zoom_labels, y=[b / 1e3 for b in orig_avg],
        text=[fmt_bytes(b) for b in orig_avg],
        textposition="auto",
        marker_color="#636EFA",
    ))
    fig.add_trace(go.Bar(
        name="Rewritten (MLT)",
        x=zoom_labels, y=[b / 1e3 for b in rewr_avg],
        text=[fmt_bytes(b) for b in rewr_avg],
        textposition="auto",
        marker_color="#EF553B",
    ))
    fig.update_layout(
        title="Average Tile Size per Zoom Level",
        xaxis_title="Zoom Level",
        yaxis_title="Average Size (KB)",
        barmode="group",
        template="plotly_white",
    )
    write_fig(fig, args.out, "size_avg_per_zoom", args.format)

    # ── 3. Size reduction percentage per zoom ─────────────────────────────────
    colors = ["#2CA02C" if p > 0 else "#D62728" for p in reduction_pct]
    fig = go.Figure()
    fig.add_trace(go.Bar(
        x=zoom_labels, y=reduction_pct,
        text=[f"{p:.1f}%" for p in reduction_pct],
        textposition="auto",
        marker_color=colors,
    ))
    fig.update_layout(
        title=f"Size Reduction by Zoom Level (overall {grand_pct:.1f}%)",
        xaxis_title="Zoom Level",
        yaxis_title="Reduction (%)",
        template="plotly_white",
    )
    write_fig(fig, args.out, "size_reduction_pct", args.format)

    # ── 4. Tile count comparison ──────────────────────────────────────────────
    fig = go.Figure()
    fig.add_trace(go.Bar(
        name="Original",
        x=zoom_labels, y=orig_count,
        text=[str(c) for c in orig_count],
        textposition="auto",
        marker_color="#636EFA",
    ))
    fig.add_trace(go.Bar(
        name="Rewritten",
        x=zoom_labels, y=rewr_count,
        text=[str(c) for c in rewr_count],
        textposition="auto",
        marker_color="#EF553B",
    ))
    dropped = sum(orig_count) - sum(rewr_count)
    fig.update_layout(
        title=f"Tile Count per Zoom Level ({dropped} tiles dropped)",
        xaxis_title="Zoom Level",
        yaxis_title="Tile Count",
        barmode="group",
        template="plotly_white",
    )
    write_fig(fig, args.out, "size_tile_count", args.format)

    # ── 5. Cumulative size ────────────────────────────────────────────────────
    cum_orig = list(itertools.accumulate(orig_total))
    cum_rewr = list(itertools.accumulate(rewr_total))
    fig = go.Figure()
    fig.add_trace(go.Scatter(
        name="Original (MVT)",
        x=zoom_labels, y=[b / 1e9 for b in cum_orig],
        mode="lines+markers",
        line=dict(color="#636EFA", width=3),
    ))
    fig.add_trace(go.Scatter(
        name="Rewritten (MLT)",
        x=zoom_labels, y=[b / 1e9 for b in cum_rewr],
        mode="lines+markers",
        line=dict(color="#EF553B", width=3),
    ))
    fig.update_layout(
        title="Cumulative Tile Data Size",
        xaxis_title="Zoom Level",
        yaxis_title="Cumulative Size (GB)",
        template="plotly_white",
    )
    write_fig(fig, args.out, "size_cumulative", args.format)

    # ── 6. Per-zoom compression ratio ─────────────────────────────────────────
    print("\nComputing compression variants for MVT...")
    mvt_sizes = query_all_compression_sizes(args.mvt, stored_compressed=True)
    print("Computing compression variants for MLT-Java...")
    mlt_java_sizes = query_all_compression_sizes(args.mlt_java, stored_compressed=False)
    print("Computing compression variants for MLT-Rust...")
    mlt_rust_sizes = query_all_compression_sizes(args.mlt_rust, stored_compressed=False)

    plot_compression_ratio(mvt_sizes, mlt_java_sizes, mlt_rust_sizes, args.out, args.format)

    print(f"\nAll figures written to {args.out}/")


if __name__ == "__main__":
    main()
