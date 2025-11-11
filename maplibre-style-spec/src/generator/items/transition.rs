use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .attr("deprecated = \"transition not implemented\"")
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
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[deprecated = "transition not implemented"]
        struct Foo(serde_json::Value);
        "#)
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},"transition": {
        "type": "transition",
        "doc": "A global transition definition to use as a default across properties, to be used for timing transitions between one value and the next when no property-specific transition is set. Collision-based symbol fading is controlled independently of the style's `transition` property.",
        "example": {
          "duration": 300,
          "delay": 0
          }
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            /// A global transition definition to use as a default across properties, to be used for timing transitions between one value and the next when no property-specific transition is set. Collision-based symbol fading is controlled independently of the style's `transition` property.
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            #[deprecated = "transition not implemented"]
            struct Transition(serde_json::Value);

            #[cfg(test)] 
            mod test {
                use super::*;

                #[test]
                fn test_example_transition_decodes() {
                    let example = serde_json::json!({"delay":0,"duration":300});
                    let _ = serde_json::from_value::<Transition>(example).expect("example should decode");
                }
            }
            "#);
    }
}
