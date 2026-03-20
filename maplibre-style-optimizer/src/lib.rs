//! JSON-tree optimizations for MapLibre style documents.
//!
//! Operates on [`serde_json::Value`] so root keys not yet in generated `spec.rs` are preserved.

use std::fs;
use std::path::Path;

use anyhow::Context;
use maplibre_style_spec::decoder::StyleReference;
use maplibre_style_spec::mir::IntermediateSpec;
use serde_json::Value;

/// Toggleable optimization passes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OptPasses {
    /// Replace `["any", expr]` with `expr` (semantics-preserving when `expr` is boolean).
    pub fold_unary_any: bool,
}

/// Load MIR from a MapLibre style reference `v8.json` on disk.
pub fn load_intermediate_spec_from_v8_path(path: &Path) -> anyhow::Result<IntermediateSpec> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read reference {}", path.display()))?;
    let reference: StyleReference = serde_json::from_str(&text)
        .with_context(|| format!("parse reference {}", path.display()))?;
    Ok(IntermediateSpec::from(reference))
}

/// Ensure the reference defines an expression operator (sanity check against wrong `v8.json`).
pub fn ensure_expression_operator(mir: &IntermediateSpec, name: &str) -> anyhow::Result<()> {
    if mir.expressions.operators.contains_key(name) {
        Ok(())
    } else {
        anyhow::bail!("reference MIR missing expression operator {name:?}");
    }
}

/// Apply enabled passes to every array and object in the style JSON tree.
pub fn optimize_style_json_value(v: &mut Value, _mir: &IntermediateSpec, passes: &OptPasses) {
    optimize_value_recursive(v, passes);
}

fn optimize_value_recursive(v: &mut Value, passes: &OptPasses) {
    match v {
        Value::Array(arr) => {
            for x in arr.iter_mut() {
                optimize_value_recursive(x, passes);
            }
            if passes.fold_unary_any && arr.len() == 2 {
                if matches!(arr.first().and_then(Value::as_str), Some("any")) {
                    let inner = arr[1].take();
                    *v = inner;
                    optimize_value_recursive(v, passes);
                }
            }
        }
        Value::Object(map) => {
            for x in map.values_mut() {
                optimize_value_recursive(x, passes);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mir() -> IntermediateSpec {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
        load_intermediate_spec_from_v8_path(&path).expect("v8.json")
    }

    #[test]
    fn fold_unary_any_in_filter() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "filter": ["any", ["==", 1, 1]]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                fold_unary_any: true,
            },
        );
        assert_eq!(v, serde_json::json!({ "filter": ["==", 1, 1] }));
    }

    #[test]
    fn fold_unary_any_disabled_is_noop() {
        let mir = sample_mir();
        let original = serde_json::json!({
            "filter": ["any", ["==", 1, 1]]
        });
        let mut v = original.clone();
        optimize_style_json_value(&mut v, &mir, &OptPasses::default());
        assert_eq!(v, original);
    }

    #[test]
    fn fold_nested_unary_any() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "filter": ["any", ["any", ["==", 1, 1]]]
        });
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                fold_unary_any: true,
            },
        );
        assert_eq!(v, serde_json::json!({ "filter": ["==", 1, 1] }));
    }

    #[test]
    fn bare_any_single_element_unchanged() {
        let mir = sample_mir();
        let mut v = serde_json::json!({ "filter": ["any"] });
        let expected = v.clone();
        optimize_style_json_value(
            &mut v,
            &mir,
            &OptPasses {
                fold_unary_any: true,
            },
        );
        assert_eq!(v, expected);
    }
}
