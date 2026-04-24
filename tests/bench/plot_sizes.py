#!/usr/bin/env -S uv run
"""
Plot tile size comparisons between original MVT, MLT-Java, and MLT-Rust mbtiles databases.

Usage:
    uv run plot_sizes.py <mvt.mbtiles> <mlt-java-idsort.mbtiles> <mlt-java-noidsort.mbtiles> <mlt-rust.mbtiles> [--out figures/] [--format png]
"""

import argparse
import gzip
import hashlib
import itertools
import sqlite3
from pathlib import Path

import brotli
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from pydantic import BaseModel
import zstandard

from plot_style import (
    IMPROVEMENT_COLOR,
    LAYOUT_DEFAULTS,
    REGRESSION_COLOR,
    IMG_HEIGHT,
    IMG_SCALE,
    IMG_WIDTH,
)

FETCH_SIZE = 5000

GZIP_LEVELS = [1, 6, 9]
ZSTD_LEVELS = [1, 3, 9, 19]
BROTLI_LEVELS = [1, 6, 11]


class ZoomCompression(BaseModel):
    """Compression totals for a single zoom level."""
    plain: int = 0
    gzip: dict[int, int] = {}
    zstd: dict[int, int] = {}
    brotli: dict[int, int] = {}
    zstd_dict: dict[int, int] | None = None

    def get(self, comp_key: str, level: int | None = None) -> int:
        if comp_key == "plain":
            return self.plain
        levels = getattr(self, comp_key, None)
        if levels is None or level is None:
            return 0
        return levels.get(level, 0)


class CompressionResult(BaseModel):
    """Per-zoom compression data for one tile format."""
    zooms: dict[int, ZoomCompression]


_EMPTY_ZOOM = ZoomCompression()


def sample_tiles(db_path: Path, stored_compressed: bool, n: int) -> list[bytes]:
    """Sample up to n random tiles for zstd dictionary training."""
    conn = sqlite3.connect(str(db_path))
    rows = conn.execute(
        "SELECT tile_data FROM tiles ORDER BY RANDOM() LIMIT ?", (n,)
    ).fetchall()
    conn.close()
    return [
        gzip.decompress(bytes(td)) if stored_compressed else bytes(td)
        for (td,) in rows
    ]


def _cache_key(db_path: Path, dict_size: int, dict_samples: int) -> str:
    """Stable hash from file identity + dict parameters."""
    stat = db_path.stat()
    raw = f"{db_path.resolve()}:{stat.st_size}:{stat.st_mtime_ns}:{dict_size}:{dict_samples}"
    return hashlib.sha256(raw.encode()).hexdigest()[:16]


def _cache_path(out: Path, name: str, key: str) -> Path:
    return out / f".cache_{name}_{key}.json"


