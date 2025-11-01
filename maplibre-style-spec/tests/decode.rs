use maplibre_style_spec::decoder::{ParsedItem, StyleReference, TopLevelItem};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};

#[test]
fn test_decode_top_level() {
    let v8_path = PathBuf::from("tests/upstream/src/reference/v8.json");

    let content = std::fs::read_to_string(v8_path).unwrap();
    let mut style: HashMap<String, Value> = serde_json::from_str(&content).unwrap();
    assert_eq!(style.remove("$version"), Some(Value::Number(8.into())));

    for (key, value) in style {
        let Value::Object(o) = value.clone() else {
            if let Err(e) = serde_json::from_value::<TopLevelItem>(value.clone()) {
                panic!("Failed to parse {key} {e:?}.\n\nWas {value:#?}.");
            }
            return;
        };
        for (subkey, subvalue) in o {
            if matches!(subvalue, Value::String(_) | Value::Number(_)) {
                if let Err(e) = serde_json::from_value::<TopLevelItem>(value.clone()) {
                    panic!("Failed to parse {key} {e:?}.\n\nWas {value:#?}.");
                }
                break;
            }
            if let Err(e) = serde_json::from_value::<ParsedItem>(subvalue.clone()) {
                panic!("Failed to parse {key}.{subkey} {e:?}.\n\nWas {subvalue:#?}.");
            }
        }
    }
}

#[test]
fn test_decode_whole_reference() {
    let v8_path = PathBuf::from("tests/upstream/src/reference/v8.json");

    let content = std::fs::read_to_string(v8_path).unwrap();
    let style: StyleReference = serde_json::from_str(&content).unwrap();
    assert_eq!(style.version, 8);
}
