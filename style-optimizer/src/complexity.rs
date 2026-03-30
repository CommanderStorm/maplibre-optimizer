//! Static complexity metrics for style JSON documents.
//!
//! Uses the MIR-guided walker from [`crate::optimize::walk`] to traverse only
//! schema-valid filter and property expressions, producing accurate operator
//! histograms and depth metrics.

use std::collections::BTreeMap;

use maplibre_style_spec::mir::MirSpec;
use serde::Serialize;
use serde_json::Value;

use crate::optimize::walk::{PropertyContext, StyleVisitor, walk_style_mut};

/// Aggregated complexity metrics for a style document.
#[derive(Debug, Clone, Serialize)]
pub struct ComplexityReport {
    /// Total number of JSON array/object nodes in expressions.
    pub ast_nodes: usize,
    /// Maximum nesting depth of any expression.
    pub max_depth: usize,
    /// Number of layers in the style.
    pub layer_count: usize,
    /// Number of layers with a filter expression.
    pub filter_count: usize,
    /// Histogram of expression operator occurrences (e.g. "match" → 5).
    pub expression_types: BTreeMap<String, usize>,
}

/// Compute complexity metrics for a style JSON value using the MIR-guided walker.
pub fn complexity_report(style: &mut Value, mir: &MirSpec) -> ComplexityReport {
    // Count layers from the JSON before walking
    let layer_count = style
        .as_object()
        .and_then(|r| r.get("layers"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);

    let mut visitor = ComplexityVisitor {
        report: ComplexityReport {
            ast_nodes: 0,
            max_depth: 0,
            layer_count,
            filter_count: 0,
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
    fn walk_value(&mut self, value: &Value, depth: usize) {
        match value {
            Value::Array(arr) => {
                self.report.ast_nodes += 1;
                self.report.max_depth = self.report.max_depth.max(depth + 1);

                // First-element string = expression operator
                if let Some(Value::String(op)) = arr.first() {
                    *self.report.expression_types.entry(op.clone()).or_insert(0) += 1;
                }

                for item in arr {
                    self.walk_value(item, depth + 1);
                }
            }
            Value::Object(obj) => {
                self.report.ast_nodes += 1;
                self.report.max_depth = self.report.max_depth.max(depth + 1);
                for v in obj.values() {
                    self.walk_value(v, depth + 1);
                }
            }
            _ => {}
        }
    }
}

impl StyleVisitor for ComplexityVisitor {
    fn visit_filter(&mut self, _layer_index: usize, _layer_type: &str, filter: &mut Value) {
        self.report.filter_count += 1;
        self.walk_value(filter, 0);
    }

    fn visit_property(&mut self, _ctx: &PropertyContext<'_>, value: &mut Value) {
        self.walk_value(value, 0);
    }
}