def load_or_compute_compression(
    name: str, db_path: Path, stored_compressed: bool,
    zstd_dict: "zstandard.ZstdCompressionDict",
    dict_size: int, dict_samples: int, cache_dir: Path,
) -> CompressionResult:
    """Load cached compression results or compute and cache them."""
    key = _cache_key(db_path, dict_size, dict_samples)
    cache = _cache_path(cache_dir, name.replace(" ", "_").replace("(", "").replace(")", ""), key)

    if cache.exists():
        print(f"  {name}: loading from cache")
        return CompressionResult.model_validate_json(cache.read_text())

    print(f"Computing compression variants for {name}...")
    result = query_all_compression_sizes(db_path, stored_compressed=stored_compressed, zstd_dict=zstd_dict)

    cache.write_text(result.model_dump_json())
    print(f"  {name}: cached to {cache.name}")

    return result


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
    db_path: Path, stored_compressed: bool,
    zstd_dict: "zstandard.ZstdCompressionDict | None" = None,
) -> CompressionResult:
    """Compute per-zoom total bytes for plain and multiple compression levels.

    Tests each algorithm at multiple settings to allow fair comparison:
      gzip  levels: 1 (fast), 6 (default), 9 (best)
      zstd  levels: 1, 3 (default), 9, 19
      zstd+dict: same levels but with a pre-trained dictionary
      brotli levels: 1, 6, 11 (max)

    If stored_compressed=True (MVT), blobs are gzip-compressed on disk and are
    decompressed first.  If False (MLT), blobs are raw.

    Streams tiles with fetchmany() to avoid loading everything into memory.
    """
    zstd_compressors = {lvl: zstandard.ZstdCompressor(level=lvl) for lvl in ZSTD_LEVELS}
    zstd_dict_compressors = (
        {lvl: zstandard.ZstdCompressor(level=lvl, dict_data=zstd_dict) for lvl in ZSTD_LEVELS}
        if zstd_dict is not None else None
    )
    result: dict[int, dict] = {}

    conn = sqlite3.connect(str(db_path))
    cur = conn.execute(
        "SELECT zoom_level, tile_data FROM tiles ORDER BY zoom_level"
    )

    def new_totals() -> dict:
        t: dict = {
            "plain": 0,
            "gzip": {lvl: 0 for lvl in GZIP_LEVELS},
            "zstd": {lvl: 0 for lvl in ZSTD_LEVELS},
            "brotli": {lvl: 0 for lvl in BROTLI_LEVELS},
        }
        if zstd_dict_compressors is not None:
            t["zstd_dict"] = {lvl: 0 for lvl in ZSTD_LEVELS}
        return t

    current_zoom = None
    totals = new_totals()

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
                totals = new_totals()

            blob = bytes(tile_data)
            raw = gzip.decompress(blob) if stored_compressed else blob

            totals["plain"] += len(raw)
            for lvl in GZIP_LEVELS:
                totals["gzip"][lvl] += len(gzip.compress(raw, compresslevel=lvl))
            for lvl in ZSTD_LEVELS:
                totals["zstd"][lvl] += len(zstd_compressors[lvl].compress(raw))
            for lvl in BROTLI_LEVELS:
                totals["brotli"][lvl] += len(brotli.compress(raw, quality=lvl))
            if zstd_dict_compressors is not None:
                for lvl in ZSTD_LEVELS:
                    totals["zstd_dict"][lvl] += len(zstd_dict_compressors[lvl].compress(raw))

    if current_zoom is not None:
        result[current_zoom] = totals
        print(f"  z{current_zoom}: {totals['plain'] / 1e6:.1f} MB plain", flush=True)

    conn.close()
    return CompressionResult(zooms={
        z: ZoomCompression(**data) for z, data in result.items()
    })


def write_fig(
    fig: go.Figure, out: Path, name: str, fmt: str,
    width: int = IMG_WIDTH, height: int = IMG_HEIGHT,
) -> None:
    if fmt == "html":
        path = out / f"{name}.html"
        fig.write_html(path)
    else:
        path_png = out / f"{name}.png"
        path_pdf = out / f"{name}.pdf"
        fig.write_image(path_png, width=width, height=height, scale=IMG_SCALE)
        fig.write_image(path_pdf, width=width, height=height, scale=IMG_SCALE)
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
    format_data: list[tuple[str, CompressionResult, str]],
    out: Path,
    fmt: str,
) -> None:
    """Plot 3×2 grid: plain + per-algorithm compression ratios across levels.

    Each panel shows one compression algorithm.  Within each panel, every
    (format × level) combination is drawn as a separate line so the user can
    see how the choice of compression level affects the result.

    format_data is a list of (name, CompressionResult, color) tuples.
    The first entry is used as the plain-MVT anchor (ratio = 1.0).
    """
    all_zooms = sorted(set().union(*(s.zooms.keys() for _, s, _ in format_data)))
    zoom_labels = [f"z{z}" for z in all_zooms]

    # Anchor: plain bytes from the first format (MVT)
    anchor = format_data[0][1]
    mvt_plain = {z: anchor.zooms[z].plain for z in all_zooms if z in anchor.zooms}

    # Check whether any format has zstd_dict data
    has_dict = any(
        next(iter(sizes.zooms.values()), _EMPTY_ZOOM).zstd_dict is not None
        for _, sizes, _ in format_data
    )

    dashes = ["solid", "dash", "dot", "dashdot", "longdash"]

    panels = [
        ("Plain (uncompressed)", "plain", [None]),
        ("GZip (levels 1 / 6 / 9)", "gzip", GZIP_LEVELS),
        ("ZSTD (levels 1 / 3 / 9 / 19)", "zstd", ZSTD_LEVELS),
    ]
    if has_dict:
        panels.append(("ZSTD + Dict (levels 1 / 3 / 9 / 19)", "zstd_dict", ZSTD_LEVELS))
    panels.append(("Brotli (levels 1 / 6 / 11)", "brotli", BROTLI_LEVELS))

    n_panels = len(panels)
    cols = 2
    rows = (n_panels + cols - 1) // cols
    positions = [(r + 1, c + 1) for r in range(rows) for c in range(cols)]

    fig = make_subplots(
        rows=rows, cols=cols,
        subplot_titles=[p[0] for p in panels] + [""] * (rows * cols - n_panels),
        vertical_spacing=0.08,
        horizontal_spacing=0.08,
    )

    group_titles = {
        "plain": "Plain",
        "gzip": "GZip",
        "zstd": "ZSTD",
        "zstd_dict": "ZSTD + Dict",
        "brotli": "Brotli",
    }

    for idx, (_, comp_key, levels) in enumerate(panels):
        row, col = positions[idx]
        for fmt_name, sizes, color in format_data:
            for i, lvl in enumerate(levels):
                ratios = []
                for z in all_zooms:
                    base = mvt_plain.get(z, 0)
                    val = sizes.zooms.get(z, _EMPTY_ZOOM).get(comp_key, lvl)
                    ratios.append(val / base if base > 0 else None)

                trace_name = fmt_name if comp_key == "plain" else f"{fmt_name} L{lvl}"

                fig.add_trace(go.Scatter(
                    name=trace_name,
                    x=zoom_labels,
                    y=ratios,
                    mode="lines+markers",
                    line=dict(color=color, width=2, dash=dashes[i % len(dashes)]),
                    marker=dict(size=5),
                    legendgroup=comp_key,
                    legendgrouptitle_text=group_titles[comp_key],
                ), row=row, col=col)

    # Plain MVT baseline (y=1.0) on every subplot
    for r in range(1, rows + 1):
        for c in range(1, cols + 1):
            fig.add_hline(
                y=1.0, line_dash="dash", line_color="#222222", line_width=1.5,
                opacity=0.6, row=r, col=c,
                annotation_text="Plain MVT" if c == cols else None,
                annotation_position="top right",
                annotation_font_size=10,
                annotation_font_color="#222222",
            )

    fig.update_yaxes(title_text="Ratio (lower is better)", col=1)
    fig.update_xaxes(title_text="Zoom Level", row=rows)
    fig.update_layout(
        **LAYOUT_DEFAULTS,
        title=None,
        legend=dict(
            groupclick="togglegroup",
            x=0.54,
            y=0.28,
            xanchor="left",
            yanchor="top",
        ),
    )

    write_fig(fig, out, "compression_levels", fmt, width=1600, height=500 * rows)


