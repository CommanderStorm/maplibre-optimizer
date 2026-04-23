"""Shared plot style constants for optimizer benchmark figures.

Provides a unified color palette and layout defaults that match the
Helvetica-based TUM thesis template.
"""

# ── Thesis color palette ────────────────────────────────────────────────────

COLORS = {
    "light_blue": "#5E94D4",
    "green": "#9FBA36",
    "orange": "#F7811E",
    "pink": "#B55CA5",
    "purple": "#8F81EA",
    "yellow": "#FED702",
    "grey": "#AAAAAA",
    "grey_dark": "#999999",
    "grey_light": "#BBBBBB",
}

# Semantic aliases for bar charts with improvement/regression semantics
IMPROVEMENT_COLOR = COLORS["green"]    # #9FBA36
REGRESSION_COLOR = COLORS["orange"]    # #F7811E
NEUTRAL_COLOR = COLORS["grey_dark"]    # #999999

# ── Layout defaults ─────────────────────────────────────────────────────────

LAYOUT_DEFAULTS = dict(
    template="plotly_white",
    font=dict(family="Helvetica, Arial, sans-serif", size=12),
    margin=dict(l=70, r=30, t=40, b=60),
)

# ── Image export dimensions ─────────────────────────────────────────────────

IMG_WIDTH = 1400
IMG_HEIGHT = 700
IMG_SCALE = 2  # 2x for retina-quality PNGs
