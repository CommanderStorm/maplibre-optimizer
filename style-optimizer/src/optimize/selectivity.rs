//! Data-driven selectivity estimation for filter predicates.

use serde_json::Value;

use super::expr::extract_json_literal;
use super::expr::util::{get_prop_name, is_get_expr, json_as_i64, json_as_u64};
use crate::stats::{LayerStats, PropertyStats, TileStatistics};

/// Estimate the selectivity (fraction of features matching) for a predicate expression,
/// given tile statistics for the layer's source and source-layer.
///
/// Returns `None` if the expression shape is not recognised or if stats are unavailable.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn estimate_selectivity(
    predicate: &Value,
    source: &str,
    source_layer: &str,
    stats: &TileStatistics,
) -> Option<f64> {
    let layer_stats = stats.layer_stats(source, source_layer)?;
    if layer_stats.total_features == 0 {
        return None;
    }
    estimate_expr(predicate, layer_stats)
}

#[allow(clippy::cast_precision_loss)]
fn estimate_expr(expr: &Value, stats: &LayerStats) -> Option<f64> {
    let Value::Array(arr) = expr else {
        return None;
    };
    if arr.is_empty() {
        return None;
    }
    let op = arr[0].as_str()?;
    let total = stats.total_features as f64;

    match op {
        "==" if arr.len() == 3 => estimate_eq(&arr[1], &arr[2], stats, total),
        "!=" if arr.len() == 3 => estimate_eq(&arr[1], &arr[2], stats, total).map(|s| 1.0 - s),

        "<" if arr.len() == 3 => estimate_range_lt(&arr[1], &arr[2], stats, total, false),
        "<=" if arr.len() == 3 => estimate_range_lt(&arr[1], &arr[2], stats, total, true),
        ">" if arr.len() == 3 => {
            estimate_range_lt(&arr[1], &arr[2], stats, total, true).map(|s| 1.0 - s)
        }
        ">=" if arr.len() == 3 => {
            estimate_range_lt(&arr[1], &arr[2], stats, total, false).map(|s| 1.0 - s)
        }

        "has" if arr.len() == 2 => {
            let prop = arr[1].as_str()?;
            let ps = stats.properties.get(prop)?;
            Some(ps.present_count() as f64 / total)
        }

        "!" if arr.len() == 2 => estimate_expr(&arr[1], stats).map(|s| 1.0 - s),

        "all" => {
            let mut product = 1.0;
            for child in arr.iter().skip(1) {
                product *= estimate_expr(child, stats)?;
            }
            Some(product)
        }

        "any" => {
            let mut product_complement = 1.0;
            for child in arr.iter().skip(1) {
                product_complement *= 1.0 - estimate_expr(child, stats)?;
            }
            Some(1.0 - product_complement)
        }

        "in" if arr.len() == 3 => estimate_in(&arr[1], &arr[2], stats, total),

        _ => None,
    }
}

/// Estimate selectivity for `["==", lhs, rhs]`.
#[allow(clippy::cast_precision_loss)]
fn estimate_eq(lhs: &Value, rhs: &Value, stats: &LayerStats, total: f64) -> Option<f64> {
    // ["==", ["geometry-type"], "Point"|"LineString"|"Polygon"]
    if is_geometry_type_expr(lhs) {
        let lit = extract_json_literal(rhs)?;
        let gt = lit.as_str()?;
        return Some(geometry_type_count(gt, stats) as f64 / total);
    }
    if is_geometry_type_expr(rhs) {
        let lit = extract_json_literal(lhs)?;
        let gt = lit.as_str()?;
        return Some(geometry_type_count(gt, stats) as f64 / total);
    }

    // ["==", ["id"], n]
    if is_id_expr(lhs) {
        return if stats.has_feature_ids {
            None
        } else {
            Some(0.0)
        };
    }
    if is_id_expr(rhs) {
        return if stats.has_feature_ids {
            None
        } else {
            Some(0.0)
        };
    }

    // ["==", ["get", prop], literal]
    let (prop, lit) = extract_get_and_literal(lhs, rhs)?;
    estimate_eq_for_prop(prop, &lit, stats, total)
}

