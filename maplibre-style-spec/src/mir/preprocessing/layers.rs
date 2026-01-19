use crate::decoder;
use crate::mir::layers::IntermediateLayers;
use crate::mir::preprocessing::pop_one_of_as_group;
use std::collections::{BTreeMap, BTreeSet};

pub fn preprocess_layers(reference: &mut decoder::StyleReference) -> IntermediateLayers {
    let decoder::PrimitiveType::Array {
        common,
        default,
        value,
        values,
        minimum,
        maximum,
        length,
    } = reference.root.remove("layers").unwrap().as_primitive()
    else {
        panic!("layers must be an array");
    };

    let mut layer = reference.fields.remove("layer").unwrap().as_group().clone();
    let (layer_type, layer_type_fields,layer_type_example) = layer.remove("type").unwrap().as_primitive().as_enum();
    let layout_docs = layer.remove("layout").unwrap().as_reference();
    let layout = pop_one_of_as_group(&mut reference.fields, "layout");
    let paint_docs = layer.remove("paint").unwrap().as_reference();
    let paint = pop_one_of_as_group(&mut reference.fields, "paint");
    assert_eq!(
        paint.keys().collect::<BTreeSet<_>>(),
        layout.keys().collect::<BTreeSet<_>>(),
        "paint and layout have the same keys"
    );
    layer_type.0.as_enum().keys()

    for key in layout.keys() {

    }

    IntermediateLayers {common_fields: layer, }
}
#[test]
fn test_decode_top_level() {
    let mut reference: decoder::StyleReference =
        serde_json::from_str(include_str!("../../../../upstream/src/reference/v8.json")).unwrap();
    let layers = preprocess_layers(&mut reference);
    insta::assert_json_snapshot!(layers, @"")
}
