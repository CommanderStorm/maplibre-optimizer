use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::generator::untagged::{self, Variant};
use crate::mir::types::PaddingField;

pub fn generate(scope: &mut Scope, name: &str, field: &PaddingField) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .doc(&field.meta.doc)
        .derive("PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    enu.new_variant("One")
        .doc("A single value applies to all four sides")
        .tuple_with_attrs(
            [fuzz::ARB_BOX_1_JSON_NUMBER],
            "Box<[serde_json::Number; 1]>",
        );
    enu.new_variant("Two")
        .doc("two values apply to `[top/bottom, left/right]`")
        .tuple_with_attrs(
            [fuzz::ARB_BOX_2_JSON_NUMBER],
            "Box<[serde_json::Number; 2]>",
        );
    enu.new_variant("Three")
        .doc("three values apply to `[top, left/right, bottom]`")
        .tuple_with_attrs(
            [fuzz::ARB_BOX_3_JSON_NUMBER],
            "Box<[serde_json::Number; 3]>",
        );
    enu.new_variant("Four")
        .doc("four values apply to `[top, right, bottom, left]`")
        .tuple_with_attrs(
            [fuzz::ARB_BOX_4_JSON_NUMBER],
            "Box<[serde_json::Number; 4]>",
        );

    untagged::emit_untagged_serde(
        scope,
        name,
        &[
            Variant {
                name: "One".into(),
                inner_type: "[serde_json::Number; 1]".into(),
                is_boxed: true,
                is_unit: false,
                skip_when: None,
            },
            Variant {
                name: "Two".into(),
                inner_type: "[serde_json::Number; 2]".into(),
                is_boxed: true,
                is_unit: false,
                skip_when: None,
            },
            Variant {
                name: "Three".into(),
                inner_type: "[serde_json::Number; 3]".into(),
                is_boxed: true,
                is_unit: false,
                skip_when: None,
            },
            Variant {
                name: "Four".into(),
                inner_type: "[serde_json::Number; 4]".into(),
                is_boxed: true,
                is_unit: false,
                skip_when: None,
            },
        ],
    );

    let mut items = String::from("Box::new([");
    let mut needs_separator = false;
    for item in &field.default {
        if needs_separator {
            items.push_str(", ");
        }
        items.push_str(&item.to_string());
        items.push_str(".into()");
        needs_separator = true;
    }
    items.push_str("])");

    let enum_variant_name = match field.default.len() {
        1 => "One",
        2 => "Two",
        3 => "Three",
        4 => "Four",
        _ => panic!("invalid padding default length"),
    };

    scope
        .new_impl(name)
        .impl_trait("Default")
        .new_fn("default")
        .ret("Self")
        .line(format!("Self::{enum_variant_name}({items})"));
    generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::types::FieldMeta;

    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &PaddingField {
                meta: FieldMeta::default(),
                default: vec![2.into()],
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub enum Foo {
            /// A single value applies to all four sides
            One(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_1_json_number))]
                 Box<[serde_json::Number; 1]>,
            ),
            /// two values apply to `[top/bottom, left/right]`
            Two(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
                 Box<[serde_json::Number; 2]>,
            ),
            /// three values apply to `[top, left/right, bottom]`
            Three(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_3_json_number))]
                 Box<[serde_json::Number; 3]>,
            ),
            /// four values apply to `[top, right, bottom, left]`
            Four(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
                 Box<[serde_json::Number; 4]>,
            ),
        }

        impl serde::Serialize for Foo {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self {
                    Self::One(v) => v.as_ref().serialize(serializer),
                    Self::Two(v) => v.as_ref().serialize(serializer),
                    Self::Three(v) => v.as_ref().serialize(serializer),
                    Self::Four(v) => v.as_ref().serialize(serializer),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for Foo {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
                let mut errors: Vec<(&str, std::string::String)> = Vec::new();
                match <[serde_json::Number; 1] as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::One(Box::new(v))),
                    Err(e) => errors.push(("One", e.to_string())),
                }
                match <[serde_json::Number; 2] as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::Two(Box::new(v))),
                    Err(e) => errors.push(("Two", e.to_string())),
                }
                match <[serde_json::Number; 3] as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::Three(Box::new(v))),
                    Err(e) => errors.push(("Three", e.to_string())),
                }
                match <[serde_json::Number; 4] as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::Four(Box::new(v))),
                    Err(e) => errors.push(("Four", e.to_string())),
                }

                let details: Vec<std::string::String> =
                    errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
                Err(serde::de::Error::custom(format!(
                    "Foo: no variant matched. Expected One([serde_json::Number; 1]) | Two([serde_json::Number; 2]) | Three([serde_json::Number; 3]) | Four([serde_json::Number; 4]). Errors: [{}]",
                    details.join("; ")
                )))
            }
        }

        impl Default for Foo {
            fn default() -> Self {
                Self::One(Box::new([2.into()]))
            }
        }
        "#)
    }
}