def main():
    parser = argparse.ArgumentParser(description="Plot tile size comparisons")
    parser.add_argument("mvt", type=Path, help="MVT mbtiles file (gzip-compressed blobs)")
    parser.add_argument("mlt_java_idsort", type=Path, help="MLT-Java mbtiles file, sort-by-id only (raw blobs)")
    parser.add_argument("mlt_java_noidsort", type=Path, help="MLT-Java mbtiles file, sort by geometry (raw blobs)")
    parser.add_argument("mlt_rust", type=Path, help="MLT-Rust mbtiles file (raw blobs)")
    parser.add_argument("--out", type=Path, default=Path("figures"), help="Output directory")
    parser.add_argument("--format", choices=["png", "html"], default="png")
    parser.add_argument("--dict-size", type=int, default=112 * 1024,
                        help="Zstd dictionary size in bytes (default: 112 KB)")
    parser.add_argument("--dict-samples", type=int, default=1000,
                        help="Number of tiles to sample for zstd dictionary training (default: 1000)")
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
        marker_color="#5E94D4",
    ))
    fig.add_trace(go.Bar(
        name="Rewritten (MLT)",
        x=zoom_labels, y=[b / 1e6 for b in rewr_total],
        text=[fmt_bytes(b) for b in rewr_total],
        textposition="auto",
        marker_color="#9FBA36",
    ))
    fig.update_layout(
        **LAYOUT_DEFAULTS,
        title=f"Total Tile Size per Zoom Level (overall: {fmt_bytes(grand_orig)} → {fmt_bytes(grand_rewr)}, {grand_pct:.1f}% reduction)",
        xaxis_title="Zoom Level",
        yaxis_title="Total Size (MB)",
        barmode="group",
    )
    write_fig(fig, args.out, "size_total_per_zoom", args.format)

    # ── 2. Average tile size per zoom ─────────────────────────────────────────
    fig = go.Figure()
    fig.add_trace(go.Bar(
        name="Original (MVT)",
        x=zoom_labels, y=[b / 1e3 for b in orig_avg],
        text=[fmt_bytes(b) for b in orig_avg],
        textposition="auto",
        marker_color="#5E94D4",
    ))
    fig.add_trace(go.Bar(
        name="Rewritten (MLT)",
        x=zoom_labels, y=[b / 1e3 for b in rewr_avg],
        text=[fmt_bytes(b) for b in rewr_avg],
        textposition="auto",
        marker_color="#9FBA36",
    ))
    fig.update_layout(
        **LAYOUT_DEFAULTS,
        title="Average Tile Size per Zoom Level",
        xaxis_title="Zoom Level",
        yaxis_title="Average Size (KB)",
        barmode="group",
    )
    write_fig(fig, args.out, "size_avg_per_zoom", args.format)

    # ── 3. Size reduction percentage per zoom ─────────────────────────────────
    colors = [IMPROVEMENT_COLOR if p > 0 else REGRESSION_COLOR for p in reduction_pct]
    fig = go.Figure()
    fig.add_trace(go.Bar(
        x=zoom_labels, y=reduction_pct,
        text=[f"{p:.1f}%" for p in reduction_pct],
        textposition="auto",
        marker_color=colors,
    ))
    fig.update_layout(
        **LAYOUT_DEFAULTS,
        title=f"Size Reduction by Zoom Level (overall {grand_pct:.1f}%)",
        xaxis_title="Zoom Level",
        yaxis_title="Reduction (%)",
    )
    write_fig(fig, args.out, "size_reduction_pct", args.format)

    # ── 4. Tile count comparison ──────────────────────────────────────────────
    fig = go.Figure()
    fig.add_trace(go.Bar(
        name="Original",
        x=zoom_labels, y=orig_count,
        text=[str(c) for c in orig_count],
        textposition="auto",
        marker_color="#5E94D4",
    ))
    fig.add_trace(go.Bar(
        name="Rewritten",
        x=zoom_labels, y=rewr_count,
        text=[str(c) for c in rewr_count],
        textposition="auto",
        marker_color="#9FBA36",
    ))
    dropped = sum(orig_count) - sum(rewr_count)
    fig.update_layout(
        **LAYOUT_DEFAULTS,
        title=f"Tile Count per Zoom Level ({dropped} tiles dropped)",
        xaxis_title="Zoom Level",
        yaxis_title="Tile Count",
        barmode="group",
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
        line=dict(color="#5E94D4", width=3),
    ))
    fig.add_trace(go.Scatter(
        name="Rewritten (MLT)",
        x=zoom_labels, y=[b / 1e9 for b in cum_rewr],
        mode="lines+markers",
        line=dict(color="#9FBA36", width=3),
    ))
    fig.update_layout(
        **LAYOUT_DEFAULTS,
        title="Cumulative Tile Data Size",
        xaxis_title="Zoom Level",
        yaxis_title="Cumulative Size (GB)",
    )
    write_fig(fig, args.out, "size_cumulative", args.format)

    # ── 6. Per-zoom compression ratio ─────────────────────────────────────────
    # Train per-format zstd dictionaries
    print(f"\nTraining zstd dictionaries ({fmt_bytes(args.dict_size)}, {args.dict_samples} samples)...")
    sources = [
        ("MVT", args.mvt, True),
        ("MLT-Java (ID-sort)", args.mlt_java_idsort, False),
        ("MLT-Java (Geo-sort)", args.mlt_java_noidsort, False),
        ("MLT-Rust", args.mlt_rust, False),
    ]
    zstd_dicts = {}
    for name, path, compressed in sources:
        samples = sample_tiles(path, stored_compressed=compressed, n=args.dict_samples)
        zstd_dicts[name] = zstandard.train_dictionary(args.dict_size, samples)
        print(f"  {name}: dict {len(zstd_dicts[name].as_bytes())} bytes from {len(samples)} samples")

    comp_results = {}
    for name, path, compressed in sources:
        comp_results[name] = load_or_compute_compression(
            name, path, compressed, zstd_dicts[name],
            args.dict_size, args.dict_samples, args.out,
        )
    mvt_comp = comp_results["MVT"]
    java_idsort_comp = comp_results["MLT-Java (ID-sort)"]
    java_noidsort_comp = comp_results["MLT-Java (Geo-sort)"]
    rust_comp = comp_results["MLT-Rust"]

    plot_compression_ratio(
        [
            ("MVT", mvt_comp, "#5E94D4"),
            ("MLT-Java (ID-sort)", java_idsort_comp, "#F7811E"),
            ("MLT-Java (Geo-sort)", java_noidsort_comp, "#B55CA5"),
            ("MLT-Rust", rust_comp, "#9FBA36"),
        ],
        args.out,
        args.format,
    )

    print(f"\nAll figures written to {args.out}/")


if __name__ == "__main__":
    main()
