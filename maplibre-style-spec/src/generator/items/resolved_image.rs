use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, _tokens: Option<bool>) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .attr("deprecated = \"resolved_image not implemented\"")
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
        generate(&mut scope, "Foo", &Fields::default(), None);
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[deprecated = "resolved_image not implemented"]
        struct Foo(serde_json::Value);
        "#)
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "icon-image": {
          "type": "resolvedImage",
          "doc": "Name of image in sprite to use for drawing an image background.",
          "tokens": true,
          "sdk-support": {
            "basic functionality": {
              "js": "0.10.0",
              "android": "2.0.1",
              "ios": "2.0.0"
            },
            "data-driven styling": {
              "js": "0.35.0",
              "android": "5.1.0",
              "ios": "3.6.0"
            }
          },
          "expression": {
            "interpolated": false,
            "parameters": [
              "zoom",
              "feature"
            ]
          },
          "property-type": "data-driven"
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            /// Name of image in sprite to use for drawing an image background.
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            #[deprecated = "resolved_image not implemented"]
            struct IconImage(serde_json::Value);

            #[cfg(test)] 
            mod test {
                use super::*;

            }
            "#);
    }
}
