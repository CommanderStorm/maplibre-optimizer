use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("Projection");
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
        struct Foo(Projection);
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "projection": {
          "type": "projection",
          "doc": "The projection configuration",
          "example": {
            "type": [
              "interpolate",
              ["linear"],
              ["zoom"],
              10, "vertical-perspective",
              12, "mercator"
            ]
          }
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            /// The projection configuration
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            struct Projection(Projection);

            #[cfg(test)] 
            mod test {
                use super::*;

                #[test]
                fn test_example_projection_decodes() {
                    let example = serde_json::json!({"type":["interpolate",["linear"],["zoom"],10,"vertical-perspective",12,"mercator"]});
                    let _ = serde_json::from_value::<Projection>(example).expect("example should decode");
                }
            }
            "#);
    }
}
