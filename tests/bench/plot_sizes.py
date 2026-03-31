#!/usr/bin/env -S uv run
"""
Plot tile size comparisons between original and rewritten mbtiles databases.

Usage:
    uv run plot_sizes.py <original.mbtiles> <rewritten.mbtiles> [--out figures/] [--format png]
"""

import argparse
import sqlite3
from pathlib import Path

import plotly.graph_objects as go
from plotly.subplots import make_subplots

IMG_WIDTH = 1400
IMG_HEIGHT = 700
IMG_SCALE = 2


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


def write_fig(fig: go.Figure, out: Path, name: str, fmt: str) -> None:
    if fmt == "html":
        path = out / f"{name}.html"
        fig.write_html(path)
    else:
        path = out / f"{name}.png"
        fig.write_image(path, width=IMG_WIDTH, height=IMG_HEIGHT, scale=IMG_SCALE)
    print(f"  {path}")


def fmt_bytes(b: float) -> str:
    if b >= 1e9:
        return f"{b / 1e9:.2f} GB"
    if b >= 1e6:
        return f"{b / 1e6:.1f} MB"
    if b >= 1e3:
        return f"{b / 1e3:.1f} KB"
    return f"{b:.0f} B"


def main():
    parser = argparse.ArgumentParser(description="Plot tile size comparisons")
    parser.add_argument("original", type=Path, help="Original mbtiles file")
    parser.add_argument("rewritten", type=Path, help="Rewritten mbtiles file")
    parser.add_argument("--out", type=Path, default=Path("figures"), help="Output directory")
    parser.add_argument("--format", choices=["png", "html"], default="png")
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)

    orig = query_sizes(args.original)
    rewr = query_sizes(args.rewritten)

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
    import itertools
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

    print(f"\nAll figures written to {args.out}/")


if __name__ == "__main__":
    main()
