use codegen::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("Sky");
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
        struct Foo(Sky);
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "sky": {
          "type": "sky",
          "doc": "The map's sky configuration. **Note:** this definition is still experimental and is under development in maplibre-gl-js.",
          "example": {
            "sky-color": "#199EF3",
            "sky-horizon-blend": 0.5,
            "horizon-color": "#ffffff",
            "horizon-fog-blend": 0.5,
            "fog-color": "#0000ff",
            "fog-ground-blend": 0.5,
            "atmosphere-blend": ["interpolate",
              ["linear"],
              ["zoom"],
              0,1,
              10,1,
              12,0
            ]
          }
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r##"
            /// This is a Maplibre Style Specification
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            pub struct MaplibreStyleSpecification;

            /// The map's sky configuration. **Note:** this definition is still experimental and is under development in maplibre-gl-js.
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            struct Sky(Sky);

            #[cfg(test)]
            mod test {
                use super::*;

                #[test]
                fn test_example_sky_decodes() {
                    let example = serde_json::json!({"atmosphere-blend":["interpolate",["linear"],["zoom"],0,1,10,1,12,0],"fog-color":"#0000ff","fog-ground-blend":0.5,"horizon-color":"#ffffff","horizon-fog-blend":0.5,"sky-color":"#199EF3","sky-horizon-blend":0.5});
                    let _ = serde_json::from_value::<Sky>(example).expect("example should decode");
                }
            }
            "##);
    }
}