#[allow(clippy::cast_precision_loss)]
fn estimate_eq_for_prop(prop: &str, lit: &Value, stats: &LayerStats, total: f64) -> Option<f64> {
    let ps = stats.properties.get(prop)?;
    match ps {
        PropertyStats::Bool {
            present_count,
            true_count,
        } => {
            let b = lit.as_bool()?;
            let count = if b {
                *true_count
            } else {
                present_count.saturating_sub(*true_count)
            };
            Some(count as f64 / total)
        }
        PropertyStats::Integer {
            value_counts,
            present_count,
            cardinality,
            ..
        } => {
            if let Some(vc) = value_counts {
                let n = json_as_i64(lit)?;
                let count = vc.get(&n).copied().unwrap_or(0);
                Some(count as f64 / total)
            } else if *cardinality > 0 {
                Some(*present_count as f64 / total / *cardinality as f64)
            } else {
                None
            }
        }
        PropertyStats::UnsignedInteger {
            value_counts,
            present_count,
            cardinality,
            ..
        } => {
            if let Some(vc) = value_counts {
                let n = json_as_u64(lit)?;
                let count = vc.get(&n).copied().unwrap_or(0);
                Some(count as f64 / total)
            } else if *cardinality > 0 {
                Some(*present_count as f64 / total / *cardinality as f64)
            } else {
                None
            }
        }
        PropertyStats::String {
            value_counts,
            present_count,
            cardinality,
            ..
        } => {
            if let Some(vc) = value_counts {
                let s = lit.as_str()?;
                let count = vc.get(s).copied().unwrap_or(0);
                Some(count as f64 / total)
            } else if *cardinality > 0 {
                Some(*present_count as f64 / total / *cardinality as f64)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Estimate selectivity for `["<", lhs, rhs]` (or `["<=", ...]` when `inclusive` is true).
#[allow(clippy::cast_precision_loss, clippy::nonminimal_bool)]
fn estimate_range_lt(
    lhs: &Value,
    rhs: &Value,
    stats: &LayerStats,
    total: f64,
    inclusive: bool,
) -> Option<f64> {
    // Only handle ["<", ["get", prop], n] form.
    let (prop, is_get_first) = if is_get_expr(lhs) {
        (get_prop_name(lhs)?, true)
    } else if is_get_expr(rhs) {
        (get_prop_name(rhs)?, false)
    } else {
        return None;
    };

    let lit = if is_get_first {
        extract_json_literal(rhs)?
    } else {
        extract_json_literal(lhs)?
    };

    let ps = stats.properties.get(prop)?;

    match ps {
        PropertyStats::Integer {
            value_counts: Some(vc),
            ..
        } => {
            let n = json_as_i64(&lit)?;
            let count: u64 = if is_get_first {
                // ["<", ["get", prop], n] or ["<=", ["get", prop], n]
                if inclusive {
                    vc.range(..=n).map(|(_, c)| c).sum()
                } else {
                    vc.range(..n).map(|(_, c)| c).sum()
                }
            } else {
                // ["<", n, ["get", prop]] means get(prop) > n
                // This is handled by the caller via 1 - estimate
                if inclusive {
                    vc.range(..=n).map(|(_, c)| c).sum()
                } else {
                    vc.range(..n).map(|(_, c)| c).sum()
                }
            };
            Some(count as f64 / total)
        }
        PropertyStats::Integer { min, max, .. } => {
            let n = json_as_i64(&lit)?;
            if (is_get_first && !inclusive && n <= *min) || (is_get_first && inclusive && n < *min)
            {
                Some(0.0)
            } else if (is_get_first && !inclusive && n > *max)
                || (is_get_first && inclusive && n >= *max)
            {
                Some(ps.present_count() as f64 / total)
            } else {
                None
            }
        }
        PropertyStats::UnsignedInteger {
            value_counts: Some(vc),
            ..
        } => {
            let n = json_as_u64(&lit)?;
            let count: u64 = if inclusive {
                vc.range(..=n).map(|(_, c)| c).sum()
            } else {
                vc.range(..n).map(|(_, c)| c).sum()
            };
            Some(count as f64 / total)
        }
        PropertyStats::Double { min, max, .. } => {
            let n = lit.as_f64()?;
            if n <= *min {
                Some(0.0)
            } else if n > *max {
                Some(ps.present_count() as f64 / total)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Estimate selectivity for `["in", ["get", prop], ["literal", [v1, v2, ...]]]`.
#[allow(clippy::cast_precision_loss)]
fn estimate_in(input: &Value, values_expr: &Value, stats: &LayerStats, total: f64) -> Option<f64> {
    let prop = get_prop_name(input)?;
    // values_expr should be ["literal", [...]]
    let Value::Array(lit_arr) = values_expr else {
        return None;
    };
    if lit_arr.len() != 2 || lit_arr[0].as_str() != Some("literal") {
        return None;
    }
    let Value::Array(values) = &lit_arr[1] else {
        return None;
    };

    let mut sum = 0.0;
    for v in values {
        sum += estimate_eq_for_prop(prop, v, stats, total)?;
    }
    Some(sum.min(1.0))
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_geometry_type_expr(v: &Value) -> bool {
    matches!(v, Value::Array(a) if a.len() == 1 && a[0].as_str() == Some("geometry-type"))
}

fn is_id_expr(v: &Value) -> bool {
    matches!(v, Value::Array(a) if a.len() == 1 && a[0].as_str() == Some("id"))
}

pub(crate) fn extract_get_and_literal<'a>(
    lhs: &'a Value,
    rhs: &'a Value,
) -> Option<(&'a str, Value)> {
    if let Some(prop) = get_prop_name(lhs) {
        let lit = extract_json_literal(rhs)?;
        return Some((prop, lit));
    }
    if let Some(prop) = get_prop_name(rhs) {
        let lit = extract_json_literal(lhs)?;
        return Some((prop, lit));
    }
    None
}

fn geometry_type_count(gt: &str, stats: &LayerStats) -> u64 {
    match gt {
        "Point" => stats.geometry_types.point,
        "LineString" => stats.geometry_types.linestring,
        "Polygon" => stats.geometry_types.polygon,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use indexmap::IndexMap;
    use serde_json::json;

    use super::*;
    use crate::stats::{GeometryTypeStats, SourceStats};

    fn sample_stats() -> TileStatistics {
        let mut props = BTreeMap::new();
        let mut vc = IndexMap::new();
        vc.insert("motorway".to_string(), 5000u64);
        vc.insert("service".to_string(), 90000u64);
        vc.insert("secondary".to_string(), 5000u64);
        props.insert(
            "class".to_string(),
            PropertyStats::String {
                present_count: 100_000,
                cardinality: 3,
                value_counts: Some(vc),
            },
        );

        let mut int_vc = BTreeMap::new();
        int_vc.insert(2i64, 500);
        int_vc.insert(4, 4000);
        int_vc.insert(6, 8000);
        props.insert(
            "admin_level".to_string(),
            PropertyStats::Integer {
                present_count: 12_500,
                min: 2,
                max: 6,
                cardinality: 3,
                value_counts: Some(int_vc),
            },
        );

        let mut layers = BTreeMap::new();
        layers.insert(
            "transportation".to_string(),
            LayerStats {
                total_features: 100_000,
                features_by_zoom: BTreeMap::new(),
                geometry_types: GeometryTypeStats {
                    unknown: 0,
                    point: 0,
                    linestring: 100_000,
                    polygon: 0,
                },
                has_feature_ids: false,
                properties: props,
            },
        );

        let mut sources = BTreeMap::new();
        sources.insert("openmaptiles".to_string(), SourceStats { layers });

        TileStatistics {
            sources,
            sample_rate: 1.0,
        }
    }

    #[test]
    fn eq_string_selectivity() {
        let stats = sample_stats();
        let pred = json!(["==", ["get", "class"], "motorway"]);
        let s = estimate_selectivity(&pred, "openmaptiles", "transportation", &stats);
        assert!((s.unwrap() - 0.05).abs() < 1e-9);
    }

    #[test]
    fn neq_selectivity() {
        let stats = sample_stats();
        let pred = json!(["!=", ["get", "class"], "motorway"]);
        let s = estimate_selectivity(&pred, "openmaptiles", "transportation", &stats);
        assert!((s.unwrap() - 0.95).abs() < 1e-9);
    }

    #[test]
    fn has_selectivity() {
        let stats = sample_stats();
        let pred = json!(["has", "class"]);
        let s = estimate_selectivity(&pred, "openmaptiles", "transportation", &stats);
        assert!((s.unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn range_lt_selectivity() {
        let stats = sample_stats();
        // admin_level < 4 → only value 2 (500 features) out of 100_000
        let pred = json!(["<", ["get", "admin_level"], 4]);
        let s = estimate_selectivity(&pred, "openmaptiles", "transportation", &stats);
        assert!((s.unwrap() - 0.005).abs() < 1e-9);
    }

    #[test]
    fn geometry_type_selectivity() {
        let stats = sample_stats();
        let pred = json!(["==", ["geometry-type"], "LineString"]);
        let s = estimate_selectivity(&pred, "openmaptiles", "transportation", &stats);
        assert!((s.unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn unknown_source_returns_none() {
        let stats = sample_stats();
        let pred = json!(["==", ["get", "class"], "motorway"]);
        let s = estimate_selectivity(&pred, "nosource", "transportation", &stats);
        assert!(s.is_none());
    }

    #[test]
    fn id_no_ids_returns_zero() {
        let stats = sample_stats();
        let pred = json!(["==", ["id"], 42]);
        let s = estimate_selectivity(&pred, "openmaptiles", "transportation", &stats);
        assert!((s.unwrap() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn all_compound_selectivity() {
        let stats = sample_stats();
        let pred = json!([
            "all",
            ["==", ["get", "class"], "motorway"],
            ["has", "class"]
        ]);
        let s = estimate_selectivity(&pred, "openmaptiles", "transportation", &stats);
        // 0.05 * 1.0 = 0.05
        assert!((s.unwrap() - 0.05).abs() < 1e-9);
    }
}
