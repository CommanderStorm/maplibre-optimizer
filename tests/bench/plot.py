#!/usr/bin/env -S uv run
"""
Plot ablation benchmark results from JSONL files produced by the bench harness.

Generates thesis-quality figures showing the marginal contribution of each
optimizer pass via cumulative ablation.

Usage:
    uv run plot.py results/bench-*.jsonl
    uv run plot.py results/bench-*.jsonl --out figures/
    uv run plot.py results/bench-*.jsonl --metric loadMs --metric p95FrameMs
    uv run plot.py results/bench-*.jsonl --format html
"""

import argparse
import json
import sys
from pathlib import Path

import numpy as np
import pandas as pd
import plotly.express as px
import plotly.graph_objects as go

IMG_WIDTH = 1400
IMG_HEIGHT = 700
IMG_SCALE = 2  # 2x for retina-quality PNGs


def write_fig(fig: go.Figure, out: Path, name: str, fmt: str) -> None:
    """Write a figure in the chosen format (png or html)."""
    if fmt == "html":
        path = out / f"{name}.html"
        fig.write_html(path)
    else:
        path = out / f"{name}.png"
        fig.write_image(path, width=IMG_WIDTH, height=IMG_HEIGHT, scale=IMG_SCALE)
    print(f"  {path}")


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
    "style_bytes": "Style Size (bytes)",
}


def ablation_step_order(df: pd.DataFrame) -> list[str]:
    """Return variant IDs sorted by ablation step number."""
    variants = df["variant"].unique().tolist()
    variants.sort(key=lambda v: (v.split("-")[1] if "-" in v else "99"))
    return variants


def _step_label(variant_id: str) -> str:
    """Extract a short label from a variant ID like 'step-04-constant_fold'."""
    parts = variant_id.split("-", 2)
    if len(parts) == 3:
        return parts[2]
    return variant_id


def _delta_pct(old: float, new: float) -> float | None:
    """Compute percentage change from old to new, or None if old is zero."""
    if old == 0:
        return None
    return ((new - old) / old) * 100


def compute_marginal_deltas(
    medians: pd.DataFrame, variants: list[str], metric: str, scenarios: list[str],
) -> dict[str, list[float | None]]:
    """
    Compute per-scenario marginal delta for each ablation step.

    Returns a dict mapping pass_name -> list of delta values (one per scenario).
    None entries indicate missing data.
    """
    result: dict[str, list[float | None]] = {}
    for i in range(1, len(variants)):
        prev_v = variants[i - 1]
        curr_v = variants[i]
        pass_name = _step_label(curr_v)

        deltas: list[float | None] = []
        for scenario in scenarios:
            sdf = medians[medians.scenario == scenario].set_index("variant")
            if prev_v in sdf.index and curr_v in sdf.index:
                deltas.append(_delta_pct(sdf.loc[prev_v, metric], sdf.loc[curr_v, metric]))
            else:
                deltas.append(None)
        result[pass_name] = deltas
    return result


def plot_ablation_waterfall(
    medians: pd.DataFrame, variants: list[str], metrics: list[str], out: Path, fmt: str
) -> None:
    """
    Ablation waterfall: X-axis is ablation step, Y-axis is metric value.
    One thin line per scenario, bold line for the mean across scenarios.
    """
    step_labels = [_step_label(v) for v in variants]

    for metric in metrics:
        fig = go.Figure()

        scenarios = medians["scenario"].unique()
        for scenario in scenarios:
            sdf = medians[medians.scenario == scenario].set_index("variant")
            sdf = sdf.reindex(variants)
            fig.add_trace(go.Scatter(
                x=step_labels,
                y=sdf[metric].values,
                mode="lines",
                line=dict(width=1, color="rgba(150,150,150,0.4)"),
                name=scenario,
                showlegend=False,
                hovertext=scenario,
            ))

        # Mean line across all scenarios
        mean_vals = []
        for v in variants:
            vdf = medians[medians.variant == v]
            mean_vals.append(vdf[metric].mean() if len(vdf) > 0 else None)

        fig.add_trace(go.Scatter(
            x=step_labels,
            y=mean_vals,
            mode="lines+markers",
            line=dict(width=3, color="#D62728"),
            marker=dict(size=8),
            name="Mean across scenarios",
        ))

        lower_better = metric in LOWER_IS_BETTER
        direction = "↓ lower is better" if lower_better else "↑ higher is better"
        fig.update_layout(
            title=f"Ablation Waterfall: {METRIC_LABELS.get(metric, metric)}",
            xaxis_title="Cumulative Pass",
            yaxis_title=f"{METRIC_LABELS.get(metric, metric)} ({direction})",
            xaxis_tickangle=-45,
            template="plotly_white",
            legend=dict(x=0.01, y=0.99),
        )
        write_fig(fig, out, f"waterfall_{metric}", fmt)


