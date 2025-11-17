use codegen::Scope;
use serde_json::Number;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: &[Number]) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .attr("serde(untagged)")
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone");
    enu.new_variant("Unwrapped")
        .annotation("#[deprecated = \"Please see [`Self::One`] instead\"]").doc("A single value applies to all four sides.\n\nOnly avaliable for backwards compatibility.").tuple("serde_json::Number");
    enu.new_variant("One")
        .doc("A single value applies to all four sides")
        .tuple("Box<[serde_json::Number; 1]>");
    enu.new_variant("Two")
        .doc("two values apply to `[top/bottom, left/right]`")
        .tuple("Box<[serde_json::Number; 2]>");
    enu.new_variant("Three")
        .doc("three values apply to `[top, left/right, bottom]`")
        .tuple("Box<[serde_json::Number; 3]>");
    enu.new_variant("Four")
        .doc("four values apply to `[top, right, bottom, left]`")
        .tuple("Box<[serde_json::Number; 4]>");

    let mut items = String::from("Box::new([");

    let mut needs_separator = false;
    for item in default {
        if needs_separator {
            items.push_str(", ");
        }

        items.push_str(&item.to_string());
        items.push_str(".into()");
        needs_separator = true;
    }
    items.push_str("])");

    let enum_variant_name = match default.len() {
        1 => "One",
        2 => "Two",
        3 => "Three",
        4 => "Four",
        _ => panic!("invalid padding length"),
    };

    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!("Self::{enum_variant_name}({items})"));
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
        generate(&mut scope, "Foo", &Fields::default(), &[2.into()]);
        insta::assert_snapshot!(scope.to_string(), @r##"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[serde(untagged)]
        pub enum Foo {
            /// A single value applies to all four sides.
            ///
            /// Only avaliable for backwards compatibility.
            #[deprecated = "Please see [`Self::One`] instead"]
            Unwrapped(serde_json::Number),
            /// A single value applies to all four sides
            One(Box<[serde_json::Number; 1]>),
            /// two values apply to `[top/bottom, left/right]`
            Two(Box<[serde_json::Number; 2]>),
            /// three values apply to `[top, left/right, bottom]`
            Three(Box<[serde_json::Number; 3]>),
            /// four values apply to `[top, right, bottom, left]`
            Four(Box<[serde_json::Number; 4]>),
        }

        impl Default for Foo {
            fn default() -> Self {
                Self::One(Box::new([2.into()]))
            }
        }
        "##)
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "icon-padding": {
          "type": "padding",
          "default": [2],
          "units": "pixels",
          "doc": "Size of additional area round the icon bounding box used for detecting symbol collisions.",
          "requires": [
            "icon-image"
          ],
          "sdk-support": {
            "basic functionality": {
              "js": "0.10.0",
              "android": "2.0.1",
              "ios": "2.0.0"
            },
            "data-driven styling": {
              "js": "2.2.0",
              "android": "https://github.com/maplibre/maplibre-native/issues/2754",
              "ios": "https://github.com/maplibre/maplibre-native/issues/2754"
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

            /// Size of additional area round the icon bounding box used for detecting symbol collisions.
            #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
            #[serde(untagged)]
            pub enum IconPadding {
                /// A single value applies to all four sides.
                ///
                /// Only avaliable for backwards compatibility.
                #[deprecated = "Please see [`Self::One`] instead"]
                Unwrapped(serde_json::Number),
                /// A single value applies to all four sides
                One(Box<[serde_json::Number; 1]>),
                /// two values apply to `[top/bottom, left/right]`
                Two(Box<[serde_json::Number; 2]>),
                /// three values apply to `[top, left/right, bottom]`
                Three(Box<[serde_json::Number; 3]>),
                /// four values apply to `[top, right, bottom, left]`
                Four(Box<[serde_json::Number; 4]>),
            }

            impl Default for IconPadding {
                fn default() -> Self {
                    Self::One(Box::new([2.into()]))
                }
            }

            #[cfg(test)]
            mod test {
                use super::*;

            }
            "#);
    }
}
