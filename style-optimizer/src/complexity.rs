//! Static complexity metrics for style JSON documents.
//!
//! Uses the MIR-guided walker from [`crate::optimize::walk`] to traverse only
//! schema-valid filter and property expressions, producing accurate operator
//! histograms and depth metrics.
//!
//! Metrics are designed to capture both structural complexity (expression nesting)
//! and surface area (total properties), so every optimizer pass — whether it removes
//! layers, strips scalar defaults, or simplifies expressions — shows up in at least
//! one metric.

use std::collections::BTreeMap;

use maplibre_style_spec::mir::MirSpec;
use serde::Serialize;
use serde_json::Value;

use crate::optimize::walk::{PropertyContext, StyleVisitor, walk_style_mut};

/// Aggregated complexity metrics for a style document.
#[derive(Debug, Clone, Serialize)]
pub struct ComplexityReport {
    /// Number of layers in the style.
    pub layer_count: usize,
    /// Number of layers with a filter expression.
    pub filter_count: usize,

    /// Total paint/layout properties across all layers (scalar + expression).
    /// Reduced by `strip_defaults`, `dead_elimination`, `cleanup`.
    pub property_count: usize,
    /// Properties whose value is a data/zoom expression (JSON array).
    pub expression_property_count: usize,
    /// Properties whose value is a plain scalar (string, number, bool).
    pub scalar_property_count: usize,

    /// Total JSON nodes (arrays, objects, scalars) within all expressions.
    /// Captures the full "weight" of expressions including literal values.
    pub total_expression_nodes: usize,
    /// Number of JSON array/object structural nodes within expressions.
    pub ast_nodes: usize,
    /// Maximum nesting depth of any expression.
    pub max_depth: usize,

    /// Histogram of expression operator occurrences (e.g. "match" → 5).
    pub expression_types: BTreeMap<String, usize>,
}

/// Compute complexity metrics for a style JSON value using the MIR-guided walker.
pub fn complexity_report(style: &mut Value, mir: &MirSpec) -> ComplexityReport {
    let layer_count = style
        .as_object()
        .and_then(|r| r.get("layers"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);

    let mut visitor = ComplexityVisitor {
        report: ComplexityReport {
            layer_count,
            filter_count: 0,
            property_count: 0,
            expression_property_count: 0,
            scalar_property_count: 0,
            total_expression_nodes: 0,
            ast_nodes: 0,
            max_depth: 0,
            expression_types: BTreeMap::new(),
        },
    };

    walk_style_mut(style, mir, &mut visitor);
    visitor.report
}

struct ComplexityVisitor {
    report: ComplexityReport,
}

impl ComplexityVisitor {
    fn walk_expression(&mut self, value: &Value, depth: usize) {
        self.report.total_expression_nodes += 1;

        match value {
            Value::Array(arr) => {
                self.report.ast_nodes += 1;
                self.report.max_depth = self.report.max_depth.max(depth + 1);

                if let Some(Value::String(op)) = arr.first() {
                    *self.report.expression_types.entry(op.clone()).or_insert(0) += 1;
                }

                for item in arr {
                    self.walk_expression(item, depth + 1);
                }
            }
            Value::Object(obj) => {
                self.report.ast_nodes += 1;
                self.report.max_depth = self.report.max_depth.max(depth + 1);
                for v in obj.values() {
                    self.walk_expression(v, depth + 1);
                }
            }
            // Scalars (strings, numbers, bools, nulls) inside expressions
            _ => {}
        }
    }
}

impl StyleVisitor for ComplexityVisitor {
    fn visit_filter(&mut self, _layer_index: usize, _layer_type: &str, filter: &mut Value) {
        self.report.filter_count += 1;
        self.walk_expression(filter, 0);
    }

    fn visit_property(&mut self, _ctx: &PropertyContext<'_>, value: &mut Value) {
        self.report.property_count += 1;

        if value.is_array() {
            self.report.expression_property_count += 1;
            self.walk_expression(value, 0);
        } else {
            self.report.scalar_property_count += 1;
        }
    }
}
