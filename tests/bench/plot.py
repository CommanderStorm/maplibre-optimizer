#!/usr/bin/env python3
"""
Plot benchmark results from JSONL files produced by the bench harness.

Usage:
    python plot.py results/bench-*.jsonl
    python plot.py results/bench-*.jsonl --out figures/
    python plot.py results/bench-*.jsonl --metric fps --metric loadMs
"""

import argparse
import json
import sys
from pathlib import Path

import pandas as pd
import plotly.express as px
import plotly.graph_objects as go
from plotly.subplots import make_subplots


def load_jsonl(paths: list[Path]) -> pd.DataFrame:
    rows = []
    for p in paths:
        with open(p) as f:
            for line in f:
                line = line.strip()
                if line:
                    rows.append(json.loads(line))
    if not rows:
        print("No data found in the provided files.", file=sys.stderr)
        sys.exit(1)
    return pd.DataFrame(rows)


METRICS = ["loadMs", "idleMs", "fps", "p50FrameMs", "p95FrameMs", "p99FrameMs", "jankCount"]
LOWER_IS_BETTER = {"loadMs", "idleMs", "p50FrameMs", "p95FrameMs", "p99FrameMs", "jankCount"}

METRIC_LABELS = {
    "loadMs": "Load Time (ms)",
    "idleMs": "Time to Idle (ms)",
    "fps": "Frames per Second",
    "p50FrameMs": "Median Frame Time (ms)",
    "p95FrameMs": "P95 Frame Time (ms)",
    "p99FrameMs": "P99 Frame Time (ms)",
    "jankCount": "Jank Frames",
}


def plot_box_comparison(df: pd.DataFrame, metrics: list[str], out: Path) -> None:
    """Box plots comparing original vs optimized for each scenario."""
    for metric in metrics:
        fig = px.box(
            df,
            x="scenario",
            y=metric,
            color="variant",
            title=f"{METRIC_LABELS.get(metric, metric)} by Scenario",
            labels={"scenario": "Scenario", metric: METRIC_LABELS.get(metric, metric)},
            color_discrete_map={"original": "#636EFA", "optimized": "#00CC96"},
        )
        fig.update_layout(
            xaxis_tickangle=-45,
            legend_title_text="Variant",
            template="plotly_white",
        )
        path = out / f"box_{metric}.html"
        fig.write_html(path)
        print(f"  {path}")


def plot_delta_bar(df: pd.DataFrame, metrics: list[str], out: Path) -> None:
    """Bar chart showing % change (optimized vs original) per scenario."""
    medians = df.groupby(["scenario", "variant"])[metrics].median().reset_index()
    orig = medians[medians.variant == "original"].set_index("scenario")
    opt = medians[medians.variant == "optimized"].set_index("scenario")

    deltas = ((opt[metrics] - orig[metrics]) / orig[metrics] * 100).reset_index()

    for metric in metrics:
        lower_better = metric in LOWER_IS_BETTER
        colors = [
            "#00CC96" if (v < 0) == lower_better else "#EF553B"
            for v in deltas[metric]
        ]
        fig = go.Figure(go.Bar(
            x=deltas["scenario"],
            y=deltas[metric],
            marker_color=colors,
            text=[f"{v:+.1f}%" for v in deltas[metric]],
            textposition="outside",
        ))
        fig.update_layout(
            title=f"Optimization Impact: {METRIC_LABELS.get(metric, metric)}",
            xaxis_title="Scenario",
            yaxis_title="Change (%)",
            xaxis_tickangle=-45,
            template="plotly_white",
        )
        path = out / f"delta_{metric}.html"
        fig.write_html(path)
        print(f"  {path}")


def plot_geo_map(df: pd.DataFrame, metrics: list[str], out: Path) -> None:
    """Scatter map showing per-location delta for each metric."""
    medians = df.groupby(["scenario", "variant", "location", "lat", "lng"])[metrics].median().reset_index()
    orig = medians[medians.variant == "original"].set_index("scenario")
    opt = medians[medians.variant == "optimized"].set_index("scenario")

    geo = orig[["location", "lat", "lng"]].copy()
    for metric in metrics:
        geo[f"delta_{metric}"] = ((opt[metric].values - orig[metric].values) / orig[metric].values * 100)

    # Aggregate per location (average across animations at same location)
    geo_agg = geo.groupby(["location", "lat", "lng"]).mean(numeric_only=True).reset_index()

    for metric in metrics:
        col = f"delta_{metric}"
        lower_better = metric in LOWER_IS_BETTER
        # For color: green = good, red = bad
        geo_agg["improvement"] = geo_agg[col] * (-1 if lower_better else 1)

        fig = px.scatter_geo(
            geo_agg,
            lat="lat",
            lon="lng",
            size=geo_agg[col].abs(),
            color="improvement",
            color_continuous_scale="RdYlGn",
            hover_name="location",
            hover_data={col: ":.1f", "lat": False, "lng": False, "improvement": False},
            title=f"Optimization Impact by Location: {METRIC_LABELS.get(metric, metric)}",
            projection="natural earth",
        )
        fig.update_layout(
            coloraxis_colorbar_title="Improvement",
            template="plotly_white",
        )
        path = out / f"geo_{metric}.html"
        fig.write_html(path)
        print(f"  {path}")


def plot_frame_time_distribution(df: pd.DataFrame, out: Path) -> None:
    """Histogram of p95 frame times across all runs, original vs optimized."""
    fig = px.histogram(
        df,
        x="p95FrameMs",
        color="variant",
        barmode="overlay",
        opacity=0.7,
        nbins=40,
        title="P95 Frame Time Distribution",
        labels={"p95FrameMs": "P95 Frame Time (ms)"},
        color_discrete_map={"original": "#636EFA", "optimized": "#00CC96"},
    )
    fig.update_layout(template="plotly_white")
    path = out / "hist_p95.html"
    fig.write_html(path)
    print(f"  {path}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Plot benchmark results from JSONL files.")
    parser.add_argument("files", nargs="+", type=Path, help="JSONL benchmark result files")
    parser.add_argument("--out", type=Path, default=Path("tests/bench/figures"), help="Output directory for figures")
    parser.add_argument("--metric", action="append", dest="metrics", help="Metrics to plot (default: all)")
    args = parser.parse_args()

    metrics = args.metrics or METRICS
    args.out.mkdir(parents=True, exist_ok=True)

    df = load_jsonl(args.files)
    print(f"Loaded {len(df)} records from {len(args.files)} file(s)\n")

    print("Box plots:")
    plot_box_comparison(df, metrics, args.out)

    print("\nDelta bar charts:")
    plot_delta_bar(df, metrics, args.out)

    print("\nGeo maps:")
    plot_geo_map(df, metrics, args.out)

    print("\nDistributions:")
    plot_frame_time_distribution(df, args.out)

    print(f"\nAll figures written to {args.out}/")


if __name__ == "__main__":
    main()
