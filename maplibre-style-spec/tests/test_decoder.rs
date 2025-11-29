use std::collections::BTreeMap;

use maplibre_style_spec::decoder::{ParsedItem, StyleReference, TopLevelItem};
use serde_json::Value;

// objects produced by errors here are too large to review;
// to produce better error messages, we need to remove keys that don't cause errors
fn minimise_object(check_still_produces_error: fn(Value) -> bool, value: Value) -> Value {
    let values_to_try_to_remove = if let Value::Object(o) = value.clone() {
        o.keys().cloned().collect::<Vec<_>>()
    } else {
        vec![]
    };
    let mut minimized = value.clone();
    for outer_key in values_to_try_to_remove {
        let outer_val = minimized
            .as_object_mut()
            .unwrap()
            .remove(&outer_key)
            .unwrap();
        if !check_still_produces_error(minimized.clone()) {
            // the current object must be causing the mishap
            minimized.as_object_mut().unwrap().clear();
            let _ = minimized
                .as_object_mut()
                .unwrap()
                .insert(outer_key.clone(), outer_val.clone());
            return minimized;
        }
    }
    minimized
}

#[test]
fn test_decode_top_level() {
    let content = include_str!("../../upstream/src/reference/v8.json");
    let mut style: BTreeMap<String, Value> = serde_json::from_str(content).unwrap();
    assert_eq!(style.remove("$version"), Some(Value::Number(8.into())));
    if let Some(root) = style.remove("$root") {
        let root_items = serde_json::from_value::<BTreeMap<String, Value>>(root.clone())
            .expect("$root is a valid map of top level items.");
        for (key, root_item) in root_items {
            serde_json::from_value::<ParsedItem>(root_item.clone()).expect(&format!(
                "$root.{key} is not a valid ParsedItem\n{root_item:#?}"
            ));
        }
    }

    for (key, value) in style {
        if let Err(e) = serde_json::from_value::<TopLevelItem>(value.clone()) {
            if !value.is_object() {
                panic!("Failed to parse {key} {e:?}.\n\nWas {value:#?}.");
            }
            let minimized = minimise_object(
                |val| serde_json::from_value::<TopLevelItem>(val).is_err(),
                value.clone(),
            );

            panic!("Failed to parse {key} {e:?}.\n\nWas {minimized:#?}.");
        }
    }
}

#[test]
fn test_decode_whole_reference() {
    let content = include_str!("../../upstream/src/reference/v8.json");
    let style: StyleReference = serde_json::from_str(content).unwrap();
    assert_eq!(style.version, 8);
    assert!(!style.root.is_empty());
    assert!(!style.fields.is_empty())
}
