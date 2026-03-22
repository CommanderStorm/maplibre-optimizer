//! Roundtrip tests for the MaplibreStyleSpecification types.
#![cfg(feature = "full")]

use maplibre_style_spec::spec::*;

#[test]
fn roundtrip_typed_fill_layer() {
    let json = serde_json::json!({
        "id": "water",
        "type": "fill",
        "source": "openmaptiles",
        "source-layer": "water",
        "filter": ["all", [">=", ["zoom"], 5], ["==", ["geometry-type"], "Polygon"]],
        "paint": { "fill-color": "#a0c8f0" }
    });
    let layer: AnyLayer = serde_json::from_value(json.clone()).expect("deserialize");
    assert!(matches!(&layer, AnyLayer::Typed(TypedLayer::Fill { .. })));
    assert_eq!(layer.layer_type(), Some("fill"));
    assert_eq!(layer.id().as_str(), "water");

    let back = serde_json::to_value(&layer).expect("serialize");
    assert_eq!(back, json);
}

#[test]
fn roundtrip_ref_layer() {
    let json = serde_json::json!({
        "id": "water-outline",
        "ref": "water"
    });
    let layer: AnyLayer = serde_json::from_value(json.clone()).expect("deserialize");
    assert!(matches!(&layer, AnyLayer::Ref(_)));
    assert_eq!(layer.id().as_str(), "water-outline");
    assert_eq!(layer.layer_type(), None);

    let back = serde_json::to_value(&layer).expect("serialize");
    assert_eq!(back, json);
}

#[test]
fn roundtrip_background_layer() {
    let json = serde_json::json!({
        "id": "background",
        "type": "background",
        "paint": { "background-color": "#f8f4f0" }
    });
    let layer: AnyLayer = serde_json::from_value(json.clone()).expect("deserialize");
    assert!(matches!(
        &layer,
        AnyLayer::Typed(TypedLayer::Background { .. })
    ));

    let back = serde_json::to_value(&layer).expect("serialize");
    assert_eq!(back, json);
}

#[test]
fn roundtrip_style_with_layers() {
    let json = serde_json::json!({
        "version": 8,
        "sources": {
            "openmaptiles": { "type": "vector", "url": "https://example/tiles.json" }
        },
        "layers": [
            {
                "id": "background",
                "type": "background",
                "paint": { "background-color": "#f8f4f0" }
            },
            {
                "id": "water",
                "type": "fill",
                "source": "openmaptiles",
                "source-layer": "water",
                "paint": { "fill-color": "#a0c8f0" }
            }
        ]
    });

    let style: MaplibreStyleSpecification =
        serde_json::from_value(json.clone()).expect("deserialize style");
    assert_eq!(style.layers.len(), 2);

    let back = serde_json::to_value(&style).expect("serialize style");
    assert_eq!(back["layers"], json["layers"]);
}

#[test]
fn typed_layer_helpers() {
    let json = serde_json::json!({
        "id": "roads",
        "type": "line",
        "source": "openmaptiles",
        "source-layer": "transportation",
        "filter": ["==", ["get", "class"], "motorway"],
        "paint": { "line-color": "#ff0000", "line-width": 2 }
    });
    let layer: AnyLayer = serde_json::from_value(json).expect("deserialize");
    assert_eq!(layer.source(), Some("openmaptiles"));
    assert_eq!(layer.source_layer(), Some("transportation"));

    if let AnyLayer::Typed(ref t) = layer {
        let common = t.common();
        assert!(common.filter.is_some());
        assert_eq!(t.layer_type(), "line");
    } else {
        panic!("expected Typed layer");
    }
}

#[test]
fn filter_value_access() {
    let json = serde_json::json!({
        "id": "x",
        "type": "fill",
        "filter": ["==", 1, 1]
    });
    let mut layer: AnyLayer = serde_json::from_value(json).expect("deserialize");
    if let AnyLayer::Typed(ref mut t) = layer {
        let common = t.common_mut();
        let filter = common.filter.as_mut().unwrap();
        let val = filter.as_value_mut();
        assert!(val.is_array());
    }
}

#[test]
fn expression_roundtrip_boolean() {
    // Verify that Boolean expressions serialize back to the ["op", ...] form.
    let json = serde_json::json!(["==", ["get", "class"], "motorway"]);
    let expr: maplibre_style_spec::spec::Boolean =
        serde_json::from_value(json.clone()).expect("deserialize Boolean");
    let back = serde_json::to_value(&expr).expect("serialize Boolean");
    assert_eq!(back, json, "Boolean roundtrip failed");
}

#[test]
fn expression_roundtrip_all() {
    let json = serde_json::json!([
        "all",
        [">=", ["zoom"], 5],
        ["==", ["geometry-type"], "Polygon"]
    ]);
    let expr: maplibre_style_spec::spec::Boolean =
        serde_json::from_value(json.clone()).expect("deserialize Boolean");
    let back = serde_json::to_value(&expr).expect("serialize Boolean");
    assert_eq!(back, json, "Boolean::All roundtrip failed");
}

#[test]
fn expression_roundtrip_step() {
    let json = serde_json::json!(["step", ["zoom"], 2, 10, 4, 15, 8]);
    let expr: maplibre_style_spec::spec::Any =
        serde_json::from_value(json.clone()).expect("deserialize Any");
    let back = serde_json::to_value(&expr).expect("serialize Any");
    assert_eq!(back, json, "Any::Step roundtrip failed");
}

#[test]
fn filter_literal_bool() {
    let json = serde_json::json!({
        "id": "x",
        "type": "fill",
        "filter": true
    });
    let layer: AnyLayer = serde_json::from_value(json).expect("deserialize");
    if let AnyLayer::Typed(ref t) = layer {
        let filter = t.common().filter.as_ref().unwrap();
        assert_eq!(filter.as_literal_bool(), Some(true));
    }
}
