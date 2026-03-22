use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::generator::untagged::{self, Variant};
use crate::mir::types::ColorArrayField;

pub fn generate(scope: &mut Scope, name: &str, field: &ColorArrayField) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .doc(&field.meta.doc)
        .derive("PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    enu.new_variant("One")
        .tuple_with_attrs([fuzz::ARB_DYNAMIC_COLOR], "color::DynamicColor")
        .doc("A color");
    enu.new_variant("Multiple")
        .tuple_with_attrs([fuzz::ARB_VEC_DYNAMIC_COLOR], "Vec<color::DynamicColor>")
        .doc("A set of colors");
    untagged::emit_untagged_serde(
        scope,
        name,
        &[
            Variant {
                name: "One".into(),
                inner_type: "color::DynamicColor".into(),
                is_boxed: false,
                is_unit: false,
            },
            Variant {
                name: "Multiple".into(),
                inner_type: "Vec<color::DynamicColor>".into(),
                is_boxed: false,
                is_unit: false,
            },
        ],
    );

    if let Some(default) = &field.default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self::One(color::parse_color(\"{default}\").expect(\"Invalid color specified as the default value\"))"));
    }
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
            &ColorArrayField {
                meta: FieldMeta::default(),
                default: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub enum Foo {
            /// A color
            One(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_dynamic_color))]
                 color::DynamicColor,
            ),
            /// A set of colors
            Multiple(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_dynamic_color))]
                 Vec<color::DynamicColor>,
            ),
        }

        impl serde::Serialize for Foo {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self {
                    Self::One(v) => v.serialize(serializer),
                    Self::Multiple(v) => v.serialize(serializer),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for Foo {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
                let mut errors: Vec<(&str, std::string::String)> = Vec::new();
                match <color::DynamicColor as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::One(v)),
                    Err(e) => errors.push(("One", e.to_string())),
                }
                match <Vec<color::DynamicColor> as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::Multiple(v)),
                    Err(e) => errors.push(("Multiple", e.to_string())),
                }

                let details: Vec<std::string::String> =
                    errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
                Err(serde::de::Error::custom(format!(
                    "Foo: no variant matched. Expected One(color::DynamicColor) | Multiple(Vec<color::DynamicColor>). Errors: [{}]",
                    details.join("; ")
                )))
            }
        }
        "#)
    }
}
