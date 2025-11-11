use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Value");
    generate_test_from_example_if_present(scope, name, common);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decoder::StyleReference;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default());
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Foo(serde_json::Value);
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "source_geojson": {
          "type": {
            "required": true,
            "type": "enum",
            "values": {
              "geojson": {
                "doc": "A GeoJSON data source."
              }
            },
            "doc": "The data type of the GeoJSON source."
          },
          "data": {
            "required": true,
            "type": "*",
            "doc": "A URL to a GeoJSON file, or inline GeoJSON."
          },
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct SourceGeojson {
                /// A URL to a GeoJSON file, or inline GeoJSON.
                pub data: SourceGeojsonData,
                /// The data type of the GeoJSON source.
                #[serde(rename="type")]
                pub r#type: SourceGeojsonType,
            }

            /// A URL to a GeoJSON file, or inline GeoJSON.
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            struct SourceGeojsonData(serde_json::Value);

            /// The data type of the GeoJSON source.
            #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
            pub enum SourceGeojsonType {
                /// A GeoJSON data source.
                #[serde(rename="geojson")]
                Geojson,
            }

            #[cfg(test)] 
            mod test {
                use super::*;

            }
            "#);
    }
}
