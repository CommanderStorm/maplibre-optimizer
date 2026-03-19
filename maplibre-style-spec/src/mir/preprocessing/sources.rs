use std::collections::BTreeMap;

use crate::decoder::r#enum::EnumValues;
use crate::decoder::{ParsedItem, PrimitiveType, TopLevelItem};
use crate::mir::lower::lower_parsed_item;
use crate::mir::preprocessing::pop_one_of_as_group;
use crate::mir::sources::{IntermediateSources, SourceTypeDef};

/// Consume the `"source"` OneOf key and all referenced `source_*` groups from `fields`,
/// returning structured source definitions keyed by source-type name.
///
/// The `"type"` discriminant field is extracted and stored separately; the `"sources"`
/// wildcard group (used only in spec validation) is also removed.
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
            let discriminant_value = extract_discriminant(&mut group, "type");

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

            (
                type_name,
                SourceTypeDef {
                    fields: mir_fields,
                    discriminant_value,
                },
            )
        })
        .collect();

    IntermediateSources { source_types }
}

/// Extract and remove a single-variant enum field, returning its sole variant name.
/// If the field is absent or not a single-variant enum, it is left in place and `None` is returned.
fn extract_discriminant(
    group: &mut BTreeMap<String, ParsedItem>,
    field_name: &str,
) -> Option<String> {
    let item = group.remove(field_name)?;
    if let ParsedItem::Primitive(PrimitiveType::Enum {
        values: EnumValues::Enum(ref vals),
        ..
    }) = item
        && vals.len() == 1
    {
        return Some(vals.keys().next().unwrap().clone());
    }
    // Not a single-variant discriminant — put it back
    group.insert(field_name.to_string(), item);
    None
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
            assert!(
                def.discriminant_value.is_some(),
                "source type '{name}' must have a discriminant value"
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
