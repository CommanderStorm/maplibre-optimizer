//! Style JSON rewrite passes (expression simplification, etc.).

use maplibre_style_spec::mir::IntermediateSpec;
use serde_json::Value;

/// Toggleable optimization passes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OptPasses {
    /// Unary boolean simplifications: `["any", e]` / `["all", e]` → `e`, and `["!", ["!", e]]` → `e`.
    pub simplify_unary: bool,
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
            if passes.simplify_unary && arr.len() == 2 {
                match arr.first().and_then(Value::as_str) {
                    Some("any" | "all") => {
                        let inner = arr[1].take();
                        *v = inner;
                        optimize_value_recursive(v, passes);
                    }
                    Some("!") => {
                        if let Value::Array(inner) = &arr[1] {
                            if inner.len() == 2 && inner[0].as_str() == Some("!") {
                                let grand = inner[1].clone();
                                *v = grand;
                                optimize_value_recursive(v, passes);
                            }
                        }
                    }
                    _ => {}
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
    use std::path::Path;

    use crate::load_intermediate_spec_from_v8_path;

    use super::*;

    fn sample_mir() -> IntermediateSpec {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
        load_intermediate_spec_from_v8_path(&path).expect("v8.json")
    }

    fn passes_on() -> OptPasses {
        OptPasses {
            simplify_unary: true,
        }
    }

    #[test]
    fn simplify_unary_any_in_filter() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "filter": ["any", ["==", 1, 1]]
        });
        optimize_style_json_value(&mut v, &mir, &passes_on());
        assert_eq!(v, serde_json::json!({ "filter": ["==", 1, 1] }));
    }

    #[test]
    fn simplify_unary_disabled_is_noop() {
        let mir = sample_mir();
        let original = serde_json::json!({
            "filter": ["any", ["==", 1, 1]]
        });
        let mut v = original.clone();
        optimize_style_json_value(&mut v, &mir, &OptPasses::default());
        assert_eq!(v, original);
    }

    #[test]
    fn simplify_nested_unary_any() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "filter": ["any", ["any", ["==", 1, 1]]]
        });
        optimize_style_json_value(&mut v, &mir, &passes_on());
        assert_eq!(v, serde_json::json!({ "filter": ["==", 1, 1] }));
    }

    #[test]
    fn simplify_unary_all() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "filter": ["all", ["==", 1, 1]]
        });
        optimize_style_json_value(&mut v, &mir, &passes_on());
        assert_eq!(v, serde_json::json!({ "filter": ["==", 1, 1] }));
    }

    #[test]
    fn bare_any_single_element_unchanged() {
        let mir = sample_mir();
        let mut v = serde_json::json!({ "filter": ["any"] });
        let expected = v.clone();
        optimize_style_json_value(&mut v, &mir, &passes_on());
        assert_eq!(v, expected);
    }

    #[test]
    fn simplify_double_not() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "filter": ["!", ["!", ["has", "x"]]]
        });
        optimize_style_json_value(&mut v, &mir, &passes_on());
        assert_eq!(v, serde_json::json!({ "filter": ["has", "x"] }));
    }

    #[test]
    fn simplify_triple_not() {
        let mir = sample_mir();
        let mut v = serde_json::json!({
            "filter": ["!", ["!", ["!", ["has", "x"]]]]
        });
        optimize_style_json_value(&mut v, &mir, &passes_on());
        assert_eq!(v, serde_json::json!({ "filter": ["!", ["has", "x"]] }));
    }
}
