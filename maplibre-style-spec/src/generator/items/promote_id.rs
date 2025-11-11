use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    scope
        .new_struct(name)
        .vis("pub")
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("String");
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
        pub struct Foo(String);
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
          "promoteId": {
            "type": "promoteId",
            "doc": "A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`."
          }
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct SourceGeojson {
                /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
                #[serde(rename="promoteId")]
                pub promote_id: Option<SourceGeojsonPromoteId>,
                /// The data type of the GeoJSON source.
                #[serde(rename="type")]
                pub r#type: SourceGeojsonType,
            }

            /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct SourceGeojsonPromoteId(String);

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
