use std::collections::BTreeMap;

use crate::decoder::r#enum::EnumValues;
use crate::decoder::{ParsedItem, PrimitiveType, TopLevelItem};
use crate::mir::lower::lower_parsed_item;
use crate::mir::preprocessing::pop_one_of_as_group;
use crate::mir::sources::{IntermediateSources, SourceTypeDef};

/// Consume the `"source"` OneOf key and all referenced `source_*` groups from `fields`,
/// returning structured source definitions keyed by source-type name.
///
/// The `"type"` discriminant field is stripped from each group (the map key is the JSON
/// `"type"` value for serde); the `"sources"` wildcard group (used only in spec validation)
/// is also removed.
pub fn preprocess_sources(fields: &mut BTreeMap<String, TopLevelItem>) -> IntermediateSources {
    // Remove the `sources` wildcard group — not needed for codegen
    fields.remove("sources");

    // If there is no "source" OneOf entry (e.g. in unit tests), return empty.
    if !fields.contains_key("source") {
        return IntermediateSources {
            source_types: BTreeMap::new(),
        };
    }

    let source_groups = pop_one_of_as_group(fields, "source");

    let source_types = source_groups
        .into_iter()
        .map(|(type_name, mut group)| {
            strip_source_type_discriminant_field(&mut group, "type");

            let mir_fields = group
                .into_iter()
                .filter(|(k, _)| k != "*" && k != "property-type")
                .filter(|(_, v)| {
                    !matches!(
                        v,
                        crate::decoder::ParsedItem::Primitive(
                            crate::decoder::PrimitiveType::PropertyType(_)
                        )
                    )
                })
                .map(|(k, v)| lower_parsed_item(&k, v))
                .collect();

            (type_name, SourceTypeDef { fields: mir_fields })
        })
        .collect();

    IntermediateSources { source_types }
}

/// Remove the source `"type"` field when it is the usual single-variant enum, so it is not
/// emitted twice (serde tag + field). The JSON discriminant string is always the map key
/// (`source_types` in [`IntermediateSources`]).
fn strip_source_type_discriminant_field(
    group: &mut BTreeMap<String, ParsedItem>,
    field_name: &str,
) {
    let Some(item) = group.remove(field_name) else {
        return;
    };
    if let ParsedItem::Primitive(PrimitiveType::Enum {
        values: EnumValues::Enum(ref vals),
        ..
    }) = item
        && vals.len() == 1
    {
        return;
    }
    group.insert(field_name.to_string(), item);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder;

    #[test]
    fn test_preprocess_sources_snapshot() {
        let mut reference: decoder::StyleReference =
            serde_json::from_str(include_str!("../../../../upstream/src/reference/v8.json"))
                .unwrap();
        let sources = preprocess_sources(&mut reference.fields);
        insta::assert_json_snapshot!(sources);
    }

    #[test]
    fn test_preprocess_sources() {
        let mut reference: decoder::StyleReference =
            serde_json::from_str(include_str!("../../../../upstream/src/reference/v8.json"))
                .unwrap();

        let sources = preprocess_sources(&mut reference.fields);

        let mut keys: Vec<&str> = sources.source_types.keys().map(|s| s.as_str()).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec![
                "geojson",
                "image",
                "raster",
                "raster_dem",
                "vector",
                "video"
            ],
            "expected exactly the six MapLibre source types"
        );

        for (name, def) in &sources.source_types {
            assert!(
                !def.fields.is_empty(),
                "source type '{name}' must have at least one field"
            );
        }

        // Spot-check: vector source should have url, tiles, minzoom, maxzoom fields
        let vector_names: Vec<&str> = sources.source_types["vector"]
            .fields
            .iter()
            .map(|f| f.meta().spec_name.as_str())
            .collect();
        assert!(vector_names.contains(&"url"), "vector source must have url");
        assert!(
            vector_names.contains(&"tiles"),
            "vector source must have tiles"
        );
        assert!(
            vector_names.contains(&"minzoom"),
            "vector source must have minzoom"
        );
        assert!(
            vector_names.contains(&"maxzoom"),
            "vector source must have maxzoom"
        );

        // Spot-check: geojson source must have data field
        let geojson_names: Vec<&str> = sources.source_types["geojson"]
            .fields
            .iter()
            .map(|f| f.meta().spec_name.as_str())
            .collect();
        assert!(
            geojson_names.contains(&"data"),
            "geojson source must have data"
        );
    }
}