def plot_marginal_contribution(
    medians: pd.DataFrame, variants: list[str], metrics: list[str], out: Path, fmt: str
) -> None:
    """
    Bar chart of marginal % change when each pass is added.
    Averaged across all scenarios with error bars (stddev).
    """
    scenarios = sorted(medians["scenario"].unique())

    for metric in metrics:
        lower_better = metric in LOWER_IS_BETTER
        marginals = compute_marginal_deltas(medians, variants, metric, scenarios)

        pass_names = []
        mean_deltas = []
        std_deltas = []
        for pass_name, deltas in marginals.items():
            valid = [d for d in deltas if d is not None]
            if valid:
                pass_names.append(pass_name)
                mean_deltas.append(np.mean(valid))
                std_deltas.append(np.std(valid))

        colors = []
        for d in mean_deltas:
            is_good = (d < 0) if lower_better else (d > 0)
            colors.append("#2CA02C" if is_good else "#D62728" if abs(d) > 0.5 else "#999999")

        fig = go.Figure()
        fig.add_trace(go.Bar(
            x=pass_names,
            y=mean_deltas,
            error_y=dict(type="data", array=std_deltas, visible=True),
            marker_color=colors,
            text=[f"{d:+.2f}%" for d in mean_deltas],
            textposition="outside",
        ))

        direction = "(negative = improvement)" if lower_better else "(positive = improvement)"
        fig.update_layout(
            title=f"Marginal Contribution per Pass: {METRIC_LABELS.get(metric, metric)}",
            xaxis_title="Pass Added",
            yaxis_title=f"Median % Change {direction}",
            xaxis_tickangle=-45,
            template="plotly_white",
        )
        write_fig(fig, out, f"marginal_{metric}", fmt)


def plot_style_size_ablation(df: pd.DataFrame, variants: list[str], out: Path, fmt: str) -> None:
    """
    Style size waterfall: X-axis is ablation step, Y-axis is style JSON bytes.
    """
    if "style_bytes" not in df.columns:
        print("  (skipped — no style_bytes in data)")
        return

    # style_bytes is the same for all runs of a variant, take the first
    size_by_variant = df.groupby("variant")["style_bytes"].first()

    sizes = []
    labels = []
    for v in variants:
        if v in size_by_variant.index:
            sizes.append(size_by_variant[v])
            labels.append(_step_label(v))

    if not sizes:
        return

    baseline = sizes[0]
    pct_labels = [
        f"{s / 1024:.1f} KB ({(1 - s / baseline) * 100:.1f}% smaller)" if i > 0
        else f"{s / 1024:.1f} KB"
        for i, s in enumerate(sizes)
    ]

    fig = go.Figure()
    fig.add_trace(go.Scatter(
        x=labels,
        y=sizes,
        mode="lines+markers+text",
        line=dict(width=3, color="#1F77B4"),
        marker=dict(size=10),
        text=pct_labels,
        textposition="top center",
        textfont=dict(size=10),
    ))

    fig.update_layout(
        title="Style Size Across Ablation Steps",
        xaxis_title="Cumulative Pass",
        yaxis_title="Style Size (bytes)",
        xaxis_tickangle=-45,
        template="plotly_white",
    )
    write_fig(fig, out, "style_size_ablation", fmt)


