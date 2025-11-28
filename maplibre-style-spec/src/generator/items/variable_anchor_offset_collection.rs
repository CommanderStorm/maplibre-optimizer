use codegen2::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    let anchor_name = "LayoutSymbolTextVariableAnchorValue";
    scope
        .new_struct(&name)
        .vis("pub")
        .doc(&common.doc)
        .derive("PartialEq, Debug, Clone")
        .tuple_field(format!(
            "Vec<({anchor_name}, (serde_json::Number, serde_json::Number))>"
        ));
    let des = scope
        .new_impl(name)
        .generic("'de")
        .impl_trait("serde::Deserialize<'de>")
        .new_fn("deserialize")
        .arg("_deserializer", "D")
        .generic("D")
        .bound("D", "serde::Deserializer<'de>")
        .ret("Result<Self, D::Error>");
    des.line("todo!()");

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
        #[derive(PartialEq, Debug, Clone)]
        pub struct Foo(Vec<(LayoutSymbolTextVariableAnchorValue, (serde_json::Number, serde_json::Number))>);

        impl<'de> serde::Deserialize<'de> for Foo {
            fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de>,
            {
                todo!()
            }
        }
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "text-variable-anchor-offset": {
          "type": "variableAnchorOffsetCollection",
          "requires": [
            "text-field",
            {
              "symbol-placement": [
                "point"
              ]
            }
          ],
          "doc": "To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations, each paired with an offset value. The renderer will attempt to place the label at each location, in order, before moving on to the next location+offset. Use `text-justify: auto` to choose justification based on anchor position. \n\n The length of the array must be even, and must alternate between enum and point entries. i.e., each anchor location must be accompanied by a point, and that point defines the offset when the corresponding anchor location is used. Positive offset values indicate right and down, while negative values indicate left and up. Anchor locations may repeat, allowing the renderer to try multiple offsets to try and place a label using the same anchor. \n\n When present, this property takes precedence over `text-anchor`, `text-variable-anchor`, `text-offset`, and `text-radial-offset`. \n\n ```json \n\n { \"text-variable-anchor-offset\": [\"top\", [0, 4], \"left\", [3,0], \"bottom\", [1, 1]] } \n\n ``` \n\n When the renderer chooses the `top` anchor, `[0, 4]` will be used for `text-offset`; the text will be shifted down by 4 ems. \n\n When the renderer chooses the `left` anchor, `[3, 0]` will be used for `text-offset`; the text will be shifted right by 3 ems.",
          "sdk-support": {
            "basic functionality": {
              "js": "3.3.0",
              "ios": "6.8.0",
              "android": "11.6.0"
            },
            "data-driven styling": {
              "js": "3.3.0",
              "ios": "https://github.com/maplibre/maplibre-native/issues/2358",
              "android": "https://github.com/maplibre/maplibre-native/issues/2358"
            }
          },
          "expression": {
            "interpolated": true,
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

        /// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations, each paired with an offset value. The renderer will attempt to place the label at each location, in order, before moving on to the next location+offset. Use `text-justify: auto` to choose justification based on anchor position. 
        ///
        ///  The length of the array must be even, and must alternate between enum and point entries. i.e., each anchor location must be accompanied by a point, and that point defines the offset when the corresponding anchor location is used. Positive offset values indicate right and down, while negative values indicate left and up. Anchor locations may repeat, allowing the renderer to try multiple offsets to try and place a label using the same anchor. 
        ///
        ///  When present, this property takes precedence over `text-anchor`, `text-variable-anchor`, `text-offset`, and `text-radial-offset`. 
        ///
        ///  ```json 
        ///
        ///  { "text-variable-anchor-offset": ["top", [0, 4], "left", [3,0], "bottom", [1, 1]] } 
        ///
        ///  ``` 
        ///
        ///  When the renderer chooses the `top` anchor, `[0, 4]` will be used for `text-offset`; the text will be shifted down by 4 ems. 
        ///
        ///  When the renderer chooses the `left` anchor, `[3, 0]` will be used for `text-offset`; the text will be shifted right by 3 ems.
        #[derive(PartialEq, Debug, Clone)]
        pub struct TextVariableAnchorOffset(Vec<(LayoutSymbolTextVariableAnchorValue, (serde_json::Number, serde_json::Number))>);

        impl<'de> serde::Deserialize<'de> for TextVariableAnchorOffset {
            fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de>,
            {
                todo!()
            }
        }

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        "#);
    }
}
