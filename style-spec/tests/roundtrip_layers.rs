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
    let layer: AnyLayer = serde_json::from_value(json).expect("deserialize");
    if let AnyLayer::Typed(ref t) = layer {
        let filter = t.common().filter.as_ref().unwrap();
        // ["==", 1, 1] is an expression, not a literal bool.
        assert!(matches!(filter, Boolean::EqualEqual(..)));
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
        assert!(filter.is_always_true());
    }
}

#[test]
fn expr_or_literal_normalize() {
    // BooleanExpr(Literal(x)) → Bool(x)
    let eol = ExprOrLiteral::BooleanExpr(Box::new(Boolean::Literal(false)));
    let json = serde_json::to_value(&eol).unwrap();
    assert!(matches!(eol.normalize(), ExprOrLiteral::Bool(false)));

    // JSON round-trip normalizes too
    assert_eq!(json, serde_json::json!(false));
    let back: ExprOrLiteral = serde_json::from_value(json).unwrap();
    assert!(matches!(back, ExprOrLiteral::Bool(false)));
}

#[test]
fn filter_roundtrip_normalizes_expr_or_literal() {
    // Construct a filter with an unnormalized ExprOrLiteral deep inside
    let filter = Boolean::EqualEqual(
        ExprOrLiteral::BooleanExpr(Box::new(Boolean::Literal(true))),
        ExprOrLiteral::StringLiteral(StringLiteral::from("x".to_string())),
        None,
    );
    let json = serde_json::to_value(&filter).unwrap();
    let back: Boolean = serde_json::from_value(json).unwrap();
    // After round-trip, BooleanExpr(Literal(true)) should become Bool(true)
    if let Boolean::EqualEqual(ref lhs, _, _) = back {
        assert!(
            matches!(lhs, ExprOrLiteral::Bool(true)),
            "Expected Bool(true), got {lhs:?}"
        );
    } else {
        panic!("Expected EqualEqual");
    }
}

#[test]
fn deep_expr_or_literal_roundtrip_normalizes() {
    // BooleanExpr(Literal(true)) deep inside a filter should normalize on JSON round-trip.
    let filter = Boolean::In(
        ExprOrLiteral::Bool(false),
        ExprOrLiteral::BooleanExpr(Box::new(Boolean::Not(Box::new(Boolean::EqualEqual(
            ExprOrLiteral::BooleanExpr(Box::new(Boolean::Literal(true))),
            ExprOrLiteral::StringLiteral(StringLiteral::from("x".to_string())),
            None,
        ))))),
    );
    let json = serde_json::to_value(&filter).unwrap();
    let back: Boolean = serde_json::from_value(json).unwrap();
    // Dig into the result: In → BooleanExpr → Not → EqualEqual → first arg
    if let Boolean::In(_, ref rhs) = back
        && let ExprOrLiteral::BooleanExpr(b) = rhs
        && let Boolean::Not(inner) = &**b
        && let Boolean::EqualEqual(lhs, _, _) = &**inner
    {
        assert!(
            matches!(lhs, ExprOrLiteral::Bool(true)),
            "Deep ExprOrLiteral should normalize, got {lhs:?}"
        );
        return;
    }
    panic!("unexpected filter structure: {back:?}");
}

#[test]
fn filter_after_fold_deserializes_normalized() {
    // After expression passes fold ["all"] → true, the JSON becomes ["!=", ["collator", {}], true].
    // This should deserialize with Bool(true), not BooleanExpr(Literal(true)).
    let json = serde_json::json!(["!=", ["collator", {}], true]);
    let filter: Boolean = serde_json::from_value(json).unwrap();
    if let Boolean::NotEqual(_, ref rhs, _) = filter {
        assert!(
            matches!(rhs, ExprOrLiteral::Bool(true)),
            "Expected Bool(true) from bare JSON true, got {rhs:?}"
        );
    } else {
        panic!("Expected NotEqual, got {filter:?}");
    }
}

#[test]
fn filter_with_boolean_all_empty_roundtrips() {
    // BooleanExpr(All([])) should roundtrip through JSON and Boolean deserialization
    let filter = Boolean::NotEqual(
        ExprOrLiteral::Null,
        ExprOrLiteral::BooleanExpr(Box::new(Boolean::All(vec![]))),
        None,
    );
    let json = serde_json::to_value(&filter).unwrap();
    eprintln!("JSON: {json}");
    let back: Boolean = serde_json::from_value(json).unwrap();
    eprintln!("Back: {back:?}");
    // The All([]) should survive the roundtrip as BooleanExpr(All([]))
    if let Boolean::NotEqual(_, ref rhs, _) = back {
        eprintln!("RHS: {rhs:?}");
        // BooleanExpr(All([])) serializes as ["all"], which is an array
        // On deserialization, ExprOrLiteral tries Bool first (fails for array),
        // then eventually tries BooleanExpr which succeeds.
        assert!(
            matches!(rhs, ExprOrLiteral::BooleanExpr(_)),
            "Expected BooleanExpr, got {rhs:?}"
        );
    }
}