def plot_scenario_heatmap(
    medians: pd.DataFrame, variants: list[str], metrics: list[str], out: Path, fmt: str
) -> None:
    """
    Heatmap: rows = scenarios, columns = passes (marginal contribution).
    Cell color = % improvement from adding that pass.
    """
    scenarios = sorted(medians["scenario"].unique())

    for metric in metrics:
        lower_better = metric in LOWER_IS_BETTER
        marginals = compute_marginal_deltas(medians, variants, metric, scenarios)

        pass_names = list(marginals.keys())
        # Build matrix: each row is a pass, each column is a scenario
        matrix = []
        for pass_name in pass_names:
            row = []
            for d in marginals[pass_name]:
                if d is None:
                    row.append(None)
                else:
                    # For lower-is-better, negate so positive = improvement
                    row.append(-d if lower_better else d)
            matrix.append(row)

        z = np.array(matrix, dtype=float).T  # scenarios as rows, passes as columns

        fig = go.Figure(data=go.Heatmap(
            z=z,
            x=pass_names,
            y=scenarios,
            colorscale="RdYlGn",
            zmid=0,
            text=np.where(np.isnan(z), "", np.char.add(np.where(z >= 0, "+", ""), np.char.mod("%.1f%%", z))),
            texttemplate="%{text}",
            colorbar_title="% Improvement",
        ))

        fig.update_layout(
            title=f"Per-Scenario Pass Impact: {METRIC_LABELS.get(metric, metric)}",
            xaxis_title="Pass Added",
            yaxis_title="Scenario",
            xaxis_tickangle=-45,
            template="plotly_white",
            height=max(500, 30 * len(scenarios) + 200),
        )
        write_fig(fig, out, f"heatmap_{metric}", fmt)


def plot_box_per_step(
    df: pd.DataFrame, variants: list[str], metrics: list[str], out: Path, fmt: str
) -> None:
    """Box plots with X-axis as ablation step."""
    variant_order = {v: i for i, v in enumerate(variants)}
    step_labels = [_step_label(v) for v in variants]
    df = df.copy()
    df["step_label"] = df["variant"].map(_step_label)
    df["step_order"] = df["variant"].map(lambda v: variant_order.get(v, 99))
    df = df.sort_values("step_order")

    for metric in metrics:
        fig = px.box(
            df,
            x="step_label",
            y=metric,
            title=f"{METRIC_LABELS.get(metric, metric)} by Ablation Step",
            labels={"step_label": "Ablation Step", metric: METRIC_LABELS.get(metric, metric)},
            category_orders={"step_label": step_labels},
        )
        fig.update_layout(
            xaxis_tickangle=-45,
            template="plotly_white",
        )
        fig.update_traces(marker_color="#1F77B4")
        write_fig(fig, out, f"box_{metric}", fmt)


def main() -> None:
    parser = argparse.ArgumentParser(description="Plot ablation benchmark results from JSONL files.")
    parser.add_argument("files", nargs="+", type=Path, help="JSONL benchmark result files")
    parser.add_argument("--out", type=Path, default=Path("tests/bench/figures"), help="Output directory for figures")
    parser.add_argument("--metric", action="append", dest="metrics", help="Metrics to plot (default: all)")
    parser.add_argument("--format", choices=["png", "html"], default="png", help="Output format (default: png)")
    args = parser.parse_args()

    metrics = args.metrics or METRICS
    fmt = args.format
    args.out.mkdir(parents=True, exist_ok=True)

    df = load_jsonl(args.files)
    print(f"Loaded {len(df)} records from {len(args.files)} file(s)")
    print(f"Variants: {sorted(df['variant'].unique())}")
    print(f"Scenarios: {sorted(df['scenario'].unique())}\n")

    # Precompute shared data
    variants = ablation_step_order(df)
    medians = df.groupby(["scenario", "variant"])[metrics].median().reset_index()

    print("Ablation waterfalls:")
    plot_ablation_waterfall(medians, variants, metrics, args.out, fmt)

    print("\nMarginal contribution bars:")
    plot_marginal_contribution(medians, variants, metrics, args.out, fmt)

    print("\nStyle size ablation:")
    plot_style_size_ablation(df, variants, args.out, fmt)

    print("\nScenario × pass heatmaps:")
    plot_scenario_heatmap(medians, variants, metrics, args.out, fmt)

    print("\nBox plots per ablation step:")
    plot_box_per_step(df, variants, metrics, args.out, fmt)

    print(f"\nAll figures written to {args.out}/")


if __name__ == "__main__":
    main()
