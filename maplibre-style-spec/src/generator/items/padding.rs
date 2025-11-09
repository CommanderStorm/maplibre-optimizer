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
    generate_test_from_example_if_present(scope, name, common);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), &[2.into()]);
        insta::assert_snapshot!(scope.to_string(), @r#"
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
        "#)
    }
}
