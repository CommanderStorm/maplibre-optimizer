use maplibre_style_spec::spec::decoder::{StyleReference, TopLevelItem};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};

// objects produced by errors here are too large to review
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
            // current object must be causing the mishap
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
    let v8_path = PathBuf::from("../upstream/src/reference/v8.json");

    let content = std::fs::read_to_string(v8_path).unwrap();
    let mut style: HashMap<String, Value> = serde_json::from_str(&content).unwrap();
    assert_eq!(style.remove("$version"), Some(Value::Number(8.into())));

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
    let v8_path = PathBuf::from("../upstream/src/reference/v8.json");

    let content = std::fs::read_to_string(v8_path).unwrap();
    let style: StyleReference = serde_json::from_str(&content).unwrap();
    assert_eq!(style.version, 8);
}
