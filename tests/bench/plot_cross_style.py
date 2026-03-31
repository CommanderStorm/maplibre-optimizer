#!/usr/bin/env -S uv run
"""
Plot cross-style generalization results from JSONL produced by cross_style.ts.

Usage:
    uv run plot_cross_style.py results/cross-style-*.jsonl
    uv run plot_cross_style.py results/cross-style-*.jsonl --out figures/ --format html
"""

import argparse
import json
import sys
from pathlib import Path

import pandas as pd
import plotly.graph_objects as go

IMG_WIDTH = 1400
IMG_HEIGHT = 700
IMG_SCALE = 2


def write_fig(fig: go.Figure, out: Path, name: str, fmt: str) -> None:
    if fmt == "html":
        path = out / f"{name}.html"
        fig.write_html(path)
    else:
        path_png = out / f"{name}.png"
        path_eps = out / f"{name}.eps"
        fig.write_image(path_png, width=IMG_WIDTH, height=IMG_HEIGHT, scale=IMG_SCALE)
        fig.write_image(path_eps, width=IMG_WIDTH, height=IMG_HEIGHT, scale=IMG_SCALE)
        print(f"  {path_png}")
        print(f"  {path_eps}")


def load_jsonl(paths: list[Path]) -> pd.DataFrame:
    rows = []
    for p in paths:
        with open(p) as f:
            for line in f:
                line = line.strip()
                if line:
                    rows.append(json.loads(line))
    if not rows:
        print("No data found.", file=sys.stderr)
        sys.exit(1)
    return pd.DataFrame(rows)


def plot_size_reduction_bar(df: pd.DataFrame, out: Path, fmt: str) -> None:
    """Bar chart: % size reduction per style, sorted."""
    df = df.sort_values("reduction_pct", ascending=True)

    fig = go.Figure()

    for col, name, color in [
        ("reduction_pct", "Raw", "#1F77B4"),
        ("gzip_reduction_pct", "Gzip", "#FF7F0E"),
        ("brotli_reduction_pct", "Brotli", "#2CA02C"),
    ]:
        if col not in df.columns:
            continue
        fig.add_trace(go.Bar(
            y=df["style_id"],
            x=df[col],
            orientation="h",
            name=name,
            marker_color=color,
            text=[f"{v:.1f}%" for v in df[col]],
            textposition="outside",
        ))

    fig.update_layout(
        title="Size Reduction per Style (Raw, Gzip, Brotli)",
        xaxis_title="% Reduction",
        yaxis_title="Style",
        barmode="group",
        template="plotly_white",
        height=max(500, 30 * len(df) + 200),
    )
    write_fig(fig, out, "cross_style_reduction", fmt)


def plot_complexity_scatter(df: pd.DataFrame, out: Path, fmt: str) -> None:
    """Scatter: original complexity vs reduction achieved."""
    if "original_ast_nodes" not in df.columns or "reduction_pct" not in df.columns:
        print("  (skipped — missing complexity or reduction columns)")
        return

    fig = go.Figure()
    fig.add_trace(go.Scatter(
        x=df["original_ast_nodes"],
        y=df["reduction_pct"],
        mode="markers+text",
        text=df["style_id"],
        textposition="top center",
        textfont=dict(size=9),
        marker=dict(size=10, color="#1F77B4"),
    ))

    fig.update_layout(
        title="Original Complexity vs Size Reduction",
        xaxis_title="Original AST Node Count",
        yaxis_title="% Size Reduction",
        template="plotly_white",
    )
    write_fig(fig, out, "cross_style_complexity_scatter", fmt)


def plot_layer_count_comparison(df: pd.DataFrame, out: Path, fmt: str) -> None:
    """Grouped bar: original vs optimized layer count per style."""
    if "original_layer_count" not in df.columns or "optimized_layer_count" not in df.columns:
        print("  (skipped — missing layer count columns)")
        return

    df = df.sort_values("original_layer_count", ascending=True)

    fig = go.Figure()
    fig.add_trace(go.Bar(
        y=df["style_id"],
        x=df["original_layer_count"],
        orientation="h",
        name="Original",
        marker_color="#1F77B4",
    ))
    fig.add_trace(go.Bar(
        y=df["style_id"],
        x=df["optimized_layer_count"],
        orientation="h",
        name="Optimized",
        marker_color="#2CA02C",
    ))

    fig.update_layout(
        title="Layer Count: Original vs Optimized",
        xaxis_title="Layer Count",
        yaxis_title="Style",
        barmode="group",
        template="plotly_white",
        height=max(500, 30 * len(df) + 200),
    )
    write_fig(fig, out, "cross_style_layers", fmt)


def main() -> None:
    parser = argparse.ArgumentParser(description="Plot cross-style benchmark results.")
    parser.add_argument("files", nargs="+", type=Path, help="JSONL result files")
    parser.add_argument("--out", type=Path, default=Path("tests/bench/figures"), help="Output directory")
    parser.add_argument("--format", choices=["png", "html"], default="png", help="Output format")
    args = parser.parse_args()

    args.out.mkdir(parents=True, exist_ok=True)
    df = load_jsonl(args.files)
    print(f"Loaded {len(df)} styles\n")

    # Summary stats
    if "reduction_pct" in df.columns:
        print(f"Mean raw reduction:    {df['reduction_pct'].mean():.1f}%")
    if "gzip_reduction_pct" in df.columns:
        print(f"Mean gzip reduction:   {df['gzip_reduction_pct'].mean():.1f}%")
    if "brotli_reduction_pct" in df.columns:
        print(f"Mean brotli reduction: {df['brotli_reduction_pct'].mean():.1f}%")
    print()

    print("Size reduction bars:")
    plot_size_reduction_bar(df, args.out, args.format)

    print("\nComplexity scatter:")
    plot_complexity_scatter(df, args.out, args.format)

    print("\nLayer count comparison:")
    plot_layer_count_comparison(df, args.out, args.format)

    print(f"\nAll figures written to {args.out}/")


if __name__ == "__main__":
    main()
