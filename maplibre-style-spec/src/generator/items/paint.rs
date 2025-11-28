use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("Paint");
    generate_test_from_example_if_present(scope, name, common.example.as_ref());
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
        struct Foo(Paint);
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "layer": {
          "paint": {
            "type": "paint",
            "doc": "Default paint properties for this layer."
          }
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Layer {
            /// Default paint properties for this layer.
            pub paint: Option<LayerPaint>,
        }

        /// Default paint properties for this layer.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct LayerPaint(Paint);

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        ");
    }
}
